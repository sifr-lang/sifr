#!/usr/bin/env python3
"""Negative self-tests for coverage-matrix readiness enforcement."""

from __future__ import annotations

import json
import tempfile
from contextlib import redirect_stderr, redirect_stdout
from datetime import date
from io import StringIO
from pathlib import Path
from typing import Any, Callable

import coverage_matrix
import profile_assignment_matrix
from sifr_verify.profiles import compare_plans


OWNERS = {
    "algorithmic/compatibility",
    "compiler-verification",
    "compiler/codegen",
    "compiler/core-language",
    "compiler/diagnostics",
    "compiler/frontend",
    "compiler/hardening",
    "compiler/package-management",
    "compiler/performance",
    "compiler/tooling",
    "compiler/verification",
    "developer-tooling",
    "release/distribution",
    "runtime/platform",
    "stdlib/parity",
}


def main() -> int:
    tests: list[tuple[str, Callable[[], list[str]], str]] = [
        ("stable guarantee with no matrix row", stable_guarantee_without_matrix_row, "stable guarantee has no matrix row"),
        ("owner unassigned", owner_unassigned, "owner must not be unassigned"),
        ("unknown owner id", owner_unknown, "unknown owner"),
        ("expired expected-missing", expired_expected_missing, "expiry has passed"),
        ("expired tests:none", expired_tests_none, "expiry has passed"),
        ("lingering red-blocker", lingering_red_blocker, "status red-blocker is illegal in strict readiness mode"),
        ("illegal stable not-applicable", illegal_stable_not_applicable, "not-applicable is illegal"),
        ("undocumented quarantine", undocumented_quarantine, "missing or invalid reproduction_command"),
        ("expired quarantine", expired_quarantine, "expiry has passed"),
        ("v1 stable-surface manifest", v1_stable_manifest, "schema_version 2"),
        ("required corpus missing checksum", required_corpus_missing_checksum, "pinned corpus requires revision and checksum"),
        ("live-network create-pr profile", live_network_profile, "live network is forbidden"),
        ("create-pr cargo not locked/offline", unlocked_cargo_profile, "locked and offline"),
        ("CI plan omitting local merge suite", ci_plan_omits_local_merge_suite, "profile plan mismatch"),
        ("profile assignment mismatch", profile_assignment_mismatch, "omits required suite"),
        ("release suite drift", release_suite_drift, "does not match release profile assignment"),
        ("undeclared release divergence", undeclared_release_divergence, "diverges from nightly without release_suite"),
        ("profile-derived release divergence", profile_derived_release_divergence, "profile-derived release coverage diverges"),
        ("expired release divergence", expired_release_divergence, "release divergence expiry has passed"),
        ("missing release divergence record", missing_release_divergence_record, "record is not indexed"),
        ("redundant release suite", redundant_release_suite, "without a profile assignment divergence"),
        ("equal release surface suite", equal_release_surface_suite, "must differ from nightly_release_suite"),
        ("orphaned release divergence metadata", orphaned_release_divergence_metadata, "metadata requires release_suite"),
        ("first-party crate without membership", missing_crate_membership, "missing merge crate-test membership"),
        ("SQL component primary target kind", wrong_component_target_kind, "target lacks classification: rlib:sql_component"),
        ("stale SQL component target kind", wrong_component_target_kind, "stale target classification: lib:sql_component"),
        ("SQL integration target omitted", missing_sql_test_target, "target lacks classification: test:runtime_types"),
    ]
    failed: list[str] = []
    for name, func, expected in tests:
        errors = func()
        if not any(expected in error for error in errors):
            failed.append(f"{name}: expected error containing {expected!r}, got {errors!r}")
    if failed:
        for failure in failed:
            print(f"coverage matrix self-test: {failure}")
        return 1
    print(f"coverage matrix readiness self-tests ok: cases={len(tests)}")
    return 0


def stable_guarantee_without_matrix_row() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_guarantees(
        [
            {
                "guarantee_id": "stable-without-row",
                "support_status": "stable",
                "owner": "compiler-verification",
                "public_doc_path": "internal",
                "merge_surface": "missing",
                "nightly_release_surface": "missing",
                "regression_surface": "regression",
            }
        ],
        [],
        OWNERS,
        errors,
    )
    return errors


def owner_unassigned() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_owner("unassigned", OWNERS, "test.owner", errors)
    return errors


def owner_unknown() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_owner("unknown/team", OWNERS, "test.owner", errors)
    return errors


def expired_expected_missing() -> list[str]:
    return validate_single_surface("expected-missing", {"resolution_gate": "9"})


def expired_tests_none() -> list[str]:
    return validate_single_surface("tests:none")


def lingering_red_blocker() -> list[str]:
    return validate_single_surface(
        "red-blocker",
        {
            "resolution_gate": "9",
            "command": "cargo test -p sifr_codegen",
            "triage_file": "plans/issues/active/codegen-test-triage.md",
            "current_failure_count": 1,
        },
    )


