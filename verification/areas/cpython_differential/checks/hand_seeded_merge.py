"""Run hand-seeded CPython-vs-Sifr differential smoke programs."""

from __future__ import annotations

import json
import subprocess
import sys
import time
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
MANIFEST = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "cpython_differential"
    / "data"
    / "hand_seeded_manifest.json"
)
PYPROJECT = REPO_ROOT / "verification" / "pyproject.toml"
TIMEOUT_SECONDS = 240


@dataclass(frozen=True)
class RuntimeResult:
    exit_code: int
    stdout: str
    stderr: str
    duration_ms: float
    timed_out: bool = False


def main() -> int:
    failures = run_suite()
    if failures:
        for failure in failures:
            print(f"cpython differential smoke error: {failure}", file=sys.stderr)
        return 1
    return 0


def run_suite() -> list[str]:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    failures = validate_python_version()
    print(f"cpython differential oracle python={sys.version}")
    for case in manifest["cases"]:
        failures.extend(run_case(case))
    if not failures:
        print(f"cpython differential hand-seeded smoke ok: cases={len(manifest['cases'])}")
    return failures


def validate_python_version() -> list[str]:
    pyproject = tomllib.loads(PYPROJECT.read_text(encoding="utf-8"))
    requires_python = pyproject["project"]["requires-python"]
    if requires_python != ">=3.11":
        return [f"unsupported requires-python policy {requires_python!r}; update runner parser"]
    if sys.version_info < (3, 11):
        return [f"python3 must satisfy {requires_python}, got {sys.version.split()[0]}"]
    return []


def run_case(case: dict[str, Any]) -> list[str]:
    case_id = str(case["id"])
    allowed_exit_codes = set(int(code) for code in case["allowed_exit_codes"])
    python_path = REPO_ROOT / str(case["python"])
    sifr_path = REPO_ROOT / str(case["sifr"])
    failures: list[str] = []

    cpython = run_command([sys.executable, str(python_path)])
    sifr = run_command(["cargo", "run", "-q", "-p", "sifr", "--", "run", str(sifr_path)])
    print(
        f"[cpython-differential] case={case_id} "
        f"python_exit={cpython.exit_code} sifr_exit={sifr.exit_code} "
        f"python_ms={cpython.duration_ms:.0f} sifr_ms={sifr.duration_ms:.0f}"
    )

    if cpython.timed_out:
        failures.append(f"{case_id} CPython timed out after {TIMEOUT_SECONDS}s")
    if sifr.timed_out:
        failures.append(f"{case_id} Sifr timed out after {TIMEOUT_SECONDS}s")
    if cpython.exit_code not in allowed_exit_codes:
        failures.append(f"{case_id} CPython exit {cpython.exit_code} not in {sorted(allowed_exit_codes)}")
    if sifr.exit_code not in allowed_exit_codes:
        failures.append(f"{case_id} Sifr exit {sifr.exit_code} not in {sorted(allowed_exit_codes)}")
    if cpython.exit_code != sifr.exit_code:
        failures.append(f"{case_id} exit codes differ: CPython={cpython.exit_code} Sifr={sifr.exit_code}")
    cpython_json = parse_exact_json_line(case_id, "CPython", cpython.stdout, failures)
    sifr_json = parse_exact_json_line(case_id, "Sifr", sifr.stdout, failures)
    if cpython_json is not None and sifr_json is not None and cpython_json != sifr_json:
        failures.append(f"{case_id} JSON values differ: CPython={cpython_json!r} Sifr={sifr_json!r}")
    if normalize_stdout(cpython.stdout) != normalize_stdout(sifr.stdout):
        failures.append(
            f"{case_id} canonical stdout differs: "
            f"CPython={normalize_stdout(cpython.stdout)!r} Sifr={normalize_stdout(sifr.stdout)!r}"
        )
    return failures


def run_command(argv: list[str]) -> RuntimeResult:
    started = time.perf_counter()
    try:
        completed = subprocess.run(
            argv,
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=TIMEOUT_SECONDS,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        stdout = decode_timeout_stream(error.stdout)
        stderr = decode_timeout_stream(error.stderr)
        return RuntimeResult(
            exit_code=124,
            stdout=stdout,
            stderr=stderr,
            duration_ms=(time.perf_counter() - started) * 1000.0,
            timed_out=True,
        )
    return RuntimeResult(
        exit_code=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
        duration_ms=(time.perf_counter() - started) * 1000.0,
    )


def decode_timeout_stream(stream: str | bytes | None) -> str:
    if stream is None:
        return ""
    if isinstance(stream, str):
        return stream
    return stream.decode("utf-8", errors="replace")


def parse_exact_json_line(
    case_id: str,
    runtime: str,
    stdout: str,
    failures: list[str],
) -> Any | None:
    normalized = normalize_stdout(stdout)
    if not normalized.endswith("\n"):
        failures.append(f"{case_id} {runtime} stdout must end with one newline")
        return None
    lines = normalized.splitlines()
    if len(lines) != 1:
        failures.append(f"{case_id} {runtime} stdout must contain exactly one JSON line")
        return None
    try:
        return json.loads(lines[0])
    except json.JSONDecodeError as error:
        failures.append(f"{case_id} {runtime} stdout is not valid JSON: {error}")
        return None


def normalize_stdout(stdout: str) -> str:
    return stdout.replace("\r\n", "\n")


if __name__ == "__main__":
    raise SystemExit(main())
