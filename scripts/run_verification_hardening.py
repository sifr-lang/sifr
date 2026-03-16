#!/usr/bin/env python3
"""Run phase-29 verification suites with baseline compare/bless support."""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

TMP_PATTERNS = (
    re.compile(r"/private/var/folders/[^\s\"']+"),
    re.compile(r"/private/tmp/[^\s\"']+"),
    re.compile(r"/tmp/[^\s\"']+"),
    re.compile(r"/var/folders/[^\s\"']+"),
)
LOCAL_PINNED_REVISION_PATTERN = re.compile(r"^local-main@([0-9a-f]{7,40})$")
STRING_LITERAL_PATTERN = re.compile(r"(\"[^\n\"]*\"|'[^\n']*')")
INTEGER_LITERAL_PATTERN = re.compile(r"(?<![A-Za-z0-9_])\d+(?![A-Za-z0-9_])")
FUNCTION_SIGNATURE_PATTERN = re.compile(r"^\s*def\s+\w+\s*\(")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        default="verification/suites/manifest.json",
        help="Path to suite manifest JSON.",
    )
    parser.add_argument(
        "--profile",
        choices=("quick", "pr", "nightly", "release", "full", "stress"),
        default="pr",
        help="Execution profile.",
    )
    parser.add_argument(
        "--suite",
        action="append",
        default=[],
        help="Suite name filter (can repeat).",
    )
    parser.add_argument(
        "--bless",
        action="store_true",
        help="Update checked-in baselines instead of verifying.",
    )
    parser.add_argument(
        "--result-json",
        default="target/verification/hardening-results.json",
        help="Path for machine-readable run result summary.",
    )
    parser.add_argument(
        "--shard-total",
        type=int,
        default=1,
        help="Deterministic suite-level shard count.",
    )
    parser.add_argument(
        "--shard-index",
        type=int,
        default=0,
        help="Deterministic suite-level shard index (0-based).",
    )
    parser.add_argument(
        "--rerun-failures",
        type=int,
        default=1,
        help="Number of rerun attempts for failed suites (flake tracking only).",
    )
    parser.add_argument(
        "--quarantine-file",
        default="verification/flake/quarantine.json",
        help="Path to quarantine metadata file.",
    )
    return parser.parse_args()


def canonicalize_profile(profile: str) -> str:
    if profile == "full":
        return "pr"
    if profile == "stress":
        return "release"
    return profile


def normalize_string(value: str, repo_root: Path) -> str:
    normalized = value.replace("\r\n", "\n").replace("\r", "\n")
    normalized = normalized.replace(str(repo_root), "<WORKSPACE>")
    for pattern in TMP_PATTERNS:
        normalized = pattern.sub("<TMP>", normalized)
    normalized = "\n".join(line.rstrip() for line in normalized.split("\n"))
    if normalized and not normalized.endswith("\n"):
        normalized += "\n"
    return normalized


def normalize_json_text(value: str, repo_root: Path) -> str:
    parsed = json.loads(value)
    return json.dumps(
        normalize_json_value(parsed, repo_root),
        indent=2,
        sort_keys=True,
        ensure_ascii=False,
    ) + "\n"


def normalize_json_value(value: Any, repo_root: Path) -> Any:
    if isinstance(value, str):
        return normalize_string(value, repo_root).rstrip("\n")
    if isinstance(value, list):
        return [normalize_json_value(item, repo_root) for item in value]
    if isinstance(value, dict):
        return {key: normalize_json_value(item, repo_root) for key, item in value.items()}
    return value


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def should_run_suite(profile: str, suite_name: str) -> bool:
    canonical_profile = canonicalize_profile(profile)
    if canonical_profile == "quick":
        return False
    if canonical_profile == "pr":
        return suite_name in {
            "diagnostics",
            "project",
            "fixedbugs",
            "crashes",
            "oss-curated",
        }
    return True


def run_variant(
    *,
    repo_root: Path,
    command_name: str,
    entry: Path,
    diagnostic_format: str | None,
    timeout_secs: int | None = None,
) -> tuple[int, str, str, float, list[str]]:
    args = ["cargo", "run", "-q", "-p", "sifr", "--"]
    if diagnostic_format is not None:
        args.extend(["--diagnostic-format", diagnostic_format])
    args.extend([command_name, str(entry)])

    started = time.perf_counter()
    try:
        proc = subprocess.run(
            args,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
            timeout=timeout_secs,
        )
        exit_code = proc.returncode
        stdout = proc.stdout
        stderr = proc.stderr
    except subprocess.TimeoutExpired as timeout_error:
        exit_code = 124
        stdout = timeout_error.stdout or ""
        stderr = (timeout_error.stderr or "") + f"\ncommand timed out after {timeout_secs} seconds"
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    return exit_code, stdout, stderr, elapsed_ms, args


def canonicalize_output(
    *,
    repo_root: Path,
    text: str,
    diagnostic_format: str | None,
    stream: str,
) -> str:
    if stream == "stdout" and diagnostic_format == "json" and text.strip():
        try:
            return normalize_json_text(text, repo_root)
        except json.JSONDecodeError:
            return normalize_string(text, repo_root)
    return normalize_string(text, repo_root)


def parse_formats(raw: Any) -> list[str | None]:
    if raw is None:
        return [None]
    if not isinstance(raw, list) or not raw:
        return []
    return [str(item) for item in raw]


