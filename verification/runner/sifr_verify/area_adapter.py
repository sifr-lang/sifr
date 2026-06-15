"""Shared adapter support for verification areas with checked baselines."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .paths import REPO_ROOT

TMP_PATTERNS = (
    re.compile(r"/private/var/folders/[^\s\"']+"),
    re.compile(r"/private/tmp/[^\s\"']+"),
    re.compile(r"/tmp/[^\s\"']+"),
    re.compile(r"/var/folders/[^\s\"']+"),
)
ARTIFACT_CACHE_LINE_PATTERN = re.compile(r"^\[sifr-artifact-cache\].*$")
BASELINE_COMMANDS = {"check", "run", "build", "test", "lint"}
CONTRACT_MATRIX_COMMAND = "contract-matrix"


@dataclass(frozen=True)
class AreaAdapterConfig:
    area: str
    owner: str
    runner_name: str
    manifest_path: Path
    actual_root: Path
    status_label: str


@dataclass(frozen=True)
class AreaRunOptions:
    suite_filters: set[str]
    bless: bool
    result_json: Path
    hardening_summary: bool


def run_area(config: AreaAdapterConfig, options: AreaRunOptions) -> int:
    manifest = load_manifest(config.manifest_path)
    suites = select_suites(config, manifest, options.suite_filters)
    config.actual_root.mkdir(parents=True, exist_ok=True)

    print(f"Running {config.status_label} verification area")
    print(f"  manifest={format_repo_relative_path(config.manifest_path)}")
    print(f"  bless={'yes' if options.bless else 'no'}")

    suite_results = [run_suite(config, suite, options) for suite in suites]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    blocking_failures = total_failures
    result_payload = {
        "schema_version": 1,
        "area": config.area,
        "bless": options.bless,
        "manifest": format_repo_relative_path(config.manifest_path),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": blocking_failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = resolve_repo_path(options.result_json)
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(result_payload, indent=2, sort_keys=True), encoding="utf-8")

    if options.bless:
        print("baselines updated")
    else:
        print(f"result_json={format_repo_relative_path(result_path)}")

    if blocking_failures > 0 and not options.bless:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={blocking_failures}, non_blocking_failures=0",
            file=sys.stderr,
        )
        print(
            f"actual outputs written under {format_repo_relative_path(config.actual_root)}",
            file=sys.stderr,
        )
        return 1

    summary_prefix = "verification ok" if options.hardening_summary else f"{config.status_label} verification ok"
    print(
        f"{summary_prefix}: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures={blocking_failures}, non_blocking_failures=0"
    )
    return 0


def load_manifest(manifest_path: Path) -> dict[str, Any]:
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def select_suites(
    config: AreaAdapterConfig,
    manifest: dict[str, Any],
    requested: set[str],
) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    if not isinstance(suites, list) or not suites:
        raise SystemExit(f"{config.area} area manifest contains no suites")
    selected = [suite for suite in suites if not requested or str(suite.get("name")) in requested]
    if requested:
        present = {str(suite.get("name")) for suite in selected}
        missing = sorted(requested.difference(present))
        if missing:
            raise SystemExit(f"unknown {config.area} suite filter(s): {', '.join(missing)}")
    return selected


def run_suite(
    config: AreaAdapterConfig,
    suite: dict[str, Any],
    options: AreaRunOptions,
) -> dict[str, Any]:
    suite_name = str(suite["name"])
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or not cases:
        raise SystemExit(f"{config.area} suite '{suite_name}' has no cases")
    print(f"  suite={suite_name} owner={config.owner} cases={len(cases)}")

    result = {
        "name": suite_name,
        "owner": config.owner,
        "blocking": True,
        "runner": config.runner_name,
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }
    validate_unique_baseline_artifact_paths(config, suite_name, cases)
    for case in cases:
        case_result, case_failed, failed_variants = run_case(config, suite_name, case, options)
        result["cases"].append(case_result)
        result["total_variants"] += len(case_result["variants"])
        result["total_failures"] += failed_variants
        if case_failed:
            result["failed_cases"] += 1
    return result


def run_case(
    config: AreaAdapterConfig,
    suite_name: str,
    case: dict[str, Any],
    options: AreaRunOptions,
) -> tuple[dict[str, Any], bool, int]:
    command = str(case["command"])
    if command == "area-check":
        return run_area_check_case(config, suite_name, case)
    if command == CONTRACT_MATRIX_COMMAND:
        return run_contract_matrix_case(config, suite_name, case)
    return run_baseline_case(config, suite_name, case, options)


def run_area_check_case(
    config: AreaAdapterConfig,
    suite_name: str,
    case: dict[str, Any],
) -> tuple[dict[str, Any], bool, int]:
    case_id = str(case["id"])
    entry = case_entry_path(config, suite_name, case_id, case)
    expected_exit = int(case["expect_exit_code"])
    started = time.perf_counter()
    proc = subprocess.run(
        [sys.executable, str(entry)],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    if proc.stdout:
        sys.stdout.write(proc.stdout)
    if proc.stderr:
        sys.stderr.write(proc.stderr)
    mismatches = []
    if proc.returncode != expected_exit:
        mismatches.append("unexpected-exit")
    status = "pass" if not mismatches else "fail"
    emit_case_timing(config.area, suite_name, case_id, "area-check", elapsed_ms, status)
    return (
        {
            "id": case_id,
            "entry": format_repo_relative_path(entry),
            "command": "area-check",
            "variants": [
                {
                    "label": "area-check",
                    "diagnostic_format": None,
                    "argv": [sys.executable, str(entry)],
                    "status": status,
                    "mismatches": mismatches,
                    "expected_exit_code": expected_exit,
                    "actual_exit_code": proc.returncode,
                    "duration_ms": round(elapsed_ms, 3),
                }
            ],
        },
        bool(mismatches),
        len(mismatches),
    )


def run_contract_matrix_case(
    config: AreaAdapterConfig,
    suite_name: str,
    case: dict[str, Any],
) -> tuple[dict[str, Any], bool, int]:
    case_id = str(case["id"])
    contract_suite = case_id
    entry = case_entry_path(config, suite_name, case_id, case)
    if not entry.is_file():
        raise SystemExit(
            f"{config.area} case '{case_id}' contract manifest does not exist: {entry}"
        )
    expected_exit = int(case["expect_exit_code"])
    argv = [
        "cargo",
        "test",
        "--locked",
        "-p",
        "sifr",
        "--test",
        "validation_contracts",
        "test_validation_contract_matrix",
        "--",
        "--ignored",
        "--nocapture",
    ]
    started = time.perf_counter()
    proc = subprocess.run(
        argv,
        cwd=REPO_ROOT,
        env={
            **_contract_matrix_env(),
            "SIFR_VALIDATION_CONTRACT_MANIFEST": str(entry),
            "SIFR_VALIDATION_CONTRACT_SUITE_FILTER": contract_suite,
        },
        text=True,
        capture_output=True,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    if proc.stdout:
        sys.stdout.write(proc.stdout)
    if proc.stderr:
        sys.stderr.write(proc.stderr)
    mismatches = []
    if proc.returncode != expected_exit:
        mismatches.append("unexpected-exit")
    status = "pass" if not mismatches else "fail"
    emit_case_timing(config.area, suite_name, case_id, CONTRACT_MATRIX_COMMAND, elapsed_ms, status)
    return (
        {
            "id": case_id,
            "entry": format_repo_relative_path(entry),
            "command": CONTRACT_MATRIX_COMMAND,
            "variants": [
                {
                    "label": CONTRACT_MATRIX_COMMAND,
                    "diagnostic_format": None,
                    "argv": argv,
                    "contract_suite": contract_suite,
                    "status": status,
                    "mismatches": mismatches,
                    "expected_exit_code": expected_exit,
                    "actual_exit_code": proc.returncode,
                    "duration_ms": round(elapsed_ms, 3),
                }
            ],
        },
        bool(mismatches),
        len(mismatches),
    )


def run_baseline_case(
    config: AreaAdapterConfig,
    suite_name: str,
    case: dict[str, Any],
    options: AreaRunOptions,
) -> tuple[dict[str, Any], bool, int]:
    case_id, entry, command_name, formats = baseline_case_metadata(config, suite_name, case)
    expected_exit = int(case["expect_exit_code"])

    case_failed = False
    failed_variants = 0
    case_result = {
        "id": case_id,
        "entry": format_repo_relative_path(entry),
        "command": command_name,
        "variants": [],
    }
    for diagnostic_format in formats:
        label = baseline_variant_label(command_name, diagnostic_format)
        exit_code, stdout, stderr, elapsed_ms, argv = run_sifr_variant(
            command_name=command_name,
            entry=entry,
            diagnostic_format=diagnostic_format,
        )
        stdout_norm = canonicalize_output(stdout, diagnostic_format, "stdout")
        stderr_norm = canonicalize_output(stderr, diagnostic_format, "stderr")
        stdout_file, stderr_file, exit_file = baseline_artifact_paths(entry, label)
        mismatches = compare_or_bless(
            config=config,
            options=options,
            case_id=case_id,
            label=label,
            stdout_norm=stdout_norm,
            stderr_norm=stderr_norm,
            exit_code=exit_code,
            stdout_file=stdout_file,
            stderr_file=stderr_file,
            exit_file=exit_file,
        )
        if exit_code != expected_exit:
            mismatches.append("unexpected-exit")

        status = "pass" if not mismatches else "fail"
        emit_case_timing(config.area, suite_name, case_id, label, elapsed_ms, status)
        if mismatches:
            case_failed = True
            failed_variants += 1
        case_result["variants"].append(
            {
                "label": label,
                "diagnostic_format": diagnostic_format,
                "argv": argv,
                "status": status,
                "mismatches": mismatches,
                "expected_exit_code": expected_exit,
                "actual_exit_code": exit_code,
                "duration_ms": round(elapsed_ms, 3),
                "baseline_stdout": format_repo_relative_path(stdout_file),
                "baseline_stderr": format_repo_relative_path(stderr_file),
                "baseline_exit_code": format_repo_relative_path(exit_file),
            }
        )
    return case_result, case_failed, failed_variants


def baseline_case_metadata(
    config: AreaAdapterConfig,
    suite_name: str,
    case: dict[str, Any],
) -> tuple[str, Path, str, list[str | None]]:
    case_id = str(case["id"])
    command_name = str(case["command"])
    if command_name not in BASELINE_COMMANDS:
        raise SystemExit(
            f"{config.area} suite '{suite_name}' case '{case_id}' has unsupported command "
            f"'{command_name}'"
        )
    entry = case_entry_path(config, suite_name, case_id, case)
    if not entry.is_file():
        raise SystemExit(f"{config.area} case '{case_id}' entry does not exist: {entry}")
    formats = parse_formats(case.get("diagnostic_formats"))
    if not formats:
        raise SystemExit(f"{config.area} suite '{suite_name}' case '{case_id}' has invalid diagnostic_formats")
    validate_unique_diagnostic_formats(suite_name, case_id, formats)
    return case_id, entry, command_name, formats


def case_entry_path(
    config: AreaAdapterConfig,
    suite_name: str,
    case_id: str,
    case: dict[str, Any],
) -> Path:
    raw_entry = Path(str(case["entry"]))
    if raw_entry.is_absolute():
        raise SystemExit(
            f"{config.area} suite '{suite_name}' case '{case_id}' entry must be repo-relative"
        )
    return resolve_repo_path(raw_entry)


def validate_unique_baseline_artifact_paths(
    config: AreaAdapterConfig,
    suite_name: str,
    cases: list[Any],
) -> None:
    seen: dict[Path, str] = {}
    for case in cases:
        if not isinstance(case, dict):
            continue
        command = str(case.get("command"))
        if command in {"area-check", CONTRACT_MATRIX_COMMAND}:
            continue
        case_id, entry, command_name, formats = baseline_case_metadata(config, suite_name, case)
        for diagnostic_format in formats:
            label = baseline_variant_label(command_name, diagnostic_format)
            for artifact_path in baseline_artifact_paths(entry, label):
                key = artifact_path.resolve()
                previous = seen.get(key)
                owner = f"{case_id}:{label}"
                if previous is not None:
                    raise SystemExit(
                        f"{config.area} suite '{suite_name}' baseline artifact path collision "
                        f"for {format_repo_relative_path(key)}: {previous} and {owner}"
                    )
                seen[key] = owner


def parse_formats(raw: object) -> list[str | None]:
    if raw is None:
        return [None]
    if not isinstance(raw, list):
        return []
    if not raw:
        return [None]
    return [str(item) for item in raw]


def validate_unique_diagnostic_formats(
    suite_name: str,
    case_id: str,
    formats: list[str | None],
) -> None:
    seen: set[str | None] = set()
    for diagnostic_format in formats:
        if diagnostic_format in seen:
            display = "default" if diagnostic_format is None else diagnostic_format
            raise SystemExit(
                f"suite '{suite_name}' case '{case_id}' lists diagnostic_format "
                f"'{display}' more than once"
            )
        seen.add(diagnostic_format)


def run_sifr_variant(
    *,
    command_name: str,
    entry: Path,
    diagnostic_format: str | None,
) -> tuple[int, str, str, float, list[str]]:
    argv = ["cargo", "run", "--locked", "-q", "-p", "sifr", "--"]
    if diagnostic_format is not None:
        argv.extend(["--diagnostic-format", diagnostic_format])
    argv.extend([command_name, str(entry)])
    started = time.perf_counter()
    proc = subprocess.run(
        argv,
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    return proc.returncode, proc.stdout, proc.stderr, elapsed_ms, argv


def compare_or_bless(
    *,
    config: AreaAdapterConfig,
    options: AreaRunOptions,
    case_id: str,
    label: str,
    stdout_norm: str,
    stderr_norm: str,
    exit_code: int,
    stdout_file: Path,
    stderr_file: Path,
    exit_file: Path,
) -> list[str]:
    if options.bless:
        write_text(stdout_file, stdout_norm)
        write_text(stderr_file, stderr_norm)
        write_text(exit_file, f"{exit_code}\n")
        return []

    mismatches: list[str] = []
    missing_files = [path for path in (stdout_file, stderr_file, exit_file) if not path.is_file()]
    if missing_files:
        mismatches.append(
            "missing-baseline:"
            + ",".join(format_repo_relative_path(path) for path in missing_files)
        )
    else:
        if stdout_norm != stdout_file.read_text(encoding="utf-8"):
            mismatches.append("stdout")
        if stderr_norm != stderr_file.read_text(encoding="utf-8"):
            mismatches.append("stderr")
        if str(exit_code) != exit_file.read_text(encoding="utf-8").strip():
            mismatches.append("exit-code")

    if mismatches:
        actual_case_dir = config.actual_root / case_id
        write_text(actual_case_dir / f"{label}.stdout.txt", stdout_norm)
        write_text(actual_case_dir / f"{label}.stderr.txt", stderr_norm)
        write_text(actual_case_dir / f"{label}.exit-code.txt", f"{exit_code}\n")
    return mismatches


def baseline_variant_label(command_name: str, diagnostic_format: str | None) -> str:
    return f"{command_name}-{diagnostic_format}" if diagnostic_format else command_name


def baseline_artifact_paths(entry: Path, label: str) -> tuple[Path, Path, Path]:
    baseline_dir = entry.parent / "baselines"
    return (
        baseline_dir / f"{label}.stdout.txt",
        baseline_dir / f"{label}.stderr.txt",
        baseline_dir / f"{label}.exit-code.txt",
    )


def canonicalize_output(text: str, diagnostic_format: str | None, stream: str) -> str:
    if stream == "stdout" and diagnostic_format == "json" and text.strip():
        try:
            parsed = json.loads(text)
        except json.JSONDecodeError:
            return normalize_string(text)
        return json.dumps(normalize_json_value(parsed), indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    return normalize_string(text)


def normalize_json_value(value: object) -> object:
    if isinstance(value, str):
        return normalize_string(value).rstrip("\n")
    if isinstance(value, list):
        return [normalize_json_value(item) for item in value]
    if isinstance(value, dict):
        return {key: normalize_json_value(item) for key, item in value.items()}
    return value


def normalize_string(value: str) -> str:
    normalized = value.replace("\r\n", "\n").replace("\r", "\n")
    normalized = normalized.replace(str(REPO_ROOT), "<WORKSPACE>")
    for pattern in TMP_PATTERNS:
        normalized = pattern.sub("<TMP>", normalized)
    normalized = "\n".join(
        line.rstrip()
        for line in normalized.split("\n")
        if not ARTIFACT_CACHE_LINE_PATTERN.fullmatch(line.strip())
    )
    if normalized and not normalized.endswith("\n"):
        normalized += "\n"
    return normalized


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def resolve_repo_path(path: Path) -> Path:
    resolved = path if path.is_absolute() else REPO_ROOT / path
    resolved = resolved.resolve()
    try:
        resolved.relative_to(REPO_ROOT)
    except ValueError as error:
        raise SystemExit(f"area path must stay under repo root: {path}") from error
    return resolved


def format_repo_relative_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def emit_case_timing(
    area: str,
    suite_name: str,
    case_id: str,
    label: str,
    elapsed_ms: float,
    status: str,
) -> None:
    print(
        f"[sifr-case-timing] bucket={area} "
        f"case={timing_token(suite_name)}/{timing_token(case_id)}/{timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}"
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


def _contract_matrix_env() -> dict[str, str]:
    env = dict(os.environ)
    env.pop("SIFR_VALIDATION_CONTRACT_SUITE_FILTER", None)
    return env
