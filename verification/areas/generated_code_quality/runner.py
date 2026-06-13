"""Generated-code quality verification area adapter."""

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
CORPUS_MANIFEST = AREA_ROOT / "data" / "corpus_manifest.json"
GATE_SCRIPT = AREA_ROOT / "generated_code_quality.py"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "generated-code-quality-results.json"

PROFILE_SUITES = {
    "smoke": [
        ("corpus", "2"),
        ("panic-scan", "2"),
        ("rustfmt", "2"),
        ("determinism", "2"),
    ],
    "representative": [
        ("corpus", "12"),
        ("panic-scan", "12"),
        ("rustfmt", "12"),
        ("clippy", "12"),
        ("determinism", "12"),
        ("demos", None),
    ],
    "full": [
        ("corpus", None),
        ("panic-scan", None),
        ("rustfmt", None),
        ("clippy", None),
        ("determinism", None),
        ("demos", None),
    ],
}
GATE_SUITES = {"corpus", "panic-scan", "rustfmt", "clippy", "determinism", "demos"}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable generated-code quality result summary.",
    )
    parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit the legacy validation summary line consumed by reports.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("generated_code_quality area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running generated-code quality verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print(f"  corpus_manifest={CORPUS_MANIFEST.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    blocking_failures = total_failures
    result_payload = {
        "schema_version": 1,
        "area": "generated_code_quality",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "corpus_manifest": str(CORPUS_MANIFEST.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": blocking_failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = REPO_ROOT / args.result_json
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(result_payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}", flush=True)

    if blocking_failures > 0:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={blocking_failures}, non_blocking_failures=0",
            file=sys.stderr,
            flush=True,
        )
        return 1

    prefix = "verification ok" if args.hardening_summary else "generated-code quality verification ok"
    print(
        f"{prefix}: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures={blocking_failures}, non_blocking_failures=0",
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
            raise SystemExit(f"unknown generated_code_quality suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no generated_code_quality suites selected")
    return selected


def run_suite(area_suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(area_suite["name"])
    cases = area_suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit(f"generated_code_quality suite '{suite_name}' must contain one case")
    case = cases[0]
    command = str(case.get("command"))
    if suite_name in PROFILE_SUITES and command != "generated-code-quality-mode":
        raise SystemExit(f"profile suite '{suite_name}' must use generated-code-quality-mode")
    if suite_name in GATE_SUITES and command != "generated-code-quality-gate":
        raise SystemExit(f"gate suite '{suite_name}' must use generated-code-quality-gate")

    gate_plan = PROFILE_SUITES.get(suite_name, [(suite_name, None)])
    variants = [run_gate(suite_name, gate, max_entries) for gate, max_entries in gate_plan]
    failed_variants = sum(1 for variant in variants if variant["status"] != "pass")
    return {
        "name": suite_name,
        "owner": "compiler/codegen",
        "blocking": True,
        "runner": "generated-code-quality",
        "cases": [
            {
                "id": str(case["id"]),
                "entry": str((REPO_ROOT / str(case["entry"])).relative_to(REPO_ROOT)),
                "command": command,
                "variants": variants,
            }
        ],
        "failed_cases": 1 if failed_variants else 0,
        "total_variants": len(variants),
        "total_failures": failed_variants,
    }


def run_gate(suite_name: str, gate: str, max_entries: str | None) -> dict[str, Any]:
    env = os.environ.copy()
    if max_entries is None:
        env.pop("SIFR_GCQ_MAX_ENTRIES", None)
    else:
        env["SIFR_GCQ_MAX_ENTRIES"] = max_entries
    argv = [sys.executable, str(GATE_SCRIPT), gate, "--manifest", str(CORPUS_MANIFEST)]
    started = time.perf_counter()
    proc = subprocess.run(argv, cwd=REPO_ROOT, env=env, text=True, check=False)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    status = "pass" if proc.returncode == 0 else "fail"
    emit_case_timing(suite_name, gate, elapsed_ms, status)
    return {
        "label": gate,
        "argv": argv,
        "max_entries": max_entries,
        "status": status,
        "mismatches": [] if status == "pass" else ["unexpected-exit"],
        "expected_exit_code": 0,
        "actual_exit_code": proc.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def emit_case_timing(suite_name: str, gate: str, elapsed_ms: float, status: str) -> None:
    print(
        f"[sifr-case-timing] bucket=generated_code_quality "
        f"case={timing_token(suite_name)}/{timing_token(gate)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
