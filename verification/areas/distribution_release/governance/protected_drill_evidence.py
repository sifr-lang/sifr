"""Semantic contract for credential-free protected release-drill evidence."""

from __future__ import annotations

from .common import (
    fail,
    require_array,
    require_exact_keys,
    require_nonempty_string,
    require_object,
    require_schema_v2,
)

GOVERNED_SCENARIO_TESTS: dict[str, tuple[str, ...]] = {
    "publication": (
        "test_ga_activation",
        "test_normal_successor",
        "test_fail_closed_identity_and_transition",
        "test_direct_transition_defenses",
        "test_cli_producer",
        "test_evidence_contract",
    ),
    "rollback": (
        "test_rollback_burns_generation_and_resumes",
        "test_site_timeout_resumes_without_second_index_mutation",
        "test_concurrency_and_credential_boundaries",
    ),
    "first-ga": (
        "test_first_ga_incident_roll_forward",
        "test_concurrency_and_credential_boundaries",
    ),
}


def validate_drill_evidence(
    payload: object,
    *,
    expected_scenarios: tuple[str, ...] | None = None,
) -> dict[str, object]:
    """Validate semantic invariants beyond the checked-in JSON Schema."""
    evidence = require_object(payload, "$")
    require_exact_keys(
        evidence,
        required={
            "schema_version",
            "environment",
            "external_network",
            "production_credentials",
            "scenarios",
            "status",
        },
        location="$",
    )
    require_schema_v2(evidence)
    if evidence["environment"] != "stable-release-drill":
        fail("$.environment", "must be stable-release-drill")
    if evidence["external_network"] != "blocked":
        fail("$.external_network", "must record the blocked network boundary")
    if evidence["production_credentials"] != "absent":
        fail("$.production_credentials", "must record absent production credentials")
    if evidence["status"] != "pass":
        fail("$.status", "completed drill evidence must pass")
    rows = require_array(evidence["scenarios"], "$.scenarios")
    if not rows:
        fail("$.scenarios", "must contain at least one drill scenario")
    names: list[str] = []
    for index, value in enumerate(rows):
        location = f"$.scenarios[{index}]"
        row = require_object(value, location)
        require_exact_keys(row, required={"name", "tests"}, location=location)
        name = require_nonempty_string(row["name"], f"{location}.name")
        if name not in GOVERNED_SCENARIO_TESTS or name in names:
            fail(f"{location}.name", "must be a unique governed drill scenario")
        tests = require_array(row["tests"], f"{location}.tests")
        for test_index, test in enumerate(tests):
            require_nonempty_string(test, f"{location}.tests[{test_index}]")
        if tests != list(GOVERNED_SCENARIO_TESTS[name]):
            fail(f"{location}.tests", "must equal the governed scenario test order")
        names.append(name)
    if expected_scenarios is not None and names != list(expected_scenarios):
        fail("$.scenarios", "does not equal the requested drill scenarios")
    return evidence
