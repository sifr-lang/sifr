from __future__ import annotations

import json
from pathlib import Path

from env import require_canonical_python
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
    passed = runtime["status"] == "passed"
    payload = {
        "schema_version": 1,
        "suite": "buffer-runtime",
        "status": "passed" if passed else "failed",
        "python_version": version,
        "runtime_release_tests": runtime,
    }
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(
        json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8"
    )
    print(
        f"python interop buffer-runtime {payload['status']}: "
        f"runtime={runtime['passed']} "
        f"report={REPORT_PATH.relative_to(REPO_ROOT)}"
    )
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
