from __future__ import annotations

import json
import os
import time
from pathlib import Path
from typing import Any

from .fixedbugs_and_crashes import contains_internal_panic
from .core import run_captured_command

OUTPUT_TAIL_BYTES = 16 * 1024


def output_tail(output: str) -> str:
    encoded = output.encode("utf-8", errors="replace")
    return encoded[-OUTPUT_TAIL_BYTES:].decode("utf-8", errors="replace")


def classify_build_failure(exit_code: int, output: str, timed_out: bool) -> str:
    lowered = output.lower()
    if timed_out:
        return "instrumented-build-timeout"
    if exit_code == 127 or "no such command: `fuzz`" in lowered:
        return "missing-fuzz-tool"
    if "offline mode" in lowered or "attempting to make an http request" in lowered:
        return "offline-dependency-failure"
    return "instrumented-build-failure"


def run_sustained_fuzz_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
) -> dict[str, Any]:
    suite_name = str(suite["name"])
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    payload = json.loads(index_path.read_text(encoding="utf-8"))
    profile = os.environ.get("SIFR_VALIDATION_PROFILE", "nightly")
    budgets = payload.get("budgets_seconds", {})
    budget = budgets.get(profile) if isinstance(budgets, dict) else None

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "sustained-fuzz",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }
    mismatches = validate_sustained_fuzz_rules(payload, repo_root)
    if profile not in {"nightly", "release"}:
        mismatches.append(f"unsupported-profile:{profile}")
    if not isinstance(budget, int) or budget < 1:
        mismatches.append(f"budget:{profile}")
    if mismatches:
        result["cases"].append(
            {
                "id": payload.get("id", "sustained-fuzz"),
                "variants": [
                    {
                        "label": "metadata",
                        "status": "fail",
                        "mismatches": sorted(set(mismatches)),
                    }
                ],
            }
        )
        result["failed_cases"] = 1
        result["total_variants"] = 1
        result["total_failures"] = 1
        return result

    assert isinstance(budget, int)
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} profile={profile} seconds_per_target={budget}")
    build_case = build_fuzz_project(repo_root=repo_root)
    result["cases"].append(build_case)
    result["total_variants"] += 1
    if build_case["variants"][0]["status"] != "pass":
        result["failed_cases"] += 1
        result["total_failures"] += 1
        return result

    for target in payload["targets"]:
        case = run_coverage_fuzz_target(
            target=target,
            repo_root=repo_root,
            seconds=budget,
        )
        result["cases"].append(case)
        result["total_variants"] += 1
        if case["variants"][0]["status"] != "pass":
            result["failed_cases"] += 1
            result["total_failures"] += 1
    return result


def build_fuzz_project(*, repo_root: Path) -> dict[str, Any]:
    argv = [
        "cargo",
        "+nightly",
        "fuzz",
        "build",
        "--fuzz-dir",
        "verification/fuzz",
    ]
    started = time.perf_counter()
    exit_code, stdout, stderr = run_captured_command(
        args=argv,
        cwd=repo_root,
        env={**os.environ, "CARGO_NET_OFFLINE": "true"},
        timeout_secs=1_200,
    )
    timed_out = exit_code == 124
    output = stdout + stderr
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    mismatches = [] if exit_code == 0 else [classify_build_failure(exit_code, output, timed_out)]
    variant = {
        "label": "sustained-fuzz-instrumented-build",
        "status": "pass" if not mismatches else "fail",
        "mismatches": mismatches,
        "actual_exit_code": exit_code,
        "duration_ms": round(elapsed_ms, 3),
        "argv": argv,
    }
    if mismatches:
        variant["output_tail"] = output_tail(output)
    return {
        "id": "fuzz_project_build",
        "variants": [variant],
    }


