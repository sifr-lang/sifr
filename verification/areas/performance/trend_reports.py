"""Trend comparison report helpers for performance benchmark runs."""

from __future__ import annotations

import time
from copy import deepcopy
from typing import Any


CORE_METRICS = ["median_ms", "p95_ms", "mad_ms", "coefficient_variation", "peak_rss_bytes"]
OPTIONAL_SIZE_METRICS = ["emitted_rust_lines", "emitted_rust_bytes", "generated_binary_bytes"]


class TrendReportError(Exception):
    pass


def build_trend_report(
    run_report: dict[str, Any],
    trend_baselines: dict[str, Any],
    runner_version: int,
) -> dict[str, Any]:
    if trend_baselines.get("schema_version") != 1:
        raise TrendReportError("trend baselines schema_version must be 1")
    if trend_baselines.get("runner_version") != runner_version:
        raise TrendReportError(f"trend baselines runner_version must be {runner_version}")
    baseline_results = trend_baselines.get("results")
    if not isinstance(baseline_results, list):
        raise TrendReportError("trend baselines results must be a list")
    baselines_by_id = {}
    for raw in baseline_results:
        if not isinstance(raw, dict) or not isinstance(raw.get("id"), str):
            raise TrendReportError("trend baseline entries must be objects with string ids")
        baselines_by_id[raw["id"]] = raw

    comparisons = []
    missing_baselines = []
    for result in run_report["results"]:
        baseline = baselines_by_id.get(result["id"])
        if baseline is None:
            missing_baselines.append(result["id"])
            continue
        comparisons.append(compare_trend_result(result, baseline))
    if missing_baselines:
        raise TrendReportError(f"trend baselines are missing benchmark ids: {missing_baselines}")

    review_required = [
        comparison["id"]
        for comparison in comparisons
        if comparison["classification"] == "regression_outside_noise"
    ]
    return {
        "schema_version": 1,
        "runner_version": runner_version,
        "run_id": run_report["run_id"],
        "generated_at_unix": int(time.time()),
        "local_trend_delta_blocking": False,
        "review_policy": "local trend deltas are advisory; regressions outside the noise band require owner review on approved reference hardware",
        "metadata": run_report["metadata"],
        "baseline_metadata": trend_baselines.get("metadata", {}),
        "summary": {
            "benchmarks_compared": len(comparisons),
            "reference_review_required": len(review_required),
            "reference_review_benchmark_ids": review_required,
        },
        "results": comparisons,
    }


def compare_trend_result(result: dict[str, Any], baseline: dict[str, Any]) -> dict[str, Any]:
    current_metrics = require_metrics(result, "benchmark result")
    baseline_metrics = require_metrics(baseline, "trend baseline result")
    current_cache = require_cache(result, "benchmark result")
    baseline_cache = require_cache(baseline, "trend baseline result")
    median_delta_percent = percent_delta(current_metrics["median_ms"], baseline_metrics["median_ms"])
    p95_delta_percent = percent_delta(current_metrics["p95_ms"], baseline_metrics["p95_ms"])
    rss_delta_percent = nullable_percent_delta(
        current_metrics.get("peak_rss_bytes"),
        baseline_metrics.get("peak_rss_bytes"),
    )
    noise_band_percent = max(5.0, float(baseline_metrics.get("coefficient_variation", 0.0)) * 200.0)
    sample_count = int(result["sample_count"])
    baseline_sample_count = int(baseline["sample_count"])
    classification = classify_trend_delta(
        median_delta_percent,
        p95_delta_percent,
        noise_band_percent,
        sample_count,
        baseline_sample_count,
    )
    return {
        "id": result["id"],
        "sample_count": sample_count,
        "baseline_sample_count": baseline_sample_count,
        "current": metric_snapshot(current_metrics) | {"cache": current_cache},
        "baseline": metric_snapshot(baseline_metrics)
        | {"cache": baseline_cache, "captured_at_unix": baseline["baseline_captured_at_unix"]},
        "delta": metric_delta(current_metrics, baseline_metrics)
        | {
            "median_ms": round(float(current_metrics["median_ms"]) - float(baseline_metrics["median_ms"]), 3),
            "median_percent": median_delta_percent,
            "p95_ms": round(float(current_metrics["p95_ms"]) - float(baseline_metrics["p95_ms"]), 3),
            "p95_percent": p95_delta_percent,
            "mad_ms": round(float(current_metrics["mad_ms"]) - float(baseline_metrics["mad_ms"]), 3),
            "peak_rss_bytes": nullable_delta(
                current_metrics.get("peak_rss_bytes"),
                baseline_metrics.get("peak_rss_bytes"),
            ),
            "peak_rss_percent": rss_delta_percent,
            "cache_hits": int(current_cache["hits"]) - int(baseline_cache["hits"]),
            "cache_misses": int(current_cache["misses"]) - int(baseline_cache["misses"]),
        },
        "noise_band_percent": round(noise_band_percent, 3),
        "classification": classification,
        "reference_review_required": classification == "regression_outside_noise",
        "local_blocking": False,
    }


def metric_snapshot(metrics: dict[str, Any]) -> dict[str, Any]:
    return {field: metrics.get(field) for field in [*CORE_METRICS, *OPTIONAL_SIZE_METRICS]}


