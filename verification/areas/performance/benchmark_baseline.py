"""Validation and construction for performance budget baselines."""

from __future__ import annotations

import hashlib
import math
from pathlib import Path
from typing import Any

from benchmark_manifest import RUNNER_VERSION, BenchmarkCase, BenchmarkError

WORK_HEADROOM_RATIO = 1.02
WORK_HEADROOM_FLOOR = 1_000_000
RSS_HEADROOM_RATIO = 1.10
RSS_HEADROOM_FLOOR = 32 * 1024 * 1024


def validate_baseline_capture(
    run_report: dict[str, Any], cases_by_id: dict[str, BenchmarkCase]
) -> None:
    for result in run_report.get("results", []):
        if not isinstance(result, dict):
            raise BenchmarkError("benchmark result entries must be objects")
        case_id = result.get("id")
        if case_id not in cases_by_id:
            raise BenchmarkError(
                f"baseline result references unknown benchmark id {case_id!r}"
            )
        if result.get("timed_out"):
            raise BenchmarkError(
                f"baseline capture rejected timed out benchmark {case_id}"
            )
        samples = result.get("samples_ms")
        metrics = result.get("metrics")
        cache = result.get("cache")
        if not isinstance(samples, list) or not all(
            isinstance(sample, int | float) for sample in samples
        ):
            raise BenchmarkError(
                f"baseline result {case_id} is missing numeric samples_ms"
            )
        if not isinstance(metrics, dict):
            raise BenchmarkError(f"baseline result {case_id} is missing metrics")
        for field in [
            "median_ms",
            "p95_ms",
            "mad_ms",
            "coefficient_variation",
            "peak_rss_bytes",
        ]:
            if field not in metrics:
                raise BenchmarkError(
                    f"baseline result {case_id} is missing metric {field}"
                )
        if not isinstance(cache, dict) or "hits" not in cache or "misses" not in cache:
            raise BenchmarkError(
                f"baseline result {case_id} is missing cache hit/miss metrics"
            )
        case = cases_by_id[str(case_id)]
        control_mode = result.get("control", {}).get("mode", "latency")
        if control_mode == "work":
            instruction_samples = result.get("samples_instructions")
            if (
                not isinstance(instruction_samples, list)
                or len(instruction_samples) != len(samples)
                or metrics.get("median_instructions") is None
            ):
                raise BenchmarkError(
                    f"work-controlled baseline result {case_id} is missing retired-instruction evidence"
                )
            cv = float(metrics["instructions_coefficient_variation"])
            stability_limit = case.work_stability_limit
        else:
            cv = float(metrics["coefficient_variation"])
            stability_limit = case.stability_limit
        if cv > stability_limit:
            raise BenchmarkError(
                f"baseline result {case_id} is unstable: coefficient_variation={cv:.6f} "
                f"limit={stability_limit:.6f}"
            )


def baseline_from_run(
    run_report: dict[str, Any], manifest: dict[str, Any], manifest_path: Path
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "runner_version": RUNNER_VERSION,
        "manifest_sha256": sha256(manifest_path),
        "default_stability_limit": manifest.get("default_stability_limit", 0.10),
        "metadata": run_report["metadata"],
        "results": run_report["results"],
    }


def work_budgets_from_run(
    budgets: dict[str, Any], run_report: dict[str, Any]
) -> dict[str, Any]:
    entries = budgets.get("budgets")
    if not isinstance(entries, list):
        raise BenchmarkError("performance budgets are missing the budgets list")
    results = run_report.get("results")
    if not isinstance(results, list):
        raise BenchmarkError("performance baseline is missing results")
    results_by_id = {
        result.get("id"): result
        for result in results
        if isinstance(result, dict) and isinstance(result.get("id"), str)
    }
    work_entries: list[dict[str, Any]] = []
    for entry in entries:
        if not isinstance(entry, dict):
            raise BenchmarkError("performance budget entries must be objects")
        case_id = entry.get("benchmark_id")
        result = results_by_id.get(case_id)
        if not isinstance(result, dict):
            raise BenchmarkError(
                f"work baseline is missing benchmark result {case_id!r}"
            )
        metrics = result.get("metrics")
        if not isinstance(metrics, dict) or not isinstance(
            metrics.get("median_instructions"), int
        ):
            raise BenchmarkError(
                f"work baseline result {case_id!r} is missing median_instructions"
            )
        if not isinstance(metrics.get("peak_rss_bytes"), int):
            raise BenchmarkError(
                f"work baseline result {case_id!r} is missing peak_rss_bytes"
            )
        median = int(metrics["median_instructions"])
        instruction_threshold = max(
            math.ceil(median * WORK_HEADROOM_RATIO), median + WORK_HEADROOM_FLOOR
        )
        peak_rss = int(metrics["peak_rss_bytes"])
        rss_threshold = max(
            math.ceil(peak_rss * RSS_HEADROOM_RATIO), peak_rss + RSS_HEADROOM_FLOOR
        )
        work_entries.append(
            {
                "benchmark_id": case_id,
                "budget_id": entry.get("budget_id"),
                "baseline_median_instructions": median,
                "threshold_median_instructions": instruction_threshold,
                "baseline_peak_rss_bytes": peak_rss,
                "threshold_peak_rss_bytes": rss_threshold,
            }
        )
    return {
        "version": 1,
        "runner_version": RUNNER_VERSION,
        "source_commit": run_report.get("metadata", {}).get(
            "work_baseline_source_commit"
        ),
        "host": {
            "host_os": run_report.get("metadata", {}).get("host_os"),
            "architecture": run_report.get("metadata", {}).get("architecture"),
            "work_counter_source": "darwin-rusage-instructions",
            "rss_counter_source": "darwin-rusage-process-tree",
        },
        "instruction_threshold_rule": (
            "max(baseline_median_instructions * 1.02, "
            "baseline_median_instructions + 1000000)"
        ),
        "rss_threshold_rule": (
            "max(baseline_peak_rss_bytes * 1.10, "
            "baseline_peak_rss_bytes + 33554432)"
        ),
        "budgets": work_entries,
    }


def sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()