def load_index(path: Path) -> list[dict[str, Any]]:
    if not path.is_file():
        raise SystemExit(f"verification index not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    entries = payload.get("entries", [])
    if not isinstance(entries, list):
        raise SystemExit(f"invalid index file '{path}': 'entries' must be a list")
    return [entry for entry in entries if isinstance(entry, dict)]


def required_missing(entry: dict[str, Any], keys: tuple[str, ...]) -> list[str]:
    missing: list[str] = []
    for key in keys:
        value = entry.get(key)
        if not isinstance(value, str) or not value.strip():
            missing.append(key)
    return missing


def latest_project_revision(repo_root: Path, project_root: str) -> str | None:
    try:
        proc = subprocess.run(
            ["git", "log", "-n", "1", "--format=%H", "--", project_root],
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError:
        return None
    if proc.returncode != 0:
        return None
    revision = proc.stdout.strip()
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        return None
    return revision


def baseline_case_result(
    *,
    suite_name: str,
    case: dict[str, Any],
    args: argparse.Namespace,
    repo_root: Path,
    actual_root: Path,
) -> tuple[dict[str, Any], bool, int]:
    case_id = case.get("id")
    case_entry = case.get("entry")
    command_name = case.get("command")
    expected_exit = case.get("expect_exit_code")
    diagnostic_formats = case.get("diagnostic_formats")

    if not isinstance(case_id, str):
        raise SystemExit(f"suite '{suite_name}' has case without valid 'id'")
    if not isinstance(case_entry, str):
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' missing string 'entry'")
    if command_name not in {"check", "run", "build", "test"}:
        raise SystemExit(
            f"suite '{suite_name}' case '{case_id}' has unsupported command '{command_name}'"
        )
    if not isinstance(expected_exit, int):
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' missing integer 'expect_exit_code'")

    entry_path = repo_root / case_entry
    if not entry_path.is_file():
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' entry does not exist: {entry_path}")

    formats = parse_formats(diagnostic_formats)
    if not formats:
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' has invalid diagnostic_formats")

    case_failed = False
    failed_variants = 0
    case_result = {
        "id": case_id,
        "entry": str(entry_path.relative_to(repo_root)),
        "command": command_name,
        "variants": [],
    }

    for diagnostic_format in formats:
        label = f"{command_name}-{diagnostic_format}" if diagnostic_format else command_name
        exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
            repo_root=repo_root,
            command_name=command_name,
            entry=entry_path,
            diagnostic_format=diagnostic_format,
        )
        stdout_norm = canonicalize_output(
            repo_root=repo_root,
            text=stdout,
            diagnostic_format=diagnostic_format,
            stream="stdout",
        )
        stderr_norm = canonicalize_output(
            repo_root=repo_root,
            text=stderr,
            diagnostic_format=diagnostic_format,
            stream="stderr",
        )

        baseline_dir = entry_path.parent / "baselines"
        stdout_file = baseline_dir / f"{label}.stdout.txt"
        stderr_file = baseline_dir / f"{label}.stderr.txt"
        exit_file = baseline_dir / f"{label}.exit-code.txt"

        mismatches: list[str] = []

        if args.bless:
            write_text(stdout_file, stdout_norm)
            write_text(stderr_file, stderr_norm)
            write_text(exit_file, f"{exit_code}\n")
        else:
            missing_files = [path for path in (stdout_file, stderr_file, exit_file) if not path.is_file()]
            if missing_files:
                mismatches.append(
                    "missing-baseline:"
                    + ",".join(str(path.relative_to(repo_root)) for path in missing_files)
                )
            else:
                expected_stdout = load_text(stdout_file)
                expected_stderr = load_text(stderr_file)
                expected_exit_raw = load_text(exit_file).strip()
                if stdout_norm != expected_stdout:
                    mismatches.append("stdout")
                if stderr_norm != expected_stderr:
                    mismatches.append("stderr")
                if str(exit_code) != expected_exit_raw:
                    mismatches.append("exit-code")

        if exit_code != expected_exit:
            mismatches.append("unexpected-exit")

        status = "pass" if not mismatches else "fail"
        if mismatches:
            case_failed = True
            failed_variants += 1

            actual_case_dir = actual_root / suite_name / case_id
            write_text(actual_case_dir / f"{label}.stdout.txt", stdout_norm)
            write_text(actual_case_dir / f"{label}.stderr.txt", stderr_norm)
            write_text(actual_case_dir / f"{label}.exit-code.txt", f"{exit_code}\n")

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
                "baseline_stdout": str(stdout_file.relative_to(repo_root)),
                "baseline_stderr": str(stderr_file.relative_to(repo_root)),
                "baseline_exit_code": str(exit_file.relative_to(repo_root)),
            }
        )

    return case_result, case_failed, failed_variants


