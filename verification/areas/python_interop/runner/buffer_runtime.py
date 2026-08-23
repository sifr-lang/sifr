from __future__ import annotations

import json
from pathlib import Path

from buffer_examples import BUFFER_EXAMPLE_CASES, build_buffer_examples_report
from env import RunnerPaths, require_canonical_python
from runtime_test_runner import run_exact_runtime_tests, validate_expected_tests

AREA_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = AREA_ROOT.parents[2]
REPORT_PATH = (
    REPO_ROOT / "target/verification/areas/python_interop/buffer-runtime.latest.json"
)
EXPECTED_RUNTIME_TESTS = {
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_admission_conflict_releases_rejected_view_exactly_once",
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_automatic_resource_drop_releases_exactly_once",
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_explicit_release_is_exact_once_and_pointer_identical",
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_store_failure_rolls_back_exactly_once",
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_validation_failure_rolls_back_exactly_once",
}


def main() -> int:
    version = require_canonical_python(AREA_ROOT)
    validate_expected_tests("buffer", EXPECTED_RUNTIME_TESTS)
    runtime = run_exact_runtime_tests(
        REPO_ROOT,
        EXPECTED_RUNTIME_TESTS,
        "buffer_ops::release_evidence_tests::",
    )
    paths = RunnerPaths(
        repo_root=REPO_ROOT,
        area_root=AREA_ROOT,
        packages_root=AREA_ROOT / "packages",
        fixtures_root=AREA_ROOT / "fixtures",
        reports_root=AREA_ROOT / "reports",
    )
    examples = build_buffer_examples_report(paths)
    passed = runtime["status"] == "passed" and compiled_examples_are_complete(examples)
    payload = {
        "schema_version": 1,
        "suite": "buffer-runtime",
        "status": "passed" if passed else "failed",
        "python_version": version,
        "runtime_release_tests": runtime,
        "compiled_examples": examples,
    }
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(
        json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8"
    )
    print(
        f"python interop buffer-runtime {payload['status']}: "
        f"runtime={runtime['passed']} examples={len(examples['cases'])} "
        f"report={REPORT_PATH.relative_to(REPO_ROOT)}"
    )
    return 0 if passed else 1


def compiled_examples_are_complete(examples: dict[str, object]) -> bool:
    cases = examples.get("cases")
    if not isinstance(cases, list):
        return False
    observed_ids = [case.get("id") for case in cases if isinstance(case, dict)]
    return (
        examples.get("status") == "examples-passed"
        and len(observed_ids) == len(cases)
        and len(observed_ids) == len(set(observed_ids))
        and set(observed_ids) == set(BUFFER_EXAMPLE_CASES)
        and all(case.get("status") == "example-passed" for case in cases)
    )


if __name__ == "__main__":
    raise SystemExit(main())
