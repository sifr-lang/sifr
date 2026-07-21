"""Python interop verification area runner."""

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
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "python-interop-results.json"

COMMAND_ARGS: dict[str, list[str]] = {
    "python-interop-self-test": ["--self-test"],
    "python-interop-scaffold": [
        "--group",
        "scaffold",
        "--report",
        "../../../target/verification/areas/python_interop/scaffold.latest.json",
    ],
    "python-interop-env": [
        "--group",
        "env",
        "--report",
        "../../../target/verification/areas/python_interop/env.latest.json",
    ],
    "python-interop-readonly-check-doctor": [],
    "python-interop-tier1": [
        "--tier",
        "tier1",
        "--report",
        "../../../target/verification/areas/python_interop/tier1.latest.json",
    ],
    "python-interop-tier2": [
        "--tier",
        "tier2",
        "--report",
        "../../../target/verification/areas/python_interop/tier2.latest.json",
    ],
    "python-interop-tier3": [
        "--tier",
        "tier3",
        "--report",
        "../../../target/verification/areas/python_interop/tier3.latest.json",
    ],
    "python-interop-tier4": [
        "--tier",
        "tier4",
        "--report",
        "../../../target/verification/areas/python_interop/tier4.latest.json",
    ],
    "python-interop-callbacks": [
        "--group",
        "callbacks",
        "--report",
        "../../../target/verification/areas/python_interop/callbacks.latest.json",
    ],
    "python-interop-callback-examples": [
        "--callback-examples",
        "--report",
        "../../../target/verification/areas/python_interop/callback-examples.latest.json",
    ],
    "python-interop-dataframes": [
        "--group",
        "dataframes",
        "--report",
        "../../../target/verification/areas/python_interop/dataframes.latest.json",
    ],
    "python-interop-dataframe-examples": [
        "--dataframe-examples",
        "--report",
        "../../../target/verification/areas/python_interop/dataframe-examples.latest.json",
    ],
    "python-interop-buffer-examples": [
        "--buffer-examples",
        "--report",
        "../../../target/verification/areas/python_interop/buffer-examples.latest.json",
    ],
    "python-interop-arrow-examples": [
        "--arrow-examples",
        "--report",
        "../../../target/verification/areas/python_interop/arrow-examples.latest.json",
    ],
    "python-interop-dlpack-examples": [
        "--dlpack-examples",
        "--report",
        "../../../target/verification/areas/python_interop/dlpack-examples.latest.json",
    ],
    "python-interop-buffer-cpython311": [],
    "python-interop-arrow-cpython311": [],
    "python-interop-dlpack-cpython311": [],
    "python-interop-ml-examples": [
        "--ml-examples",
        "--report",
        "../../../target/verification/areas/python_interop/ml-examples.latest.json",
    ],
    "python-interop-library-examples": [
        "--library-examples",
        "--report",
        "../../../target/verification/areas/python_interop/library-examples.latest.json",
    ],
    "python-interop-async-declaration-examples": [
        "--async-declaration-examples",
        "--report",
        "../../../target/verification/areas/python_interop/async-declaration-examples.latest.json",
    ],
    "python-interop-async-context-examples": [
        "--async-context-examples",
        "--report",
        "../../../target/verification/areas/python_interop/async-context-examples.latest.json",
    ],
    "python-interop-cloud-boto3": [
        "--group",
        "cloud",
        "--package",
        "boto3",
        "--report",
        "../../../target/verification/areas/python_interop/package.latest.json",
    ],
    "python-interop-live-policy": [
        "--live-policy",
        "--report",
        "../../../target/verification/areas/python_interop/live-policy.latest.json",
    ],
    "python-interop-live-examples": [
        "--live-examples",
        "--report",
        "../../../target/verification/areas/python_interop/live-examples.latest.json",
    ],
}

AREA_PROJECT_COMMANDS = {
    "python-interop-callback-examples",
    "python-interop-dataframe-examples",
    "python-interop-buffer-examples",
    "python-interop-arrow-examples",
    "python-interop-dlpack-examples",
    "python-interop-library-examples",
    "python-interop-async-declaration-examples",
    "python-interop-async-context-examples",
    "python-interop-ml-examples",
    "python-interop-live-examples",
}
CPYTHON311_PROJECT_COMMANDS = {
    "python-interop-arrow-cpython311",
    "python-interop-dlpack-cpython311",
    "python-interop-buffer-cpython311",
}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable python interop result summary.",
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
        raise SystemExit("python_interop area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running python interop verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "python_interop",
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
    prefix = "verification ok" if args.hardening_summary else "python interop verification ok"
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
            raise SystemExit(f"unknown python_interop suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no python_interop suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or not cases:
        raise SystemExit(f"python_interop {suite_name} suite must contain at least one case")
    case_results = [run_case(case) for case in cases]
    failures = sum(1 for variant in case_results if variant["status"] == "fail")
    return {
        "name": suite_name,
        "owner": "runtime/python-interop",
        "blocking": True,
        "runner": "python-interop",
        "cases": [
            {
                "id": str(case["id"]),
                "entry": str(case["entry"]),
                "command": str(case["command"]),
                "variants": [variant],
            }
            for case, variant in zip(cases, case_results, strict=True)
        ],
        "failed_cases": failures,
        "total_variants": len(case_results),
        "total_failures": failures,
    }


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    command = str(case["command"])
    if command not in COMMAND_ARGS:
        raise SystemExit(f"unsupported python_interop command: {command}")
    entry = REPO_ROOT / str(case["entry"])
    if not entry.is_file():
        raise SystemExit(f"python_interop case entry does not exist: {entry}")
    expected_exit = int(case["expect_exit_code"])
    env = None
    if command in CPYTHON311_PROJECT_COMMANDS:
        argv = [
            "uv",
            "run",
            "--project",
            str(AREA_ROOT / "cpython311"),
            "--locked",
            "--python",
            "3.11",
            "--managed-python",
            "--no-python-downloads",
            "python",
            str(entry),
            *COMMAND_ARGS[command],
        ]
        env = os.environ.copy()
        env.pop("VIRTUAL_ENV", None)
    elif command in AREA_PROJECT_COMMANDS:
        argv = [
            "uv",
            "run",
            "--project",
            str(AREA_ROOT),
            "--locked",
            "python",
            str(entry),
            *COMMAND_ARGS[command],
        ]
        env = os.environ.copy()
        env.pop("VIRTUAL_ENV", None)
    else:
        argv = [sys.executable, str(entry), *COMMAND_ARGS[command]]
    started = time.perf_counter()
    proc = subprocess.run(
        argv,
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
        env=env,
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
    print(
        "[sifr-case-timing] "
        f"bucket=python_interop case={case['id']} elapsed_ms={round(elapsed_ms)} status={status}",
        flush=True,
    )
    return {
        "label": command,
        "diagnostic_format": None,
        "argv": argv,
        "status": status,
        "mismatches": mismatches,
        "expected_exit_code": expected_exit,
        "actual_exit_code": proc.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


if __name__ == "__main__":
    raise SystemExit(main())