def run_baseline_suite(
    *,
    suite: dict[str, Any],
    args: argparse.Namespace,
    repo_root: Path,
    actual_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or not cases:
        raise SystemExit(f"suite '{suite_name}' has no cases")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} cases={len(cases)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "baseline",
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    for case in cases:
        case_result, case_failed, failed_variants = baseline_case_result(
            suite_name=suite_name,
            case=case,
            args=args,
            repo_root=repo_root,
            actual_root=actual_root,
        )
        result["cases"].append(case_result)
        result["total_variants"] += len(case_result["variants"])
        result["total_failures"] += failed_variants
        if case_failed:
            result["failed_cases"] += 1

    return result


def run_fixedbugs_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
    actual_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "fixedbugs",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    required = (
        "id",
        "issue",
        "root_cause_category",
        "suite_location",
        "command",
        "note",
    )

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        case_result = {
            "id": case_id,
            "issue": entry.get("issue"),
            "root_cause_category": entry.get("root_cause_category"),
            "entry": entry.get("suite_location"),
            "command": entry.get("command"),
            "variants": [],
        }
        case_failed = False

        missing = required_missing(entry, required)
        expected_exit = entry.get("expect_exit_code")
        command_name = entry.get("command")
        entry_path_raw = entry.get("suite_location")
        formats = parse_formats(entry.get("diagnostic_formats"))
        if not formats:
            formats = [None]

        metadata_mismatches = list(missing)
        if not isinstance(expected_exit, int):
            metadata_mismatches.append("expect_exit_code")
        if command_name not in {"check", "run", "build", "test"}:
            metadata_mismatches.append("command")
        entry_path = repo_root / str(entry_path_raw) if isinstance(entry_path_raw, str) else None
        if entry_path is None or not entry_path.is_file():
            metadata_mismatches.append("suite_location")

        if metadata_mismatches:
            case_failed = True
            result["total_failures"] += 1
            result["total_variants"] += 1
            case_result["variants"].append(
                {
                    "label": "metadata",
                    "status": "fail",
                    "mismatches": sorted(set(metadata_mismatches)),
                }
            )
            result["cases"].append(case_result)
            result["failed_cases"] += 1
            continue

        assert entry_path is not None
        for diagnostic_format in formats:
            label = f"{command_name}-{diagnostic_format}" if diagnostic_format else str(command_name)
            exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
                repo_root=repo_root,
                command_name=str(command_name),
                entry=entry_path,
                diagnostic_format=diagnostic_format,
            )
            stdout_norm = canonicalize_output(
                repo_root=repo_root,
                text=stdout,
                diagnostic_format=diagnostic_format,
                stream="stdout",
            )
            stderr_norm = canonicalize_output(
                repo_root=repo_root,
                text=stderr,
                diagnostic_format=diagnostic_format,
                stream="stderr",
            )
            mismatches: list[str] = []
            if exit_code != expected_exit:
                mismatches.append("unexpected-exit")

            status = "pass" if not mismatches else "fail"
            result["total_variants"] += 1
            if mismatches:
                case_failed = True
                result["total_failures"] += 1
                actual_case_dir = actual_root / suite_name / case_id
                write_text(actual_case_dir / f"{label}.stdout.txt", stdout_norm)
                write_text(actual_case_dir / f"{label}.stderr.txt", stderr_norm)
                write_text(actual_case_dir / f"{label}.exit-code.txt", f"{exit_code}\n")

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
                }
            )

        result["cases"].append(case_result)
        if case_failed:
            result["failed_cases"] += 1

    return result


def collect_fixedbug_ids(repo_root: Path, suites: list[dict[str, Any]]) -> set[str]:
    fixedbug_ids: set[str] = set()
    for suite in suites:
        if suite.get("runner", "baseline") != "fixedbugs":
            continue
        index_raw = suite.get("index")
        if not isinstance(index_raw, str):
            continue
        entries = load_index(repo_root / index_raw)
        for entry in entries:
            bug_id = entry.get("id")
            if isinstance(bug_id, str) and bug_id:
                fixedbug_ids.add(bug_id)
    return fixedbug_ids


def run_crashes_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
    fixedbug_ids: set[str],
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "crashes",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
        "unresolved_count": 0,
    }

    required = (
        "id",
        "issue",
        "owner",
        "status",
        "root_cause_category",
        "source_reference",
        "reproducer_fixture",
        "promotion_target_suite",
        "note",
    )

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        mismatches = required_missing(entry, required)
        status_raw = entry.get("status")
        source_ref = entry.get("source_reference")
        reproducer_ref = entry.get("reproducer_fixture")
        promotion_target = entry.get("promotion_target_suite")

        if status_raw not in {"unresolved", "promoted"}:
            mismatches.append("status")
        if status_raw == "unresolved":
            result["unresolved_count"] += 1
        if not isinstance(source_ref, str) or not (repo_root / source_ref).is_file():
            mismatches.append("source_reference")
        if not isinstance(reproducer_ref, str) or not (repo_root / reproducer_ref).is_file():
            mismatches.append("reproducer_fixture")
        if promotion_target != "fixedbugs":
            mismatches.append("promotion_target_suite")
        if status_raw == "promoted":
            promoted_id = entry.get("promoted_fixedbug_id")
            if not isinstance(promoted_id, str) or promoted_id not in fixedbug_ids:
                mismatches.append("promoted_fixedbug_id")

        variant_status = "pass" if not mismatches else "fail"
        result["total_variants"] += 1
        if mismatches:
            result["total_failures"] += 1
            result["failed_cases"] += 1

        result["cases"].append(
            {
                "id": case_id,
                "issue": entry.get("issue"),
                "status": status_raw,
                "root_cause_category": entry.get("root_cause_category"),
                "source_reference": source_ref,
                "reproducer_fixture": reproducer_ref,
                "promotion_target_suite": promotion_target,
                "variants": [
                    {
                        "label": "metadata",
                        "status": variant_status,
                        "mismatches": sorted(set(mismatches)),
                    }
                ],
            }
        )

    if result["unresolved_count"] == 0:
        result["total_failures"] += 1
        result["failed_cases"] += 1
        result["total_variants"] += 1
        result["cases"].append(
            {
                "id": "sentinel-unresolved-count",
                "variants": [
                    {
                        "label": "policy",
                        "status": "fail",
                        "mismatches": ["missing-unresolved-sentinels"],
                    }
                ],
            }
        )

    return result


def contains_internal_panic(text: str) -> bool:
    lowered = text.lower()
    return "internal compiler panic" in lowered or "panicked at" in lowered


