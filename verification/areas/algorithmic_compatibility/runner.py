"""Algorithmic compatibility verification area adapter."""

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
RESULT_JSON = (
    REPO_ROOT
    / "target"
    / "verification"
    / "areas"
    / "algorithmic-compatibility-results.json"
)
LEETCODE_ROOT = AREA_ROOT / "corpora" / "leetcode" / "src"
TAXONOMY_SMOKE_RESULTS = AREA_ROOT / "data" / "taxonomy_smoke_results.json"
PROFILE_MANIFEST = AREA_ROOT / "data" / "leetcode_profile_manifest.json"
DEFAULT_SIFR_BIN = REPO_ROOT / "target" / "debug" / "sifr"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable algorithmic compatibility result summary.",
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
        raise SystemExit("algorithmic_compatibility area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running algorithmic compatibility verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "algorithmic_compatibility",
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
    prefix = (
        "verification ok"
        if args.hardening_summary
        else "algorithmic compatibility verification ok"
    )
    print(
        f"{prefix}: variants={total_variants}, failures={total_failures}, "
        "blocking_failures=0, non_blocking_failures=0",
        flush=True,
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
                f"unknown algorithmic_compatibility suite filter(s): {', '.join(missing)}"
            )
    if not selected:
        raise SystemExit("no algorithmic_compatibility suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit(
            f"algorithmic_compatibility suite '{suite_name}' must contain exactly one case"
        )
    case = cases[0]
    variants = run_case(case)
    failures = sum(1 for variant in variants if variant["status"] == "fail")
    return {
        "name": suite_name,
        "owner": "algorithmic/compatibility",
        "blocking": True,
        "runner": "algorithmic-compatibility",
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


def run_case(case: dict[str, Any]) -> list[dict[str, Any]]:
    entry = REPO_ROOT / str(case["entry"])
    if not entry.exists():
        raise SystemExit(f"algorithmic_compatibility case entry does not exist: {entry}")
    command = str(case["command"])
    expected_exit = int(case.get("expect_exit_code", 0))
    started = time.perf_counter()
    if command == "algorithmic-profile-manifest":
        variants = run_profile_manifest()
    elif command == "algorithmic-representative-subset":
        variants = run_representative_subset()
    elif command == "algorithmic-leetcode-full":
        variants = run_leetcode_full()
    elif command == "algorithmic-taxonomy-smoke":
        argv, result = run_taxonomy_smoke()
        variants = [completed_process_variant(str(case["id"]), argv, result, expected_exit, started)]
    elif command == "algorithmic-leetcode-check":
        argv, result = run_leetcode_check()
        variants = [completed_process_variant(str(case["id"]), argv, result, expected_exit, started)]
    else:
        raise SystemExit(f"unsupported algorithmic_compatibility command: {command}")
    return variants


def completed_process_variant(
    label: str,
    argv: list[str],
    result: subprocess.CompletedProcess[str],
    expected_exit: int,
    started: float,
) -> dict[str, Any]:
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    failures = []
    if result.returncode != expected_exit:
        failures.append(f"exit={result.returncode} expected={expected_exit}")
    if result.stdout:
        sys.stdout.write(result.stdout)
    if result.stderr:
        sys.stderr.write(result.stderr)
    status = "fail" if failures else "pass"
    print_case_timing(label, elapsed_ms, status)
    return {
        "label": label,
        "argv": argv,
        "status": status,
        "mismatches": failures,
        "expected_exit_code": expected_exit,
        "actual_exit_code": result.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def run_profile_manifest() -> list[dict[str, Any]]:
    started = time.perf_counter()
    payload = load_profile_manifest()
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    variant = {
        "label": "leetcode-profile-manifest",
        "argv": ["json-validate", str(PROFILE_MANIFEST.relative_to(REPO_ROOT))],
        "status": "pass",
        "mismatches": [],
        "expected_exit_code": 0,
        "actual_exit_code": 0,
        "duration_ms": round(elapsed_ms, 3),
        "representative_cases": len(payload["representative_subset"]),
        "required_categories": list(payload["taxonomy"]["required_categories"]),
        "full_corpus_expected_fixture_count": int(payload["full_corpus"]["expected_fixture_count"]),
    }
    print_case_timing("leetcode-profile-manifest", elapsed_ms, "pass")
    return [variant]


def load_profile_manifest() -> dict[str, Any]:
    payload = json.loads(PROFILE_MANIFEST.read_text(encoding="utf-8"))
    validate_profile_manifest(payload)
    return payload


def validate_profile_manifest(payload: object) -> None:
    if not isinstance(payload, dict):
        raise SystemExit("leetcode profile manifest must be a JSON object")
    if payload.get("schema_version") != 1:
        raise SystemExit("leetcode profile manifest schema_version must be 1")
    if payload.get("owner") != "algorithmic/compatibility":
        raise SystemExit("leetcode profile manifest owner mismatch")
    corpus_root = REPO_ROOT / required_string(payload, "corpus_root")
    if corpus_root != LEETCODE_ROOT or not corpus_root.is_dir():
        raise SystemExit("leetcode profile manifest corpus_root must point at the checked-in corpus")
    taxonomy = payload.get("taxonomy")
    if not isinstance(taxonomy, dict):
        raise SystemExit("leetcode profile manifest taxonomy must be an object")
    required_categories = taxonomy.get("required_categories")
    if not isinstance(required_categories, list) or not required_categories:
        raise SystemExit("leetcode profile manifest requires categories")
    categories = [str(category) for category in required_categories]
    if len(categories) != len(set(categories)):
        raise SystemExit("leetcode profile manifest categories must be unique")
    for path_key in ("baseline_taxonomy", "baseline_results"):
        path = REPO_ROOT / required_string(taxonomy, path_key)
        if not path.is_file():
            raise SystemExit(f"leetcode profile manifest missing {path_key}: {path}")
    required_string(taxonomy, "generated_on")
    subset = payload.get("representative_subset")
    if not isinstance(subset, list) or not subset:
        raise SystemExit("leetcode profile manifest requires representative_subset")
    seen_ids: set[str] = set()
    seen_categories: set[str] = set()
    result_artifacts: set[str] = set()
    for row in subset:
        validate_representative_row(row, seen_ids, set(categories))
        seen_categories.add(str(row["category"]))
        result_artifacts.add(str(row["result_artifact"]))
    if len(result_artifacts) != 1:
        raise SystemExit("representative subset rows must share one result_artifact")
    missing_categories = sorted(set(categories).difference(seen_categories))
    if missing_categories:
        raise SystemExit("representative subset misses categories: " + ", ".join(missing_categories))
    full_corpus = payload.get("full_corpus")
    if not isinstance(full_corpus, dict):
        raise SystemExit("leetcode profile manifest requires full_corpus")
    expected_count = full_corpus.get("expected_fixture_count")
    actual_count = len(list(LEETCODE_ROOT.glob("*.sifr")))
    if expected_count != actual_count:
        raise SystemExit(f"full corpus expected_fixture_count={expected_count} actual={actual_count}")
    for key in ("id", "command", "result_artifact", "taxonomy_artifact", "taxonomy_markdown", "delta_markdown"):
        required_string(full_corpus, key)
    timeout = full_corpus.get("timeout_seconds")
    if not isinstance(timeout, int) or timeout <= 0:
        raise SystemExit("full corpus timeout_seconds must be a positive integer")


def validate_representative_row(row: object, seen_ids: set[str], categories: set[str]) -> None:
    if not isinstance(row, dict):
        raise SystemExit("representative rows must be objects")
    allowed_keys = {
        "category",
        "command",
        "expected_classification",
        "id",
        "owner",
        "path",
        "result_artifact",
        "timeout_seconds",
    }
    unknown = sorted(set(row).difference(allowed_keys))
    if unknown:
        raise SystemExit(f"representative row has unknown field(s): {', '.join(unknown)}")
    row_id = required_string(row, "id")
    if row_id in seen_ids:
        raise SystemExit(f"duplicate representative row id: {row_id}")
    seen_ids.add(row_id)
    if required_string(row, "owner") != "algorithmic/compatibility":
        raise SystemExit(f"representative row {row_id} owner mismatch")
    if required_string(row, "category") not in categories:
        raise SystemExit(f"representative row {row_id} has unknown category")
    if required_string(row, "expected_classification") != "PASS":
        raise SystemExit(f"representative row {row_id} expected classification must be PASS")
    path = REPO_ROOT / required_string(row, "path")
    if not path.is_file() or path.parent != LEETCODE_ROOT:
        raise SystemExit(f"representative row {row_id} path must be a checked-in LeetCode fixture")
    command = required_string(row, "command")
    expected_command = f"target/debug/sifr check {path.relative_to(REPO_ROOT)}"
    if command != expected_command:
        raise SystemExit(f"representative row {row_id} command must be {expected_command}")
    timeout = row.get("timeout_seconds")
    if not isinstance(timeout, int) or timeout <= 0:
        raise SystemExit(f"representative row {row_id} timeout_seconds must be positive")
    required_string(row, "result_artifact")


def required_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"required string field missing: {key}")
    return value


def run_representative_subset() -> list[dict[str, Any]]:
    payload = load_profile_manifest()
    ensure_sifr_bin(DEFAULT_SIFR_BIN)
    result_path = REPO_ROOT / payload["representative_subset"][0]["result_artifact"]
    results = []
    variants = []
    for row in payload["representative_subset"]:
        variant, result_entry = run_manifest_fixture(row)
        variants.append(variant)
        results.append(result_entry)
    write_results_artifact(result_path, results)
    return variants


def run_manifest_fixture(row: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
    path = REPO_ROOT / str(row["path"])
    started = time.perf_counter()
    try:
        result = run_fixture(DEFAULT_SIFR_BIN, path, timeout_seconds=int(row["timeout_seconds"]))
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        status = "pass" if result.returncode == 0 else "fail"
        failures = [] if result.returncode == 0 else [f"exit={result.returncode} expected=0"]
    except subprocess.TimeoutExpired as exc:
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        result = subprocess.CompletedProcess(args=[], returncode=124, stdout=exc.stdout or "", stderr=exc.stderr or "")
        status = "fail"
        failures = [f"timeout after {exc.timeout}s"]
    print_case_timing(str(row["id"]), elapsed_ms, status)
    result_entry = result_entry_for_fixture(path, result)
    variant = {
        "label": str(row["id"]),
        "argv": ["target/debug/sifr", "check", str(path.relative_to(REPO_ROOT))],
        "status": status,
        "mismatches": failures,
        "expected_exit_code": 0,
        "actual_exit_code": result.returncode,
        "duration_ms": round(elapsed_ms, 3),
        "category": str(row["category"]),
        "owner": str(row["owner"]),
        "expected_classification": str(row["expected_classification"]),
        "result_artifact": str(row["result_artifact"]),
    }
    return variant, result_entry


def run_leetcode_full() -> list[dict[str, Any]]:
    payload = load_profile_manifest()
    full_corpus = payload["full_corpus"]
    result_path = REPO_ROOT / str(full_corpus["result_artifact"])
    taxonomy_path = REPO_ROOT / str(full_corpus["taxonomy_artifact"])
    taxonomy_md = REPO_ROOT / str(full_corpus["taxonomy_markdown"])
    delta_md = REPO_ROOT / str(full_corpus["delta_markdown"])
    ensure_sifr_bin(DEFAULT_SIFR_BIN)
    variants = []
    results = []
    for path in sorted(LEETCODE_ROOT.glob("*.sifr")):
        started = time.perf_counter()
        result = run_fixture(DEFAULT_SIFR_BIN, path, timeout_seconds=30)
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        status = "pass" if result.returncode == 0 else "fail"
        failures = [] if result.returncode == 0 else [f"exit={result.returncode} expected=0"]
        print_case_timing(path.stem, elapsed_ms, status)
        variants.append(
            {
                "label": path.stem,
                "argv": ["target/debug/sifr", "check", str(path.relative_to(REPO_ROOT))],
                "status": status,
                "mismatches": failures,
                "expected_exit_code": 0,
                "actual_exit_code": result.returncode,
                "duration_ms": round(elapsed_ms, 3),
                "result_artifact": str(result_path.relative_to(REPO_ROOT)),
            }
        )
        results.append(result_entry_for_fixture(path, result))
    write_results_artifact(result_path, results)
    build_taxonomy_artifacts(
        results_path=result_path,
        taxonomy_path=taxonomy_path,
        taxonomy_md=taxonomy_md,
        delta_md=delta_md,
        profile_manifest=payload,
    )
    return variants


def write_results_artifact(path: Path, results: list[dict[str, Any]]) -> None:
    status_counts = {key: 0 for key in ["PASS", "CHECK_ERROR", "RUN_ERROR", "NO_ORACLE"]}
    for result in results:
        status_counts[str(result["status"])] += 1
    payload = {
        "summary": {
            "case_count": len(results),
            "status_counts": status_counts,
        },
        "results": results,
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def result_entry_for_fixture(path: Path, result: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    fixture_slug = path.stem
    if result.returncode == 0:
        return {"fixture_slug": fixture_slug, "status": "PASS", "stages": []}
    return {
        "fixture_slug": fixture_slug,
        "status": "CHECK_ERROR",
        "failure_stage": "check",
        "stages": [
            {
                "stage": "check",
                "stdout": result.stdout[-4000:] if result.stdout else "",
                "stderr": result.stderr[-4000:] if result.stderr else "",
            }
        ],
    }


def build_taxonomy_artifacts(
    *,
    results_path: Path,
    taxonomy_path: Path,
    taxonomy_md: Path,
    delta_md: Path,
    profile_manifest: dict[str, Any],
) -> None:
    taxonomy = profile_manifest["taxonomy"]
    argv = [
        "python3",
        str(AREA_ROOT / "tools" / "build_full_corpus_failure_taxonomy.py"),
        "--results",
        str(results_path.relative_to(REPO_ROOT)),
        "--output-json",
        str(taxonomy_path.relative_to(REPO_ROOT)),
        "--output-md",
        str(taxonomy_md.relative_to(REPO_ROOT)),
        "--name",
        "leetcode-full",
        "--generated-on",
        str(taxonomy["generated_on"]),
        "--baseline-taxonomy",
        str(taxonomy["baseline_taxonomy"]),
        "--baseline-results",
        str(taxonomy["baseline_results"]),
        "--delta-md",
        str(delta_md.relative_to(REPO_ROOT)),
    ]
    result = subprocess.run(argv, cwd=REPO_ROOT, text=True, capture_output=True, check=False)
    if result.stdout:
        sys.stdout.write(result.stdout)
    if result.stderr:
        sys.stderr.write(result.stderr)
    if result.returncode != 0:
        raise SystemExit(f"failed to build LeetCode taxonomy artifacts: exit={result.returncode}")


def run_taxonomy_smoke() -> tuple[list[str], subprocess.CompletedProcess[str]]:
    output_json = (
        REPO_ROOT
        / "target"
        / "verification"
        / "areas"
        / "algorithmic_compatibility"
        / "taxonomy-smoke.json"
    )
    output_md = output_json.with_suffix(".md")
    argv = [
        "python3",
        str(AREA_ROOT / "tools" / "build_full_corpus_failure_taxonomy.py"),
        "--results",
        str(TAXONOMY_SMOKE_RESULTS.relative_to(REPO_ROOT)),
        "--output-json",
        str(output_json.relative_to(REPO_ROOT)),
        "--output-md",
        str(output_md.relative_to(REPO_ROOT)),
        "--name",
        "taxonomy-smoke",
        "--generated-on",
        "2026-06-13",
    ]
    result = subprocess.run(argv, cwd=REPO_ROOT, text=True, capture_output=True, check=False)
    if result.returncode == 0:
        payload = json.loads(output_json.read_text(encoding="utf-8"))
        category_counts = payload["summary"]["category_counts"]
        if payload["summary"]["failing_cases"] != 1:
            result = completed_with_failure(argv, "taxonomy smoke expected exactly one failing case")
        elif category_counts.get("python_stdlib_and_builtin_parity_gap") != 1:
            result = completed_with_failure(argv, "taxonomy smoke category classification changed")
    return argv, result


def run_leetcode_check() -> tuple[list[str], subprocess.CompletedProcess[str]]:
    if not LEETCODE_ROOT.is_dir():
        raise SystemExit(f"missing LeetCode corpus root: {LEETCODE_ROOT.relative_to(REPO_ROOT)}")
    paths = sorted(LEETCODE_ROOT.glob("*.sifr"))
    if not paths:
        raise SystemExit(f"no LeetCode fixtures found under {LEETCODE_ROOT.relative_to(REPO_ROOT)}")
    ensure_sifr_bin(DEFAULT_SIFR_BIN)
    failures: list[tuple[Path, subprocess.CompletedProcess[str]]] = []
    stdout_lines = [f"validating {len(paths)} LeetCode fixture(s) with sifr check"]
    started = time.monotonic()
    for index, path in enumerate(paths, start=1):
        result = run_fixture(DEFAULT_SIFR_BIN, path)
        if result.returncode == 0:
            stdout_lines.append(f"[{index}/{len(paths)}] PASS {path.relative_to(REPO_ROOT)}")
            continue
        stdout_lines.append(f"[{index}/{len(paths)}] FAIL {path.relative_to(REPO_ROOT)}")
        failures.append((path, result))
    elapsed = time.monotonic() - started
    stdout_lines.append(
        f"LeetCode corpus check completed: {len(paths) - len(failures)}/{len(paths)} passed in {elapsed:.1f}s"
    )
    stderr = render_failures(failures)
    argv = [str(DEFAULT_SIFR_BIN), "check", str(LEETCODE_ROOT.relative_to(REPO_ROOT))]
    return argv, subprocess.CompletedProcess(
        args=argv,
        returncode=1 if failures else 0,
        stdout="\n".join(stdout_lines) + "\n",
        stderr=stderr,
    )


def ensure_sifr_bin(sifr_bin: Path) -> None:
    if sifr_bin == DEFAULT_SIFR_BIN:
        subprocess.run(["cargo", "build", "--locked", "-q", "-p", "sifr"], cwd=REPO_ROOT, check=True)
    elif not sifr_bin.exists():
        raise SystemExit(f"missing Sifr CLI binary: {sifr_bin}")


def run_fixture(sifr_bin: Path, path: Path, *, timeout_seconds: int | None = None) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("SIFR_ARTIFACT_CACHE", "1")
    return subprocess.run(
        [str(sifr_bin), "check", str(path.relative_to(REPO_ROOT))],
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout_seconds,
        check=False,
    )


def render_failures(failures: list[tuple[Path, subprocess.CompletedProcess[str]]]) -> str:
    lines: list[str] = []
    for path, result in failures:
        lines.append(f"--- {path.relative_to(REPO_ROOT)} exit={result.returncode} ---")
        if result.stdout:
            lines.append(result.stdout[-4000:])
        if result.stderr:
            lines.append(result.stderr[-4000:])
    return "\n".join(lines)


def completed_with_failure(argv: list[str], stderr: str) -> subprocess.CompletedProcess[str]:
    return subprocess.CompletedProcess(args=argv, returncode=1, stdout="", stderr=f"{stderr}\n")


def print_case_timing(label: str, elapsed_ms: float, status: str) -> None:
    print(
        f"[sifr-case-timing] bucket=algorithmic_compatibility case={timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
