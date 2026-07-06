"""Run generated CPython-vs-Sifr differential suites."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from generated_programs import generate_program, minimized_candidate

REPO_ROOT = Path(__file__).resolve().parents[4]
MANIFEST = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "cpython_differential"
    / "data"
    / "generated_seed_manifest.json"
)
MINIMIZED_FAILURES = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "cpython_differential"
    / "data"
    / "minimized_failures.json"
)
PYPROJECT = REPO_ROOT / "verification" / "pyproject.toml"
ACTUAL_ROOT = REPO_ROOT / "target" / "verification" / "actual" / "cpython_differential" / "generated"
INTEGER_MIN = -1_000_000
INTEGER_MAX = 1_000_000
MAX_DEPTH = 4


@dataclass(frozen=True)
class RuntimeResult:
    exit_code: int
    stdout: str
    stderr: str
    duration_ms: float
    timed_out: bool = False


def main(suite_name: str) -> int:
    failures = run_suite(suite_name)
    if failures:
        for failure in failures:
            print(f"cpython generated differential error: {failure}", file=sys.stderr)
        return 1
    return 0


def run_suite(suite_name: str) -> list[str]:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    failures = validate_python_version()
    suite = manifest["suites"][suite_name]
    build_info = build_release_binary(manifest, failures)
    if failures:
        return failures
    deadline = time.monotonic() + int(suite["overall_timeout_seconds"])
    minimized_manifest = json.loads(MINIMIZED_FAILURES.read_text(encoding="utf-8"))
    if minimized_manifest.get("schema_version") != 1:
        failures.append("minimized_failures schema_version must be 1")

    print(f"cpython generated oracle python={sys.version}")
    print(
        "cpython generated release binary="
        f"{build_info['binary']} sha256={build_info['binary_sha256']} source_digest={build_info['source_digest']}"
    )
    suite_actual_root = ACTUAL_ROOT / suite_name
    suite_actual_root.mkdir(parents=True, exist_ok=True)
    minimized_failures: list[dict[str, Any]] = []
    for case in suite["cases"]:
        if time.monotonic() >= deadline:
            failures.append(f"{suite_name} exceeded overall timeout {suite['overall_timeout_seconds']}s")
            break
        case_failures = run_case(suite_name, case, suite, build_info, suite_actual_root)
        failures.extend(case_failures)
        if case_failures:
            minimized_failures.append(write_minimized_candidate(suite_name, case, suite_actual_root, case_failures))
    if minimized_failures:
        minimized_path = suite_actual_root / "minimized_failures.actual.json"
        minimized_path.write_text(json.dumps(minimized_failures, indent=2, sort_keys=True), encoding="utf-8")
    if not failures:
        print(f"cpython generated differential ok: suite={suite_name} cases={len(suite['cases'])}")
    return failures


def validate_python_version() -> list[str]:
    if sys.version_info < (3, 11):
        return [f"python3 must be >=3.11 to read verification/pyproject.toml, got {sys.version.split()[0]}"]
    import tomllib

    pyproject = tomllib.loads(PYPROJECT.read_text(encoding="utf-8"))
    requires_python = pyproject["project"]["requires-python"]
    if requires_python != ">=3.11":
        return [f"unsupported requires-python policy {requires_python!r}; update generated runner parser"]
    return []


def build_release_binary(manifest: dict[str, Any], failures: list[str]) -> dict[str, str]:
    release = manifest["release_binary"]
    build_command = [str(part) for part in release["build_command"]]
    timeout = int(release["build_timeout_seconds"])
    env = os.environ.copy()
    env.pop("CARGO_TARGET_DIR", None)
    started = time.perf_counter()
    try:
        completed = subprocess.run(
            build_command,
            cwd=REPO_ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        failures.append(f"release binary build timed out after {timeout}s")
        stdout = decode_timeout_stream(error.stdout)
        stderr = decode_timeout_stream(error.stderr)
        if stdout:
            sys.stdout.write(stdout)
        if stderr:
            sys.stderr.write(stderr)
        return {
            "binary": str(Path(str(release["path"]))),
            "binary_sha256": "timeout",
            "source_digest": source_digest(release["source_digest_inputs"]),
        }
    duration_ms = (time.perf_counter() - started) * 1000.0
    if completed.stdout:
        sys.stdout.write(completed.stdout)
    if completed.stderr:
        sys.stderr.write(completed.stderr)
    if completed.returncode != 0:
        failures.append(f"release binary build failed with exit {completed.returncode}")
    binary = REPO_ROOT / str(release["path"])
    if not binary.is_file():
        failures.append(f"release binary missing after build: {binary.relative_to(REPO_ROOT)}")
        binary_sha = "missing"
    else:
        binary_sha = sha256_file(binary)
    print(f"[cpython-generated] release_build_ms={duration_ms:.0f}")
    return {
        "binary": str(binary.relative_to(REPO_ROOT)),
        "binary_sha256": binary_sha,
        "source_digest": source_digest(release["source_digest_inputs"]),
    }


def run_case(
    suite_name: str,
    case: dict[str, Any],
    suite: dict[str, Any],
    build_info: dict[str, str],
    suite_actual_root: Path,
) -> list[str]:
    case_id = str(case["id"])
    program = generate_program(case)
    case_root = suite_actual_root / case_id
    case_root.mkdir(parents=True, exist_ok=True)
    python_path = case_root / "case.py"
    sifr_path = case_root / "main.sifr"
    metadata_path = case_root / "seed.json"
    python_path.write_text(program.python_source, encoding="utf-8")
    sifr_path.write_text(program.sifr_source, encoding="utf-8")
    metadata_path.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "case": case,
                "generated_shape": program.grammar_shape,
                "parameters": program.parameters,
                "release_binary": build_info,
            },
            indent=2,
            sort_keys=True,
        ),
        encoding="utf-8",
    )
    timeout = int(suite["per_program_timeout_seconds"])
    cpython = run_command([sys.executable, str(python_path)], timeout)
    sifr = run_command(
        [
            str(REPO_ROOT / "target" / "release" / "sifr"),
            "--sysroot",
            str(REPO_ROOT),
            "run",
            str(sifr_path),
        ],
        timeout,
    )
    print(
        f"[cpython-generated] suite={suite_name} case={case_id} "
        f"python_exit={cpython.exit_code} sifr_exit={sifr.exit_code} "
        f"python_ms={cpython.duration_ms:.0f} sifr_ms={sifr.duration_ms:.0f}"
    )

    failures: list[str] = []
    compare_runtime(case_id, "CPython", cpython, case, failures)
    compare_runtime(case_id, "Sifr", sifr, case, failures)
    if exit_bucket(cpython.exit_code) != exit_bucket(sifr.exit_code):
        failures.append(
            f"{case_id} exit buckets differ: CPython={exit_bucket(cpython.exit_code)} "
            f"Sifr={exit_bucket(sifr.exit_code)}"
        )
    if error_presence("CPython", cpython) != error_presence("Sifr", sifr):
        failures.append(
            f"{case_id} error presence differs: CPython={error_presence('CPython', cpython)} "
            f"Sifr={error_presence('Sifr', sifr)}"
        )
    cpython_json = parse_exact_json_line(case_id, "CPython", cpython.stdout, failures)
    sifr_json = parse_exact_json_line(case_id, "Sifr", sifr.stdout, failures)
    if cpython_json is not None:
        failures.extend(validate_value_grammar(case_id, "CPython", cpython_json))
    if sifr_json is not None:
        failures.extend(validate_value_grammar(case_id, "Sifr", sifr_json))
    if normalize_stdout(cpython.stdout) != normalize_stdout(sifr.stdout):
        failures.append(
            f"{case_id} stdout differs after CRLF normalization: "
            f"CPython={normalize_stdout(cpython.stdout)!r} Sifr={normalize_stdout(sifr.stdout)!r}"
        )
    return failures


def compare_runtime(
    case_id: str,
    runtime: str,
    result: RuntimeResult,
    case: dict[str, Any],
    failures: list[str],
) -> None:
    if result.timed_out:
        failures.append(f"{case_id} {runtime} timed out")
    expected_bucket = str(case["expected_exit_bucket"])
    actual_bucket = exit_bucket(result.exit_code)
    if actual_bucket != expected_bucket:
        failures.append(f"{case_id} {runtime} exit bucket {actual_bucket} != expected {expected_bucket}")
    expected_presence = str(case["expected_error_presence"])
    actual_presence = error_presence(runtime, result)
    if actual_presence != expected_presence:
        failures.append(f"{case_id} {runtime} error presence {actual_presence} != expected {expected_presence}")


def run_command(argv: list[str], timeout: int) -> RuntimeResult:
    started = time.perf_counter()
    try:
        completed = subprocess.run(
            argv,
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        return RuntimeResult(
            exit_code=124,
            stdout=decode_timeout_stream(error.stdout),
            stderr=decode_timeout_stream(error.stderr),
            duration_ms=(time.perf_counter() - started) * 1000.0,
            timed_out=True,
        )
    return RuntimeResult(
        exit_code=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
        duration_ms=(time.perf_counter() - started) * 1000.0,
    )


def parse_exact_json_line(case_id: str, runtime: str, stdout: str, failures: list[str]) -> Any | None:
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


def validate_value_grammar(case_id: str, runtime: str, value: Any) -> list[str]:
    failures: list[str] = []
    validate_value(case_id, runtime, value, 0, failures)
    return failures


def validate_value(case_id: str, runtime: str, value: Any, depth: int, failures: list[str]) -> str:
    if depth > MAX_DEPTH:
        failures.append(f"{case_id} {runtime} JSON value exceeds depth {MAX_DEPTH}")
        return "invalid"
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "bool"
    if isinstance(value, int):
        if not INTEGER_MIN <= value <= INTEGER_MAX:
            failures.append(f"{case_id} {runtime} integer {value} outside [{INTEGER_MIN}, {INTEGER_MAX}]")
        return "int"
    if isinstance(value, float):
        failures.append(f"{case_id} {runtime} floats are outside value grammar v1")
        return "float"
    if isinstance(value, str):
        return "str"
    if isinstance(value, list):
        element_types = {validate_value(case_id, runtime, item, depth + 1, failures) for item in value}
        if len(element_types - {"invalid"}) > 1:
            failures.append(f"{case_id} {runtime} list is not homogeneous")
        return f"list:{next(iter(element_types), 'empty')}"
    if isinstance(value, dict):
        keys = list(value.keys())
        if not all(isinstance(key, str) for key in keys):
            failures.append(f"{case_id} {runtime} dict keys must be strings")
        if keys != sorted(keys):
            failures.append(f"{case_id} {runtime} dict keys are not canonical sorted order")
        value_types = {validate_value(case_id, runtime, item, depth + 1, failures) for item in value.values()}
        if len(value_types - {"invalid"}) > 1:
            failures.append(f"{case_id} {runtime} dict values are not homogeneous")
        return f"dict:{next(iter(value_types), 'empty')}"
    failures.append(f"{case_id} {runtime} unsupported JSON value type {type(value).__name__}")
    return "invalid"


def write_minimized_candidate(
    suite_name: str,
    case: dict[str, Any],
    suite_actual_root: Path,
    failures: list[str],
) -> dict[str, Any]:
    candidate = minimized_candidate(case)
    case_id = str(case["id"])
    minimized_root = suite_actual_root / case_id / "minimized"
    minimized_root.mkdir(parents=True, exist_ok=True)
    (minimized_root / "case.py").write_text(candidate.python_source, encoding="utf-8")
    (minimized_root / "main.sifr").write_text(candidate.sifr_source, encoding="utf-8")
    metadata = {
        "schema_version": 1,
        "suite": suite_name,
        "case_id": case_id,
        "original_seed": case["seed"],
        "shape": case["shape"],
        "failures": failures,
        "minimized_python": str((minimized_root / "case.py").relative_to(REPO_ROOT)),
        "minimized_sifr": str((minimized_root / "main.sifr").relative_to(REPO_ROOT)),
    }
    (minimized_root / "metadata.json").write_text(json.dumps(metadata, indent=2, sort_keys=True), encoding="utf-8")
    return metadata


def exit_bucket(exit_code: int) -> str:
    return "0" if exit_code == 0 else "non-zero"


def error_presence(runtime: str, result: RuntimeResult) -> str:
    if result.exit_code == 0:
        return "no-error"
    # CPython generated cases are never classified as compile errors; Sifr may fail during static checking.
    if runtime == "Sifr" and "error[" in result.stderr:
        return "compile-error"
    return "runtime-error"


def normalize_stdout(stdout: str) -> str:
    return stdout.replace("\r\n", "\n")


def decode_timeout_stream(stream: str | bytes | None) -> str:
    if stream is None:
        return ""
    if isinstance(stream, str):
        return stream
    return stream.decode("utf-8", errors="replace")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def source_digest(inputs: list[str]) -> str:
    digest = hashlib.sha256()
    for pattern in inputs:
        paths = sorted(REPO_ROOT.glob(pattern))
        if not paths:
            path = REPO_ROOT / pattern
            paths = [path] if path.exists() else []
        for path in paths:
            if path.is_file():
                relative = str(path.relative_to(REPO_ROOT))
                digest.update(relative.encode("utf-8"))
                digest.update(b"\0")
                digest.update(path.read_bytes())
                digest.update(b"\0")
    return digest.hexdigest()