def run_property_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "property",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        case_result = {
            "id": case_id,
            "entry": entry.get("entry"),
            "command": entry.get("command"),
            "variants": [],
        }
        case_failed = False
        mismatches = required_missing(
            entry,
            (
                "id",
                "entry",
                "command",
                "diagnostic_format",
                "note",
            ),
        )

        entry_path_raw = entry.get("entry")
        command_name = entry.get("command")
        diagnostic_format = entry.get("diagnostic_format")
        expected_exit = entry.get("expect_exit_code")
        repeat_runs = entry.get("repeat_runs", 2)
        assert_no_panic = bool(entry.get("assert_no_panic", True))

        if command_name not in {"check", "run", "build", "test"}:
            mismatches.append("command")
        if not isinstance(expected_exit, int):
            mismatches.append("expect_exit_code")
        if not isinstance(repeat_runs, int) or repeat_runs < 2:
            mismatches.append("repeat_runs")
        entry_path = repo_root / str(entry_path_raw) if isinstance(entry_path_raw, str) else None
        if entry_path is None or not entry_path.is_file():
            mismatches.append("entry")
        if not isinstance(diagnostic_format, str) or not diagnostic_format:
            mismatches.append("diagnostic_format")

        if mismatches:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "metadata",
                    "status": "fail",
                    "mismatches": sorted(set(mismatches)),
                }
            )
            result["cases"].append(case_result)
            continue

        assert entry_path is not None
        outputs: list[tuple[int, str, str]] = []
        for run_index in range(repeat_runs):
            exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
                repo_root=repo_root,
                command_name=str(command_name),
                entry=entry_path,
                diagnostic_format=str(diagnostic_format),
            )
            stdout_norm = canonicalize_output(
                repo_root=repo_root,
                text=stdout,
                diagnostic_format=str(diagnostic_format),
                stream="stdout",
            )
            stderr_norm = canonicalize_output(
                repo_root=repo_root,
                text=stderr,
                diagnostic_format=str(diagnostic_format),
                stream="stderr",
            )
            run_mismatches: list[str] = []
            if exit_code != expected_exit:
                run_mismatches.append("unexpected-exit")
            if assert_no_panic and contains_internal_panic(stdout_norm + stderr_norm):
                run_mismatches.append("panic-signal")
            status = "pass" if not run_mismatches else "fail"
            result["total_variants"] += 1
            if run_mismatches:
                case_failed = True
                result["total_failures"] += 1
            case_result["variants"].append(
                {
                    "label": f"run-{run_index + 1}",
                    "diagnostic_format": diagnostic_format,
                    "argv": argv,
                    "status": status,
                    "mismatches": run_mismatches,
                    "expected_exit_code": expected_exit,
                    "actual_exit_code": exit_code,
                    "duration_ms": round(elapsed_ms, 3),
                }
            )
            outputs.append((exit_code, stdout_norm, stderr_norm))

        if len(outputs) >= 2:
            baseline = outputs[0]
            for idx, current in enumerate(outputs[1:], start=2):
                compare_mismatches: list[str] = []
                if current[0] != baseline[0]:
                    compare_mismatches.append("exit-code-drift")
                if current[1] != baseline[1]:
                    compare_mismatches.append("stdout-drift")
                if current[2] != baseline[2]:
                    compare_mismatches.append("stderr-drift")
                result["total_variants"] += 1
                status = "pass" if not compare_mismatches else "fail"
                if compare_mismatches:
                    case_failed = True
                    result["total_failures"] += 1
                case_result["variants"].append(
                    {
                        "label": f"determinism-1-vs-{idx}",
                        "status": status,
                        "mismatches": compare_mismatches,
                    }
                )

        if case_failed:
            result["failed_cases"] += 1
        result["cases"].append(case_result)

    return result


def deterministic_mutations(seed_source: str, iterations: int, random_seed: int) -> list[str]:
    rng = random.Random(random_seed)
    lines = seed_source.splitlines()
    corpus: list[str] = []
    for _ in range(iterations):
        if not lines:
            lines = ["print(\"seed\")"]
        candidate = list(lines)
        op = rng.randint(0, 8)
        if op == 0:
            insert_line = rng.choice(
                [
                    "x: int = 1",
                    "y: int = x + 1",
                    "if x > 0:",
                    "    print(str(x))",
                    "from missing_mutation_module import bad",
                    "value: int = \"bad\"",
                ]
            )
            idx = rng.randint(0, len(candidate))
            candidate.insert(idx, insert_line)
        elif op == 1 and candidate:
            idx = rng.randrange(len(candidate))
            candidate[idx] = candidate[idx] + " # fuzz"
        elif op == 2 and len(candidate) > 1:
            idx = rng.randrange(len(candidate))
            del candidate[idx]
        elif op == 3 and candidate:
            idx = rng.randrange(len(candidate))
            candidate[idx] = candidate[idx].replace("main", "main_mut")
        elif op == 4 and candidate:
            idx = rng.randrange(len(candidate))
            candidate[idx] = candidate[idx].replace("int", "str", 1)
        elif op == 5:
            import_line = rng.choice(
                [
                    "from typing import Callable",
                    "from missing_mutation_module import bad",
                    "from helper import value",
                ]
            )
            idx = rng.randint(0, len(candidate))
            candidate.insert(idx, import_line)
        elif op == 6 and candidate:
            idx = rng.randrange(len(candidate))
            line = candidate[idx]
            if STRING_LITERAL_PATTERN.search(line):
                candidate[idx] = STRING_LITERAL_PATTERN.sub('"mutated"', line, count=1)
            else:
                candidate.insert(rng.randint(0, len(candidate)), 'label: str = "mutated"')
        elif op == 7 and candidate:
            idx = rng.randrange(len(candidate))
            line = candidate[idx]
            if INTEGER_LITERAL_PATTERN.search(line):
                replacement = str(rng.choice([0, 1, 2, 7, 42, 99, 1000]))
                candidate[idx] = INTEGER_LITERAL_PATTERN.sub(replacement, line, count=1)
            else:
                candidate.insert(rng.randint(0, len(candidate)), "counter: int = 42")
        elif op == 8:
            signature = rng.choice(
                [
                    "def fuzz_helper(value: int) -> int:",
                    "def fuzz_helper(value: str) -> str:",
                ]
            )
            if candidate:
                signature_indices = [
                    idx for idx, line in enumerate(candidate) if FUNCTION_SIGNATURE_PATTERN.search(line)
                ]
                if signature_indices:
                    idx = rng.choice(signature_indices)
                    replacement = signature.replace("fuzz_helper", f"fuzz_helper_{rng.randint(0, 9)}")
                    candidate[idx] = replacement
                else:
                    insert_at = rng.randint(0, len(candidate))
                    body = (
                        "    return value + 1"
                        if "-> int" in signature
                        else '    return value + "_mut"'
                    )
                    candidate[insert_at:insert_at] = [signature, body]
            else:
                candidate.extend([signature, "    return value"])
        corpus.append("\n".join(candidate).strip() + "\n")
    return corpus


