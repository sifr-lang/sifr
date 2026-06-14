"""Local prerequisite checks for verification profiles."""

from __future__ import annotations

import json
import os
import platform
import shutil
import subprocess
import sys
from dataclasses import dataclass

from .errors import VerificationError
from .paths import REPO_ROOT

MIN_PYTHON = (3, 11)


class DoctorError(VerificationError):
    """Local prerequisite validation failed."""


@dataclass(frozen=True)
class CheckResult:
    name: str
    status: str
    detail: str


def run_command(command: list[str], *, env: dict[str, str] | None = None) -> tuple[int, str]:
    proc = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    return proc.returncode, proc.stdout.strip()


def run_doctor() -> int:
    results = [
        check_python_version(),
        check_tool("cargo", ["cargo", "--version"]),
        check_tool("rustc", ["rustc", "--version"]),
        check_tool("uv", ["uv", "--version"]),
        check_uv_lock(),
        check_cargo_offline_metadata(),
        check_sanitizer_tools(),
        check_host_metadata(),
    ]
    payload = {
        "schema_version": 1,
        "host": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
            "python": platform.python_version(),
        },
        "checks": [result.__dict__ for result in results],
    }
    print(json.dumps(payload, indent=2, sort_keys=True))
    failures = [result for result in results if result.status == "fail"]
    if failures:
        names = ", ".join(result.name for result in failures)
        raise DoctorError(f"doctor failed required checks: {names}")
    return 0


def check_python_version() -> CheckResult:
    actual = sys.version_info[:2]
    if actual < MIN_PYTHON:
        return CheckResult(
            "python-version",
            "fail",
            f"requires >= {MIN_PYTHON[0]}.{MIN_PYTHON[1]}, found {platform.python_version()}",
        )
    return CheckResult("python-version", "pass", platform.python_version())


def check_tool(name: str, command: list[str]) -> CheckResult:
    if shutil.which(command[0]) is None:
        return CheckResult(name, "fail", f"{command[0]} not found on PATH")
    code, output = run_command(command)
    status = "pass" if code == 0 else "fail"
    return CheckResult(name, status, output.splitlines()[0] if output else f"exit={code}")


def check_uv_lock() -> CheckResult:
    code, output = run_command(["uv", "lock", "--project", "verification", "--check"])
    return CheckResult("uv-lock", "pass" if code == 0 else "fail", output or f"exit={code}")


def check_cargo_offline_metadata() -> CheckResult:
    code, output = run_command(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        env={**os.environ, "CARGO_NET_OFFLINE": "true"},
    )
    if code == 0:
        return CheckResult("cargo-offline-metadata", "pass", "cargo metadata --locked resolved offline")
    return CheckResult("cargo-offline-metadata", "fail", output or f"exit={code}")


def check_sanitizer_tools() -> CheckResult:
    candidates = ["llvm-symbolizer", "clang", "rustup"]
    found = [tool for tool in candidates if shutil.which(tool) is not None]
    missing = sorted(set(candidates) - set(found))
    if missing:
        return CheckResult(
            "sanitizer-tools",
            "skip",
            "optional sanitizer tools missing for broad lanes: " + ", ".join(missing),
        )
    return CheckResult("sanitizer-tools", "pass", "optional sanitizer tools available")


def check_host_metadata() -> CheckResult:
    host = {
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
    }
    if not host["system"] or not host["machine"]:
        return CheckResult("host-metadata", "fail", "host system or machine is unavailable")
    return CheckResult("host-metadata", "pass", json.dumps(host, sort_keys=True))


if __name__ == "__main__":
    raise SystemExit(run_doctor())
