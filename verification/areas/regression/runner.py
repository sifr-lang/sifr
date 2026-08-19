"""Regression verification area adapter."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO_ROOT / "verification" / "runner"))

from sifr_verify.hardening.fixedbugs_and_crashes import (  # noqa: E402
    collect_fixedbug_ids,
    run_crashes_suite,
    run_fixedbugs_suite,
)

MANIFEST_PATH = Path(__file__).resolve().with_name("manifest.json")
ACTUAL_ROOT = REPO_ROOT / "target" / "verification" / "actual" / "regression"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "regression-results.json"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable regression area result summary.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))
    if args.bless:
        raise SystemExit("regression area does not support --bless")

    print("Running regression verification area")
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}")
    print("  bless=no")

    fixedbug_ids = collect_fixedbug_ids(REPO_ROOT, area_suites_for_hardening(manifest["suites"]))
    suite_results = [run_suite(suite, fixedbug_ids) for suite in selected]
    for suite_result in suite_results:
        emit_case_timings(str(suite_result["name"]), suite_result)
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    blocking_failures = total_failures
    result_payload = {
        "schema_version": 1,
        "area": "regression",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
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
    print(f"result_json={result_path.relative_to(REPO_ROOT)}")

    if blocking_failures > 0:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={blocking_failures}, non_blocking_failures=0",
            file=sys.stderr,
        )
        return 1

    summary_prefix = "regression verification ok"
    print(
        f"{summary_prefix}: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures={blocking_failures}, non_blocking_failures=0"
    )
    return 0


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [suite for suite in suites if not requested or str(suite.get("name")) in requested]
    if requested:
        present = {str(suite.get("name")) for suite in selected}
        missing = sorted(requested.difference(present))
        if missing:
            raise SystemExit(f"unknown regression suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no regression suites selected")
    return selected


def area_suites_for_hardening(area_suites: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [hardening_suite(suite) for suite in area_suites]


def hardening_suite(area_suite: dict[str, Any]) -> dict[str, Any]:
    name = str(area_suite["name"])
    cases = area_suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit(f"regression suite '{name}' must contain exactly one index case")
    case = cases[0]
    command = str(case.get("command"))
    if name == "fixedbugs" and command != "fixedbugs-index":
        raise SystemExit("fixedbugs suite must use fixedbugs-index command")
    if name == "crashes" and command != "crashes-index":
        raise SystemExit("crashes suite must use crashes-index command")
    entry = str(case.get("entry"))
    return {
        "name": name,
        "runner": "fixedbugs" if name == "fixedbugs" else "crashes",
        "owner": "compiler/hardening",
        "blocking": True,
        "index": entry,
    }


def run_suite(area_suite: dict[str, Any], fixedbug_ids: set[str]) -> dict[str, Any]:
    suite = hardening_suite(area_suite)
    if suite["runner"] == "fixedbugs":
        return run_fixedbugs_suite(suite=suite, repo_root=REPO_ROOT, actual_root=ACTUAL_ROOT)
    return run_crashes_suite(suite=suite, repo_root=REPO_ROOT, fixedbug_ids=fixedbug_ids)


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
                f"[sifr-case-timing] bucket=regression "
                f"case={timing_token(suite_name)}/{case_id}/{label} "
                f"elapsed_ms={elapsed_ms} status={status}"
            )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