def run_fuzz_smoke_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    if not index_path.is_file():
        raise SystemExit(f"suite '{suite_name}' index not found: {index_path}")
    payload = json.loads(index_path.read_text(encoding="utf-8"))

    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} manifest=1")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "fuzz-smoke",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    required = (
        "id",
        "command",
        "diagnostic_format",
        "note",
    )
    mismatches = required_missing(payload, required)
    seed_files = payload.get("seed_files")
    iterations = payload.get("iterations")
    random_seed = payload.get("random_seed")
    min_unique = payload.get("min_unique_cases")
    allow_exit_codes = payload.get("allow_exit_codes")
    assert_no_panic = bool(payload.get("assert_no_panic", True))

    if not isinstance(seed_files, list) or not seed_files:
        mismatches.append("seed_files")
    if not isinstance(iterations, int) or iterations < 1:
        mismatches.append("iterations")
    if not isinstance(random_seed, int):
        mismatches.append("random_seed")
    if not isinstance(min_unique, int) or min_unique < 1:
        mismatches.append("min_unique_cases")
    if not isinstance(allow_exit_codes, list) or not all(
        isinstance(code, int) for code in allow_exit_codes
    ):
        mismatches.append("allow_exit_codes")

    sources: list[str] = []
    if isinstance(seed_files, list):
        for seed in seed_files:
            if not isinstance(seed, str):
                mismatches.append("seed_file_path")
                continue
            seed_path = repo_root / seed
            if not seed_path.is_file():
                mismatches.append(f"seed_missing:{seed}")
                continue
            sources.append(seed_path.read_text(encoding="utf-8"))

    case_result = {
        "id": payload.get("id", "fuzz-smoke"),
        "variants": [],
    }

    if mismatches:
        result["total_variants"] += 1
        result["total_failures"] += 1
        result["failed_cases"] += 1
        case_result["variants"].append(
            {
                "label": "metadata",
                "status": "fail",
                "mismatches": sorted(set(mismatches)),
            }
        )
        result["cases"].append(case_result)
        return result

    assert isinstance(iterations, int)
    assert isinstance(random_seed, int)
    assert isinstance(min_unique, int)
    assert isinstance(allow_exit_codes, list)

    generated: list[str] = []
    for idx, source in enumerate(sources):
        generated.extend(
            deterministic_mutations(
                seed_source=source,
                iterations=max(1, iterations // max(1, len(sources))),
                random_seed=random_seed + (idx * 17),
            )
        )

    if len(generated) < iterations:
        while len(generated) < iterations:
            generated.extend(
                deterministic_mutations(
                    seed_source=sources[len(generated) % len(sources)],
                    iterations=1,
                    random_seed=random_seed + len(generated),
                )
            )
    generated = generated[:iterations]

    unique_hashes: set[str] = set()
    case_failed = False
    for i, snippet in enumerate(generated, start=1):
        snippet_hash = hashlib.sha256(snippet.encode("utf-8")).hexdigest()[:16]
        unique_hashes.add(snippet_hash)
        tmp_file = repo_root / "target/verification/tmp" / f"fuzz_smoke_{i:03d}_{snippet_hash}.sifr"
        write_text(tmp_file, snippet)

        exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
            repo_root=repo_root,
            command_name=str(payload["command"]),
            entry=tmp_file,
            diagnostic_format=str(payload["diagnostic_format"]),
        )
        stdout_norm = canonicalize_output(
            repo_root=repo_root,
            text=stdout,
            diagnostic_format=str(payload["diagnostic_format"]),
            stream="stdout",
        )
        stderr_norm = canonicalize_output(
            repo_root=repo_root,
            text=stderr,
            diagnostic_format=str(payload["diagnostic_format"]),
            stream="stderr",
        )

        run_mismatches: list[str] = []
        if exit_code not in allow_exit_codes:
            run_mismatches.append("unexpected-exit")
        if assert_no_panic and contains_internal_panic(stdout_norm + stderr_norm):
            run_mismatches.append("panic-signal")

        status = "pass" if not run_mismatches else "fail"
        result["total_variants"] += 1
        if run_mismatches:
            case_failed = True
            result["total_failures"] += 1

        case_result["variants"].append(
            {
                "label": f"fuzz-{i:03d}",
                "status": status,
                "mismatches": run_mismatches,
                "source_hash": snippet_hash,
                "actual_exit_code": exit_code,
                "duration_ms": round(elapsed_ms, 3),
                "argv": argv,
            }
        )

    uniqueness_mismatch: list[str] = []
    if len(unique_hashes) < min_unique:
        uniqueness_mismatch.append("insufficient-unique-cases")
        case_failed = True
        result["total_failures"] += 1

    result["total_variants"] += 1
    case_result["variants"].append(
        {
            "label": "uniqueness",
            "status": "pass" if not uniqueness_mismatch else "fail",
            "mismatches": uniqueness_mismatch,
            "unique_cases": len(unique_hashes),
            "required_min_unique_cases": min_unique,
        }
    )

    if case_failed:
        result["failed_cases"] += 1
    result["cases"].append(case_result)
    return result


