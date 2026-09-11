"""Pinned SDK construction for explicitly delivered R2 credentials.

No factory call performs a service request. Use the returned client through
the archive adapters; configuration clients expose only lifecycle GETs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date, datetime
from functools import partial
from urllib.parse import urlsplit

import boto3
import botocore.session
from botocore.config import Config
from botocore.configprovider import ConstantProvider
from botocore.credentials import CredentialResolver
from botocore.exceptions import ClientError
from botocore.loaders import Loader

from .archive_r2_config import PolicyObservation, R2Target, _label, _require
from .archive_r2_control import make_observation
from .archive_r2_store import _error_parts, _failure, validate_client
from .common import GovernanceError, canonical_json_bytes


@dataclass(frozen=True)
class R2Credentials:
    reference: str
    access_key: str = field(repr=False)
    secret_key: str = field(repr=False)
    session_token: str | None = field(default=None, repr=False)

    def __post_init__(self) -> None:
        _require(_label(self.reference), "invalid credential reference")
        for value in (self.access_key, self.secret_key):
            _require(type(value) is str and bool(value) and value.isascii()
                     and all(33 <= ord(c) <= 126 for c in value), "explicit SDK credentials required")
        _require(self.session_token is None or (
            type(self.session_token) is str and bool(self.session_token)
            and self.session_token.isascii()
            and all(33 <= ord(c) <= 126 for c in self.session_token)), "invalid explicit session token")


class _IsolatedSession(botocore.session.Session):
    """Never inspect shared profiles, credential files or ambient SDK models."""

    def get_scoped_config(self):
        return {}

    @property
    def full_config(self):
        return {"profiles": {}}


def _restrict_operation(target, purpose, params, model, **_):
    permitted = {"configuration": {"GetBucketLifecycleConfiguration"},
                 "reader": {"GetObject"},
                 "producer": {"GetObject", "PutObject"},
                 "receipt-writer": {"PutObject"}}[purpose]
    _require(model.name in permitted and params.get("Bucket") == target.bucket,
             "SDK operation or bucket is outside the bound role")
    if model.name == "GetBucketLifecycleConfiguration":
        _require(set(params) == {"Bucket"}, "unsupported lifecycle request parameters")
        return
    target.validate_key(params.get("Key"), receipt_only=purpose == "receipt-writer")
    if model.name == "GetObject":
        _require(set(params) == {"Bucket", "Key"}, "fresh complete SDK read required")
    else:
        _require(set(params) == {"Bucket", "Key", "Body", "ContentLength", "IfNoneMatch", "StorageClass"}
                 and params["IfNoneMatch"] == "*" and params["StorageClass"] == "STANDARD"
                 and type(params["Body"]) is bytes and type(params["ContentLength"]) is int
                 and params["ContentLength"] == len(params["Body"]), "conditional exact-byte SDK PUT required")


def _restrict_wire(target, purpose, request, **_):
    url = urlsplit(request.url)
    bound = urlsplit(target.endpoint)
    _require(url.scheme == "https" and url.netloc == bound.netloc and not url.fragment,
             "SDK serialized endpoint mismatch")
    if purpose == "configuration":
        _require(request.method == "GET" and url.path == f"/{target.bucket}"
                 and url.query == "lifecycle", "SDK serialized lifecycle target mismatch")
    else:
        prefix = f"/{target.bucket}/"
        _require(url.path.startswith(prefix) and not url.query, "SDK serialized object target mismatch")
        target.validate_key(url.path[len(prefix):], receipt_only=purpose == "receipt-writer")
        _require(request.method in ({"GET"} if purpose == "reader" else
                                   {"PUT"} if purpose == "receipt-writer" else {"GET", "PUT"}),
                 "SDK serialized method mismatch")
        if request.method == "PUT":
            _require(request.headers.get("If-None-Match") == b"*", "SDK omitted conditional create")


def _stop_failed_attempt(response=None, caught_exception=None, operation=None, **_):
    # Run before S3 region redirect handling, which can issue its own HEAD and
    # retry independently of standard retry limits. Never discover/retarget.
    if caught_exception is not None:
        raise caught_exception
    if response is not None and response[0].status_code >= 300:
        raise ClientError(response[1], operation.name)


def create_r2_client(target: R2Target, credentials: R2Credentials, *, purpose: str):
    """Create a new client; no global session, credential search or network IO."""
    _require(type(target) is R2Target and type(credentials) is R2Credentials,
             "explicit R2 target and credentials required")
    _require(purpose in ("producer", "reader", "receipt-writer", "configuration"), "invalid SDK role")
    _require(credentials.reference == target.credential_ref, "SDK credential reference mismatch")
    client = None
    try:
        variables = {name: (None, None, spec[2], spec[3])
                     for name, spec in botocore.session.Session.SESSION_VARIABLES.items()}
        session = _IsolatedSession(session_vars=variables)
        session.register_component("credential_provider", CredentialResolver([]))
        session.register_component("data_loader", Loader(
            extra_search_paths=[Loader.BUILTIN_DATA_PATH], include_default_search_paths=False))
        session.get_component("config_store").set_config_provider(
            "s3", ConstantProvider({"addressing_style": "path"}))
        session.set_credentials(credentials.access_key, credentials.secret_key, credentials.session_token)
        config = Config(
            region_name="auto", signature_version="s3v4", s3={"addressing_style": "path"},
            retries={"total_max_attempts": 1, "mode": "standard"},
            request_checksum_calculation="when_required", response_checksum_validation="when_required",
            proxies={}, connect_timeout=10, read_timeout=30,
            endpoint_discovery_enabled=False, ignore_configured_endpoint_urls=True,
            use_dualstack_endpoint=False, use_fips_endpoint=False,
            disable_request_compression=True,
        )
        client = boto3.Session(botocore_session=session).client(
            "s3", endpoint_url=target.endpoint, region_name="auto", use_ssl=True, verify=True,
            aws_access_key_id=credentials.access_key, aws_secret_access_key=credentials.secret_key,
            aws_session_token=credentials.session_token, config=config,
        )
        # Check caller bytes before the SDK's PutObject-specific handler wraps
        # Body in its own seekable stream for signing and transport.
        client.meta.events.register_first("provide-client-params.s3", partial(_restrict_operation, target, purpose))
        client.meta.events.register_first("before-send.s3", partial(_restrict_wire, target, purpose))
        client.meta.events.register_first("needs-retry.s3", _stop_failed_attempt)
        client._sifr_archive_binding = (target.binding, credentials.reference, purpose)
        validate_client(target, client)
        return client
    except Exception:
        if client is not None:
            try:
                client.close()
            except Exception:
                pass
        raise GovernanceError("R2 SDK construction failed") from None


def _json_dates(value):
    if isinstance(value, (date, datetime)):
        return value.isoformat()
    if isinstance(value, list):
        return [_json_dates(part) for part in value]
    if isinstance(value, dict):
        return {key: _json_dates(part) for key, part in value.items()}
    return value


def observe_lifecycle(target: R2Target, client, *, authority_ref: str) -> PolicyObservation:
    """One SDK GET; the retained raw_response is canonical SDK-shaped JSON."""
    _require(_label(authority_ref) and authority_ref != target.credential_ref,
             "separate configuration authority required")
    _require(getattr(client, "_sifr_archive_binding", None) == (target.binding, authority_ref, "configuration"),
             "lifecycle client authority/target mismatch")
    validate_client(target, client)
    try:
        try:
            response = client.get_bucket_lifecycle_configuration(Bucket=target.bucket)
        except ClientError as exc:
            if _error_parts(exc) != (404, "NoSuchLifecycleConfiguration"):
                raise
            response = {"ResponseMetadata": exc.response["ResponseMetadata"],
                        "Error": {"Code": "NoSuchLifecycleConfiguration"}}
        metadata = response["ResponseMetadata"]
        status, request_id = metadata["HTTPStatusCode"], metadata.get("RequestId")
        # Transport headers/messages may contain opaque or sensitive values;
        # retain only the fields consumed by the existing observation contract.
        raw = canonical_json_bytes(_json_dates({
            "ResponseMetadata": {"HTTPStatusCode": status, "RequestId": request_id},
            **{key: response[key] for key in ("Rules", "Error") if key in response},
        }))
        return make_observation(target, "lifecycle", raw, status, request_id, authority_ref)
    except Exception as exc:
        raise _failure("lifecycle observation", exc) from None
