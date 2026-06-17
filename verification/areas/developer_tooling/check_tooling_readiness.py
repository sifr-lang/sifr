#!/usr/bin/env python3
"""Validate developer tooling wiring and performance-policy evidence."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
PROFILE_RUNNER = REPO_ROOT / "verification" / "runner" / "sifr_verify" / "profile_runner.py"
TOOLING_VERIFICATION_DOC = REPO_ROOT / "internal_docs" / "tooling_verification.md"
REUSE_DOC = REPO_ROOT / "internal_docs" / "tooling_reuse_strategy.md"
TOOLING_ROOT = REPO_ROOT / "verification" / "areas" / "developer_tooling"
TOOLING_RUNNER = TOOLING_ROOT / "runner.py"
PERF_ROOT = REPO_ROOT / "verification" / "areas" / "performance"
PERF_DATA = PERF_ROOT / "data"
PERF_MANIFEST = PERF_ROOT / "manifest.json"

REQUIRED_TOOLING_CHECKS = [
    "check_tooling_rules_lock.py",
    "check_tooling_dependency_boundaries.py",
    "check_lsp_split_brain.py",
    "check_linter_diagnostic_class.py",
    "check_vscode_extension_rules.py",
    "check_vscode_extension.py",
    "check_formatter_rules.py",
    "check_rule_suppression_rules.py",
    "check_analysis_snapshot_rules.py",
    "check_analysis_snapshot_coherence.py",
    "check_analysis_split_brain.py",
    "run_tooling_parity.py",
    "check_completion_quality.py",
    "lsp_protocol_smoke.py",
    "lsp_protocol_stress.py",
    "check_editor_assets.py",
    "check_tooling_readiness.py",
]

REQUIRED_PERFORMANCE_CHECKS = [
    "run_benchmarks.py",
    "check_budgets.py",
]

def load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ReadinessError(f"expected JSON object at {path.relative_to(REPO_ROOT)}")
    return payload


class ReadinessError(Exception):
    pass


def validate_profile_runner_wiring(runner_text: str) -> list[str]:
    failures: list[str] = []
    for suite_name in [
        "typescript-go-transfer",
        "diagnostic-rules",
    ]:
        if suite_name not in runner_text:
            failures.append(f"developer_tooling suite {suite_name} is not wired into profile_runner.py")
    if "tooling_suites" not in runner_text:
        failures.append("developer_tooling profile suite dispatch is not wired into profile_runner.py")
    if '"developer_tooling"' not in runner_text:
        failures.append("developer_tooling area is not wired into profile_runner.py")
    if '"performance"' not in runner_text or "performance_budget_mode" not in runner_text:
        failures.append("performance profile suite dispatch is not wired into profile_runner.py")
    return failures


def validate_area_suite_wiring(tooling_runner_text: str, perf_manifest_text: str) -> list[str]:
    failures: list[str] = []
    for script_name in REQUIRED_TOOLING_CHECKS:
        if script_name not in tooling_runner_text:
            failures.append(f"{script_name} is not wired into developer_tooling area runner")
        elif f'"{script_name}"), "--self-test"' not in tooling_runner_text:
            failures.append(f"{script_name} self-test is not wired into developer_tooling area runner")
    for script_name in REQUIRED_PERFORMANCE_CHECKS:
        if script_name not in perf_manifest_text:
            failures.append(f"{script_name} is not wired into performance area manifest")
    if "benchmark-self-test" not in perf_manifest_text:
        failures.append("run_benchmarks.py self-test is not wired into performance area manifest")
    if "budget-self-test" not in perf_manifest_text:
        failures.append("check_budgets.py self-test is not wired into performance area manifest")
    return failures


def validate_required_files() -> list[str]:
    failures: list[str] = []
    for script_name in REQUIRED_TOOLING_CHECKS:
        path = TOOLING_ROOT / script_name
        if not path.is_file():
            failures.append(f"required tooling check missing: {path.relative_to(REPO_ROOT)}")
    for script_name in REQUIRED_PERFORMANCE_CHECKS:
        path = PERF_ROOT / script_name
        if not path.is_file():
            failures.append(f"required performance check missing: {path.relative_to(REPO_ROOT)}")
    for path in [TOOLING_VERIFICATION_DOC, REUSE_DOC]:
        if not path.is_file():
            failures.append(f"required tooling rules doc missing: {path.relative_to(REPO_ROOT)}")
    return failures


def validate_tracking_docs() -> list[str]:
    failures: list[str] = []
    tooling_text = TOOLING_VERIFICATION_DOC.read_text(encoding="utf-8")
    for required in [
        "check_analysis_snapshot_coherence.py",
        "check_completion_quality.py",
        "check_tooling_readiness.py",
        "scripts/run_all_tests.sh --profile create-pr",
        "scripts/run_all_tests.sh --profile merge",
    ]:
        if required not in tooling_text:
            failures.append(f"tooling verification doc missing required evidence marker: {required}")
    return failures


def validate_lsp_performance_policy() -> list[str]:
    failures: list[str] = []
    manifest = load_json(PERF_DATA / "benchmark_manifest.json")
    budgets = load_json(PERF_DATA / "budgets.json")
    waivers = load_json(PERF_DATA / "waivers.json")
    matrix = load_json(TOOLING_ROOT / "lsp_protocol_matrix.json")
    budget_doc = (PERF_ROOT / "lsp_query_budget_ids.md").read_text(encoding="utf-8")

    lsp_cases = [case for case in manifest.get("cases", []) if isinstance(case, dict) and case.get("group") == "lsp-query"]
    if not lsp_cases:
        failures.append("performance manifest must include at least one lsp-query case")
    if not any(case.get("id") == "lsp-query-001-request-families" for case in lsp_cases):
        failures.append("performance manifest missing lsp-query-001-request-families")
    budget_entries = {
        entry.get("benchmark_id"): entry
        for entry in budgets.get("budgets", [])
        if isinstance(entry, dict)
    }
    if "lsp-query-001-request-families" not in budget_entries:
        failures.append("budgets missing lsp-query-001-request-families")
    if any(waiver for waiver in waivers.get("waivers", []) if isinstance(waiver, dict) and "lsp" in str(waiver)):
        failures.append("developer tooling readiness must not carry active LSP budget waivers")

    matrix_budget_ids = {
        item.get("budget_id")
        for item in matrix.get("required_methods", [])
        if isinstance(item, dict) and item.get("budget_id")
    }
    for budget_id in sorted(matrix_budget_ids):
        if f"`{budget_id}`" not in budget_doc:
            failures.append(f"LSP budget coverage doc missing matrix budget label {budget_id}")
    if "`perf.lsp.request_families`" not in budget_doc:
        failures.append("LSP budget coverage doc missing perf.lsp.request_families")
    return failures


def validate() -> list[str]:
    failures = validate_required_files()
    if failures:
        return failures
    failures.extend(validate_profile_runner_wiring(PROFILE_RUNNER.read_text(encoding="utf-8")))
    failures.extend(
        validate_area_suite_wiring(
            TOOLING_RUNNER.read_text(encoding="utf-8"),
            PERF_MANIFEST.read_text(encoding="utf-8"),
        )
    )
    failures.extend(validate_tracking_docs())
    failures.extend(validate_lsp_performance_policy())
    return failures


def run_self_test() -> None:
    runner_text = PROFILE_RUNNER.read_text(encoding="utf-8")
    bad_text = runner_text.replace('"developer_tooling"', '"missing_developer_tooling"')
    failures = validate_profile_runner_wiring(bad_text)
    if not any("developer_tooling area" in failure for failure in failures):
        raise SystemExit("tooling readiness self-test failed: missing developer_tooling area wiring passed")

    tooling_runner_text = TOOLING_RUNNER.read_text(encoding="utf-8")
    bad_tooling_runner_text = tooling_runner_text.replace("check_completion_quality.py", "missing_completion_quality.py")
    failures = validate_area_suite_wiring(bad_tooling_runner_text, PERF_MANIFEST.read_text(encoding="utf-8"))
    if not any("check_completion_quality.py" in failure for failure in failures):
        raise SystemExit("tooling readiness self-test failed: missing area runner wiring passed")

    manifest = load_json(PERF_DATA / "benchmark_manifest.json")
    bad_manifest = copy.deepcopy(manifest)
    bad_manifest["cases"] = [
        case
        for case in bad_manifest.get("cases", [])
        if not (isinstance(case, dict) and case.get("id") == "lsp-query-001-request-families")
    ]
    if any(case.get("id") == "lsp-query-001-request-families" for case in bad_manifest.get("cases", [])):
        raise SystemExit("tooling readiness self-test setup failed")
    print("tooling readiness self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = validate()
    if failures:
        print("tooling readiness: FAIL")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("tooling readiness: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
