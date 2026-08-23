from __future__ import annotations

import json
from pathlib import Path

from env import require_canonical_python
from runtime_test_runner import run_exact_runtime_tests, validate_expected_tests

AREA_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = AREA_ROOT.parents[2]
REPORT_PATH = (
    REPO_ROOT / "target/verification/areas/python_interop/arrow-runtime.latest.json"
)
EXPECTED_TESTS = {
    "python::arrow_ops::abi::tests::rejects_invalid_device_metadata",
    "python::arrow_ops::abi::tests::rejects_missing_release_and_inconsistent_pairs",
    "python::arrow_ops::abi::tests::rejects_misaligned_payloads_and_reserved_device_types",
    "python::arrow_ops::tests::arrow_array_stream_schema_track_metadata_and_release",
    "python::arrow_ops::tests::arrow_declaration_certification_requires_exact_producer_identity",
    "python::arrow_ops::tests::arrow_marks_pandas_like_producers_copy_possible",
    "python::arrow_ops::tests::arrow_omitted_schema_calls_the_protocol_without_an_argument",
    "python::arrow_ops::tests::arrow_owned_argument_reconciles_full_partial_and_failed_consumption",
    "python::arrow_ops::tests::arrow_owned_argument_proxy_is_one_shot",
    "python::arrow_ops::tests::arrow_rejects_capsules_without_destructors",
    "python::arrow_ops::tests::arrow_rejects_malformed_capsule_and_double_release",
    "python::arrow_ops::tests::arrow_requested_schema_consumes_the_schema_capsule",
    "python::arrow_ops::tests::real_pyarrow_requested_schema_is_a_one_shot_transfer",
}


def main() -> int:
    version = require_canonical_python(AREA_ROOT)
    validate_expected_tests("Arrow", EXPECTED_TESTS)
    runtime = run_exact_runtime_tests(
        REPO_ROOT,
        EXPECTED_TESTS,
        "python::arrow_ops",
        extra_env={"SIFR_ARROW_REAL_PRODUCER_TEST": "1"},
    )
    payload = {
        "schema_version": 1,
        "suite": "arrow-runtime",
        "status": runtime["status"],
        "python_version": version,
        **runtime,
    }
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(
        json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8"
    )
    print(
        f"python interop arrow-runtime {payload['status']}: "
        f"tests={runtime['passed']} report={REPORT_PATH.relative_to(REPO_ROOT)}"
    )
    return 0 if runtime["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
