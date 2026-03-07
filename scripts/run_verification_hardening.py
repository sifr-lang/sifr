#!/usr/bin/env python3
"""Run phase-29 verification suites with baseline compare/bless support."""

from __future__ import annotations

import argparse
import json
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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        default="verification/suites/manifest.json",
        help="Path to suite manifest JSON.",
    )
    parser.add_argument(
        "--profile",
        choices=("quick", "full", "stress"),
        default="full",
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
    return parser.parse_args()


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
    if profile == "quick":
        return suite_name in {"diagnostics", "project", "fixedbugs", "crashes"}
    return True


def run_variant(
    *,
    repo_root: Path,
    command_name: str,
    entry: Path,
    diagnostic_format: str | None,
) -> tuple[int, str, str, float, list[str]]:
    args = ["cargo", "run", "-q", "-p", "sifr", "--"]
    if diagnostic_format is not None:
        args.extend(["--diagnostic-format", diagnostic_format])
    args.extend([command_name, str(entry)])

    started = time.perf_counter()
    proc = subprocess.run(
        args,
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    return proc.returncode, proc.stdout, proc.stderr, elapsed_ms, args


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
        "promotion_target_suite",
        "note",
    )

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        mismatches = required_missing(entry, required)
        status_raw = entry.get("status")
        source_ref = entry.get("source_reference")
        promotion_target = entry.get("promotion_target_suite")

        if status_raw not in {"unresolved", "promoted"}:
            mismatches.append("status")
        if status_raw == "unresolved":
            result["unresolved_count"] += 1
        if not isinstance(source_ref, str) or not (repo_root / source_ref).is_file():
            mismatches.append("source_reference")
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


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parent.parent
    manifest_path = (repo_root / args.manifest).resolve()
    result_json_path = (repo_root / args.result_json).resolve()
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

    run_results: list[dict[str, Any]] = []
    total_variants = 0
    total_failures = 0
    blocking_failures = 0

    print("Running phase-29 verification suites")
    print(f"  profile={args.profile}")
    print(f"  manifest={manifest_path.relative_to(repo_root)}")
    print(f"  bless={'yes' if args.bless else 'no'}")

    fixedbug_ids = collect_fixedbug_ids(repo_root, selected_suites)

    for suite in selected_suites:
        runner = str(suite.get("runner", "baseline"))
        if runner == "baseline":
            suite_result = run_baseline_suite(
                suite=suite,
                args=args,
                repo_root=repo_root,
                actual_root=actual_root,
            )
        elif runner == "fixedbugs":
            suite_result = run_fixedbugs_suite(
                suite=suite,
                repo_root=repo_root,
                actual_root=actual_root,
            )
        elif runner == "crashes":
            suite_result = run_crashes_suite(
                suite=suite,
                repo_root=repo_root,
                fixedbug_ids=fixedbug_ids,
            )
        else:
            raise SystemExit(
                f"unsupported runner '{runner}' for suite '{suite.get('name', '<unknown>')}'"
            )

        run_results.append(suite_result)
        total_variants += int(suite_result.get("total_variants", 0))
        total_failures += int(suite_result.get("total_failures", 0))
        if int(suite_result.get("failed_cases", 0)) > 0 and bool(suite_result.get("blocking")):
            blocking_failures += int(suite_result.get("failed_cases", 0))

    result_payload = {
        "schema_version": 1,
        "profile": args.profile,
        "bless": args.bless,
        "manifest": str(manifest_path.relative_to(repo_root)),
        "generated_at_unix_secs": int(time.time()),
        "suites": run_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": blocking_failures,
        },
    }
    result_json_path.parent.mkdir(parents=True, exist_ok=True)
    result_json_path.write_text(json.dumps(result_payload, indent=2, sort_keys=True), encoding="utf-8")

    if args.bless:
        print("baselines updated")
    else:
        print(f"result_json={result_json_path.relative_to(repo_root)}")

    if total_failures > 0 and not args.bless:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={blocking_failures}",
            file=sys.stderr,
        )
        print("actual outputs written under target/verification/actual", file=sys.stderr)
        return 1

    print(
        f"verification ok: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures={blocking_failures}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
