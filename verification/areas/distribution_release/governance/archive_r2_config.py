"""Pure R2 target and observed-policy admission; no credentials or network.

An accepted observation records supplied bytes, not authenticated live access.
The online owner must obtain these observations and prove their custody/freshness.
"""

from __future__ import annotations

import ipaddress
import re
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any

from .archive_manifest import parse_manifest
from .common import GovernanceError, load_json_bytes_strict, sha256_bytes


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise GovernanceError(message)


def _shape(value: Any, required: set[str], optional: set[str] = frozenset()) -> dict:
    _require(isinstance(value, dict), "invalid R2 observation object")
    _require(required <= value.keys() <= required | optional, "unknown or missing R2 policy fields")
    return value


def _label(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}", value) is not None


@dataclass(frozen=True)
class R2Target:
    account: str
    jurisdiction: str
    bucket: str
    credential_ref: str
    manifest_bytes: bytes = field(repr=False)
    manifest_sha256: str
    expected_source: str
    expected_run: int
    expected_attempt: int
    storage_class: str = "STANDARD"
    root: str = field(init=False)

    def __post_init__(self) -> None:
        _require(isinstance(self.account, str) and re.fullmatch(r"[0-9a-f]{32}", self.account) is not None,
                 "invalid R2 account")
        _require(self.jurisdiction in ("default", "eu", "us", "fedramp"), "invalid R2 jurisdiction")
        _require(isinstance(self.bucket, str) and re.fullmatch(r"[a-z0-9][a-z0-9-]{1,61}[a-z0-9]", self.bucket) is not None,
                 "invalid R2 bucket")
        try:
            ipaddress.ip_address(self.bucket)
        except ValueError:
            pass
        else:
            raise GovernanceError("IP bucket alias is forbidden")
        _require(not self.bucket.startswith(("xn--", "sthree-", "amzn-s3-demo-"))
                 and not self.bucket.endswith(("-s3alias", "--ol-s3", "--x-s3", "--table-s3")),
                 "reserved bucket alias")
        _require(_label(self.credential_ref), "invalid nonsecret credential reference")
        _require(self.storage_class == "STANDARD", "R2 archive requires STANDARD")
        _require(type(self.manifest_bytes) is bytes, "manifest must be exact bytes")
        manifest = parse_manifest(self.manifest_bytes, manifest_sha256=self.manifest_sha256,
                                  expected_source=self.expected_source, expected_run=self.expected_run,
                                  expected_attempt=self.expected_attempt)
        object.__setattr__(self, "root", manifest["archive_root"])

    @property
    def endpoint(self) -> str:
        jurisdiction = "" if self.jurisdiction == "default" else self.jurisdiction + "."
        return f"https://{self.account}.{jurisdiction}r2.cloudflarestorage.com"

    @property
    def location(self) -> str:
        return f"r2://{self.account}/{self.jurisdiction}/{self.bucket}"

    @property
    def binding(self) -> tuple:
        return (self.location, self.root, self.manifest_sha256, self.storage_class)

    def validate_key(self, key: str, *, receipt_only: bool = False) -> None:
        _require(isinstance(key, str) and key.isascii() and len(key) <= 1024
                 and key.startswith(self.root), "key does not match bound archive root")
        suffix = key[len(self.root):]
        pattern = r"receipts/[0-9a-f]{64}\.json"
        if not receipt_only:
            pattern = rf"(?:{pattern}|objects/sha256/[0-9a-f]{{64}}|manifest\.json)"
        _require(re.fullmatch(pattern, suffix) is not None, "unsafe or unsupported archive key")


