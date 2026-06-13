"""Ecosystem compatibility verification area adapter."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO_ROOT / "verification" / "runner"))

from sifr_verify.hardening.oss_and_determinism import run_oss_suite  # noqa: E402

MANIFEST_PATH = Path(__file__).resolve().with_name("manifest.json")
RESULT_JSON = (
    REPO_ROOT
    / "target"
    / "verification"
    / "areas"
    / "ecosystem-compatibility-results.json"
)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable ecosystem compatibility area result summary.",
    )
    parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit the legacy hardening summary line consumed by validation reports.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("ecosystem_compatibility area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running ecosystem compatibility verification area")
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}")
    print("  bless=no")

    suite_results = [run_suite(suite) for suite in selected]
    for suite_result in suite_results:
        emit_case_timings(str(suite_result["name"]), suite_result)
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    blocking_failures = sum(
        int(result["total_failures"]) for result in suite_results if result.get("blocking")
    )
    non_blocking_failures = total_failures - blocking_failures
    result_payload = {
        "schema_version": 1,
        "area": "ecosystem_compatibility",
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

    summary_prefix = (
        "verification ok" if args.hardening_summary else "ecosystem compatibility verification ok"
    )
    print(
        f"{summary_prefix}: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures={blocking_failures}, non_blocking_failures={non_blocking_failures}"
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
                f"unknown ecosystem_compatibility suite filter(s): {', '.join(missing)}"
            )
    if not selected:
        raise SystemExit("no ecosystem_compatibility suites selected")
    return selected


def hardening_suite(area_suite: dict[str, Any]) -> dict[str, Any]:
    name = str(area_suite["name"])
    cases = area_suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit(
            f"ecosystem_compatibility suite '{name}' must contain exactly one index case"
        )
    case = cases[0]
    command = str(case.get("command"))
    if name == "oss-curated" and command != "oss-curated-index":
        raise SystemExit("oss-curated suite must use oss-curated-index command")
    if name == "ecosystem-broader" and command != "ecosystem-broader-index":
        raise SystemExit("ecosystem-broader suite must use ecosystem-broader-index command")
    return {
        "name": name,
        "runner": name,
        "owner": "compiler/verification",
        "blocking": name == "oss-curated",
        "index": str(case.get("entry")),
    }


def run_suite(area_suite: dict[str, Any]) -> dict[str, Any]:
    suite = hardening_suite(area_suite)
    return run_oss_suite(
        suite=suite,
        repo_root=REPO_ROOT,
        runner_name=str(suite["runner"]),
    )


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
                f"[sifr-case-timing] bucket=ecosystem_compatibility "
                f"case={timing_token(suite_name)}/{case_id}/{label} "
                f"elapsed_ms={elapsed_ms} status={status}"
            )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
