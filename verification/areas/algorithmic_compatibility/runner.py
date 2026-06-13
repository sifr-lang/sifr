"""Algorithmic compatibility verification area adapter."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
MANIFEST_PATH = AREA_ROOT / "manifest.json"
RESULT_JSON = (
    REPO_ROOT
    / "target"
    / "verification"
    / "areas"
    / "algorithmic-compatibility-results.json"
)
LEETCODE_ROOT = AREA_ROOT / "corpora" / "leetcode" / "src"
TAXONOMY_SMOKE_RESULTS = AREA_ROOT / "data" / "taxonomy_smoke_results.json"
DEFAULT_SIFR_BIN = REPO_ROOT / "target" / "debug" / "sifr"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable algorithmic compatibility result summary.",
    )
    parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit a legacy verification summary line for direct area invocations.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("algorithmic_compatibility area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running algorithmic compatibility verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "algorithmic_compatibility",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": total_failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = REPO_ROOT / args.result_json
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}", flush=True)

    if total_failures:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={total_failures}, non_blocking_failures=0",
            file=sys.stderr,
            flush=True,
        )
        return 1
    prefix = (
        "verification ok"
        if args.hardening_summary
        else "algorithmic compatibility verification ok"
    )
    print(
        f"{prefix}: variants={total_variants}, failures={total_failures}, "
        "blocking_failures=0, non_blocking_failures=0",
        flush=True,
    )
    return 0


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [
        suite for suite in suites if not requested or str(suite.get("name")) in requested
    ]
    if requested:
        present = {str(suite.get("name")) for suite in selected}
        missing = sorted(requested.difference(present))
        if missing:
            raise SystemExit(
                f"unknown algorithmic_compatibility suite filter(s): {', '.join(missing)}"
            )
    if not selected:
        raise SystemExit("no algorithmic_compatibility suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit(
            f"algorithmic_compatibility suite '{suite_name}' must contain exactly one case"
        )
    case = cases[0]
    variant = run_case(case)
    failures = 1 if variant["status"] == "fail" else 0
    return {
        "name": suite_name,
        "owner": "algorithmic/compatibility",
        "blocking": True,
        "runner": "algorithmic-compatibility",
        "cases": [
            {
                "id": str(case["id"]),
                "entry": str(case["entry"]),
                "command": str(case["command"]),
                "variants": [variant],
            }
        ],
        "failed_cases": failures,
        "total_variants": 1,
        "total_failures": failures,
    }


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    entry = REPO_ROOT / str(case["entry"])
    if not entry.exists():
        raise SystemExit(f"algorithmic_compatibility case entry does not exist: {entry}")
    command = str(case["command"])
    expected_exit = int(case.get("expect_exit_code", 0))
    started = time.perf_counter()
    if command == "algorithmic-taxonomy-smoke":
        argv, result = run_taxonomy_smoke()
    elif command == "algorithmic-leetcode-check":
        argv, result = run_leetcode_check()
    else:
        raise SystemExit(f"unsupported algorithmic_compatibility command: {command}")
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    failures = []
    if result.returncode != expected_exit:
        failures.append(f"exit={result.returncode} expected={expected_exit}")
    if result.stdout:
        sys.stdout.write(result.stdout)
    if result.stderr:
        sys.stderr.write(result.stderr)
    status = "fail" if failures else "pass"
    print_case_timing(str(case["id"]), elapsed_ms, status)
    return {
        "label": str(case["id"]),
        "argv": argv,
        "status": status,
        "mismatches": failures,
        "expected_exit_code": expected_exit,
        "actual_exit_code": result.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def run_taxonomy_smoke() -> tuple[list[str], subprocess.CompletedProcess[str]]:
    output_json = (
        REPO_ROOT
        / "target"
        / "verification"
        / "areas"
        / "algorithmic_compatibility"
        / "taxonomy-smoke.json"
    )
    output_md = output_json.with_suffix(".md")
    argv = [
        "python3",
        str(AREA_ROOT / "tools" / "build_full_corpus_failure_taxonomy.py"),
        "--results",
        str(TAXONOMY_SMOKE_RESULTS.relative_to(REPO_ROOT)),
        "--output-json",
        str(output_json.relative_to(REPO_ROOT)),
        "--output-md",
        str(output_md.relative_to(REPO_ROOT)),
        "--name",
        "taxonomy-smoke",
        "--generated-on",
        "2026-06-13",
    ]
    result = subprocess.run(argv, cwd=REPO_ROOT, text=True, capture_output=True, check=False)
    if result.returncode == 0:
        payload = json.loads(output_json.read_text(encoding="utf-8"))
        category_counts = payload["summary"]["category_counts"]
        if payload["summary"]["failing_cases"] != 1:
            result = completed_with_failure(argv, "taxonomy smoke expected exactly one failing case")
        elif category_counts.get("python_stdlib_and_builtin_parity_gap") != 1:
            result = completed_with_failure(argv, "taxonomy smoke category classification changed")
    return argv, result


def run_leetcode_check() -> tuple[list[str], subprocess.CompletedProcess[str]]:
    if not LEETCODE_ROOT.is_dir():
        raise SystemExit(f"missing LeetCode corpus root: {LEETCODE_ROOT.relative_to(REPO_ROOT)}")
    paths = sorted(LEETCODE_ROOT.glob("*.sifr"))
    if not paths:
        raise SystemExit(f"no LeetCode fixtures found under {LEETCODE_ROOT.relative_to(REPO_ROOT)}")
    ensure_sifr_bin(DEFAULT_SIFR_BIN)
    failures: list[tuple[Path, subprocess.CompletedProcess[str]]] = []
    stdout_lines = [f"validating {len(paths)} LeetCode fixture(s) with sifr check"]
    started = time.monotonic()
    for index, path in enumerate(paths, start=1):
        result = run_fixture(DEFAULT_SIFR_BIN, path)
        if result.returncode == 0:
            stdout_lines.append(f"[{index}/{len(paths)}] PASS {path.relative_to(REPO_ROOT)}")
            continue
        stdout_lines.append(f"[{index}/{len(paths)}] FAIL {path.relative_to(REPO_ROOT)}")
        failures.append((path, result))
    elapsed = time.monotonic() - started
    stdout_lines.append(
        f"LeetCode corpus check completed: {len(paths) - len(failures)}/{len(paths)} passed in {elapsed:.1f}s"
    )
    stderr = render_failures(failures)
    argv = [str(DEFAULT_SIFR_BIN), "check", str(LEETCODE_ROOT.relative_to(REPO_ROOT))]
    return argv, subprocess.CompletedProcess(
        args=argv,
        returncode=1 if failures else 0,
        stdout="\n".join(stdout_lines) + "\n",
        stderr=stderr,
    )


def ensure_sifr_bin(sifr_bin: Path) -> None:
    if sifr_bin == DEFAULT_SIFR_BIN:
        subprocess.run(["cargo", "build", "-q", "-p", "sifr"], cwd=REPO_ROOT, check=True)
    elif not sifr_bin.exists():
        raise SystemExit(f"missing Sifr CLI binary: {sifr_bin}")


def run_fixture(sifr_bin: Path, path: Path) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("SIFR_ARTIFACT_CACHE", "1")
    return subprocess.run(
        [str(sifr_bin), "check", str(path.relative_to(REPO_ROOT))],
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def render_failures(failures: list[tuple[Path, subprocess.CompletedProcess[str]]]) -> str:
    lines: list[str] = []
    for path, result in failures:
        lines.append(f"--- {path.relative_to(REPO_ROOT)} exit={result.returncode} ---")
        if result.stdout:
            lines.append(result.stdout[-4000:])
        if result.stderr:
            lines.append(result.stderr[-4000:])
    return "\n".join(lines)


def completed_with_failure(argv: list[str], stderr: str) -> subprocess.CompletedProcess[str]:
    return subprocess.CompletedProcess(args=argv, returncode=1, stdout="", stderr=f"{stderr}\n")


def print_case_timing(label: str, elapsed_ms: float, status: str) -> None:
    print(
        f"[sifr-case-timing] bucket=algorithmic_compatibility case={timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
