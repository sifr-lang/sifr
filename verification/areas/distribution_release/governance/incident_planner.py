"""Pure rollback and incident roll-forward planning over governed index bytes."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .common import (
    GovernanceError,
    load_json_strict,
    require_positive_int,
    require_sha256,
    sha256_file,
)
from .incident import validate_incident_request
from .release_index import (
    propose_incident_roll_forward,
    propose_rollback,
    validate_release_index,
)
from .release_plan import validate_release_plan


@dataclass(frozen=True)
class IncidentMutation:
    """Validated inputs and the one proposed immutable index generation."""

    operation: str
    request: dict[str, Any]
    request_sha256: str
    affected_plan_sha256: str
    successor_plan_sha256: str
    previous_index: dict[str, Any]
    proposed_index: dict[str, Any]
    affected_version: str
    successor_version: str


def materialize_incident_mutation(
    *,
    request_path: Path,
    live_index_path: Path,
    affected_plan_path: Path,
    successor_plan_path: Path,
    expected_generation: int,
    expected_sha256: str,
    proposed_generation: int,
) -> IncidentMutation:
    """Validate exact canonical evidence and return a deterministic mutation."""
    request = validate_incident_request(_load_canonical(request_path))
    live_index = validate_release_index(_load_canonical(live_index_path))
    affected_plan = validate_release_plan(_load_canonical(affected_plan_path))
    successor_plan = validate_release_plan(_load_canonical(successor_plan_path))
    request_sha256 = sha256_file(request_path)
    live_sha256 = sha256_file(live_index_path)
    affected_plan_sha256 = sha256_file(affected_plan_path)
    successor_plan_sha256 = sha256_file(successor_plan_path)
    require_positive_int(expected_generation, "expected_generation")
    require_sha256(expected_sha256, "expected_sha256")
    if live_index["generation"] != expected_generation:
        raise GovernanceError("live index generation does not match expected_generation")
    if live_sha256 != expected_sha256:
        raise GovernanceError("live index bytes do not match expected_sha256")

    affected_ref = request.get("affected_release")
    affected_version = affected_ref.get("version") if isinstance(affected_ref, dict) else ""
    if affected_plan.get("version") != affected_version:
        raise GovernanceError("affected plan version does not match the incident request")
    if affected_plan_sha256 != affected_ref.get("plan_sha256"):
        raise GovernanceError("affected plan bytes do not match the incident request")
    live_release = live_index["releases"].get(affected_version)
    if live_release != affected_plan.get("desired_release"):
        raise GovernanceError("affected plan release bytes do not match the live index")

    operation = request.get("operation")
    approved = {affected_version: affected_plan_sha256}
    if operation == "rollback":
        target_ref = request.get("rollback_target")
        target_version = target_ref.get("version") if isinstance(target_ref, dict) else ""
        approved[target_version] = successor_plan_sha256
        validate_incident_request(
            request,
            live_index=live_index,
            approved_plan_digests=approved,
        )
        _validate_rollback_plans(
            affected_plan=affected_plan,
            target_plan=successor_plan,
            target_version=target_version,
            target_plan_sha256=successor_plan_sha256,
            live_index=live_index,
        )
        proposed = propose_rollback(
            live_index,
            incident_id=request["incident_id"],
            affected_version=affected_version,
            target_version=target_version,
            proposed_generation=proposed_generation,
        )
        successor_version = target_version
    elif operation == "incident-roll-forward":
        validate_incident_request(
            request,
            live_index=live_index,
            approved_plan_digests=approved,
        )
        _validate_roll_forward_plan(
            plan=successor_plan,
            request_sha256=request_sha256,
            affected_version=affected_version,
            affected_plan_sha256=affected_plan_sha256,
            live_index=live_index,
        )
        successor_version = successor_plan["version"]
        proposed = propose_incident_roll_forward(
            live_index,
            incident_id=request["incident_id"],
            affected_version=affected_version,
            successor_version=successor_version,
            successor_release=successor_plan["desired_release"],
            proposed_generation=proposed_generation,
        )
    else:
        validate_incident_request(request)
        raise AssertionError("validated incident operation was not handled")

    return IncidentMutation(
        operation=operation,
        request=request,
        request_sha256=request_sha256,
        affected_plan_sha256=affected_plan_sha256,
        successor_plan_sha256=successor_plan_sha256,
        previous_index=live_index,
        proposed_index=proposed,
        affected_version=affected_version,
        successor_version=successor_version,
    )


def _validate_rollback_plans(
    *,
    affected_plan: dict[str, Any],
    target_plan: dict[str, Any],
    target_version: str,
    target_plan_sha256: str,
    live_index: dict[str, Any],
) -> None:
    if affected_plan["transition"] != "normal":
        raise GovernanceError("rollback requires an affected normal release plan")
    expected = {"version": target_version, "plan_sha256": target_plan_sha256}
    if affected_plan["rollback_target"] != expected:
        raise GovernanceError("affected plan does not authorize the requested rollback target")
    predecessor = affected_plan["expected_stable_predecessor"]
    if (
        not isinstance(predecessor, dict)
        or predecessor.get("version") != target_version
        or predecessor.get("plan_sha256") != target_plan_sha256
    ):
        raise GovernanceError("affected plan predecessor does not authorize rollback")
    if target_plan["version"] != target_version:
        raise GovernanceError("rollback target plan version does not match the request")
    if live_index["releases"].get(target_version) != target_plan["desired_release"]:
        raise GovernanceError("rollback target plan release bytes do not match the retained index")


def _validate_roll_forward_plan(
    *,
    plan: dict[str, Any],
    request_sha256: str,
    affected_version: str,
    affected_plan_sha256: str,
    live_index: dict[str, Any],
) -> None:
    validate_release_plan(
        plan,
        active_index=live_index,
        incident_request_sha256=request_sha256,
    )
    if plan["transition"] != "incident-roll-forward":
        raise GovernanceError("successor plan must use incident-roll-forward")
    if plan["rollback_target"] != "none":
        raise GovernanceError("incident roll-forward successor must have rollback_target none")
    expected = plan["expected_stable_predecessor"]
    if (
        not isinstance(expected, dict)
        or expected.get("version") != affected_version
        or expected.get("plan_sha256") != affected_plan_sha256
    ):
        raise GovernanceError("successor plan does not bind the affected release plan")


def _load_canonical(path: Path) -> dict[str, Any]:
    value = load_json_strict(path, require_canonical=True)
    if not isinstance(value, dict):
        raise GovernanceError(f"{path}: expected a JSON object")
    return value
