#!/usr/bin/env python3
"""Performance trend baseline and stale-baseline policy gate."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from datetime import UTC, date, datetime
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
PERF_ROOT = REPO_ROOT / "verification" / "areas" / "performance"
PERF_DATA = PERF_ROOT / "data"
TREND_DATA = PERF_DATA / "trend"
DEFAULT_MANIFEST = PERF_DATA / "benchmark_manifest.json"
DEFAULT_TREND_BASELINES = TREND_DATA / "current.json"
DEFAULT_POLICY = TREND_DATA / "trend_policy.json"
RUNNER_VERSION = 1
SECONDS_PER_DAY = 86_400


class TrendPolicyError(Exception):
    pass


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    parser.add_argument("--trend-baselines", default=str(DEFAULT_TREND_BASELINES))
    parser.add_argument("--policy", default=str(DEFAULT_POLICY))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    try:
        if args.self_test:
            run_self_test()
            print("performance trend policy self-test passed")
            return 0

        manifest = load_json(Path(args.manifest))
        trend_baselines = load_json(Path(args.trend_baselines))
        policy = load_json(Path(args.policy))
        validate_trend_policy(manifest, trend_baselines, policy, manifest_path=Path(args.manifest))
        print("performance trend policy check passed")
        return 0
    except TrendPolicyError as error:
        print(f"performance trend policy error: {error}", file=sys.stderr)
        return 1


def validate_trend_policy(
    manifest: dict[str, Any],
    trend_baselines: dict[str, Any],
    policy: dict[str, Any],
    *,
    manifest_path: Path | None = None,
    today: date | None = None,
    now_unix: int | None = None,
) -> None:
    today = today or date.today()
    now_unix = int(datetime.now(tz=UTC).timestamp()) if now_unix is None else now_unix
    cases = validate_manifest(manifest)
    validated_policy = validate_policy(policy)
    deferrals = validate_deferrals(trend_baselines, cases, today)
    validate_renames(trend_baselines.get("renames", []), cases, today)
    validate_manifest_hash(trend_baselines, manifest_path)
    validate_metadata(trend_baselines, validated_policy, deferrals)
    validate_results(trend_baselines, cases, validated_policy, deferrals, now_unix)


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    if manifest.get("version") != 1:
        raise TrendPolicyError("benchmark manifest version must be 1")
    if manifest.get("runner_version") != RUNNER_VERSION:
        raise TrendPolicyError(f"benchmark manifest runner_version must be {RUNNER_VERSION}")
    cases_raw = manifest.get("cases")
    if not isinstance(cases_raw, list):
        raise TrendPolicyError("benchmark manifest cases must be a list")
    ids: list[str] = []
    cases: dict[str, dict[str, Any]] = {}
    for raw in cases_raw:
        if not isinstance(raw, dict):
            raise TrendPolicyError("benchmark manifest case entries must be objects")
        case_id = require_string(raw, "id", "manifest case")
        ids.append(case_id)
        cases[case_id] = raw
    if ids != sorted(ids):
        raise TrendPolicyError("benchmark ids must be sorted lexicographically for stable trend history")
    if len(ids) != len(set(ids)):
        raise TrendPolicyError("benchmark ids must be unique for stable trend history")
    return cases


def validate_policy(policy: dict[str, Any]) -> dict[str, Any]:
    if policy.get("schema_version") != 1:
        raise TrendPolicyError("trend policy schema_version must be 1")
    if require_string(policy, "owner", "trend policy") != "compiler/performance":
        raise TrendPolicyError("trend policy owner must be compiler/performance")
    baseline_window_days = policy.get("baseline_window_days")
    if not isinstance(baseline_window_days, int) or baseline_window_days <= 0:
        raise TrendPolicyError("trend policy baseline_window_days must be a positive integer")
    future_skew_days = policy.get("max_future_clock_skew_days", 0)
    if not isinstance(future_skew_days, int) or future_skew_days < 0:
        raise TrendPolicyError("trend policy max_future_clock_skew_days must be a non-negative integer")
    blocking = policy.get("blocking_policy")
    if not isinstance(blocking, dict):
        raise TrendPolicyError("trend policy must define blocking_policy")
    for field in [
        "create_pr",
        "merge",
        "nightly_release",
        "local_trend_delta",
        "checked_in_baseline_updates",
    ]:
        require_string(blocking, field, "trend policy blocking_policy")
    return {
        "baseline_window_days": baseline_window_days,
        "max_future_clock_skew_days": future_skew_days,
        "required_result_fields": require_string_list(policy, "required_result_fields", "trend policy"),
        "required_metrics": require_string_list(policy, "required_metrics", "trend policy"),
        "required_cache_fields": require_string_list(policy, "required_cache_fields", "trend policy"),
        "required_metadata_fields": require_string_list(policy, "required_metadata_fields", "trend policy"),
        "tracked_optional_metrics": optional_string_list(policy, "tracked_optional_metrics"),
    }


def validate_deferrals(
    trend_baselines: dict[str, Any],
    cases: dict[str, dict[str, Any]],
    today: date,
) -> list[dict[str, Any]]:
    raw_deferrals = trend_baselines.get("deferrals", [])
    if not isinstance(raw_deferrals, list):
        raise TrendPolicyError("trend baselines deferrals must be a list")
    validated: list[dict[str, Any]] = []
    seen: set[str] = set()
    for raw in raw_deferrals:
        if not isinstance(raw, dict):
            raise TrendPolicyError("trend deferrals must be objects")
        deferral_id = require_string(raw, "id", "trend deferral")
        if deferral_id in seen:
            raise TrendPolicyError(f"duplicate trend deferral id {deferral_id}")
        seen.add(deferral_id)
        require_string(raw, "owner", f"trend deferral {deferral_id}")
        reviewed_at = parse_iso_date(require_string(raw, "reviewed_at", f"trend deferral {deferral_id}"), deferral_id)
        expires = parse_iso_date(require_string(raw, "expires", f"trend deferral {deferral_id}"), deferral_id)
        if expires < today:
            raise TrendPolicyError(f"trend deferral {deferral_id} expired on {expires.isoformat()}")
        if reviewed_at > today:
            raise TrendPolicyError(f"trend deferral {deferral_id} reviewed_at is in the future")
        if not raw.get("benchmark_ids") and not raw.get("metadata_fields"):
            raise TrendPolicyError(f"trend deferral {deferral_id} must name benchmark_ids or metadata_fields")
        benchmark_ids = optional_string_list(raw, "benchmark_ids")
        if "*" in benchmark_ids:
            raise TrendPolicyError(f"trend deferral {deferral_id} cannot use wildcard benchmark_ids")
        unknown = sorted(set(benchmark_ids) - set(cases))
        if unknown:
            raise TrendPolicyError(f"trend deferral {deferral_id} references unknown benchmark ids: {unknown}")
        metadata_fields = optional_string_list(raw, "metadata_fields")
        require_string(raw, "rationale", f"trend deferral {deferral_id}")
        validated.append(raw | {"benchmark_ids": benchmark_ids, "metadata_fields": metadata_fields})
    return validated


def validate_renames(raw_renames: Any, cases: dict[str, dict[str, Any]], today: date) -> None:
    if not isinstance(raw_renames, list):
        raise TrendPolicyError("trend baselines renames must be a list")
    seen_old: set[str] = set()
    for raw in raw_renames:
        if not isinstance(raw, dict):
            raise TrendPolicyError("trend rename entries must be objects")
        old_id = require_string(raw, "old_id", "trend rename")
        new_id = require_string(raw, "new_id", f"trend rename {old_id}")
        if old_id == new_id:
            raise TrendPolicyError(f"trend rename {old_id} maps an id to itself")
        if old_id in cases:
            raise TrendPolicyError(f"trend rename {old_id} old_id is still an active benchmark id")
        if old_id in seen_old:
            raise TrendPolicyError(f"duplicate trend rename old_id {old_id}")
        seen_old.add(old_id)
        if new_id not in cases:
            raise TrendPolicyError(f"trend rename {old_id} points at unknown benchmark id {new_id}")
        parse_iso_date(require_string(raw, "reviewed_at", f"trend rename {old_id}"), old_id)
        expires = parse_iso_date(require_string(raw, "expires", f"trend rename {old_id}"), old_id)
        if expires < today:
            raise TrendPolicyError(f"trend rename {old_id} expired on {expires.isoformat()}")
        require_string(raw, "owner", f"trend rename {old_id}")
        require_string(raw, "rationale", f"trend rename {old_id}")


def validate_manifest_hash(trend_baselines: dict[str, Any], manifest_path: Path | None) -> None:
    manifest_hash = require_string(trend_baselines, "manifest_sha256", "trend baselines")
    if manifest_path is not None and manifest_hash != sha256(manifest_path):
        raise TrendPolicyError("trend baselines manifest_sha256 does not match benchmark_manifest.json")


def validate_metadata(
    trend_baselines: dict[str, Any],
    policy: dict[str, Any],
    deferrals: list[dict[str, Any]],
) -> None:
    if trend_baselines.get("schema_version") != 1:
        raise TrendPolicyError("trend baselines schema_version must be 1")
    if trend_baselines.get("runner_version") != RUNNER_VERSION:
        raise TrendPolicyError(f"trend baselines runner_version must be {RUNNER_VERSION}")
    metadata = trend_baselines.get("metadata")
    if not isinstance(metadata, dict):
        raise TrendPolicyError("trend baselines metadata must be an object")
    missing = sorted(
        field
        for field in policy["required_metadata_fields"]
        if field not in metadata and not has_metadata_deferral(field, deferrals)
    )
    if missing:
        raise TrendPolicyError(f"trend baselines metadata missing required fields without deferral: {missing}")


def validate_results(
    trend_baselines: dict[str, Any],
    cases: dict[str, dict[str, Any]],
    policy: dict[str, Any],
    deferrals: list[dict[str, Any]],
    now_unix: int,
) -> None:
    entries = trend_baselines.get("results")
    if not isinstance(entries, list):
        raise TrendPolicyError("trend baselines results must be a list")
    ids: list[str] = []
    seen: set[str] = set()
    for raw in entries:
        if not isinstance(raw, dict):
            raise TrendPolicyError("trend baseline result entries must be objects")
        for field in policy["required_result_fields"]:
            if field not in raw:
                raise TrendPolicyError(f"trend baseline result is missing required field {field}")
        result_id = require_string(raw, "id", "trend baseline result")
        if result_id not in cases:
            raise TrendPolicyError(f"trend baseline result references unknown benchmark id {result_id}")
        if result_id in seen:
            raise TrendPolicyError(f"duplicate trend baseline result id {result_id}")
        seen.add(result_id)
        ids.append(result_id)
        validate_result_shape(result_id, raw, policy)
        captured_at = require_int(raw, "baseline_captured_at_unix", f"trend baseline result {result_id}")
        validate_freshness(result_id, captured_at, policy, deferrals, now_unix)
    if ids != sorted(ids):
        raise TrendPolicyError("trend baseline results must be sorted lexicographically by benchmark id")
    missing = sorted(set(cases) - seen)
    missing_without_deferral = [case_id for case_id in missing if not has_benchmark_deferral(case_id, deferrals)]
    if missing_without_deferral:
        raise TrendPolicyError(f"benchmark ids missing current trend baselines: {missing_without_deferral}")


def validate_result_shape(result_id: str, raw: dict[str, Any], policy: dict[str, Any]) -> None:
    sample_count = require_int(raw, "sample_count", f"trend baseline result {result_id}")
    if sample_count <= 0:
        raise TrendPolicyError(f"trend baseline result {result_id} sample_count must be positive")
    samples = raw.get("samples_ms")
    if not isinstance(samples, list) or not all(isinstance(sample, int | float) for sample in samples):
        raise TrendPolicyError(f"trend baseline result {result_id} must include numeric samples_ms")
    if len(samples) != sample_count:
        raise TrendPolicyError(f"trend baseline result {result_id} sample_count does not match samples_ms")
    metrics = raw.get("metrics")
    if not isinstance(metrics, dict):
        raise TrendPolicyError(f"trend baseline result {result_id} metrics must be an object")
    for field in policy["required_metrics"]:
        if field not in metrics:
            raise TrendPolicyError(f"trend baseline result {result_id} is missing metric {field}")
        require_number(metrics, field, f"trend baseline result {result_id} metrics")
    for field in policy["tracked_optional_metrics"]:
        if field not in metrics:
            raise TrendPolicyError(f"trend baseline result {result_id} is missing tracked metric {field}")
        if metrics[field] is not None:
            require_number(metrics, field, f"trend baseline result {result_id} metrics")
    cache = raw.get("cache")
    if not isinstance(cache, dict):
        raise TrendPolicyError(f"trend baseline result {result_id} cache must be an object")
    for field in policy["required_cache_fields"]:
        if not isinstance(cache.get(field), int):
            raise TrendPolicyError(f"trend baseline result {result_id} cache.{field} must be an integer")


def validate_freshness(
    result_id: str,
    captured_at: int,
    policy: dict[str, Any],
    deferrals: list[dict[str, Any]],
    now_unix: int,
) -> None:
    future_skew_seconds = policy["max_future_clock_skew_days"] * SECONDS_PER_DAY
    if captured_at > now_unix + future_skew_seconds:
        raise TrendPolicyError(f"trend baseline result {result_id} captured_at is too far in the future")
    age_seconds = now_unix - captured_at
    if age_seconds > policy["baseline_window_days"] * SECONDS_PER_DAY and not has_benchmark_deferral(
        result_id,
        deferrals,
    ):
        age_days = age_seconds // SECONDS_PER_DAY
        raise TrendPolicyError(f"stale trend baseline for {result_id}: age_days={age_days}")


def has_benchmark_deferral(case_id: str, deferrals: list[dict[str, Any]]) -> bool:
    return any(case_id in deferral["benchmark_ids"] for deferral in deferrals)


def has_metadata_deferral(field: str, deferrals: list[dict[str, Any]]) -> bool:
    return any(field in deferral["metadata_fields"] or "*" in deferral["metadata_fields"] for deferral in deferrals)


def run_self_test() -> None:
    manifest = load_json(DEFAULT_MANIFEST)
    trend_baselines = load_json(DEFAULT_TREND_BASELINES)
    policy = load_json(DEFAULT_POLICY)
    validate_trend_policy(manifest, trend_baselines, policy, manifest_path=DEFAULT_MANIFEST)

    stale = copy.deepcopy(trend_baselines)
    stale["results"][0]["baseline_captured_at_unix"] = 1
    assert_fails(
        lambda: validate_trend_policy(manifest, stale, policy, manifest_path=DEFAULT_MANIFEST),
        "stale trend baseline",
    )

    missing = copy.deepcopy(trend_baselines)
    missing["results"] = missing["results"][1:]
    assert_fails(
        lambda: validate_trend_policy(manifest, missing, policy, manifest_path=DEFAULT_MANIFEST),
        "missing current trend baselines",
    )

    unknown = copy.deepcopy(trend_baselines)
    unknown["renames"] = [
        {
            "old_id": "old-benchmark-id",
            "new_id": "missing-new-id",
            "owner": "compiler/performance",
            "reviewed_at": "2026-06-16",
            "expires": "2026-07-31",
            "rationale": "negative self-test",
        }
    ]
    assert_fails(
        lambda: validate_trend_policy(manifest, unknown, policy, manifest_path=DEFAULT_MANIFEST),
        "unknown benchmark id",
    )

    active_old_id = copy.deepcopy(trend_baselines)
    manifest_ids = sorted(case["id"] for case in manifest["cases"])
    active_old_id["renames"] = [
        {
            "old_id": manifest_ids[0],
            "new_id": manifest_ids[1],
            "owner": "compiler/performance",
            "reviewed_at": "2026-06-16",
            "expires": "2026-07-31",
            "rationale": "negative self-test",
        }
    ]
    assert_fails(
        lambda: validate_trend_policy(manifest, active_old_id, policy, manifest_path=DEFAULT_MANIFEST),
        "old_id is still an active benchmark id",
    )

    wildcard = copy.deepcopy(trend_baselines)
    wildcard["deferrals"].append(
        {
            "id": "wildcard-benchmark-deferral",
            "owner": "compiler/performance",
            "reviewed_at": "2026-06-16",
            "expires": "2026-07-31",
            "benchmark_ids": ["*"],
            "rationale": "negative self-test",
        }
    )
    assert_fails(
        lambda: validate_trend_policy(manifest, wildcard, policy, manifest_path=DEFAULT_MANIFEST),
        "cannot use wildcard benchmark_ids",
    )

    wrong_hash = copy.deepcopy(trend_baselines)
    wrong_hash["manifest_sha256"] = "0" * 64
    assert_fails(
        lambda: validate_trend_policy(manifest, wrong_hash, policy, manifest_path=DEFAULT_MANIFEST),
        "manifest_sha256",
    )

    null_required_metric = copy.deepcopy(trend_baselines)
    null_required_metric["results"][0]["metrics"]["median_ms"] = None
    assert_fails(
        lambda: validate_trend_policy(manifest, null_required_metric, policy, manifest_path=DEFAULT_MANIFEST),
        "must be numeric",
    )

    no_metadata_deferral = copy.deepcopy(trend_baselines)
    no_metadata_deferral["deferrals"] = [
        deferral
        for deferral in no_metadata_deferral.get("deferrals", [])
        if "metadata_fields" not in deferral
    ]
    assert_fails(
        lambda: validate_trend_policy(manifest, no_metadata_deferral, policy, manifest_path=DEFAULT_MANIFEST),
        "metadata missing required fields",
    )


def assert_fails(action: Any, expected: str) -> None:
    try:
        action()
    except TrendPolicyError as error:
        if expected not in str(error):
            raise TrendPolicyError(f"negative trend policy self-test failed with wrong diagnostic: {error}") from error
        return
    raise TrendPolicyError(f"negative trend policy self-test did not fail; expected {expected!r}")


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise TrendPolicyError(f"failed to read {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise TrendPolicyError(f"malformed JSON in {path}: {error}") from error
    if not isinstance(value, dict):
        raise TrendPolicyError(f"{path} root must be an object")
    return value


def require_string(raw: dict[str, Any], field: str, owner: str) -> str:
    value = raw.get(field)
    if not isinstance(value, str) or not value:
        raise TrendPolicyError(f"{owner} field {field} must be a non-empty string")
    return value


def require_string_list(raw: dict[str, Any], field: str, owner: str) -> list[str]:
    value = raw.get(field)
    if not isinstance(value, list) or not value or not all(isinstance(item, str) and item for item in value):
        raise TrendPolicyError(f"{owner} field {field} must be a non-empty string list")
    return value


def optional_string_list(raw: dict[str, Any], field: str) -> list[str]:
    value = raw.get(field, [])
    if not isinstance(value, list) or not all(isinstance(item, str) and item for item in value):
        raise TrendPolicyError(f"field {field} must be a string list when present")
    return value


def require_number(raw: dict[str, Any], field: str, owner: str) -> float:
    value = raw.get(field)
    if not isinstance(value, int | float) or isinstance(value, bool):
        raise TrendPolicyError(f"{owner} field {field} must be numeric")
    return float(value)


def require_int(raw: dict[str, Any], field: str, owner: str) -> int:
    value = raw.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TrendPolicyError(f"{owner} field {field} must be an integer")
    return value


def parse_iso_date(value: str, owner: str) -> date:
    try:
        return date.fromisoformat(value)
    except ValueError as error:
        raise TrendPolicyError(f"{owner} date fields must use ISO date YYYY-MM-DD") from error


def sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


if __name__ == "__main__":
    raise SystemExit(main())
