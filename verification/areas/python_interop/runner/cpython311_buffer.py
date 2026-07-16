from __future__ import annotations

import json
import os
import platform
import subprocess
import sys
from pathlib import Path

from buffer_examples import build_buffer_examples_report
from env import RunnerPaths, cargo_env_for_repo_manifest


AREA_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = AREA_ROOT.parents[2]
COMPATIBILITY_ROOT = AREA_ROOT / "cpython311"
REPORT_PATH = REPO_ROOT / "target/verification/areas/python_interop/buffer-cpython311.latest.json"


def main() -> int:
    version = platform.python_version()
    if sys.version_info[:2] != (3, 11):
        raise SystemExit(f"buffer CPython compatibility requires 3.11, found {version}")

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
        if runtime["status"] == "passed" and examples["status"] == "examples-passed"
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
    return {
        "status": "passed" if proc.returncode == 0 else "failed",
        "passed": 5 if proc.returncode == 0 else 0,
        "expected": 5,
        "command": " ".join(command),
        "exit_code": proc.returncode,
    }


if __name__ == "__main__":
    raise SystemExit(main())