@dataclass(frozen=True)
class PolicyObservation:
    """Exact sanitized transport envelope and strict raw JSON response bytes."""

    envelope: bytes

    def parse(self, target: R2Target, operation: str) -> dict:
        value = _shape(load_json_bytes_strict(self.envelope, source="R2 observation"), {
            "account", "jurisdiction", "bucket", "archive_root", "source_commit", "run_id",
            "run_attempt", "endpoint", "method", "path", "operation", "headers", "observed_at",
            "status", "request_id", "authority_ref", "raw_response", "response_sha256",
        })
        bucket_path = f"/accounts/{target.account}/r2/buckets/{target.bucket}"
        paths = {"metadata": bucket_path, "locks": bucket_path + "/lock",
                 "lifecycle": f"/{target.bucket}?lifecycle"}
        _require(operation in paths, "unsupported configuration observation")
        expected = {
            "account": target.account, "jurisdiction": target.jurisdiction, "bucket": target.bucket,
            "archive_root": target.root, "source_commit": target.expected_source,
            "run_id": target.expected_run, "run_attempt": target.expected_attempt,
            "endpoint": target.endpoint if operation == "lifecycle" else "https://api.cloudflare.com/client/v4",
            "method": "GET", "path": paths[operation], "operation": operation,
            "headers": ({"cf-r2-jurisdiction": target.jurisdiction}
                        if operation != "lifecycle" and target.jurisdiction != "default" else {}),
        }
        _require(all(type(value[k]) is type(v) and value[k] == v for k, v in expected.items()),
                 "configuration observation target/request mismatch")
        _require(_label(value["authority_ref"]) and value["authority_ref"] != target.credential_ref,
                 "configuration authority must be separate from producer reference")
        _require(value["request_id"] is None or _label(value["request_id"]), "invalid request identity")
        try:
            timestamp = value["observed_at"]
            _require(isinstance(timestamp, str) and timestamp.endswith("Z"), "observation requires UTC time")
            datetime.fromisoformat(timestamp)
        except ValueError:
            raise GovernanceError("invalid observation UTC time") from None
        _require(type(value["status"]) is int and value["status"] in (200, 404), "observation request failed")
        _require(isinstance(value["raw_response"], str), "missing raw response bytes")
        raw = value["raw_response"].encode("utf-8")
        _require(sha256_bytes(raw) == value["response_sha256"], "observation response digest mismatch")
        response = load_json_bytes_strict(raw, source="R2 policy response")
        if operation == "lifecycle":
            response = _shape(response, {"ResponseMetadata"}, {"Rules", "Error"})
            metadata = _shape(response["ResponseMetadata"], {"HTTPStatusCode"}, {"RequestId", "HTTPHeaders", "RetryAttempts"})
            _require(type(metadata["HTTPStatusCode"]) is int and metadata["HTTPStatusCode"] == value["status"],
                     "lifecycle status mismatch")
            _require(metadata.get("RequestId") == value["request_id"], "lifecycle request identity mismatch")
            if value["status"] == 404:
                error = _shape(response.get("Error"), {"Code"}, {"Message"})
                _require(error["Code"] == "NoSuchLifecycleConfiguration" and "Rules" not in response,
                         "lifecycle is unavailable, not empty")
                return {"Rules": []}
            _require("Error" not in response and "Rules" in response, "missing lifecycle rules")
            return response
        _require(value["status"] == 200, "control-plane request failed")
        response = _shape(response, {"success", "errors", "messages", "result"}, {"result_info"})
        _require(response["success"] is True and response["errors"] == [] and isinstance(response["messages"], list),
                 "control-plane observation failed")
        return response["result"]


def _prefix(value: Any) -> str:
    _require(isinstance(value, str) and value.isascii() and len(value) <= 1024
             and "\\" not in value and "%" not in value and not value.startswith("/")
             and all(ord(c) >= 32 for c in value)
             and all(p not in (".", "..") for p in value.split("/")), "unsupported policy prefix")
    return value


def _locks(result: Any, root: str) -> None:
    result = _shape(result, {"rules"})
    _require(isinstance(result["rules"], list), "missing lock rules")
    covered = False
    seen = set()
    for rule in result["rules"]:
        rule = _shape(rule, {"id", "enabled", "condition"}, {"prefix"})
        _require(_label(rule["id"]) and rule["id"] not in seen and type(rule["enabled"]) is bool,
                 "invalid lock identity/enabled state")
        seen.add(rule["id"])
        prefix = _prefix(rule.get("prefix", ""))
        condition = _shape(rule["condition"], {"type"}, {"maxAgeSeconds", "date"})
        kind = condition["type"]
        if kind == "Indefinite":
            _shape(condition, {"type"})
        elif kind == "Age":
            _shape(condition, {"type", "maxAgeSeconds"})
            _require(type(condition["maxAgeSeconds"]) is int and condition["maxAgeSeconds"] > 0, "invalid lock age")
        elif kind == "Date":
            _shape(condition, {"type", "date"})
            _require(isinstance(condition["date"], str), "invalid lock date")
            try:
                datetime.fromisoformat(condition["date"])
            except ValueError:
                raise GovernanceError("invalid lock date") from None
        else:
            raise GovernanceError("unknown lock condition")
        covered |= rule["enabled"] and kind == "Indefinite" and root.startswith(prefix)
    _require(covered, "indefinite lock does not cover complete archive root")


