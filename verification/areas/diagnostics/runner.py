"""Diagnostics verification area adapter."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = Path(__file__).resolve().with_name("manifest.json")
ACTUAL_ROOT = REPO_ROOT / "target" / "verification" / "actual" / "diagnostics"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "diagnostics-results.json"

TMP_PATTERNS = (
    re.compile(r"/private/var/folders/[^\s\"']+"),
    re.compile(r"/private/tmp/[^\s\"']+"),
    re.compile(r"/tmp/[^\s\"']+"),
    re.compile(r"/var/folders/[^\s\"']+"),
)
ARTIFACT_CACHE_LINE_PATTERN = re.compile(r"^\[sifr-artifact-cache\].*$")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Update checked-in baselines.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable diagnostics area result summary.",
    )
    parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit the legacy hardening summary line consumed by validation reports.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    manifest = load_manifest()
    suites = select_suites(manifest, set(args.suite))
    ACTUAL_ROOT.mkdir(parents=True, exist_ok=True)

    print("Running diagnostics verification area")
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}")
    print(f"  bless={'yes' if args.bless else 'no'}")

    suite_results = [run_suite(suite, args) for suite in suites]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    blocking_failures = total_failures
    result_payload = {
        "schema_version": 1,
        "area": "diagnostics",
        "bless": args.bless,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": blocking_failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = (REPO_ROOT / args.result_json).resolve()
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(result_payload, indent=2, sort_keys=True), encoding="utf-8")

    if args.bless:
        print("baselines updated")
    else:
        print(f"result_json={result_path.relative_to(REPO_ROOT)}")

    if blocking_failures > 0 and not args.bless:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={blocking_failures}, non_blocking_failures=0",
            file=sys.stderr,
        )
        print(f"actual outputs written under {ACTUAL_ROOT.relative_to(REPO_ROOT)}", file=sys.stderr)
        return 1

    summary_prefix = "verification ok" if args.hardening_summary else "diagnostics verification ok"
    print(
        f"{summary_prefix}: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures={blocking_failures}, non_blocking_failures=0"
    )
    return 0


def load_manifest() -> dict[str, Any]:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    if not isinstance(suites, list) or not suites:
        raise SystemExit("diagnostics area manifest contains no suites")
    selected = [
        suite
        for suite in suites
        if not requested or str(suite.get("name")) in requested
    ]
    if requested:
        present = {str(suite.get("name")) for suite in selected}
        missing = sorted(requested.difference(present))
        if missing:
            raise SystemExit(f"unknown diagnostics suite filter(s): {', '.join(missing)}")
    return selected


def run_suite(suite: dict[str, Any], args: argparse.Namespace) -> dict[str, Any]:
    suite_name = str(suite["name"])
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or not cases:
        raise SystemExit(f"diagnostics suite '{suite_name}' has no cases")
    print(f"  suite={suite_name} owner=compiler/diagnostics cases={len(cases)}")

    result = {
        "name": suite_name,
        "owner": "compiler/diagnostics",
        "blocking": True,
        "runner": "diagnostics-area",
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }
    for case in cases:
        case_result, case_failed, failed_variants = run_case(suite_name, case, args)
        result["cases"].append(case_result)
        result["total_variants"] += len(case_result["variants"])
        result["total_failures"] += failed_variants
        if case_failed:
            result["failed_cases"] += 1
    return result


def run_case(
    suite_name: str,
    case: dict[str, Any],
    args: argparse.Namespace,
) -> tuple[dict[str, Any], bool, int]:
    command = str(case["command"])
    if command == "area-check":
        return run_area_check_case(suite_name, case)
    return run_baseline_case(suite_name, case, args)


def run_area_check_case(
    suite_name: str,
    case: dict[str, Any],
) -> tuple[dict[str, Any], bool, int]:
    case_id = str(case["id"])
    entry = REPO_ROOT / str(case["entry"])
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
    emit_case_timing(suite_name, case_id, "area-check", elapsed_ms, status)
    return (
        {
            "id": case_id,
            "entry": str(entry.relative_to(REPO_ROOT)),
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


def run_baseline_case(
    suite_name: str,
    case: dict[str, Any],
    args: argparse.Namespace,
) -> tuple[dict[str, Any], bool, int]:
    case_id = str(case["id"])
    entry = REPO_ROOT / str(case["entry"])
    if not entry.is_file():
        raise SystemExit(f"diagnostics case '{case_id}' entry does not exist: {entry}")
    expected_exit = int(case["expect_exit_code"])
    formats = parse_formats(case.get("diagnostic_formats"))
    if not formats:
        raise SystemExit(f"diagnostics baseline case '{case_id}' has no diagnostic formats")
    validate_unique_formats(suite_name, case_id, formats)

    case_failed = False
    failed_variants = 0
    case_result = {
        "id": case_id,
        "entry": str(entry.relative_to(REPO_ROOT)),
        "command": str(case["command"]),
        "variants": [],
    }
    for diagnostic_format in formats:
        label = f"{case['command']}-{diagnostic_format}"
        exit_code, stdout, stderr, elapsed_ms, argv = run_sifr_variant(
            command_name=str(case["command"]),
            entry=entry,
            diagnostic_format=diagnostic_format,
        )
        stdout_norm = canonicalize_output(stdout, diagnostic_format, "stdout")
        stderr_norm = canonicalize_output(stderr, diagnostic_format, "stderr")
        stdout_file, stderr_file, exit_file = baseline_artifact_paths(entry, label)
        mismatches = compare_or_bless(
            args=args,
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
        emit_case_timing(suite_name, case_id, label, elapsed_ms, status)
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
                "baseline_stdout": str(stdout_file.relative_to(REPO_ROOT)),
                "baseline_stderr": str(stderr_file.relative_to(REPO_ROOT)),
                "baseline_exit_code": str(exit_file.relative_to(REPO_ROOT)),
            }
        )
    return case_result, case_failed, failed_variants


def parse_formats(raw: object) -> list[str]:
    if not isinstance(raw, list):
        return []
    return [str(item) for item in raw]


def validate_unique_formats(suite_name: str, case_id: str, formats: list[str]) -> None:
    seen: set[str] = set()
    for diagnostic_format in formats:
        if diagnostic_format in seen:
            raise SystemExit(
                f"suite '{suite_name}' case '{case_id}' lists diagnostic_format "
                f"'{diagnostic_format}' more than once"
            )
        seen.add(diagnostic_format)


def run_sifr_variant(
    *,
    command_name: str,
    entry: Path,
    diagnostic_format: str,
) -> tuple[int, str, str, float, list[str]]:
    argv = [
        "cargo",
        "run",
        "-q",
        "-p",
        "sifr",
        "--",
        "--diagnostic-format",
        diagnostic_format,
        command_name,
        str(entry),
    ]
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
    args: argparse.Namespace,
    case_id: str,
    label: str,
    stdout_norm: str,
    stderr_norm: str,
    exit_code: int,
    stdout_file: Path,
    stderr_file: Path,
    exit_file: Path,
) -> list[str]:
    if args.bless:
        write_text(stdout_file, stdout_norm)
        write_text(stderr_file, stderr_norm)
        write_text(exit_file, f"{exit_code}\n")
        return []

    mismatches: list[str] = []
    missing_files = [path for path in (stdout_file, stderr_file, exit_file) if not path.is_file()]
    if missing_files:
        mismatches.append(
            "missing-baseline:"
            + ",".join(str(path.relative_to(REPO_ROOT)) for path in missing_files)
        )
    else:
        if stdout_norm != stdout_file.read_text(encoding="utf-8"):
            mismatches.append("stdout")
        if stderr_norm != stderr_file.read_text(encoding="utf-8"):
            mismatches.append("stderr")
        if str(exit_code) != exit_file.read_text(encoding="utf-8").strip():
            mismatches.append("exit-code")

    if mismatches:
        actual_case_dir = ACTUAL_ROOT / case_id
        write_text(actual_case_dir / f"{label}.stdout.txt", stdout_norm)
        write_text(actual_case_dir / f"{label}.stderr.txt", stderr_norm)
        write_text(actual_case_dir / f"{label}.exit-code.txt", f"{exit_code}\n")
    return mismatches


def baseline_artifact_paths(entry: Path, label: str) -> tuple[Path, Path, Path]:
    baseline_dir = entry.parent / "baselines"
    return (
        baseline_dir / f"{label}.stdout.txt",
        baseline_dir / f"{label}.stderr.txt",
        baseline_dir / f"{label}.exit-code.txt",
    )


def canonicalize_output(text: str, diagnostic_format: str, stream: str) -> str:
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


def emit_case_timing(
    suite_name: str,
    case_id: str,
    label: str,
    elapsed_ms: float,
    status: str,
) -> None:
    print(
        f"[sifr-case-timing] bucket=diagnostics "
        f"case={timing_token(suite_name)}/{timing_token(case_id)}/{timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}"
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
