"""Schema-first SQL platform verification area adapter."""

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
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "sql-platform-results.json"
PLATFORM_CHECKER = AREA_ROOT / "tools" / "check_contracts.py"
BASELINE_RESOLVER = AREA_ROOT / "tools" / "resolve_dependency_baseline.py"
COMMANDS = {
    "sql-platform-contracts": [sys.executable, str(PLATFORM_CHECKER)],
    "sql-dependency-baseline": [sys.executable, str(BASELINE_RESOLVER), "--check"],
    "sql-platform-contract-mutations": [sys.executable, str(PLATFORM_CHECKER), "--self-test"],
    "sql-dependency-baseline-mutations": [sys.executable, str(BASELINE_RESOLVER), "--self-test"],
    "sql-component-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_component_qualification.py"),
    ],
    "sql-component-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_component_qualification.py"),
        "--self-test",
    ],
    "sql-schema-profile-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_schema_profiles.py"),
    ],
    "sql-schema-profile-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_schema_profiles.py"),
        "--self-test",
    ],
    "sql-schema-profile-rust-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test", "schema_profiles",
    ],
    "sql-schema-profile-driver-tests": [
        "cargo", "test", "--locked", "-p", "sifr_driver", "sql_profiles_tests",
    ],
    "sql-common-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_common_sql.py"),
    ],
    "sql-common-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_common_sql.py"),
        "--self-test",
    ],
    "sql-common-contract-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test", "common_sql_contracts",
    ],
    "sql-common-runtime-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_runtime",
    ],
    "sql-query-substrate-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_query_substrate.py"),
    ],
    "sql-query-substrate-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_query_substrate.py"),
        "--self-test",
    ],
    "sql-query-contract-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test", "query_fragments",
    ],
    "sql-query-runtime-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_runtime", "--test", "query_substrate",
    ],
    "sql-query-frontend-tests": [
        "cargo", "test", "--locked", "-p", "sifr_frontend", "--test", "sql_queries",
    ],
}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter. This option can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for runner parity. This area rejects it.")
    parser.add_argument("--result-json", default=str(RESULT_JSON.relative_to(REPO_ROOT)))
    return parser.parse_args(argv)


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [suite for suite in suites if not requested or suite.get("name") in requested]
    missing = sorted(requested.difference(str(suite.get("name")) for suite in selected))
    if missing:
        raise SystemExit(f"unknown SQL platform suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no SQL platform suites selected")
    return selected


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    name = str(case["command"])
    command = COMMANDS.get(name)
    if command is None:
        raise SystemExit(f"unsupported SQL platform command: {name}")
    started = time.perf_counter()
    result = subprocess.run(command, cwd=REPO_ROOT, check=False)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    status = "pass" if result.returncode == case["expect_exit_code"] else "fail"
    print(
        f"[sifr-case-timing] bucket=sql_platform case={case['id']} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )
    return {
        "id": case["id"],
        "entry": case["entry"],
        "command": name,
        "variants": [
            {
                "label": name,
                "argv": command,
                "status": status,
                "mismatches": [] if status == "pass" else ["unexpected-exit"],
                "expected_exit_code": case["expect_exit_code"],
                "actual_exit_code": result.returncode,
                "duration_ms": round(elapsed_ms, 3),
            }
        ],
    }


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    cases = [run_case(case) for case in suite["cases"]]
    failures = sum(variant["status"] != "pass" for case in cases for variant in case["variants"])
    variants = sum(len(case["variants"]) for case in cases)
    return {
        "name": suite["name"],
        "owner": "compiler/sql-platform",
        "blocking": True,
        "runner": "sql_platform",
        "cases": cases,
        "failed_cases": failures,
        "total_variants": variants,
        "total_failures": failures,
    }


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("sql_platform area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))
    print("Running schema-first SQL platform verification area", flush=True)
    results = [run_suite(suite) for suite in selected]
    variants = sum(int(result["total_variants"]) for result in results)
    failures = sum(int(result["total_failures"]) for result in results)
    payload = {
        "schema_version": 1,
        "area": "sql_platform",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": results,
        "summary": {
            "total_variants": variants,
            "total_failures": failures,
            "blocking_failures": failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = REPO_ROOT / args.result_json
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}", flush=True)
    if failures:
        print(f"SQL platform verification failed: failures={failures}", file=sys.stderr)
        return 1
    print(f"SQL platform verification ok: variants={variants}, failures=0", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