def _lifecycle(result: dict, root: str) -> None:
    rules = result["Rules"]
    _require(isinstance(rules, list), "invalid lifecycle rules")
    actions = {"Expiration", "Transitions", "NoncurrentVersionExpiration", "NoncurrentVersionTransitions"}
    seen = set()
    for rule in rules:
        rule = _shape(rule, {"Status"}, {"ID", "Prefix", "Filter", "AbortIncompleteMultipartUpload"} | actions)
        _require(rule["Status"] in ("Enabled", "Disabled"), "unknown lifecycle status")
        if "ID" in rule:
            _require(_label(rule["ID"]) and rule["ID"] not in seen, "invalid lifecycle identity")
            seen.add(rule["ID"])
        _require(("Prefix" in rule) != ("Filter" in rule), "missing or ambiguous lifecycle filter")
        if "Filter" in rule:
            filter_value = _shape(rule["Filter"], set(), {"Prefix"})
            prefix = _prefix(filter_value.get("Prefix", ""))
        else:
            prefix = _prefix(rule["Prefix"])
        _require(bool(actions.intersection(rule) or "AbortIncompleteMultipartUpload" in rule), "missing lifecycle action")
        if "AbortIncompleteMultipartUpload" in rule:
            abort = _shape(rule["AbortIncompleteMultipartUpload"], {"DaysAfterInitiation"})
            _require(type(abort["DaysAfterInitiation"]) is int and abort["DaysAfterInitiation"] > 0,
                     "invalid multipart-abort lifecycle")
        for action in actions.intersection(rule):
            records = rule[action] if action.endswith("Transitions") else [rule[action]]
            _require(isinstance(records, list) and bool(records), "invalid lifecycle action")
            for record in records:
                if action.startswith("Noncurrent"):
                    required = {"NoncurrentDays"} | ({"StorageClass"} if action.endswith("Transitions") else set())
                    record = _shape(record, required)
                    _require(type(record["NoncurrentDays"]) is int and record["NoncurrentDays"] >= 0,
                             "invalid noncurrent lifecycle age")
                else:
                    record = _shape(record, set(), {"Days", "Date", "StorageClass"})
                    _require(("Days" in record) != ("Date" in record), "missing lifecycle age/date")
                    _require(("StorageClass" in record) == action.endswith("Transitions"), "invalid lifecycle class action")
                    if "Days" in record:
                        _require(type(record["Days"]) is int and record["Days"] >= 0, "invalid lifecycle age")
                    else:
                        _require(isinstance(record["Date"], str), "invalid lifecycle date")
                        try:
                            datetime.fromisoformat(record["Date"])
                        except ValueError:
                            raise GovernanceError("invalid lifecycle date") from None
                if "StorageClass" in record:
                    _require(record["StorageClass"] in ("STANDARD", "STANDARD_IA"), "unknown lifecycle class")
        overlap = root.startswith(prefix) or prefix.startswith(root)
        _require(not (rule["Status"] == "Enabled" and overlap and actions.intersection(rule)),
                 "completed archive objects have active expiry or class transition")


@dataclass(frozen=True)
class R2Admission:
    target_binding: tuple
    observations: tuple[PolicyObservation, PolicyObservation, PolicyObservation]
    assurance: str = "supplied-policy-observations-only"


def admit_configuration(target: R2Target, *, metadata: PolicyObservation,
                        locks: PolicyObservation, lifecycle: PolicyObservation) -> R2Admission:
    """Admit exact supplied observations, never assert live protection or custody."""
    result = _shape(metadata.parse(target, "metadata"), {"name", "jurisdiction", "storageClass", "creationDate"})
    _require(result["name"] == target.bucket and result["jurisdiction"] == target.jurisdiction
             and result["storageClass"] == "Standard", "bucket metadata/Standard selection mismatch")
    _require(isinstance(result["creationDate"], str), "missing bucket creation date")
    try:
        datetime.fromisoformat(result["creationDate"])
    except ValueError:
        raise GovernanceError("invalid bucket creation date") from None
    _locks(locks.parse(target, "locks"), target.root)
    _lifecycle(lifecycle.parse(target, "lifecycle"), target.root)
    return R2Admission(target.binding, (metadata, locks, lifecycle))