def run_oss_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
    runner_name: str,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": runner_name,
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    allowed_classifications = {"pass", "known-failure", "investigate"}
    pinned_revision_cache: dict[str, str | None] = {}

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        case_result = {
            "id": case_id,
            "project_root": entry.get("project_root"),
            "pinned_revision": entry.get("pinned_revision"),
            "expected_result_classification": entry.get("expected_result_classification"),
            "variants": [],
        }

        mismatches = required_missing(
            entry,
            (
                "id",
                "project_root",
                "pinned_revision",
                "owner",
                "rationale",
                "expected_result_classification",
            ),
        )
        classification = entry.get("expected_result_classification")
        if classification not in allowed_classifications:
            mismatches.append("expected_result_classification")

        project_root_raw = entry.get("project_root")
        pinned_revision_raw = entry.get("pinned_revision")
        commands = entry.get("commands")
        project_root = repo_root / str(project_root_raw) if isinstance(project_root_raw, str) else None
        if project_root is None or not project_root.is_dir():
            mismatches.append("project_root")
        if not isinstance(commands, list) or not commands:
            mismatches.append("commands")

        case_failed = False
        if mismatches:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "metadata",
                    "status": "fail",
                    "mismatches": sorted(set(mismatches)),
                }
            )
            result["cases"].append(case_result)
            continue

        assert project_root is not None
        assert isinstance(project_root_raw, str)
        assert isinstance(pinned_revision_raw, str)
        pinned_match = LOCAL_PINNED_REVISION_PATTERN.fullmatch(pinned_revision_raw)
        if pinned_match is None:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "pinned-revision",
                    "status": "fail",
                    "mismatches": ["pinned_revision_format"],
                }
            )
            result["cases"].append(case_result)
            continue
        expected_sha = pinned_match.group(1)
        latest_sha = pinned_revision_cache.get(project_root_raw)
        if project_root_raw not in pinned_revision_cache:
            latest_sha = latest_project_revision(repo_root, project_root_raw)
            pinned_revision_cache[project_root_raw] = latest_sha
        if latest_sha is None:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "pinned-revision",
                    "status": "fail",
                    "mismatches": ["pinned_revision_unresolvable"],
                }
            )
            result["cases"].append(case_result)
            continue
        if not latest_sha.startswith(expected_sha):
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "pinned-revision",
                    "status": "fail",
                    "mismatches": ["pinned_revision_mismatch"],
                    "expected_pinned_revision": pinned_revision_raw,
                    "latest_project_revision": f"local-main@{latest_sha[:len(expected_sha)]}",
                }
            )
            result["cases"].append(case_result)
            continue
        result["total_variants"] += 1
        case_result["variants"].append(
            {
                "label": "pinned-revision",
                "status": "pass",
                "mismatches": [],
                "expected_pinned_revision": pinned_revision_raw,
                "latest_project_revision": f"local-main@{latest_sha[:len(expected_sha)]}",
            }
        )

        for idx, command_meta in enumerate(commands, start=1):
            if not isinstance(command_meta, dict):
                result["total_variants"] += 1
                result["total_failures"] += 1
                case_failed = True
                case_result["variants"].append(
                    {
                        "label": f"command-{idx}",
                        "status": "fail",
                        "mismatches": ["command-metadata"],
                    }
                )
                continue

            command_name = command_meta.get("command")
            entrypoint_raw = command_meta.get("entrypoint")
            expected_exit = command_meta.get("expect_exit_code")
            timeout_secs = command_meta.get("timeout_secs")

            command_mismatches: list[str] = []
            if command_name not in {"check", "run", "build", "test"}:
                command_mismatches.append("command")
            if not isinstance(entrypoint_raw, str) or not entrypoint_raw:
                command_mismatches.append("entrypoint")
            if not isinstance(expected_exit, int):
                command_mismatches.append("expect_exit_code")
            if not isinstance(timeout_secs, int) or timeout_secs < 1:
                command_mismatches.append("timeout_secs")

            entrypoint_path = project_root / str(entrypoint_raw) if isinstance(entrypoint_raw, str) else None
            if entrypoint_path is None or not entrypoint_path.is_file():
                command_mismatches.append("entrypoint")

            if command_mismatches:
                result["total_variants"] += 1
                result["total_failures"] += 1
                case_failed = True
                case_result["variants"].append(
                    {
                        "label": f"command-{idx}",
                        "status": "fail",
                        "mismatches": sorted(set(command_mismatches)),
                    }
                )
                continue

            assert entrypoint_path is not None
            assert isinstance(timeout_secs, int)
            exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
                repo_root=repo_root,
                command_name=str(command_name),
                entry=entrypoint_path,
                diagnostic_format=None,
                timeout_secs=timeout_secs,
            )
            stdout_norm = normalize_string(stdout, repo_root)
            stderr_norm = normalize_string(stderr, repo_root)

            variant_mismatches: list[str] = []
            if exit_code != expected_exit:
                variant_mismatches.append("unexpected-exit")
            if contains_internal_panic(stdout_norm + stderr_norm):
                variant_mismatches.append("panic-signal")

            result["total_variants"] += 1
            status = "pass" if not variant_mismatches else "fail"
            if variant_mismatches:
                case_failed = True
                result["total_failures"] += 1

            case_result["variants"].append(
                {
                    "label": f"{command_name}-{idx}",
                    "status": status,
                    "mismatches": variant_mismatches,
                    "argv": argv,
                    "expected_exit_code": expected_exit,
                    "actual_exit_code": exit_code,
                    "duration_ms": round(elapsed_ms, 3),
                    "timeout_secs": timeout_secs,
                }
            )

        if case_failed:
            result["failed_cases"] += 1
        result["cases"].append(case_result)

    return result


