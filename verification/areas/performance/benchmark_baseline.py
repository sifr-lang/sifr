"""Validation and construction for performance budget baselines."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from benchmark_manifest import RUNNER_VERSION, BenchmarkCase, BenchmarkError


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
        cv = float(metrics["coefficient_variation"])
        if cv > case.stability_limit:
            raise BenchmarkError(
                f"baseline result {case_id} is unstable: coefficient_variation={cv:.6f} "
                f"limit={case.stability_limit:.6f}"
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


def sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()
