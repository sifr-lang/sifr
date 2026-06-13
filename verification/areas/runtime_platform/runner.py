"""Runtime platform verification area adapter."""

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
GOLDEN_MANIFEST = AREA_ROOT / "golden" / "manifest.json"
PLATFORM_CONTRACT = AREA_ROOT / "platform_contract.json"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "runtime-platform-results.json"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable runtime platform result summary.",
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
        raise SystemExit("runtime_platform area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running runtime platform verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "runtime_platform",
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
    prefix = "verification ok" if args.hardening_summary else "runtime platform verification ok"
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
            raise SystemExit(f"unknown runtime_platform suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no runtime_platform suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    if suite_name == "platform-golden":
        variants = run_platform_golden()
    elif suite_name == "platform-contract":
        variants = [run_contract_variant()]
    else:
        raise SystemExit(f"unsupported runtime_platform suite: {suite_name}")
    failures = sum(1 for variant in variants if variant["status"] == "fail")
    case = suite["cases"][0]
    return {
        "name": suite_name,
        "owner": "runtime/platform",
        "blocking": True,
        "runner": "runtime-platform",
        "cases": [
            {
                "id": str(case["id"]),
                "entry": str(case["entry"]),
                "command": str(case["command"]),
                "variants": variants,
            }
        ],
        "failed_cases": 1 if failures else 0,
        "total_variants": len(variants),
        "total_failures": failures,
    }


def run_contract_variant() -> dict[str, Any]:
    started = time.perf_counter()
    status = "pass"
    failures: list[str] = []
    try:
        payload = json.loads(PLATFORM_CONTRACT.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            failures.append("platform contract must be a JSON object")
    except Exception as exc:  # noqa: BLE001 - validation result captures parse failure.
        failures.append(str(exc))
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    if failures:
        status = "fail"
        for failure in failures:
            print(f"[platform-contract] fail {failure}", file=sys.stderr, flush=True)
    print_case_timing("platform-contract", "platform-contract", elapsed_ms, status)
    return {
        "label": "platform-contract",
        "argv": ["json-validate", str(PLATFORM_CONTRACT.relative_to(REPO_ROOT))],
        "status": status,
        "mismatches": failures,
        "expected_exit_code": 0,
        "actual_exit_code": 0 if status == "pass" else 1,
        "duration_ms": round(elapsed_ms, 3),
    }


def run_platform_golden() -> list[dict[str, Any]]:
    manifest = json.loads(GOLDEN_MANIFEST.read_text(encoding="utf-8"))
    closed = {item.strip() for item in os.environ.get("SIFR_PLATFORM_CLOSED_MILESTONES", "").split(",") if item.strip()}
    variants = []
    passed = 0
    skipped = 0
    for entry in manifest.get("entries", []):
        variant = run_platform_entry(entry, closed)
        variants.append(variant)
        if variant["status"] == "skip":
            skipped += 1
        elif variant["status"] == "pass":
            passed += 1
    print(f"[platform-golden] summary pass={passed} skip={skipped}", flush=True)
    return variants


def run_platform_entry(entry: dict[str, Any], closed: set[str]) -> dict[str, Any]:
    program = str(entry["program"])
    missing = [milestone for milestone in entry.get("blocked_until", []) if milestone not in closed]
    if missing:
        print(f"[platform-golden] skip {program} blocked_until={','.join(missing)}", flush=True)
        return {
            "label": program,
            "argv": [str(entry.get("command", ""))],
            "status": "skip",
            "mismatches": [],
            "expected_exit_code": int(entry.get("expected_exit", 0)),
            "actual_exit_code": None,
            "duration_ms": 0.0,
            "blocked_until": missing,
        }

    started = time.perf_counter()
    result = subprocess.run(
        str(entry["command"]),
        cwd=REPO_ROOT,
        shell=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    combined = result.stdout + result.stderr
    expected_exit = int(entry.get("expected_exit", 0))
    failures = []
    if result.returncode != expected_exit:
        failures.append(f"exit={result.returncode} expected={expected_exit}")
    for needle in entry.get("expected_stdout_contains", []):
        if needle not in result.stdout:
            failures.append(f"missing stdout: {needle}")
    for needle in entry.get("expected_diagnostic_contains", []):
        if needle not in combined:
            failures.append(f"missing diagnostic: {needle}")
    status = "fail" if failures else "pass"
    if failures:
        print(f"[platform-golden] fail {program} {'; '.join(failures)}", file=sys.stderr, flush=True)
        print(combined, file=sys.stderr, flush=True)
    else:
        print(f"[platform-golden] pass {program}", flush=True)
    print_case_timing("platform-golden", program, elapsed_ms, status)
    return {
        "label": program,
        "argv": [str(entry["command"])],
        "status": status,
        "mismatches": failures,
        "expected_exit_code": expected_exit,
        "actual_exit_code": result.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def print_case_timing(suite_name: str, label: str, elapsed_ms: float, status: str) -> None:
    print(
        f"[sifr-case-timing] bucket=runtime_platform case={timing_token(suite_name)}/{timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