def metric_delta(current_metrics: dict[str, Any], baseline_metrics: dict[str, Any]) -> dict[str, Any]:
    deltas: dict[str, Any] = {}
    for field in OPTIONAL_SIZE_METRICS:
        deltas[field] = nullable_delta(current_metrics.get(field), baseline_metrics.get(field))
        deltas[f"{field}_percent"] = nullable_percent_delta(current_metrics.get(field), baseline_metrics.get(field))
    return deltas


def classify_trend_delta(
    median_delta_percent: float,
    p95_delta_percent: float,
    noise_band_percent: float,
    sample_count: int,
    baseline_sample_count: int,
) -> str:
    if sample_count < baseline_sample_count:
        return "sample_count_below_baseline"
    if median_delta_percent > noise_band_percent or p95_delta_percent > noise_band_percent:
        return "regression_outside_noise"
    if median_delta_percent < -noise_band_percent or p95_delta_percent < -noise_band_percent:
        return "improvement_outside_noise"
    return "within_noise"


def require_metrics(result: dict[str, Any], owner: str) -> dict[str, Any]:
    metrics = result.get("metrics")
    if not isinstance(metrics, dict):
        raise TrendReportError(f"{owner} {result.get('id')!r} is missing metrics")
    for field in ["median_ms", "p95_ms", "mad_ms", "coefficient_variation"]:
        if not isinstance(metrics.get(field), int | float):
            raise TrendReportError(f"{owner} {result.get('id')!r} metric {field} must be numeric")
    return metrics


def require_cache(result: dict[str, Any], owner: str) -> dict[str, int]:
    cache = result.get("cache")
    if not isinstance(cache, dict) or not isinstance(cache.get("hits"), int) or not isinstance(cache.get("misses"), int):
        raise TrendReportError(f"{owner} {result.get('id')!r} is missing cache hit/miss metrics")
    return {"hits": int(cache["hits"]), "misses": int(cache["misses"])}


def percent_delta(current: float, baseline: float) -> float:
    baseline = float(baseline)
    if baseline == 0.0:
        # Zero baselines are invalid upstream data; keep local trend reports non-crashing.
        return 0.0
    return round(((float(current) - baseline) / baseline) * 100.0, 3)


def nullable_delta(current: Any, baseline: Any) -> int | None:
    if not isinstance(current, int | float) or not isinstance(baseline, int | float):
        return None
    return int(current) - int(baseline)


def nullable_percent_delta(current: Any, baseline: Any) -> float | None:
    if not isinstance(current, int | float) or not isinstance(baseline, int | float):
        return None
    return percent_delta(float(current), float(baseline))


def run_self_test() -> None:
    current = sample_result("build-sample", emitted_lines=12, emitted_bytes=240, binary_bytes=1024)
    baseline = sample_result("build-sample", emitted_lines=10, emitted_bytes=200, binary_bytes=1000)
    baseline["baseline_captured_at_unix"] = 1
    report = build_trend_report(
        {"run_id": "self-test", "metadata": {}, "results": [current]},
        {"schema_version": 1, "runner_version": 1, "metadata": {}, "results": [baseline]},
        1,
    )
    comparison = report["results"][0]
    if comparison["current"]["emitted_rust_lines"] != 12:
        raise TrendReportError("self-test did not preserve current emitted Rust line count")
    if comparison["delta"]["generated_binary_bytes"] != 24:
        raise TrendReportError("self-test did not compute generated binary byte delta")
    assert_fails(
        lambda: build_trend_report(
            {"run_id": "self-test", "metadata": {}, "results": [current]},
            {"schema_version": 1, "runner_version": 1, "metadata": {}, "results": []},
            1,
        ),
        "missing benchmark ids",
    )
    invalid_metric = deepcopy(current)
    invalid_metric["metrics"]["median_ms"] = None
    assert_fails(
        lambda: build_trend_report(
            {"run_id": "self-test", "metadata": {}, "results": [invalid_metric]},
            {"schema_version": 1, "runner_version": 1, "metadata": {}, "results": [baseline]},
            1,
        ),
        "median_ms",
    )


def sample_result(
    case_id: str,
    *,
    emitted_lines: int,
    emitted_bytes: int,
    binary_bytes: int,
) -> dict[str, Any]:
    return {
        "id": case_id,
        "sample_count": 1,
        "samples_ms": [10.0],
        "metrics": {
            "median_ms": 10.0,
            "p95_ms": 10.0,
            "mad_ms": 0.0,
            "coefficient_variation": 0.0,
            "peak_rss_bytes": 100,
            "emitted_rust_lines": emitted_lines,
            "emitted_rust_bytes": emitted_bytes,
            "generated_binary_bytes": binary_bytes,
        },
        "cache": {"hits": 0, "misses": 0},
    }


def assert_fails(action: Any, expected: str) -> None:
    try:
        action()
    except TrendReportError as error:
        if expected not in str(error):
            raise TrendReportError(f"negative trend report self-test failed with wrong diagnostic: {error}") from error
        return
    raise TrendReportError(f"negative trend report self-test did not fail; expected {expected!r}")
