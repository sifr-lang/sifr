"""Stdlib parity verification area adapter."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
MANIFEST_PATH = AREA_ROOT / "manifest.json"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "stdlib-parity-results.json"

COMMAND_ARGS = {
    "stdlib-complexity-resource": [],
    "stdlib-namespace-demos-check": ["--scope", "demos", "--command", "check"],
    "stdlib-namespace-leetcode-check": ["--scope", "leetcode", "--command", "check"],
}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable stdlib parity result summary.",
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
        raise SystemExit("stdlib_parity area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running stdlib parity verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "stdlib_parity",
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
    prefix = "verification ok" if args.hardening_summary else "stdlib parity verification ok"
    print(
        f"{prefix}: variants={total_variants}, failures={total_failures}, "
        "blocking_failures=0, non_blocking_failures=0",
        flush=True,
    )
    return 0


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [suite for suite in suites if not requested or str(suite.get("name")) in requested]
    if requested:
        present = {str(suite.get("name")) for suite in selected}
        missing = sorted(requested.difference(present))
        if missing:
            raise SystemExit(f"unknown stdlib_parity suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no stdlib_parity suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit(f"stdlib_parity suite '{suite_name}' must contain exactly one case")
    case = cases[0]
    variant = run_case(case)
    failures = 1 if variant["status"] == "fail" else 0
    return {
        "name": suite_name,
        "owner": "stdlib/parity",
        "blocking": True,
        "runner": "stdlib-parity",
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
    if not entry.is_file():
        raise SystemExit(f"stdlib_parity case entry does not exist: {entry}")
    command = str(case["command"])
    if command not in COMMAND_ARGS:
        raise SystemExit(f"unsupported stdlib_parity command: {command}")
    expected_exit = int(case.get("expect_exit_code", 0))
    argv = ["python3", str(entry.relative_to(REPO_ROOT)), *COMMAND_ARGS[command]]
    started = time.perf_counter()
    result = subprocess.run(argv, cwd=REPO_ROOT, text=True, capture_output=True, check=False)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    if result.stdout:
        sys.stdout.write(result.stdout)
    if result.stderr:
        sys.stderr.write(result.stderr)
    failures = []
    if result.returncode != expected_exit:
        failures.append(f"exit={result.returncode} expected={expected_exit}")
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


def print_case_timing(label: str, elapsed_ms: float, status: str) -> None:
    print(
        f"[sifr-case-timing] bucket=stdlib_parity case={timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
