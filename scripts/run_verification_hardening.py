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
        return suite_name in {"diagnostics", "project"}
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

    for suite in selected_suites:
        suite_name = suite["name"]
        suite_owner = suite.get("owner", "unknown")
        suite_blocking = bool(suite.get("blocking", False))
        cases = suite.get("cases", [])
        if not isinstance(cases, list) or not cases:
            raise SystemExit(f"suite '{suite_name}' has no cases")

        suite_result = {
            "name": suite_name,
            "owner": suite_owner,
            "blocking": suite_blocking,
            "cases": [],
            "failed_cases": 0,
            "total_variants": 0,
        }
        print(f"  suite={suite_name} owner={suite_owner} cases={len(cases)}")

        for case in cases:
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
                raise SystemExit(
                    f"suite '{suite_name}' case '{case_id}' missing integer 'expect_exit_code'"
                )

            entry_path = repo_root / case_entry
            if not entry_path.is_file():
                raise SystemExit(
                    f"suite '{suite_name}' case '{case_id}' entry does not exist: {entry_path}"
                )

            formats: list[str | None]
            if diagnostic_formats is None:
                formats = [None]
            else:
                if not isinstance(diagnostic_formats, list) or not diagnostic_formats:
                    raise SystemExit(
                        f"suite '{suite_name}' case '{case_id}' has invalid diagnostic_formats"
                    )
                formats = [str(fmt) for fmt in diagnostic_formats]

            case_failed = False
            case_result = {
                "id": case_id,
                "entry": str(entry_path.relative_to(repo_root)),
                "command": command_name,
                "variants": [],
            }

            for diagnostic_format in formats:
                label = f"{command_name}-{diagnostic_format}" if diagnostic_format else command_name
                total_variants += 1
                suite_result["total_variants"] += 1

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
                    missing_files = [
                        path
                        for path in (stdout_file, stderr_file, exit_file)
                        if not path.is_file()
                    ]
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
                    total_failures += 1

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

            if case_failed:
                suite_result["failed_cases"] += 1
                if suite_blocking:
                    blocking_failures += 1
            suite_result["cases"].append(case_result)

        run_results.append(suite_result)

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
