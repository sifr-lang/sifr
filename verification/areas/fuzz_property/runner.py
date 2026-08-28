"""Fuzz/property verification area adapter."""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO_ROOT / "verification" / "runner"))

from sifr_verify.hardening.coverage_fuzz import run_sustained_fuzz_suite  # noqa: E402
from sifr_verify.hardening.property_and_fuzz import (  # noqa: E402
    run_mutation_smoke_suite,
    run_property_suite,
)

MANIFEST_PATH = Path(__file__).resolve().with_name("manifest.json")
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "fuzz-property-results.json"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable fuzz/property area result summary.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    os.environ.setdefault("CARGO_NET_OFFLINE", "true")
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("fuzz_property area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running fuzz/property verification area")
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}")
    print("  bless=no")

    suite_results = [run_suite(suite) for suite in selected]
    for suite_result in suite_results:
        emit_case_timings(str(suite_result["name"]), suite_result)
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    blocking_failures = sum(
        int(result["total_failures"])
        for result in suite_results
        if bool(result.get("blocking"))
    )
    non_blocking_failures = total_failures - blocking_failures
    result_payload = {
        "schema_version": 1,
        "area": "fuzz_property",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": blocking_failures,
            "non_blocking_failures": non_blocking_failures,
        },
    }
    result_path = REPO_ROOT / args.result_json
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(result_payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}")

    if blocking_failures > 0:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={blocking_failures}, non_blocking_failures={non_blocking_failures}",
            file=sys.stderr,
        )
        return 1

    summary_prefix = "fuzz/property verification ok"
    print(
        f"{summary_prefix}: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures={blocking_failures}, non_blocking_failures={non_blocking_failures}"
    )
    return 0


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [suite for suite in suites if not requested or str(suite.get("name")) in requested]
    if requested:
        present = {str(suite.get("name")) for suite in selected}
        missing = sorted(requested.difference(present))
        if missing:
            raise SystemExit(f"unknown fuzz_property suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no fuzz_property suites selected")
    return selected


def hardening_suite(area_suite: dict[str, Any]) -> dict[str, Any]:
    name = str(area_suite["name"])
    cases = area_suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit(f"fuzz_property suite '{name}' must contain exactly one index case")
    case = cases[0]
    command = str(case.get("command"))
    if name == "property" and command != "property-index":
        raise SystemExit("property suite must use property-index command")
    if name == "mutation-smoke" and command != "mutation-smoke-index":
        raise SystemExit("mutation-smoke suite must use mutation-smoke-index command")
    if name == "sustained-fuzz" and command != "sustained-fuzz-index":
        raise SystemExit("sustained-fuzz suite must use sustained-fuzz-index command")
    return {
        "name": name,
        "runner": name,
        "owner": "compiler/hardening",
        "blocking": name != "sustained-fuzz",
        "index": str(case.get("entry")),
    }


def run_suite(area_suite: dict[str, Any]) -> dict[str, Any]:
    if str(area_suite["name"]) == "cargo-smoke":
        return run_cargo_smoke_suite(area_suite)
    suite = hardening_suite(area_suite)
    if suite["runner"] == "property":
        return run_property_suite(suite=suite, repo_root=REPO_ROOT)
    if suite["runner"] == "mutation-smoke":
        return run_mutation_smoke_suite(suite=suite, repo_root=REPO_ROOT)
    return run_sustained_fuzz_suite(suite=suite, repo_root=REPO_ROOT)


def run_cargo_smoke_suite(area_suite: dict[str, Any]) -> dict[str, Any]:
    import subprocess
    import time

    suite_name = str(area_suite["name"])
    result = {
        "name": suite_name,
        "owner": "compiler/hardening",
        "blocking": True,
        "runner": "cargo-smoke",
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }
    for case in area_suite.get("cases", []):
        case_id = str(case["id"])
        expected_exit = int(case["expect_exit_code"])
        if str(case.get("command")) != "cargo-e2e-test":
            raise SystemExit(f"cargo-smoke case '{case_id}' must use cargo-e2e-test command")
        entry = REPO_ROOT / str(case["entry"])
        if not entry.is_file():
            raise SystemExit(f"cargo-smoke case '{case_id}' entry does not exist: {entry}")
        argv = [
            "cargo",
            "test",
            "-p",
            "sifr",
            "--test",
            "e2e",
            case_id,
            "--",
            "--nocapture",
        ]
        started = time.perf_counter()
        proc = subprocess.run(argv, cwd=REPO_ROOT, text=True, capture_output=True, check=False)
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        if proc.stdout:
            sys.stdout.write(proc.stdout)
        if proc.stderr:
            sys.stderr.write(proc.stderr)
        mismatches = []
        if proc.returncode != expected_exit:
            mismatches.append("unexpected-exit")
        status = "pass" if not mismatches else "fail"
        result["total_variants"] += 1
        if mismatches:
            result["total_failures"] += 1
            result["failed_cases"] += 1
        result["cases"].append(
            {
                "id": case_id,
                "entry": str(entry.relative_to(REPO_ROOT)),
                "command": "cargo-e2e-test",
                "variants": [
                    {
                        "label": case_id,
                        "argv": argv,
                        "status": status,
                        "mismatches": mismatches,
                        "expected_exit_code": expected_exit,
                        "actual_exit_code": proc.returncode,
                        "duration_ms": round(elapsed_ms, 3),
                    }
                ],
            }
        )
    return result


def emit_case_timings(suite_name: str, suite_result: dict[str, Any]) -> None:
    for case in suite_result.get("cases", []):
        if not isinstance(case, dict):
            continue
        case_id = timing_token(case.get("id", "unknown"))
        for variant in case.get("variants", []):
            if not isinstance(variant, dict) or "duration_ms" not in variant:
                continue
            label = timing_token(variant.get("label", "variant"))
            status = "pass" if variant.get("status") == "pass" else "fail"
            elapsed_ms = int(float(variant["duration_ms"]))
            print(
                f"[sifr-case-timing] bucket=fuzz_property "
                f"case={timing_token(suite_name)}/{case_id}/{label} "
                f"elapsed_ms={elapsed_ms} status={status}"
            )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
