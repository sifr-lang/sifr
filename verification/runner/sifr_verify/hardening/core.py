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
ARTIFACT_CACHE_LINE_PATTERN = re.compile(r"^\[sifr-artifact-cache\].*$")
BASELINE_COMMANDS = {"check", "run", "build", "test"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        default="verification/runner/sifr_verify/hardening/data/determinism_suites.json",
        help="Path to suite manifest JSON.",
    )
    parser.add_argument(
        "--profile",
        choices=("create-pr", "merge", "nightly", "release"),
        default="merge",
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
        default="verification/policy/flake_quarantine.json",
        help="Path to quarantine metadata file.",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run verification harness self-tests without executing suites.",
    )
    return parser.parse_args()


def normalize_string(value: str, repo_root: Path) -> str:
    normalized = value.replace("\r\n", "\n").replace("\r", "\n")
    normalized = normalized.replace(str(repo_root), "<WORKSPACE>")
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
    if profile == "create-pr":
        return False
    if profile == "merge":
        return suite_name in {
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


def validate_unique_diagnostic_formats(
    *,
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


def baseline_variant_label(command_name: str, diagnostic_format: str | None) -> str:
    return f"{command_name}-{diagnostic_format}" if diagnostic_format else command_name


def baseline_artifact_paths(entry_path: Path, label: str) -> tuple[Path, Path, Path]:
    baseline_dir = entry_path.parent / "baselines"
    return (
        baseline_dir / f"{label}.stdout.txt",
        baseline_dir / f"{label}.stderr.txt",
        baseline_dir / f"{label}.exit-code.txt",
    )


def baseline_artifact_key(path: Path) -> Path:
    return path.resolve()


def format_repo_relative_path(path: Path, repo_root: Path) -> str:
    try:
        return str(path.relative_to(repo_root))
    except ValueError:
        return str(path)


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


def baseline_case_metadata(
    *,
    suite_name: str,
    case: Any,
    repo_root: Path,
) -> tuple[str, Path, str, list[str | None]]:
    if not isinstance(case, dict):
        raise SystemExit(f"suite '{suite_name}' has non-object case entry")
    case_id = case.get("id")
    case_entry = case.get("entry")
    command_name = case.get("command")
    diagnostic_formats = case.get("diagnostic_formats")
    if not isinstance(case_id, str):
        raise SystemExit(f"suite '{suite_name}' has case without valid 'id'")
    if not isinstance(case_entry, str):
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' missing string 'entry'")
    if Path(case_entry).is_absolute():
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' entry must be repo-relative")
    if command_name not in BASELINE_COMMANDS:
        raise SystemExit(
            f"suite '{suite_name}' case '{case_id}' has unsupported command '{command_name}'"
        )
    formats = parse_formats(diagnostic_formats)
    if not formats:
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' has invalid diagnostic_formats")
    validate_unique_diagnostic_formats(
        suite_name=suite_name,
        case_id=case_id,
        formats=formats,
    )

    entry_path = (repo_root / case_entry).resolve()
    try:
        entry_path.relative_to(repo_root)
    except ValueError as error:
        raise SystemExit(
            f"suite '{suite_name}' case '{case_id}' entry must stay under repo root"
        ) from error
    return case_id, entry_path, command_name, formats


def validate_unique_baseline_artifact_paths(
    *,
    suite_name: str,
    cases: list[Any],
    repo_root: Path,
) -> None:
    seen: dict[Path, str] = {}
    for case in cases:
        case_id, entry_path, command_name, formats = baseline_case_metadata(
            suite_name=suite_name,
            case=case,
            repo_root=repo_root,
        )
        for diagnostic_format in formats:
            label = baseline_variant_label(command_name, diagnostic_format)
            for artifact_path in baseline_artifact_paths(entry_path, label):
                key = baseline_artifact_key(artifact_path)
                previous = seen.get(key)
                owner = f"{case_id}:{label}"
                if previous is not None:
                    rel = format_repo_relative_path(key, repo_root)
                    raise SystemExit(
                        f"suite '{suite_name}' baseline artifact path collision for {rel}: "
                        f"{previous} and {owner}"
                    )
                seen[key] = owner


def assert_self_test_failure(description: str, expected: str, callback: Any) -> None:
    try:
        callback()
    except SystemExit as error:
        message = str(error)
        if expected not in message:
            raise AssertionError(
                f"{description}: expected failure containing {expected!r}, got {message!r}"
            ) from error
        return
    raise AssertionError(f"{description}: expected SystemExit")