def illegal_stable_not_applicable() -> list[str]:
    return validate_single_surface("not-applicable")


def undocumented_quarantine() -> list[str]:
    return validate_single_surface("quarantined", omit_reproduction=True)


def expired_quarantine() -> list[str]:
    return validate_single_surface("quarantined")


def validate_single_surface(
    status: str,
    extra: dict[str, Any] | None = None,
    *,
    omit_reproduction: bool = False,
) -> list[str]:
    errors: list[str] = []
    row = {
        "surface_id": f"surface-{status}",
        "guarantee_id": "stable-guarantee",
        "owner": "compiler-verification",
        "status": status,
        "merge_suite": "coverage_matrix:readiness",
        "nightly_release_suite": "coverage_matrix:readiness",
        "regression_suite": "regression:fixedbugs",
        "reproduction_command": "uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix --suite readiness",
        "issue": "compiler-codebase-verification-readiness",
        "expiry": "2000-01-01",
    }
    if omit_reproduction:
        row.pop("reproduction_command")
    if extra:
        row.update(extra)
    coverage_matrix.validate_surfaces(
        [row],
        [{"guarantee_id": "stable-guarantee", "support_status": "stable"}],
        OWNERS,
        True,
        errors,
    )
    return errors


def v1_stable_manifest() -> list[str]:
    return validate_manifest_policy({"schema_version": 1, "name": "tmp_area", "owner": "compiler-verification"})


def required_corpus_missing_checksum() -> list[str]:
    return validate_manifest_policy(
        {
            "schema_version": 2,
            "name": "tmp_area",
            "owner": "compiler-verification",
            "network_mode": "offline",
            "pinned_corpus": {"required": True, "revision": "local"},
        }
    )


def validate_manifest_policy(manifest_payload: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        area = root / "tmp_area"
        area.mkdir()
        (area / "manifest.json").write_text(json.dumps(manifest_payload), encoding="utf-8")
        previous = coverage_matrix.AREAS_DIR
        coverage_matrix.AREAS_DIR = root
        try:
            coverage_matrix.validate_readiness_area_manifest_policy(
                [{"merge_suite": "tmp_area:case", "nightly_release_suite": "tmp_area:case", "regression_suite": "tmp_area:case"}],
                True,
                errors,
            )
        finally:
            coverage_matrix.AREAS_DIR = previous
    return errors


def live_network_profile() -> list[str]:
    profile = minimal_profile()
    profile["network_policy"] = {"mode": "offline", "live_network_allowed": True}
    errors: list[str] = []
    coverage_matrix.validate_profile_readiness_policy("create-pr", profile, errors)
    return errors


def unlocked_cargo_profile() -> list[str]:
    profile = minimal_profile()
    profile["cargo_policy"] = {"locked": False, "offline": True}
    errors: list[str] = []
    coverage_matrix.validate_profile_readiness_policy("create-pr", profile, errors)
    return errors


def minimal_profile() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "network_policy": {"mode": "offline", "live_network_allowed": False},
        "cargo_policy": {"locked": True, "offline": True},
        "selected_areas": [{"area": "coverage_matrix", "suites": ["readiness"]}],
        "toolchain_steps": [],
        "guardrail_steps": [],
    }


def ci_plan_omits_local_merge_suite() -> list[str]:
    errors: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        local = root / "local.json"
        ci = root / "ci.json"
        base_plan = {
            "schema_version": 2,
            "profile": "merge",
            "selected_areas": [{"area": "coverage_matrix", "suites": ["readiness"]}],
            "toolchain_steps": [],
            "guardrail_steps": [],
            "crate_test_membership": {},
            "e2e": {},
            "network_policy": {"mode": "offline"},
            "cargo_policy": {"locked": True, "offline": True},
            "reference_host": {},
            "execution_sandbox": {},
        }
        local.write_text(json.dumps(base_plan), encoding="utf-8")
        ci_plan = dict(base_plan)
        ci_plan["selected_areas"] = []
        ci.write_text(json.dumps(ci_plan), encoding="utf-8")
        stdout = StringIO()
        stderr = StringIO()
        with redirect_stdout(stdout), redirect_stderr(stderr):
            result = compare_plans(str(local), str(ci))
        if result != 1:
            errors.append("compare_plans did not report mismatch")
        elif "profile plan mismatch" not in stderr.getvalue():
            errors.append("compare_plans mismatch diagnostic missing")
        else:
            errors.append("profile plan mismatch detected")
    return errors


def profile_assignment_mismatch() -> list[str]:
    errors: list[str] = []
    profile_assignment_matrix.validate_expected_tokens(
        "parser_acceptance_rejection",
        "merge",
        ["core_language:syntax_parser_lexer_matrix"],
        {"core_language": {"syntax_parser_lexer_matrix"}},
        errors,
    )
    profile_assignment_matrix.validate_row_membership(
        "parser_acceptance_rejection",
        "merge",
        ["core_language:syntax_parser_lexer_matrix"],
        {"core_language:hir_analysis_behaviors"},
        errors,
    )
    return errors