def run_external_command(
    *,
    repo_root: Path,
    argv: list[str],
    timeout_secs: int | None,
) -> tuple[int, str, str, float]:
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            argv,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
            timeout=timeout_secs,
        )
        exit_code = proc.returncode
        stdout = proc.stdout
        stderr = proc.stderr
    except subprocess.TimeoutExpired as timeout_error:
        exit_code = 124
        stdout = timeout_error.stdout or ""
        stderr = (timeout_error.stderr or "") + f"\ncommand timed out after {timeout_secs} seconds"
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    return exit_code, stdout, stderr, elapsed_ms


def run_determinism_scale_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "determinism-scale",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        description = entry.get("description")
        command_raw = entry.get("command")
        expected_exit = entry.get("expect_exit_code")
        timeout_secs = entry.get("timeout_secs")
        mismatches = []

        if not isinstance(description, str) or not description:
            mismatches.append("description")
        if not isinstance(command_raw, list) or not command_raw or not all(
            isinstance(token, str) and token for token in command_raw
        ):
            mismatches.append("command")
        if not isinstance(expected_exit, int):
            mismatches.append("expect_exit_code")
        if not isinstance(timeout_secs, int) or timeout_secs < 1:
            mismatches.append("timeout_secs")

        case_result = {
            "id": case_id,
            "description": description,
            "variants": [],
        }
        case_failed = False

        if mismatches:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "metadata",
                    "status": "fail",
                    "mismatches": sorted(set(mismatches)),
                }
            )
            result["cases"].append(case_result)
            continue

        assert isinstance(command_raw, list)
        assert isinstance(timeout_secs, int)
        exit_code, stdout, stderr, elapsed_ms = run_external_command(
            repo_root=repo_root,
            argv=command_raw,
            timeout_secs=timeout_secs,
        )
        stdout_norm = normalize_string(stdout, repo_root)
        stderr_norm = normalize_string(stderr, repo_root)
        variant_mismatches: list[str] = []
        if exit_code != expected_exit:
            variant_mismatches.append("unexpected-exit")
        if contains_internal_panic(stdout_norm + stderr_norm):
            variant_mismatches.append("panic-signal")

        status = "pass" if not variant_mismatches else "fail"
        result["total_variants"] += 1
        if variant_mismatches:
            case_failed = True
            result["total_failures"] += 1
        case_result["variants"].append(
            {
                "label": "command",
                "status": status,
                "mismatches": variant_mismatches,
                "argv": command_raw,
                "expected_exit_code": expected_exit,
                "actual_exit_code": exit_code,
                "duration_ms": round(elapsed_ms, 3),
                "timeout_secs": timeout_secs,
            }
        )

        result["cases"].append(case_result)
        if case_failed:
            result["failed_cases"] += 1

    return result


def deterministic_suite_shard(name: str, shard_total: int) -> int:
    if shard_total <= 1:
        return 0
    digest = hashlib.sha256(name.encode("utf-8")).hexdigest()
    return int(digest[:16], 16) % shard_total


