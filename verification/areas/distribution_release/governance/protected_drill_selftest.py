"""Credential-free protected stable-publication and incident drill."""

from __future__ import annotations

import argparse
import copy
import os
from collections.abc import Callable
from pathlib import Path

from .common import (
    GovernanceError,
    PRODUCTION_CREDENTIAL_NAMES,
    write_canonical_json,
)
from .incident_recovery_selftest import (
    test_concurrency_and_credential_boundaries,
    test_first_ga_incident_roll_forward,
    test_rollback_burns_generation_and_resumes,
    test_site_timeout_resumes_without_second_index_mutation,
)
from .protected_drill_evidence import (
    GOVERNED_SCENARIO_TESTS,
    validate_drill_evidence,
)
from .stable_planner_selftest import (
    test_cli_producer,
    test_direct_transition_defenses,
    test_fail_closed_identity_and_transition,
    test_ga_activation,
    test_normal_successor,
)


def test_evidence_contract() -> None:
    evidence: dict[str, object] = {
        "schema_version": 2,
        "environment": "stable-release-drill",
        "external_network": "blocked",
        "production_credentials": "absent",
        "scenarios": [
            {
                "name": "rollback",
                "tests": [
                    test_rollback_burns_generation_and_resumes.__name__,
                    test_site_timeout_resumes_without_second_index_mutation.__name__,
                    test_concurrency_and_credential_boundaries.__name__,
                ],
            }
        ],
        "status": "pass",
    }
    validate_drill_evidence(evidence, expected_scenarios=("rollback",))
    mutations = (
        lambda value: value.update(
            {"schema_version": value["schema_version"] - 1}
        ),
        lambda value: value.update({"environment": "stable-release"}),
        lambda value: value.update({"external_network": "available"}),
        lambda value: value.update({"production_credentials": "present"}),
        lambda value: value.update({"status": "failed"}),
        lambda value: value["scenarios"].append(copy.deepcopy(value["scenarios"][0])),
        lambda value: value["scenarios"][0]["tests"].pop(),
    )
    for mutate in mutations:
        changed = copy.deepcopy(evidence)
        mutate(changed)
        try:
            validate_drill_evidence(changed)
        except GovernanceError:
            pass
        else:
            raise AssertionError("protected drill evidence mutation passed")


SCENARIOS: dict[str, tuple[Callable[[], None], ...]] = {
    "publication": (
        test_ga_activation,
        test_normal_successor,
        test_fail_closed_identity_and_transition,
        test_direct_transition_defenses,
        test_cli_producer,
        test_evidence_contract,
    ),
    "rollback": (
        test_rollback_burns_generation_and_resumes,
        test_site_timeout_resumes_without_second_index_mutation,
        test_concurrency_and_credential_boundaries,
    ),
    "first-ga": (
        test_first_ga_incident_roll_forward,
        test_concurrency_and_credential_boundaries,
    ),
}

if {
    name: tuple(test.__name__ for test in tests)
    for name, tests in SCENARIOS.items()
} != GOVERNED_SCENARIO_TESTS:
    raise AssertionError("protected drill runner drifted from its evidence contract")


def run_drill(scenario: str, *, report_path: Path | None = None) -> int:
    if scenario not in {*SCENARIOS, "all"}:
        raise GovernanceError(f"unsupported protected drill scenario: {scenario}")
    present = sorted(
        name for name in PRODUCTION_CREDENTIAL_NAMES if os.environ.get(name)
    )
    if present:
        raise GovernanceError(
            "protected drill refuses production credential(s): " + ", ".join(present)
        )
    selected = (
        tuple(SCENARIOS)
        if scenario == "all"
        else (scenario,)
    )
    for name in selected:
        for test in SCENARIOS[name]:
            test()
            print(f"protected-drill pass: scenario={name} test={test.__name__}")
    if report_path is not None:
        report = {
            "schema_version": 2,
            "environment": "stable-release-drill",
            "external_network": "blocked",
            "production_credentials": "absent",
            "scenarios": [
                {
                    "name": name,
                    "tests": [test.__name__ for test in SCENARIOS[name]],
                }
                for name in selected
            ],
            "status": "pass",
        }
        validate_drill_evidence(report, expected_scenarios=selected)
        write_canonical_json(
            report_path,
            report,
            refuse_existing=True,
        )
    print(f"protected stable release drill ok: scenarios={','.join(selected)}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--scenario",
        choices=(*SCENARIOS, "all"),
        default="all",
    )
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    try:
        return run_drill(args.scenario, report_path=args.report)
    except GovernanceError as exc:
        print(f"protected stable release drill: {exc}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
