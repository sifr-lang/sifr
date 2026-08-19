from __future__ import annotations

import json
import sys
import time
from pathlib import Path
from typing import Any

from .core import parse_args, should_run_suite
from .fixedbugs_and_crashes import collect_fixedbug_ids, run_crashes_suite, run_fixedbugs_suite
from .oss_and_determinism import (
    deterministic_suite_shard,
    failed_case_ids,
    load_quarantine_metadata,
    run_determinism_scale_suite,
    run_oss_suite,
)
from .property_and_fuzz import run_fuzz_smoke_suite, run_property_suite
from .self_tests_and_baselines import run_baseline_suite, run_self_tests


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


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
                f"[sifr-case-timing] bucket=verification_hardening "
                f"case={timing_token(suite_name)}/{case_id}/{label} "
                f"elapsed_ms={elapsed_ms} status={status}"
            )


def main() -> int:
    args = parse_args()
    if args.shard_total < 1:
        raise SystemExit("--shard-total must be >= 1")
    if args.shard_index < 0 or args.shard_index >= args.shard_total:
        raise SystemExit("--shard-index must satisfy 0 <= shard-index < shard-total")
    if args.rerun_failures < 0:
        raise SystemExit("--rerun-failures must be >= 0")

    repo_root = Path(__file__).resolve().parents[4]
    if args.self_test:
        return run_self_tests()

    manifest_path = (repo_root / args.manifest).resolve()
    result_json_path = (repo_root / args.result_json).resolve()
    quarantine_path = (repo_root / args.quarantine_file).resolve()
    actual_root = repo_root / "target/verification/actual"
    actual_root.mkdir(parents=True, exist_ok=True)

    if not manifest_path.is_file():
        raise SystemExit(f"verification manifest not found: {manifest_path}")

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    suites = manifest.get("suites", [])
    if not isinstance(suites, list):
        raise SystemExit("invalid manifest: 'suites' must be a list")

    selected_suites = []
    explicit_suites = set(args.suite)
    for suite in suites:
        name = suite.get("name")
        if not isinstance(name, str):
            raise SystemExit("invalid manifest: suite missing string 'name'")
        if explicit_suites and name not in explicit_suites:
            continue
        if not explicit_suites and not should_run_suite(args.profile, name):
            continue
        selected_suites.append(suite)

    if explicit_suites:
        missing = sorted(explicit_suites.difference({suite.get("name") for suite in suites}))
        if missing:
            raise SystemExit(f"unknown suite filter(s): {', '.join(missing)}")

    if not selected_suites:
        raise SystemExit("no verification suites selected")

    selected_suites = [
        suite
        for suite in selected_suites
        if deterministic_suite_shard(str(suite.get("name")), args.shard_total) == args.shard_index
    ]

    quarantine_entries = load_quarantine_metadata(quarantine_path, suites)

    run_results: list[dict[str, Any]] = []
    total_variants = 0
    total_failures = 0
    blocking_failures = 0
    non_blocking_failures = 0

    print("Running verification hardening suites")
    print(f"  profile={args.profile}")
    print(f"  manifest={manifest_path.relative_to(repo_root)}")
    print(f"  bless={'yes' if args.bless else 'no'}")
    print(f"  shard={args.shard_index}/{args.shard_total}")
    print(f"  rerun_failures={args.rerun_failures}")
    print(f"  quarantine_entries={len(quarantine_entries)}")

    fixedbug_ids = collect_fixedbug_ids(repo_root, selected_suites)

    def execute_suite_once(suite: dict[str, Any]) -> dict[str, Any]:
        runner = str(suite.get("runner", "baseline"))
        if runner == "baseline":
            return run_baseline_suite(
                suite=suite,
                args=args,
                repo_root=repo_root,
                actual_root=actual_root,
            )
        if runner == "fixedbugs":
            return run_fixedbugs_suite(
                suite=suite,
                repo_root=repo_root,
                actual_root=actual_root,
            )
        if runner == "crashes":
            return run_crashes_suite(
                suite=suite,
                repo_root=repo_root,
                fixedbug_ids=fixedbug_ids,
            )
        if runner == "property":
            return run_property_suite(
                suite=suite,
                repo_root=repo_root,
            )
        if runner == "fuzz-smoke":
            return run_fuzz_smoke_suite(
                suite=suite,
                repo_root=repo_root,
            )
        if runner == "oss-curated":
            return run_oss_suite(
                suite=suite,
                repo_root=repo_root,
                runner_name="oss-curated",
            )
        if runner == "ecosystem-broader":
            return run_oss_suite(
                suite=suite,
                repo_root=repo_root,
                runner_name="ecosystem-broader",
            )
        if runner == "determinism-scale":
            return run_determinism_scale_suite(
                suite=suite,
                repo_root=repo_root,
                profile=args.profile,
            )
        raise SystemExit(f"unsupported runner '{runner}' for suite '{suite.get('name', '<unknown>')}'")

    for suite in selected_suites:
        suite_result = execute_suite_once(suite)
        suite_name = str(suite.get("name"))
        emit_case_timings(suite_name, suite_result)
        suite_quarantine = [entry for entry in quarantine_entries if entry.get("suite") == suite_name]
        if suite_quarantine:
            suite_result["quarantine_entries"] = suite_quarantine

        if not args.bless and args.rerun_failures > 0 and int(suite_result.get("total_failures", 0)) > 0:
            initial_failed = failed_case_ids(suite_result)
            rerun_attempts: list[dict[str, Any]] = []
            flake_events: list[dict[str, Any]] = []
            previous_failed = set(initial_failed)
            for attempt in range(1, args.rerun_failures + 1):
                rerun_result = execute_suite_once(suite)
                rerun_failed = failed_case_ids(rerun_result)
                transitioned = sorted(previous_failed.difference(rerun_failed))
                rerun_attempt = {
                    "attempt": attempt,
                    "failed_case_count": len(rerun_failed),
                    "failed_cases": sorted(rerun_failed),
                    "total_failures": int(rerun_result.get("total_failures", 0)),
                }
                if transitioned:
                    rerun_attempt["flaky_fail_to_pass_cases"] = transitioned
                    flake_events.append(
                        {
                            "attempt": attempt,
                            "flaky_fail_to_pass_cases": transitioned,
                        }
                    )
                rerun_attempts.append(rerun_attempt)
                previous_failed = rerun_failed
                if not rerun_failed:
                    break
            suite_result["rerun_attempts"] = rerun_attempts
            if flake_events:
                suite_result["flake_events"] = flake_events

        run_results.append(suite_result)
        total_variants += int(suite_result.get("total_variants", 0))
        suite_failures = int(suite_result.get("total_failures", 0))
        total_failures += suite_failures
        if suite_failures > 0:
            if bool(suite_result.get("blocking")):
                blocking_failures += suite_failures
            else:
                non_blocking_failures += suite_failures

    result_payload = {
        "schema_version": 1,
        "profile": args.profile,
        "bless": args.bless,
        "manifest": str(manifest_path.relative_to(repo_root)),
        "shard_total": args.shard_total,
        "shard_index": args.shard_index,
        "rerun_failures": args.rerun_failures,
        "quarantine_file": str(quarantine_path.relative_to(repo_root)),
        "quarantine_entry_count": len(quarantine_entries),
        "generated_at_unix_secs": int(time.time()),
        "suites": run_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": blocking_failures,
            "non_blocking_failures": non_blocking_failures,
        },
    }
    result_json_path.parent.mkdir(parents=True, exist_ok=True)
    result_json_path.write_text(json.dumps(result_payload, indent=2, sort_keys=True), encoding="utf-8")

    if args.bless:
        print("baselines updated")
    else:
        print(f"result_json={result_json_path.relative_to(repo_root)}")

    if blocking_failures > 0 and not args.bless:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={blocking_failures}, non_blocking_failures={non_blocking_failures}",
            file=sys.stderr,
        )
        print("actual outputs written under target/verification/actual", file=sys.stderr)
        return 1

    return 0
