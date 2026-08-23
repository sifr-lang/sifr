from __future__ import annotations

import json
from pathlib import Path

from env import require_canonical_python
from runtime_test_runner import run_exact_runtime_tests, validate_expected_tests

AREA_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = AREA_ROOT.parents[2]
REPORT_PATH = (
    REPO_ROOT / "target/verification/areas/python_interop/dlpack-runtime.latest.json"
)
EXPECTED_TESTS = {
    "python::dlpack_ops::abi::tests::metadata_accepts_null_deleter_and_preserves_offset",
    "python::dlpack_ops::abi::tests::metadata_allows_null_data_for_an_empty_tensor",
    "python::dlpack_ops::abi::tests::metadata_rejects_invalid_dimensions_and_pointers",
    "python::dlpack_ops::abi::tests::metadata_rejects_shape_stride_dtype_and_offset_drift",
    "python::dlpack_ops::declaration_tests::acquisition_uses_full_signature_once_without_legacy_retry",
    "python::dlpack_ops::declaration_tests::attach_failure_leaves_the_deleter_with_the_capsule_owner",
    "python::dlpack_ops::declaration_tests::capsule_device_mismatch_releases_the_acquired_tensor",
    "python::dlpack_ops::declaration_tests::consumed_argument_transfers_deleter_ownership_exactly_once",
    "python::dlpack_ops::declaration_tests::cuda_and_any_require_a_matching_explicit_stream",
    "python::dlpack_ops::declaration_tests::normalized_stream_metadata_is_closed_and_checked",
    "python::dlpack_ops::declaration_tests::test_runtime_reset_releases_an_outstanding_tensor_once",
    "python::dlpack_ops::declaration_tests::unconsumed_argument_releases_once_during_reconciliation",
    "python::dlpack_ops::declaration_tests::versioned_capsule_is_accepted_and_released_once",
    "python::dlpack_ops::declaration_tests::versioned_capsule_rejects_incompatible_major_version_without_leaking",
    "python::dlpack_ops::declaration_tests::versioned_copied_flag_is_rejected_without_leaking",
    "python::dlpack_ops::tests::dlpack_cpu_tensor_tracks_metadata_and_release",
    "python::dlpack_ops::tests::dlpack_rejects_double_consumption_and_double_release",
    "python::dlpack_ops::tests::dlpack_rejects_invalid_capsule_name_dtype_and_device",
    "python::dlpack_ops::tests::dlpack_scalar_tensor_allows_null_shape_pointer",
}


def main() -> int:
    version = require_canonical_python(AREA_ROOT)
    validate_expected_tests("DLPack", EXPECTED_TESTS)
    runtime = run_exact_runtime_tests(REPO_ROOT, EXPECTED_TESTS, "python::dlpack_ops")
    payload = {
        "schema_version": 1,
        "suite": "dlpack-runtime",
        "status": runtime["status"],
        "python_version": version,
        **runtime,
    }
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(
        json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8"
    )
    print(
        f"python interop dlpack-runtime {payload['status']}: "
        f"tests={runtime['passed']} report={REPORT_PATH.relative_to(REPO_ROOT)}"
    )
    return 0 if runtime["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
