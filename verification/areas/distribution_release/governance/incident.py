"""Stable incident request and protected sign-off contracts."""

from __future__ import annotations

from typing import Any

from .common import (
    fail,
    require_array,
    require_enum,
    require_exact_keys,
    require_incident_id,
    require_nonempty_string,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    version_channel,
)
from .release_index import validate_release_index

REQUEST_REQUIRED = {
    "schema_version",
    "incident_id",
    "operation",
    "trigger",
    "affected_release",
    "withdrawal",
}
SIGNOFF_REQUIRED = {
    "schema_version",
    "incident_id",
    "operation",
    "request_sha256",
    "attempts",
    "index_mutation",
    "site_reconciliation",
    "validation",
    "communications",
    "closure",
}


def validate_incident_request(
    payload: Any,
    *,
    live_index: Any | None = None,
    approved_plan_digests: dict[str, str] | None = None,
) -> dict[str, Any]:
    request = require_object(payload, "$")
    operation = request.get("operation")
    required = set(REQUEST_REQUIRED)
    if operation == "rollback":
        required.add("rollback_target")
    require_exact_keys(
        request,
        required=required,
        location="$",
    )
    require_schema_v2(request)
    require_incident_id(request["incident_id"], "$.incident_id")
    operation = require_enum(
        operation,
        {"rollback", "incident-roll-forward"},
        "$.operation",
    )
    require_nonempty_string(request["trigger"], "$.trigger")

    affected = require_object(request["affected_release"], "$.affected_release")
    require_exact_keys(affected, required={"version", "plan_sha256"}, location="$.affected_release")
    if version_channel(affected["version"], "$.affected_release.version") != "stable":
        fail("$.affected_release.version", "must be a stable version")
    require_sha256(affected["plan_sha256"], "$.affected_release.plan_sha256")
    if (
        approved_plan_digests is not None
        and approved_plan_digests.get(affected["version"]) != affected["plan_sha256"]
    ):
        fail("$.affected_release.plan_sha256", "does not match the approved release plan")

    withdrawal = require_object(request["withdrawal"], "$.withdrawal")
    require_exact_keys(
        withdrawal,
        required={"reason", "evidence_sha256"},
        location="$.withdrawal",
    )
    require_nonempty_string(withdrawal["reason"], "$.withdrawal.reason")
    require_sha256(withdrawal["evidence_sha256"], "$.withdrawal.evidence_sha256")

    if operation == "rollback":
        target = require_object(request["rollback_target"], "$.rollback_target")
        require_exact_keys(target, required={"version", "plan_sha256"}, location="$.rollback_target")
        if version_channel(target["version"], "$.rollback_target.version") != "stable":
            fail("$.rollback_target.version", "must be a stable version")
        require_sha256(target["plan_sha256"], "$.rollback_target.plan_sha256")
        if (
            approved_plan_digests is not None
            and approved_plan_digests.get(target["version"]) != target["plan_sha256"]
        ):
            fail("$.rollback_target.plan_sha256", "does not match the approved release plan")
        if target["version"] == affected["version"]:
            fail("$.rollback_target.version", "must differ from the affected version")

    if live_index is not None:
        index = validate_release_index(live_index)
        if index["ga_status"] != "active" or index["channels"].get("stable") != affected["version"]:
            fail("$.affected_release.version", "is not the live expected stable predecessor")
        if operation == "rollback":
            target_release = index["releases"].get(request["rollback_target"]["version"])
            if not isinstance(target_release, dict) or target_release.get("status") != "active":
                fail("$.rollback_target.version", "must name an active release in the live index")
    return request


