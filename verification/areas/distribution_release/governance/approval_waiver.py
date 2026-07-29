"""Validation for the temporary single-maintainer publication approval waiver."""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from .common import (
    fail,
    require_array,
    require_enum,
    require_exact_keys,
    require_nonempty_string,
    require_object,
    require_sha256,
)

DISTINCT_REVIEWER = "distinct-reviewer"
SINGLE_MAINTAINER_WAIVER = "single-maintainer-waiver"
APPROVAL_MODES = {DISTINCT_REVIEWER, SINGLE_MAINTAINER_WAIVER}
WAIVED_OPERATIONS = {"bootstrap-alpha", "bootstrap-index", "ga-activation"}


def validate_single_maintainer_waiver(
    payload: Any,
    *,
    repository: str,
    environment: str,
    operation: str | None,
    initiator: str,
    require_unexpired: bool = False,
    now: datetime | None = None,
) -> dict[str, Any]:
    waiver = require_object(payload, "$")
    require_exact_keys(
        waiver,
        required={
            "schema_version",
            "repository",
            "environment",
            "owner_login",
            "allowed_operations",
            "expires_at",
            "reason",
        },
        location="$",
    )
    if waiver["schema_version"] != 2:
        fail("$.schema_version", "must be integer 2")
    if waiver["repository"] != repository:
        fail("$.repository", "does not match the publication repository")
    if waiver["environment"] != environment:
        fail("$.environment", "does not match the protected environment")
    owner = require_nonempty_string(waiver["owner_login"], "$.owner_login")
    if owner.casefold() != initiator.casefold():
        fail("$.owner_login", "must equal the workflow initiator")
    operations = require_array(waiver["allowed_operations"], "$.allowed_operations")
    if not operations:
        fail("$.allowed_operations", "must be non-empty")
    normalized_operations: list[str] = []
    for index, value in enumerate(operations):
        normalized_operations.append(
            require_enum(
                value,
                WAIVED_OPERATIONS,
                f"$.allowed_operations[{index}]",
            )
        )
    if len(set(normalized_operations)) != len(normalized_operations):
        fail("$.allowed_operations", "must contain unique operations")
    if operation is not None and operation not in normalized_operations:
        fail("$.allowed_operations", f"does not authorize {operation}")
    expires_at = require_nonempty_string(waiver["expires_at"], "$.expires_at")
    try:
        expiry = datetime.fromisoformat(expires_at.replace("Z", "+00:00"))
    except ValueError:
        fail("$.expires_at", "must be an ISO-8601 timestamp")
    if expiry.tzinfo is None:
        fail("$.expires_at", "must include a timezone")
    if require_unexpired:
        current = now or datetime.now(timezone.utc)
        if current.tzinfo is None:
            fail("$.expires_at", "comparison time must include a timezone")
        if expiry <= current:
            fail("$.expires_at", "single-maintainer approval waiver has expired")
    require_nonempty_string(waiver["reason"], "$.reason")
    return waiver


def validate_approval_policy(payload: Any, location: str) -> dict[str, str]:
    policy = require_object(payload, location)
    require_exact_keys(
        policy,
        required={"mode", "waiver_sha256"},
        location=location,
    )
    mode = require_enum(policy["mode"], APPROVAL_MODES, f"{location}.mode")
    waiver_sha256 = policy["waiver_sha256"]
    if mode == DISTINCT_REVIEWER:
        if waiver_sha256 != "none":
            fail(f"{location}.waiver_sha256", "must be none for distinct review")
    else:
        require_sha256(waiver_sha256, f"{location}.waiver_sha256")
    return {"mode": mode, "waiver_sha256": waiver_sha256}