def run_coverage_fuzz_target(
    *,
    target: dict[str, Any],
    repo_root: Path,
    seconds: int,
) -> dict[str, Any]:
    target_name = str(target["fuzz_target"])
    corpus_dir = repo_root / "target" / "verification" / "fuzz" / "corpus" / target_name
    artifact_dir = repo_root / "target" / "verification" / "fuzz" / "artifacts" / target_name
    corpus_dir.mkdir(parents=True, exist_ok=True)
    artifact_dir.mkdir(parents=True, exist_ok=True)
    artifacts_before = {path.name for path in artifact_dir.iterdir() if path.is_file()}
    argv = [
        "cargo",
        "+nightly",
        "fuzz",
        "run",
        "--fuzz-dir",
        "verification/fuzz",
        target_name,
        str(corpus_dir),
        "--",
        f"-max_total_time={seconds}",
        f"-artifact_prefix={artifact_dir}/",
        "-print_final_stats=1",
    ]
    started = time.perf_counter()
    exit_code, stdout, stderr = run_captured_command(
        args=argv,
        cwd=repo_root,
        env={**os.environ, "CARGO_NET_OFFLINE": "true"},
        timeout_secs=seconds + 120,
    )
    timed_out = exit_code == 124
    output = stdout + stderr
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    mismatches: list[str] = []
    artifacts_after = {path.name for path in artifact_dir.iterdir() if path.is_file()}
    new_artifacts = sorted(artifacts_after - artifacts_before)
    finding = bool(new_artifacts) or "ERROR: libFuzzer" in output or "Test unit written to" in output
    if timed_out:
        mismatches.append("target-timeout")
    elif finding:
        mismatches.append("fuzz-finding")
    elif exit_code != 0:
        mismatches.append("target-execution-failure")
    if contains_internal_panic(output):
        mismatches.append("panic-signal")
    variant = {
        "label": f"sustained-fuzz-{target_name}",
        "status": "pass" if not mismatches else "fail",
        "mismatches": mismatches,
        "actual_exit_code": exit_code,
        "duration_ms": round(elapsed_ms, 3),
        "argv": argv,
        "new_artifacts": new_artifacts,
    }
    if mismatches:
        variant["output_tail"] = output_tail(output)
    return {
        "id": str(target["id"]),
        "fuzz_target": target_name,
        "variants": [variant],
    }


def validate_sustained_fuzz_rules(payload: dict[str, Any], repo_root: Path) -> list[str]:
    mismatches: list[str] = []
    if payload.get("schema_version") != 1:
        mismatches.append("schema_version")
    if payload.get("id") != "SUSTAINED-FUZZ-0001":
        mismatches.append("id")
    manifest_path = payload.get("cargo_manifest")
    if not isinstance(manifest_path, str) or not (repo_root / manifest_path).is_file():
        mismatches.append("cargo_manifest")
    targets = payload.get("targets")
    if not isinstance(targets, list) or not targets:
        mismatches.append("targets")
        return mismatches
    expected = {
        "parser",
        "lowering",
        "ownership",
        "codegen_validation",
        "diagnostics",
        "project_graph",
    }
    observed: set[str] = set()
    labels: set[str] = set()
    for target in targets:
        if not isinstance(target, dict):
            mismatches.append("target")
            continue
        target_id = target.get("id")
        fuzz_target = target.get("fuzz_target")
        if not isinstance(target_id, str) or not target_id:
            mismatches.append("target.id")
        if not isinstance(fuzz_target, str) or fuzz_target not in expected:
            mismatches.append(f"target.fuzz_target:{fuzz_target}")
        elif fuzz_target in observed:
            mismatches.append(f"target.duplicate:{fuzz_target}")
        else:
            observed.add(fuzz_target)
            label = f"sustained-fuzz-{fuzz_target}"
            if label in labels:
                mismatches.append(f"target.label:{label}")
            labels.add(label)
        if not isinstance(target.get("finding_promotion"), str) or not target["finding_promotion"]:
            mismatches.append(f"{target_id}.finding_promotion")
    if observed != expected:
        mismatches.append("target.coverage")
    budgets = payload.get("budgets_seconds")
    if not isinstance(budgets, dict):
        mismatches.append("budgets_seconds")
    else:
        for profile in ("nightly", "release"):
            if not isinstance(budgets.get(profile), int) or int(budgets[profile]) < 1:
                mismatches.append(f"budgets_seconds.{profile}")
    return mismatches
