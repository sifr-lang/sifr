from __future__ import annotations

import json
import os
import platform
import subprocess
import sys
from pathlib import Path

from env import cargo_env_for_repo_manifest


AREA_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = AREA_ROOT.parents[2]
REPORT_PATH = REPO_ROOT / "target/verification/areas/python_interop/dlpack-cpython311.latest.json"
EXPECTED_TESTS = {
    "python::dlpack_ops::abi::tests::metadata_accepts_null_deleter_and_preserves_offset",
    "python::dlpack_ops::abi::tests::metadata_allows_null_data_for_an_empty_tensor",
    "python::dlpack_ops::abi::tests::metadata_rejects_invalid_dimensions_and_pointers",
    "python::dlpack_ops::abi::tests::metadata_rejects_shape_stride_dtype_and_offset_drift",
    "python::dlpack_ops::declaration_tests::acquisition_uses_full_signature_once_without_legacy_retry",
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


def observed_tests(output: str) -> list[str]:
    return [
        line.removeprefix("test ").removesuffix(" ... ok")
        for line in output.splitlines()
        if line.startswith("test ") and line.endswith(" ... ok")
    ]


def exact_tests_observed(observed: list[str]) -> bool:
    return len(observed) == len(EXPECTED_TESTS) and set(observed) == EXPECTED_TESTS


def self_test() -> None:
    exact = "\n".join(f"test {name} ... ok" for name in EXPECTED_TESTS)
    if not exact_tests_observed(observed_tests(exact)):
        raise SystemExit("DLPack CPython 3.11 runner rejected its exact test set")
    if exact_tests_observed(observed_tests("\n".join(exact.splitlines()[:-1]))):
        raise SystemExit("DLPack CPython 3.11 runner accepted a missing test")


def main() -> int:
    version = platform.python_version()
    if sys.version_info[:2] != (3, 11):
        raise SystemExit(f"DLPack CPython compatibility requires 3.11, found {version}")
    self_test()
    env = cargo_env_for_repo_manifest(REPO_ROOT)
    env["CARGO_TARGET_DIR"] = str(REPO_ROOT / "target/cpython311")
    env["PYO3_PYTHON"] = sys.executable
    env["PATH"] = str(Path(sys.executable).parent) + os.pathsep + env.get("PATH", "")
    command = [
        "cargo",
        "test",
        "-p",
        "sifr_runtime",
        "--features",
        "python",
        "python::dlpack_ops",
    ]
    proc = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.stdout:
        sys.stdout.write(proc.stdout)
    if proc.stderr:
        sys.stderr.write(proc.stderr)
    observed = observed_tests(proc.stdout)
    passed = proc.returncode == 0 and exact_tests_observed(observed)
    payload = {
        "schema_version": 1,
        "suite": "dlpack-cpython311",
        "status": "compatibility-passed" if passed else "compatibility-failed",
        "cpython_version": version,
        "expected": len(EXPECTED_TESTS),
        "observed_tests": sorted(observed),
        "command": " ".join(command),
        "exit_code": proc.returncode,
    }
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(
        f"python interop dlpack-cpython311 {payload['status']}: "
        f"tests={len(observed)} report={REPORT_PATH.relative_to(REPO_ROOT)}"
    )
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