def validate_incident_signoff(
    payload: Any,
    *,
    incident_request: Any | None = None,
) -> dict[str, Any]:
    signoff = require_object(payload, "$")
    require_exact_keys(signoff, required=SIGNOFF_REQUIRED, location="$")
    require_schema_v2(signoff)
    require_incident_id(signoff["incident_id"], "$.incident_id")
    operation = require_enum(
        signoff["operation"],
        {"rollback", "incident-roll-forward"},
        "$.operation",
    )
    require_sha256(signoff["request_sha256"], "$.request_sha256")
    validate_attempts(signoff["attempts"], "$.attempts")
    attempts = signoff["attempts"]
    run_ids = [attempt["run_id"] for attempt in attempts]
    if run_ids != sorted(set(run_ids)):
        fail("$.attempts", "run_id values must be unique and strictly increasing")
    if any(attempt["status"] == "started" for attempt in attempts):
        fail("$.attempts", "completed sign-off cannot contain a pending attempt")
    completed_attempts = sum(attempt["status"] == "completed" for attempt in attempts)
    if completed_attempts != 1 or attempts[-1]["status"] != "completed":
        fail("$.attempts", "must contain exactly one final completed attempt")

    mutation = require_object(signoff["index_mutation"], "$.index_mutation")
    require_exact_keys(
        mutation,
        required={
            "previous_generation",
            "previous_sha256",
            "realized_generation",
            "realized_sha256",
            "affected_version",
            "successor_version",
        },
        location="$.index_mutation",
    )
    previous = require_positive_int(mutation["previous_generation"], "$.index_mutation.previous_generation")
    realized = require_positive_int(mutation["realized_generation"], "$.index_mutation.realized_generation")
    if realized <= previous:
        fail("$.index_mutation.realized_generation", "must exceed previous_generation")
    require_sha256(mutation["previous_sha256"], "$.index_mutation.previous_sha256")
    require_sha256(mutation["realized_sha256"], "$.index_mutation.realized_sha256")
    version_channel(mutation["affected_version"], "$.index_mutation.affected_version")
    version_channel(mutation["successor_version"], "$.index_mutation.successor_version")

    for name in ("site_reconciliation", "validation", "communications", "closure"):
        evidence = require_object(signoff[name], f"$.{name}")
        require_exact_keys(
            evidence,
            required={"status", "evidence_sha256"},
            location=f"$.{name}",
        )
        if evidence["status"] != "pass":
            fail(f"$.{name}.status", "must be pass")
        require_sha256(evidence["evidence_sha256"], f"$.{name}.evidence_sha256")
    if incident_request is not None:
        request = validate_incident_request(incident_request)
        if request["incident_id"] != signoff["incident_id"]:
            fail("$.incident_id", "does not match the incident request")
        if request["operation"] != operation:
            fail("$.operation", "does not match the incident request")
    return signoff


def validate_attempts(payload: Any, location: str) -> None:
    attempts = require_array(payload, location)
    if not attempts:
        fail(location, "must contain at least one protected attempt")
    for index, value in enumerate(attempts):
        attempt_location = f"{location}[{index}]"
        attempt = require_object(value, attempt_location)
        require_exact_keys(
            attempt,
            required={"run_id", "mode", "approver", "status", "mutations"},
            location=attempt_location,
        )
        require_positive_int(attempt["run_id"], f"{attempt_location}.run_id")
        require_enum(
            attempt["mode"],
            {"initial", "resume"},
            f"{attempt_location}.mode",
        )
        require_nonempty_string(attempt["approver"], f"{attempt_location}.approver")
        require_enum(
            attempt["status"],
            {"started", "failed", "completed"},
            f"{attempt_location}.status",
        )
        mutations = require_array(attempt["mutations"], f"{attempt_location}.mutations")
        if not mutations:
            fail(f"{attempt_location}.mutations", "must contain mutation evidence")
        for mutation_index, mutation_value in enumerate(mutations):
            mutation_location = f"{attempt_location}.mutations[{mutation_index}]"
            mutation = require_object(mutation_value, mutation_location)
            require_exact_keys(
                mutation,
                required={"kind", "identity", "sha256"},
                location=mutation_location,
            )
            require_nonempty_string(mutation["kind"], f"{mutation_location}.kind")
            require_nonempty_string(mutation["identity"], f"{mutation_location}.identity")
            require_sha256(mutation["sha256"], f"{mutation_location}.sha256")
