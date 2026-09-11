"""Bound, read-only Cloudflare control transport. No redirects or proxy use."""

from __future__ import annotations

import http.client
import ssl
from dataclasses import dataclass, field
from datetime import datetime, timezone

from botocore.httpsession import get_cert_path

from .archive_r2_config import PolicyObservation, R2Target, _label, _require
from .common import GovernanceError, canonical_json_bytes, sha256_bytes

CONTROL_HOST = "api.cloudflare.com"
CONTROL_ENDPOINT = "https://api.cloudflare.com/client/v4"
MAX_CONTROL_BYTES = 1024 * 1024


@dataclass(frozen=True)
class ControlCredentials:
    reference: str
    bearer_token: str = field(repr=False)

    def __post_init__(self) -> None:
        _require(_label(self.reference), "invalid control credential reference")
        value = self.bearer_token
        _require(type(value) is str and bool(value) and value.isascii()
                 and all(33 <= ord(c) <= 126 for c in value), "explicit control credential required")


def make_observation(target, operation, raw, status, request_id, authority_ref):
    """Bind captured bytes to the request actually made; never include auth."""
    path = f"/accounts/{target.account}/r2/buckets/{target.bucket}"
    if operation == "locks":
        path += "/lock"
    if operation == "lifecycle":
        path = f"/{target.bucket}?lifecycle"
    value = {
        "account": target.account, "jurisdiction": target.jurisdiction, "bucket": target.bucket,
        "archive_root": target.root, "source_commit": target.expected_source,
        "run_id": target.expected_run, "run_attempt": target.expected_attempt,
        "endpoint": target.endpoint if operation == "lifecycle" else CONTROL_ENDPOINT,
        "method": "GET", "path": path, "operation": operation,
        "headers": ({"cf-r2-jurisdiction": target.jurisdiction}
                    if operation != "lifecycle" and target.jurisdiction != "default" else {}),
        "observed_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "status": status, "request_id": request_id, "authority_ref": authority_ref,
        "raw_response": raw.decode("utf-8"), "response_sha256": sha256_bytes(raw),
    }
    observation = PolicyObservation(canonical_json_bytes(value))
    observation.parse(target, operation)
    return observation


def observe_control(target: R2Target, credentials: ControlCredentials, *, operation: str) -> PolicyObservation:
    """Fetch fresh bucket metadata or lock rules once, through verified HTTPS."""
    _require(type(target) is R2Target and type(credentials) is ControlCredentials,
             "explicit control target and credentials required")
    _require(credentials.reference != target.credential_ref, "separate configuration authority required")
    _require(operation in ("metadata", "locks"), "unsupported control GET")
    path = f"/client/v4/accounts/{target.account}/r2/buckets/{target.bucket}"
    if operation == "locks":
        path += "/lock"
    headers = {"Authorization": "Bearer " + credentials.bearer_token, "Accept": "application/json",
               "Accept-Encoding": "identity"}
    if target.jurisdiction != "default":
        headers["cf-r2-jurisdiction"] = target.jurisdiction
    connection = response = None
    try:
        # Pin the SDK-owned CA bundle instead of SSL_CERT_FILE/SSL_CERT_DIR.
        context = ssl.create_default_context(cafile=get_cert_path(True))
        connection = http.client.HTTPSConnection(CONTROL_HOST, timeout=30, context=context)
        connection.request("GET", path, headers=headers)
        response = connection.getresponse()
        _require(response.status == 200, "control GET failed")
        pairs = response.getheaders()
        names = [name.lower() for name, _ in pairs]
        _require(len(names) == len(set(names)), "duplicate control response headers")
        observed_headers = {name.lower(): value for name, value in pairs}
        _require("content-encoding" not in observed_headers and "content-range" not in observed_headers,
                 "encoded or partial control response")
        length = observed_headers.get("content-length")
        if length is not None:
            _require(length.isascii() and length.isdecimal() and int(length) <= MAX_CONTROL_BYTES,
                     "invalid control response length")
        _require(not (length is not None and "transfer-encoding" in observed_headers),
                 "ambiguous control response framing")
        raw = response.read(MAX_CONTROL_BYTES + 1)
        _require(type(raw) is bytes and len(raw) <= MAX_CONTROL_BYTES,
                 "over-limit control response")
        _require(length is None or len(raw) == int(length), "truncated control response")
        _require(response.read(1) == b"", "incomplete control response")
        return make_observation(target, operation, raw, response.status,
                                observed_headers.get("cf-ray"), credentials.reference)
    except Exception:
        raise GovernanceError("R2 control observation failed") from None
    finally:
        cleanup_failed = False
        try:
            if response is not None:
                response.close()
        except Exception:
            cleanup_failed = True
        finally:
            try:
                if connection is not None:
                    connection.close()
            except Exception:
                cleanup_failed = True
        if cleanup_failed:
            raise GovernanceError("R2 control cleanup failed") from None