def load_quarantine_metadata(path: Path, suites: list[dict[str, Any]]) -> list[dict[str, Any]]:
    if not path.is_file():
        raise SystemExit(f"quarantine file not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    entries = payload.get("entries", [])
    if not isinstance(entries, list):
        raise SystemExit(f"invalid quarantine file '{path}': 'entries' must be a list")
    suite_names = {suite.get("name") for suite in suites}
    validated: list[dict[str, Any]] = []
    for entry in entries:
        if not isinstance(entry, dict):
            raise SystemExit(f"invalid quarantine entry in '{path}': expected object")
        missing = required_missing(
            entry,
            (
                "suite",
                "case_id",
                "reason",
                "owner",
                "added_on",
                "reenable_criteria",
            ),
        )
        if missing:
            raise SystemExit(
                f"invalid quarantine entry in '{path}': missing fields {', '.join(sorted(set(missing)))}"
            )
        if entry.get("suite") not in suite_names:
            raise SystemExit(
                f"invalid quarantine entry in '{path}': unknown suite '{entry.get('suite')}'"
            )
        validated.append(entry)
    return validated


def failed_case_ids(suite_result: dict[str, Any]) -> set[str]:
    failed: set[str] = set()
    for case in suite_result.get("cases", []):
        if not isinstance(case, dict):
            continue
        case_id = case.get("id")
        variants = case.get("variants", [])
        if isinstance(case_id, str) and isinstance(variants, list):
            if any(isinstance(variant, dict) and variant.get("status") == "fail" for variant in variants):
                failed.add(case_id)
    return failed


def main() -> int:
    args = parse_args()
    args.profile = canonicalize_profile(args.profile)
    if args.shard_total < 1:
        raise SystemExit("--shard-total must be >= 1")
    if args.shard_index < 0 or args.shard_index >= args.shard_total:
        raise SystemExit("--shard-index must satisfy 0 <= shard-index < shard-total")
    if args.rerun_failures < 0:
        raise SystemExit("--rerun-failures must be >= 0")

    repo_root = Path(__file__).resolve().parent.parent
    manifest_path = (repo_root / args.manifest).resolve()
    result_json_path = (repo_root / args.result_json).resolve()
    quarantine_path = (repo_root / args.quarantine_file).resolve()
    actual_root = repo_root / "target/verification/actual"
    actual_root.mkdir(parents=True, exist_ok=True)

    if not manifest_path.is_file():
        raise SystemExit(f"verification manifest not found: {manifest_path}")

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    suites = manifest.get("suites", [])
    if not isinstance(suites, list):
        raise SystemExit("invalid manifest: 'suites' must be a list")

    selected_suites = []
    explicit_suites = set(args.suite)
    for suite in suites:
        name = suite.get("name")
        if not isinstance(name, str):
            raise SystemExit("invalid manifest: suite missing string 'name'")
        if explicit_suites and name not in explicit_suites:
            continue
        if not explicit_suites and not should_run_suite(args.profile, name):
            continue
        selected_suites.append(suite)

    if explicit_suites:
        missing = sorted(explicit_suites.difference({suite.get("name") for suite in suites}))
        if missing:
            raise SystemExit(f"unknown suite filter(s): {', '.join(missing)}")

    if not selected_suites:
        raise SystemExit("no verification suites selected")

    selected_suites = [
        suite
        for suite in selected_suites
        if deterministic_suite_shard(str(suite.get("name")), args.shard_total) == args.shard_index
    ]

    quarantine_entries = load_quarantine_metadata(quarantine_path, suites)

    run_results: list[dict[str, Any]] = []
    total_variants = 0
    total_failures = 0
    blocking_failures = 0
    non_blocking_failures = 0

    print("Running phase-29 verification suites")
    print(f"  profile={args.profile}")
    print(f"  manifest={manifest_path.relative_to(repo_root)}")
    print(f"  bless={'yes' if args.bless else 'no'}")
    print(f"  shard={args.shard_index}/{args.shard_total}")
    print(f"  rerun_failures={args.rerun_failures}")
    print(f"  quarantine_entries={len(quarantine_entries)}")

    fixedbug_ids = collect_fixedbug_ids(repo_root, selected_suites)

    def execute_suite_once(suite: dict[str, Any]) -> dict[str, Any]:
        runner = str(suite.get("runner", "baseline"))
        if runner == "baseline":
            return run_baseline_suite(
                suite=suite,
                args=args,
                repo_root=repo_root,
                actual_root=actual_root,
            )
        if runner == "fixedbugs":
            return run_fixedbugs_suite(
                suite=suite,
                repo_root=repo_root,
                actual_root=actual_root,
            )
        if runner == "crashes":
            return run_crashes_suite(
                suite=suite,
                repo_root=repo_root,
                fixedbug_ids=fixedbug_ids,
            )
        if runner == "property":
            return run_property_suite(
                suite=suite,
                repo_root=repo_root,
            )
        if runner == "fuzz-smoke":
            return run_fuzz_smoke_suite(
                suite=suite,
                repo_root=repo_root,
            )
        if runner == "oss-curated":
            return run_oss_suite(
                suite=suite,
                repo_root=repo_root,
                runner_name="oss-curated",
            )
        if runner == "ecosystem-broader":
            return run_oss_suite(
                suite=suite,
                repo_root=repo_root,
                runner_name="ecosystem-broader",
            )
        if runner == "determinism-scale":
            return run_determinism_scale_suite(
                suite=suite,
                repo_root=repo_root,
            )
        raise SystemExit(f"unsupported runner '{runner}' for suite '{suite.get('name', '<unknown>')}'")

    for suite in selected_suites:
        suite_result = execute_suite_once(suite)
        suite_name = str(suite.get("name"))
        suite_quarantine = [entry for entry in quarantine_entries if entry.get("suite") == suite_name]
        if suite_quarantine:
            suite_result["quarantine_entries"] = suite_quarantine

        if not args.bless and args.rerun_failures > 0 and int(suite_result.get("total_failures", 0)) > 0:
            initial_failed = failed_case_ids(suite_result)
            rerun_attempts: list[dict[str, Any]] = []
            flake_events: list[dict[str, Any]] = []
            previous_failed = set(initial_failed)
            for attempt in range(1, args.rerun_failures + 1):
                rerun_result = execute_suite_once(suite)
                rerun_failed = failed_case_ids(rerun_result)
                transitioned = sorted(previous_failed.difference(rerun_failed))
                rerun_attempt = {
                    "attempt": attempt,
                    "failed_case_count": len(rerun_failed),
                    "failed_cases": sorted(rerun_failed),
                    "total_failures": int(rerun_result.get("total_failures", 0)),
                }
                if transitioned:
                    rerun_attempt["flaky_fail_to_pass_cases"] = transitioned
                    flake_events.append(
                        {
                            "attempt": attempt,
                            "flaky_fail_to_pass_cases": transitioned,
                        }
                    )
                rerun_attempts.append(rerun_attempt)
                previous_failed = rerun_failed
                if not rerun_failed:
                    break
            suite_result["rerun_attempts"] = rerun_attempts
            if flake_events:
                suite_result["flake_events"] = flake_events

        run_results.append(suite_result)
        total_variants += int(suite_result.get("total_variants", 0))
        suite_failures = int(suite_result.get("total_failures", 0))
        total_failures += suite_failures
        if suite_failures > 0:
            if bool(suite_result.get("blocking")):
                blocking_failures += suite_failures
            else:
                non_blocking_failures += suite_failures

    result_payload = {
        "schema_version": 1,
        "profile": args.profile,
        "bless": args.bless,
        "manifest": str(manifest_path.relative_to(repo_root)),
        "shard_total": args.shard_total,
        "shard_index": args.shard_index,
        "rerun_failures": args.rerun_failures,
        "quarantine_file": str(quarantine_path.relative_to(repo_root)),
        "quarantine_entry_count": len(quarantine_entries),
        "generated_at_unix_secs": int(time.time()),
        "suites": run_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": blocking_failures,
            "non_blocking_failures": non_blocking_failures,
        },
    }
    result_json_path.parent.mkdir(parents=True, exist_ok=True)
    result_json_path.write_text(json.dumps(result_payload, indent=2, sort_keys=True), encoding="utf-8")

    if args.bless:
        print("baselines updated")
    else:
        print(f"result_json={result_json_path.relative_to(repo_root)}")

    if blocking_failures > 0 and not args.bless:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={blocking_failures}, non_blocking_failures={non_blocking_failures}",
            file=sys.stderr,
        )
        print("actual outputs written under target/verification/actual", file=sys.stderr)
        return 1

    print(
        f"verification ok: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures={blocking_failures}, non_blocking_failures={non_blocking_failures}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
