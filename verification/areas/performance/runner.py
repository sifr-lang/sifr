"""Performance verification area adapter."""

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
DATA_ROOT = AREA_ROOT / "data"
MANIFEST_PATH = AREA_ROOT / "manifest.json"
BENCHMARK_MANIFEST = DATA_ROOT / "benchmark_manifest.json"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "performance-results.json"
RUN_BENCHMARKS = AREA_ROOT / "run_benchmarks.py"
CHECK_BUDGETS = AREA_ROOT / "check_budgets.py"
CHECK_TREND_POLICY = AREA_ROOT / "check_trend_policy.py"

SMOKE_CASES = [
    "formatter-corpus-001-project-check",
    "formatter-large-file-001-check",
    "incremental-local-loop-001-unchanged-file-update",
    "interactive-tooling-foundation-002-warm-diagnostics-query",
    "lsp-query-003-diagnostics",
]
REPRESENTATIVE_CASES = [
    "check-single-file-001-arithmetic",
    "check-project-004-project-graph",
    "build-single-file-001-break-continue",
    "build-project-001-additional-modules",
    "formatter-corpus-001-project-check",
    "formatter-large-file-001-check",
    "incremental-local-loop-001-unchanged-file-update",
    "interactive-tooling-foundation-002-warm-diagnostics-query",
    "lsp-query-003-diagnostics",
    "diagnostic-non-regression-002-json-diagnostic-schema",
]


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable performance area result summary.",
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
        raise SystemExit("performance area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running performance verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print(f"  benchmark_manifest={BENCHMARK_MANIFEST.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "performance",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "benchmark_manifest": str(BENCHMARK_MANIFEST.relative_to(REPO_ROOT)),
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
    prefix = "verification ok" if args.hardening_summary else "performance verification ok"
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
            raise SystemExit(f"unknown performance suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no performance suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    variants: list[dict[str, Any]] = []
    if suite_name in {"smoke", "representative", "full"}:
        variants.extend(run_rules_variants(suite_name))
        variants.extend(run_profile_variants(suite_name))
        case_id = suite_name
        command = "performance-profile"
        entry = str(BENCHMARK_MANIFEST.relative_to(REPO_ROOT))
    else:
        for case in suite.get("cases", []):
            variants.append(run_manifest_case(suite_name, case))
        case_id = suite_name
        command = "area-suite"
        entry = str(MANIFEST_PATH.relative_to(REPO_ROOT))

    failures = sum(1 for variant in variants if variant["status"] != "pass")
    return {
        "name": suite_name,
        "owner": "compiler/performance",
        "blocking": True,
        "runner": "performance",
        "cases": [
            {
                "id": case_id,
                "entry": entry,
                "command": command,
                "variants": variants,
            }
        ],
        "failed_cases": 1 if failures else 0,
        "total_variants": len(variants),
        "total_failures": failures,
    }


def run_rules_variants(suite_name: str) -> list[dict[str, Any]]:
    return [
        run_command_variant(
            suite_name,
            "benchmark-manifest",
            [sys.executable, str(RUN_BENCHMARKS), "--validate-only"],
        ),
        run_command_variant(
            suite_name,
            "benchmark-runner-self-test",
            [sys.executable, str(RUN_BENCHMARKS), "--self-test"],
        ),
        run_command_variant(suite_name, "budget-policy", [sys.executable, str(CHECK_BUDGETS)]),
        run_command_variant(
            suite_name,
            "budget-policy-self-test",
            [sys.executable, str(CHECK_BUDGETS), "--self-test"],
        ),
        run_command_variant(suite_name, "trend-policy", [sys.executable, str(CHECK_TREND_POLICY)]),
        run_command_variant(
            suite_name,
            "trend-policy-self-test",
            [sys.executable, str(CHECK_TREND_POLICY), "--self-test"],
        ),
    ]


def run_profile_variants(suite_name: str) -> list[dict[str, Any]]:
    if suite_name == "smoke":
        argv = [sys.executable, str(RUN_BENCHMARKS), "--sample-scale", "smoke"]
        for case_id in SMOKE_CASES:
            argv.extend(["--case", case_id])
        return [run_command_variant(suite_name, "benchmark-smoke", argv)]

    results = f"target/performance/{suite_name}.budget.latest.json"
    run_argv = [sys.executable, str(RUN_BENCHMARKS), "--json-out", results]
    for case_id in REPRESENTATIVE_CASES:
        run_argv.extend(["--case", case_id])
    check_argv = [sys.executable, str(CHECK_BUDGETS), "--results", results, "--allow-subset"]
    return [
        run_command_variant(suite_name, "benchmark-subset", run_argv),
        run_command_variant(suite_name, "budget-subset", check_argv),
    ]


def run_manifest_case(suite_name: str, case: dict[str, Any]) -> dict[str, Any]:
    command = str(case["command"])
    entry = REPO_ROOT / str(case["entry"])
    if command == "python-script":
        return run_command_variant(suite_name, str(case["id"]), [sys.executable, str(entry)])
    if command == "python-script-self-test":
        return run_command_variant(suite_name, str(case["id"]), [sys.executable, str(entry), "--self-test"])
    if command == "benchmark-validate":
        return run_command_variant(suite_name, str(case["id"]), [sys.executable, str(RUN_BENCHMARKS), "--validate-only"])
    if command == "benchmark-self-test":
        return run_command_variant(suite_name, str(case["id"]), [sys.executable, str(RUN_BENCHMARKS), "--self-test"])
    if command == "budget-check":
        return run_command_variant(suite_name, str(case["id"]), [sys.executable, str(CHECK_BUDGETS)])
    if command == "budget-self-test":
        return run_command_variant(suite_name, str(case["id"]), [sys.executable, str(CHECK_BUDGETS), "--self-test"])
    raise SystemExit(f"unsupported performance case command: {command}")


def run_command_variant(suite_name: str, label: str, argv: list[str]) -> dict[str, Any]:
    started = time.perf_counter()
    proc = subprocess.run(argv, cwd=REPO_ROOT, text=True, check=False)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    status = "pass" if proc.returncode == 0 else "fail"
    print(
        f"[sifr-case-timing] bucket=performance case={timing_token(suite_name)}/{timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )
    return {
        "label": label,
        "argv": argv,
        "status": status,
        "mismatches": [] if status == "pass" else ["unexpected-exit"],
        "expected_exit_code": 0,
        "actual_exit_code": proc.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
