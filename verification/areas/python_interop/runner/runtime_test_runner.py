from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from env import cargo_env_for_repo_manifest


def observed_tests(output: str) -> list[str]:
    return [
        line.removeprefix("test ").removesuffix(" ... ok")
        for line in output.splitlines()
        if line.startswith("test ") and line.endswith(" ... ok")
    ]


def exact_tests_observed(observed: list[str], expected: set[str]) -> bool:
    return len(observed) == len(expected) and set(observed) == expected


def validate_expected_tests(label: str, expected: set[str]) -> None:
    if not expected:
        raise SystemExit(f"{label} runtime test registry is empty")
    exact_output = "\n".join(f"test {name} ... ok" for name in expected)
    if not exact_tests_observed(observed_tests(exact_output), expected):
        raise SystemExit(f"{label} runtime runner rejected its exact test set")
    missing_output = "\n".join(exact_output.splitlines()[:-1])
    if exact_tests_observed(observed_tests(missing_output), expected):
        raise SystemExit(f"{label} runtime runner accepted a missing test")
    duplicate_output = f"{exact_output}\n{exact_output.splitlines()[0]}"
    if exact_tests_observed(observed_tests(duplicate_output), expected):
        raise SystemExit(f"{label} runtime runner accepted a duplicate test")


def run_exact_runtime_tests(
    repo_root: Path,
    expected: set[str],
    test_filter: str,
    *,
    extra_env: dict[str, str] | None = None,
) -> dict[str, object]:
    env = cargo_env_for_repo_manifest(repo_root)
    env["CARGO_TARGET_DIR"] = str(repo_root / "target/python")
    env["PYO3_PYTHON"] = sys.executable
    env["PATH"] = str(Path(sys.executable).parent) + os.pathsep + env.get("PATH", "")
    if extra_env is not None:
        env.update(extra_env)
    command = [
        "cargo",
        "test",
        "-p",
        "sifr_runtime",
        "--features",
        "python",
        test_filter,
    ]
    process = subprocess.run(
        command,
        cwd=repo_root,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if process.stdout:
        sys.stdout.write(process.stdout)
    if process.stderr:
        sys.stderr.write(process.stderr)
    observed = observed_tests(process.stdout)
    return {
        "status": (
            "passed"
            if process.returncode == 0 and exact_tests_observed(observed, expected)
            else "failed"
        ),
        "passed": len(set(observed).intersection(expected)),
        "expected": len(expected),
        "observed_tests": sorted(observed),
        "command": " ".join(command),
        "exit_code": process.returncode,
    }
