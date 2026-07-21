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
REPORT_PATH = REPO_ROOT / "target/verification/areas/python_interop/arrow-cpython311.latest.json"
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
        raise SystemExit("Arrow CPython 3.11 runner rejected its exact test set")
    if exact_tests_observed(observed_tests("\n".join(exact.splitlines()[:-1]))):
        raise SystemExit("Arrow CPython 3.11 runner accepted a missing test")


def main() -> int:
    version = platform.python_version()
    if sys.version_info[:2] != (3, 11):
        raise SystemExit(f"Arrow CPython compatibility requires 3.11, found {version}")
    self_test()
    env = cargo_env_for_repo_manifest(REPO_ROOT)
    env["CARGO_TARGET_DIR"] = str(REPO_ROOT / "target/cpython311")
    env["PYO3_PYTHON"] = sys.executable
    env["PATH"] = str(Path(sys.executable).parent) + os.pathsep + env.get("PATH", "")
    env["SIFR_ARROW_REAL_PRODUCER_TEST"] = "1"
    command = [
        "cargo",
        "test",
        "-p",
        "sifr_runtime",
        "--features",
        "python",
        "python::arrow_ops",
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
        "suite": "arrow-cpython311",
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
        f"python interop arrow-cpython311 {payload['status']}: "
        f"tests={len(observed)} report={REPORT_PATH.relative_to(REPO_ROOT)}"
    )
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