def release_suite_drift() -> list[str]:
    errors: list[str] = []
    profile_assignment_matrix.validate_release_suite_alignment(
        "algorithmic_compatibility_profile",
        ["algorithmic_compatibility:representative-subset"],
        ["algorithmic_compatibility:taxonomy-smoke"],
        errors,
    )
    return errors


def undeclared_release_divergence() -> list[str]:
    errors: list[str] = []
    profile_assignment_matrix.validate_release_divergence_declaration(
        "algorithmic_compatibility_profile",
        ["algorithmic_compatibility:leetcode-full"],
        ["algorithmic_compatibility:representative-subset"],
        False,
        errors,
    )
    return errors


def profile_derived_release_divergence() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_release_surface_profile_policy(
        [
            {
                "surface_id": "algorithmic_compatibility_profile",
                "nightly_release_suite": (
                    "algorithmic_compatibility:leetcode-full,"
                    "algorithmic_compatibility:taxonomy-smoke"
                ),
            }
        ],
        errors,
        nightly_profile={
            "selected_areas": [
                {
                    "area": "algorithmic_compatibility",
                    "suites": ["leetcode-full", "taxonomy-smoke"],
                }
            ]
        },
        release_profile={
            "selected_areas": [
                {
                    "area": "algorithmic_compatibility",
                    "suites": ["representative-subset", "taxonomy-smoke"],
                }
            ]
        },
    )
    return errors


def expired_release_divergence() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_release_divergence(
        {
            "nightly_release_suite": "algorithmic_compatibility:leetcode-full",
            "release_suite": "algorithmic_compatibility:representative-subset",
            "release_divergence_record": "ALG-CORPUS",
            "release_divergence_expiry": "2000-01-01",
        },
        "algorithmic_compatibility_profile",
        date.today(),
        errors,
    )
    return errors


def missing_release_divergence_record() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_release_divergence(
        {
            "nightly_release_suite": "algorithmic_compatibility:leetcode-full",
            "release_suite": "algorithmic_compatibility:representative-subset",
            "release_divergence_record": "MISSING-RECORD",
            "release_divergence_expiry": "2999-01-01",
        },
        "algorithmic_compatibility_profile",
        date.today(),
        errors,
    )
    return errors


def redundant_release_suite() -> list[str]:
    errors: list[str] = []
    profile_assignment_matrix.validate_release_divergence_declaration(
        "algorithmic_compatibility_profile",
        ["algorithmic_compatibility:representative-subset"],
        ["algorithmic_compatibility:representative-subset"],
        True,
        errors,
    )
    return errors


def equal_release_surface_suite() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_release_divergence(
        {
            "nightly_release_suite": "algorithmic_compatibility:representative-subset",
            "release_suite": "algorithmic_compatibility:representative-subset",
            "release_divergence_record": "ALG-CORPUS",
            "release_divergence_expiry": "2999-01-01",
        },
        "algorithmic_compatibility_profile",
        date.today(),
        errors,
    )
    return errors


def orphaned_release_divergence_metadata() -> list[str]:
    errors: list[str] = []
    row = {
        "surface_id": "algorithmic_compatibility_profile",
        "guarantee_id": "stable-guarantee",
        "owner": "compiler-verification",
        "status": "blocking",
        "merge_suite": "algorithmic_compatibility:representative-subset",
        "nightly_release_suite": "algorithmic_compatibility:leetcode-full",
        "release_divergence_record": "ALG-CORPUS",
        "release_divergence_expiry": "2999-01-01",
        "regression_suite": "regression:fixedbugs",
        "reproduction_command": "sifr check fixture.sifr",
    }
    coverage_matrix.validate_surfaces(
        [row],
        [{"guarantee_id": "stable-guarantee", "support_status": "stable"}],
        OWNERS,
        True,
        errors,
    )
    return errors


def wrong_component_target_kind() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_targets(
        "sql_component",
        {"targets": [{"name": "sql_component", "kind": ["rlib", "cdylib"]}]},
        {"targets": [{"name": "sql_component", "kind": "lib", "classification": "first_party_compiler", "profile_assignment": "merge"}]},
        errors,
    )
    return errors


def missing_sql_test_target() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_targets(
        "sql_runtime",
        {"targets": [{"name": "runtime_types", "kind": ["test"]}]},
        {"targets": []},
        errors,
    )
    return errors


def missing_crate_membership() -> list[str]:
    errors: list[str] = []
    coverage_matrix.validate_first_party_compiler_membership("sifr_missing", [], errors)
    return errors


if __name__ == "__main__":
    raise SystemExit(main())
