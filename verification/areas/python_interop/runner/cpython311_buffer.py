from __future__ import annotations

import json
import os
import platform
import subprocess
import sys
from pathlib import Path

from buffer_examples import BUFFER_EXAMPLE_CASES, build_buffer_examples_report
from env import RunnerPaths, cargo_env_for_repo_manifest


AREA_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = AREA_ROOT.parents[2]
COMPATIBILITY_ROOT = AREA_ROOT / "cpython311"
REPORT_PATH = REPO_ROOT / "target/verification/areas/python_interop/buffer-cpython311.latest.json"
EXPECTED_RUNTIME_TESTS = {
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_admission_conflict_releases_rejected_view_exactly_once",
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_automatic_resource_drop_releases_exactly_once",
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_explicit_release_is_exact_once_and_pointer_identical",
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_store_failure_rolls_back_exactly_once",
    "python::buffer_ops::release_evidence_tests::instrumented_exporter_validation_failure_rolls_back_exactly_once",
}


def main() -> int:
    version = platform.python_version()
    if sys.version_info[:2] != (3, 11):
        raise SystemExit(f"buffer CPython compatibility requires 3.11, found {version}")

    run_compatibility_self_tests()
    os.environ["CARGO_TARGET_DIR"] = str(REPO_ROOT / "target/cpython311")
    runtime = run_runtime_release_tests()
    paths = RunnerPaths(
        repo_root=REPO_ROOT,
        area_root=COMPATIBILITY_ROOT,
        packages_root=AREA_ROOT / "packages",
        fixtures_root=AREA_ROOT / "fixtures",
        reports_root=AREA_ROOT / "reports",
    )
    examples = build_buffer_examples_report(paths)
    status = (
        "compatibility-passed"
        if runtime["status"] == "passed" and compiled_examples_are_complete(examples)
        else "compatibility-failed"
    )
    payload = {
        "schema_version": 1,
        "suite": "buffer-cpython311",
        "status": status,
        "cpython_version": version,
        "runtime_release_tests": runtime,
        "compiled_examples": examples,
    }
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(
        f"python interop buffer-cpython311 {status}: "
        f"runtime={runtime['passed']} examples={len(examples['cases'])} "
        f"report={REPORT_PATH.relative_to(REPO_ROOT)}"
    )
    return 0 if status == "compatibility-passed" else 1


def run_runtime_release_tests() -> dict[str, object]:
    env = cargo_env_for_repo_manifest(REPO_ROOT)
    interpreter_root = str(Path(sys.executable).resolve().parent)
    env["PATH"] = interpreter_root + os.pathsep + env.get("PATH", "")
    env["PYO3_PYTHON"] = sys.executable
    command = [
        "cargo",
        "test",
        "-p",
        "sifr_runtime",
        "--features",
        "python",
        "buffer_ops::release_evidence_tests::",
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
    observed_tests = observed_runtime_tests(proc.stdout)
    exact_tests_observed = observed_tests == EXPECTED_RUNTIME_TESTS
    return {
        "status": "passed" if proc.returncode == 0 and exact_tests_observed else "failed",
        "passed": len(observed_tests.intersection(EXPECTED_RUNTIME_TESTS)),
        "expected": len(EXPECTED_RUNTIME_TESTS),
        "observed_tests": sorted(observed_tests),
        "command": " ".join(command),
        "exit_code": proc.returncode,
    }


def observed_runtime_tests(output: str) -> set[str]:
    return {
        line.removeprefix("test ").removesuffix(" ... ok")
        for line in output.splitlines()
        if line.startswith("test ") and line.endswith(" ... ok")
    }


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


def run_compatibility_self_tests() -> None:
    if not EXPECTED_RUNTIME_TESTS:
        raise SystemExit("buffer CPython compatibility runtime test registry is empty")
    if observed_runtime_tests("test result: ok. 0 passed; 0 failed"):
        raise SystemExit("buffer CPython compatibility accepted a zero-test runtime result")
    exact_output = "\n".join(f"test {name} ... ok" for name in EXPECTED_RUNTIME_TESTS)
    if observed_runtime_tests(exact_output) != EXPECTED_RUNTIME_TESTS:
        raise SystemExit("buffer CPython compatibility rejected the exact runtime test set")
    if compiled_examples_are_complete({"status": "examples-passed", "cases": []}):
        raise SystemExit("buffer CPython compatibility accepted an empty compiled result set")


if __name__ == "__main__":
    raise SystemExit(main())
