#!/usr/bin/env python3
"""performance budget and waiver gate."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from datetime import date
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
PERF_ROOT = REPO_ROOT / "verification" / "areas" / "performance"
PERF_DATA = PERF_ROOT / "data"
DEFAULT_MANIFEST = PERF_DATA / "benchmark_manifest.json"
DEFAULT_BASELINES = PERF_DATA / "baselines.json"
DEFAULT_BUDGETS = PERF_DATA / "budgets.json"
DEFAULT_WAIVERS = PERF_DATA / "waivers.json"
NEGATIVE_ROOT = PERF_ROOT / "negative_seeds"
RUNNER_VERSION = 1
ALLOWED_WAIVER_OVERRIDE_KEYS = {"median_ms", "p95_ms", "peak_rss_bytes", "cache_hits"}
# The nearest-rank p95 used by the benchmark runner collapses to the maximum
# sample below 20 samples, making quick representative runs scheduler-bound.
MIN_P95_SAMPLE_COUNT = 20
EMPTY_WAIVERS = {
    "version": 1,
    "waivers": [],
}


class BudgetError(Exception):
    pass


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    parser.add_argument("--results", default=str(DEFAULT_BASELINES))
    parser.add_argument("--budgets", default=str(DEFAULT_BUDGETS))
    parser.add_argument("--waivers", default=str(DEFAULT_WAIVERS))
    parser.add_argument("--allow-subset", action="store_true")
    parser.add_argument("--expected-invocation-id", default="")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    try:
        if args.self_test:
            run_self_test()
            print("performance budget checker self-test passed")
            return 0

        manifest = load_json(Path(args.manifest))
        budgets = load_json(Path(args.budgets))
        waivers = load_json(Path(args.waivers))
        results = load_json(Path(args.results))
        check_budgets(
            manifest,
            budgets,
            waivers,
            results,
            allow_subset=args.allow_subset,
            report_skipped_p95=True,
            expected_invocation_id=args.expected_invocation_id or None,
        )
        print("performance budget check passed")
        return 0
    except BudgetError as error:
        print(f"performance budget error: {error}", file=sys.stderr)
        return 1


def check_budgets(
    manifest: dict[str, Any],
    budgets: dict[str, Any],
    waivers: dict[str, Any],
    results: dict[str, Any],
    *,
    allow_subset: bool = False,
    report_skipped_p95: bool = False,
    expected_invocation_id: str | None = None,
) -> None:
    cases = validate_manifest(manifest)
    budget_entries = validate_budgets(budgets, cases)
    waiver_entries = validate_waivers(waivers, cases, budget_entries)
    validate_results_shape(
        results,
        cases,
        allow_subset=allow_subset,
        expected_invocation_id=expected_invocation_id,
    )

    results_by_id = {result["id"]: result for result in results["results"]}
    failures: list[str] = []
    for case_id, budget in budget_entries.items():
        result = results_by_id.get(case_id)
        if result is None:
            if not allow_subset:
                failures.append(f"{case_id}: missing result for budget {budget['budget_id']}")
            continue
        failures.extend(compare_result(result, cases[case_id], budget, waiver_entries))

    if failures:
        raise BudgetError("budget check failed:\n" + "\n".join(failures))
    if report_skipped_p95:
        report_p95_skips(results_by_id, budget_entries)


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    if manifest.get("version") != 1:
        raise BudgetError("benchmark manifest version must be 1")
    cases_raw = manifest.get("cases")
    if not isinstance(cases_raw, list):
        raise BudgetError("benchmark manifest cases must be a list")
    cases: dict[str, dict[str, Any]] = {}
    budget_ids: set[str] = set()
    for raw in cases_raw:
        if not isinstance(raw, dict):
            raise BudgetError("benchmark manifest case entries must be objects")
        case_id = require_string(raw, "id", "manifest case")
        budget_id = require_string(raw, "budget_id", f"manifest case {case_id}")
        if case_id in cases:
            raise BudgetError(f"duplicate benchmark case id {case_id}")
        if budget_id in budget_ids:
            raise BudgetError(f"duplicate benchmark budget id {budget_id}")
        cases[case_id] = raw
        budget_ids.add(budget_id)
    return cases


def validate_budgets(
    budgets: dict[str, Any],
    cases: dict[str, dict[str, Any]],
) -> dict[str, dict[str, Any]]:
    if budgets.get("version") != 1:
        raise BudgetError("budgets version must be 1")
    entries = budgets.get("budgets")
    if not isinstance(entries, list):
        raise BudgetError("budgets must contain a budgets list")
    by_case: dict[str, dict[str, Any]] = {}
    by_budget_id: set[str] = set()
    for raw in entries:
        if not isinstance(raw, dict):
            raise BudgetError("budget entries must be objects")
        case_id = require_string(raw, "benchmark_id", "budget entry")
        budget_id = require_string(raw, "budget_id", f"budget entry {case_id}")
        if case_id not in cases:
            raise BudgetError(f"budget {budget_id} references unknown benchmark id {case_id}")
        if budget_id != cases[case_id]["budget_id"]:
            raise BudgetError(f"budget {budget_id} does not match manifest budget id {cases[case_id]['budget_id']} for {case_id}")
        if case_id in by_case:
            raise BudgetError(f"duplicate budget for benchmark id {case_id}")
        if budget_id in by_budget_id:
            raise BudgetError(f"duplicate budget id {budget_id}")
        thresholds = raw.get("thresholds")
        if not isinstance(thresholds, dict):
            raise BudgetError(f"budget {budget_id} is missing thresholds")
        for field in ["median_ms", "p95_ms", "timeout_ms"]:
            require_number(thresholds, field, f"budget {budget_id} thresholds")
        if "peak_rss_bytes" in thresholds and thresholds["peak_rss_bytes"] is not None:
            require_number(thresholds, "peak_rss_bytes", f"budget {budget_id} thresholds")
        cache = raw.get("cache", {})
        if not isinstance(cache, dict):
            raise BudgetError(f"budget {budget_id} cache policy must be an object")
        for field in ["min_hits", "max_misses"]:
            if field in cache and (not isinstance(cache[field], int) or cache[field] < 0):
                raise BudgetError(f"budget {budget_id} cache.{field} must be a non-negative integer")
        by_case[case_id] = raw
        by_budget_id.add(budget_id)

    missing = sorted(set(cases) - set(by_case))
    if missing:
        raise BudgetError(f"budgets are missing benchmark ids: {missing}")
    return by_case


def validate_waivers(
    waivers: dict[str, Any],
    cases: dict[str, dict[str, Any]],
    budgets: dict[str, dict[str, Any]],
) -> list[dict[str, Any]]:
    if waivers.get("version") != 1:
        raise BudgetError("waivers version must be 1")
    entries = waivers.get("waivers")
    if not isinstance(entries, list):
        raise BudgetError("waivers must contain a waivers list")
    budget_ids = {budget["budget_id"] for budget in budgets.values()}
    today = date.today()
    seen: set[str] = set()
    validated = []
    for raw in entries:
        if not isinstance(raw, dict):
            raise BudgetError("waiver entries must be objects")
        waiver_id = require_string(raw, "id", "waiver")
        if waiver_id in seen:
            raise BudgetError(f"duplicate waiver id {waiver_id}")
        seen.add(waiver_id)
        owner = require_string(raw, "owner", f"waiver {waiver_id}")
        issue = require_string(raw, "issue", f"waiver {waiver_id}")
        if not owner:
            raise BudgetError(f"waiver {waiver_id} owner must not be empty")
        if not (issue.startswith("https://github.com/") or "#" in issue):
            raise BudgetError(f"waiver {waiver_id} must link to a GitHub issue")
        expires = parse_date(require_string(raw, "expires", f"waiver {waiver_id}"), waiver_id, "expires")
        parse_date(require_string(raw, "created", f"waiver {waiver_id}"), waiver_id, "created")
        if expires < today:
            raise BudgetError(f"waiver {waiver_id} expired on {expires.isoformat()}")
        benchmark_ids = require_string_list(raw, "benchmark_ids", f"waiver {waiver_id}")
        waiver_budget_ids = require_string_list(raw, "budget_ids", f"waiver {waiver_id}")
        unknown_benchmarks = sorted(set(benchmark_ids) - set(cases))
        unknown_budgets = sorted(set(waiver_budget_ids) - budget_ids)
        if unknown_benchmarks:
            raise BudgetError(f"waiver {waiver_id} references unknown benchmark ids: {unknown_benchmarks}")
        if unknown_budgets:
            raise BudgetError(f"waiver {waiver_id} references unknown budget ids: {unknown_budgets}")
        override = raw.get("override")
        if not isinstance(override, dict) or not override:
            raise BudgetError(f"waiver {waiver_id} override must be a non-empty object")
        unknown_override_keys = sorted(set(override) - ALLOWED_WAIVER_OVERRIDE_KEYS)
        if unknown_override_keys:
            raise BudgetError(f"waiver {waiver_id} attempts to waive non-performance or unsupported fields: {unknown_override_keys}")
        require_string(raw, "rationale", f"waiver {waiver_id}")
        require_string(raw, "removal_criteria", f"waiver {waiver_id}")
        validated.append(raw)
    return validated


def validate_results_shape(
    results: dict[str, Any],
    cases: dict[str, dict[str, Any]],
    *,
    allow_subset: bool = False,
    expected_invocation_id: str | None = None,
) -> None:
    if results.get("schema_version") != 1:
        raise BudgetError("benchmark results schema_version must be 1")
    if results.get("runner_version") != RUNNER_VERSION:
        raise BudgetError(f"benchmark results runner_version must be {RUNNER_VERSION}")
    if expected_invocation_id is not None and results.get("invocation_id") != expected_invocation_id:
        raise BudgetError(
            "benchmark results invocation_id does not match this producer invocation: "
            f"expected={expected_invocation_id!r} actual={results.get('invocation_id')!r}"
        )
    entries = results.get("results")
    if not isinstance(entries, list):
        raise BudgetError("benchmark results must contain a results list")
    seen: set[str] = set()
    for raw in entries:
        if not isinstance(raw, dict):
            raise BudgetError("benchmark result entries must be objects")
        result_id = require_string(raw, "id", "benchmark result")
        if result_id not in cases:
            raise BudgetError(f"benchmark result references unknown benchmark id {result_id}")
        if result_id in seen:
            raise BudgetError(f"duplicate benchmark result id {result_id}")
        seen.add(result_id)
        require_string(raw, "budget_id", f"benchmark result {result_id}")
        sample_count = raw.get("sample_count")
        if not isinstance(sample_count, int) or isinstance(sample_count, bool) or sample_count <= 0:
            raise BudgetError(f"benchmark result {result_id} field sample_count must be a positive integer")
        samples = raw.get("samples_ms")
        if not isinstance(samples, list) or len(samples) != sample_count:
            raise BudgetError(f"benchmark result {result_id} samples_ms length must match sample_count")
        if not all(isinstance(sample, int | float) and not isinstance(sample, bool) for sample in samples):
            raise BudgetError(f"benchmark result {result_id} samples_ms entries must be numeric")
        metrics = raw.get("metrics")
        if not isinstance(metrics, dict):
            raise BudgetError(f"benchmark result {result_id} is missing metrics")
        for field in [
            "median_ms",
            "p95_ms",
            "mad_ms",
            "coefficient_variation",
            "peak_rss_bytes",
        ]:
            if field not in metrics:
                raise BudgetError(f"benchmark result {result_id} is missing metric {field}")
        require_number(metrics, "median_ms", f"benchmark result {result_id} metrics")
        require_number(metrics, "p95_ms", f"benchmark result {result_id} metrics")
        require_number(metrics, "mad_ms", f"benchmark result {result_id} metrics")
        require_number(metrics, "coefficient_variation", f"benchmark result {result_id} metrics")
        if metrics["peak_rss_bytes"] is not None:
            require_number(metrics, "peak_rss_bytes", f"benchmark result {result_id} metrics")
        cache = raw.get("cache")
        if not isinstance(cache, dict) or not isinstance(cache.get("hits"), int) or not isinstance(cache.get("misses"), int):
            raise BudgetError(f"benchmark result {result_id} is missing cache hit/miss metrics")

    missing = sorted(set(cases) - seen)
    if missing and not allow_subset:
        raise BudgetError(f"benchmark results are missing benchmark ids: {missing}")


def compare_result(
    result: dict[str, Any],
    case: dict[str, Any],
    budget: dict[str, Any],
    waivers: list[dict[str, Any]],
) -> list[str]:
    case_id = result["id"]
    budget_id = budget["budget_id"]
    failures: list[str] = []
    if result.get("timed_out"):
        failures.append(
            format_failure(
                case_id,
                budget_id,
                "timeout",
                "true",
                "false",
                waivers,
                waiverable=False,
            )
        )
        return failures
    metrics = result["metrics"]
    thresholds = budget["thresholds"]
    stability_limit = float(case.get("stability_limit", 0.10))
    if float(metrics["coefficient_variation"]) > stability_limit:
        failures.append(
            format_failure(
                case_id,
                budget_id,
                "coefficient_variation",
                metrics["coefficient_variation"],
                stability_limit,
                waivers,
                waiverable=False,
            )
        )
    checks = [
        ("median_ms", metrics["median_ms"], thresholds["median_ms"]),
    ]
    if should_enforce_p95(result):
        checks.append(("p95_ms", metrics["p95_ms"], thresholds["p95_ms"]))
    if thresholds.get("peak_rss_bytes") is not None and metrics.get("peak_rss_bytes") is not None:
        checks.append(("peak_rss_bytes", metrics["peak_rss_bytes"], thresholds["peak_rss_bytes"]))
    for metric, measured, threshold in checks:
        if float(measured) > float(threshold) and not has_waiver(case_id, budget_id, metric, waivers):
            failures.append(format_failure(case_id, budget_id, metric, measured, threshold, waivers))

    cache_policy = budget.get("cache", {})
    cache = result["cache"]
    if "min_hits" in cache_policy and int(cache["hits"]) < int(cache_policy["min_hits"]):
        metric = "cache_hits"
        if not has_waiver(case_id, budget_id, metric, waivers):
            failures.append(
                format_failure(
                    case_id,
                    budget_id,
                    metric,
                    cache["hits"],
                    cache_policy["min_hits"],
                    waivers,
                )
            )
    if "max_misses" in cache_policy and int(cache["misses"]) > int(cache_policy["max_misses"]):
        failures.append(
            format_failure(
                case_id,
                budget_id,
                "cache_misses",
                cache["misses"],
                cache_policy["max_misses"],
                waivers,
                waiverable=False,
            )
        )
    return failures


def should_enforce_p95(result: dict[str, Any]) -> bool:
    return int(result["sample_count"]) >= MIN_P95_SAMPLE_COUNT


def report_p95_skips(results_by_id: dict[str, dict[str, Any]], budget_entries: dict[str, dict[str, Any]]) -> None:
    skipped = [
        f"{case_id}:{results_by_id[case_id]['sample_count']}"
        for case_id in sorted(budget_entries)
        if case_id in results_by_id and not should_enforce_p95(results_by_id[case_id])
    ]
    if skipped:
        print(
            f"performance budget note: skipped p95_ms for {len(skipped)} benchmark(s) "
            f"with sample_count < {MIN_P95_SAMPLE_COUNT}: {', '.join(skipped)}",
            file=sys.stderr,
        )


def has_waiver(case_id: str, budget_id: str, metric: str, waivers: list[dict[str, Any]]) -> bool:
    for waiver in waivers:
        if case_id in waiver["benchmark_ids"] and budget_id in waiver["budget_ids"] and metric in waiver["override"]:
            return True
    return False


def format_failure(
    case_id: str,
    budget_id: str,
    metric: str,
    measured: Any,
    threshold: Any,
    waivers: list[dict[str, Any]],
    *,
    waiverable: bool = True,
) -> str:
    status = "waiver_available_but_not_matching" if waiverable and waivers else "no_waiver"
    if not waiverable:
        status = "not_waiverable"
    return f"{case_id} ({budget_id}) {metric} regression: " f"measured={measured} threshold={threshold} waiver_status={status}"


def run_self_test() -> None:
    manifest = load_json(DEFAULT_MANIFEST)
    budgets = load_json(DEFAULT_BUDGETS)
    waivers = load_json(DEFAULT_WAIVERS)
    baselines = load_json(DEFAULT_BASELINES)
    check_budgets(manifest, budgets, waivers, baselines)

    assert_budget_fails(
        "budget_median_regression_result.json",
        "median_ms regression",
        allow_subset=True,
    )
    assert_result_fails(
        make_result_seed(spike_twenty_sample_p95),
        "p95_ms regression",
        allow_subset=True,
    )
    assert_budget_fails(
        "budget_rss_regression_result.json",
        "peak_rss_bytes regression",
        allow_subset=True,
    )
    assert_budget_fails("budget_timeout_result.json", "timeout regression", allow_subset=True)
    assert_budget_fails("budget_missing_result.json", "missing benchmark ids")
    assert_budget_fails("budget_unknown_id_result.json", "unknown benchmark id", allow_subset=True)
    assert_budget_fails("budget_malformed_result.json", "missing metric", allow_subset=True)
    assert_budget_fails("unstable_result.json", "coefficient_variation regression", allow_subset=True)
    assert_waiver_fails("expired_waiver.json", "expired")
    assert_waiver_fails("malformed_waiver.json", "owner")
    assert_waiver_fails("correctness_waiver.json", "non-performance")

    assert_result_fails(
        make_result_seed(spike_five_sample_median),
        "median_ms regression",
        allow_subset=True,
    )
    check_budgets(manifest, budgets, EMPTY_WAIVERS, make_result_seed(spike_p95_below_threshold))
    assert_result_fails(make_result_seed(spike_p95_at_threshold), "p95_ms regression", allow_subset=True)

    active_waiver = load_json(NEGATIVE_ROOT / "active_median_waiver.json")
    median_regression = load_json(NEGATIVE_ROOT / "budget_median_regression_result.json")
    check_budgets(manifest, budgets, active_waiver, median_regression, allow_subset=True)

    stale_result = copy.deepcopy(baselines)
    stale_result["invocation_id"] = "prior-invocation"
    assert_result_fails(
        stale_result,
        "invocation_id does not match",
        expected_invocation_id="current-invocation",
    )


def assert_budget_fails(seed: str, expected: str, *, allow_subset: bool = False) -> None:
    result = load_json(NEGATIVE_ROOT / seed)
    assert_result_fails(result, expected, allow_subset=allow_subset, seed=seed)


def assert_result_fails(
    result: dict[str, Any],
    expected: str,
    *,
    allow_subset: bool = False,
    seed: str = "generated result",
    expected_invocation_id: str | None = None,
) -> None:
    try:
        check_budgets(
            load_json(DEFAULT_MANIFEST),
            load_json(DEFAULT_BUDGETS),
            EMPTY_WAIVERS,
            result,
            allow_subset=allow_subset,
            expected_invocation_id=expected_invocation_id,
        )
    except BudgetError as error:
        if expected not in str(error):
            raise BudgetError(f"negative budget seed {seed} failed with wrong diagnostic: {error}") from error
        return
    raise BudgetError(f"negative budget seed {seed} did not fail")


def assert_waiver_fails(seed: str, expected: str) -> None:
    try:
        check_budgets(
            load_json(DEFAULT_MANIFEST),
            load_json(DEFAULT_BUDGETS),
            load_json(NEGATIVE_ROOT / seed),
            load_json(DEFAULT_BASELINES),
        )
    except BudgetError as error:
        if expected not in str(error):
            raise BudgetError(f"negative waiver seed {seed} failed with wrong diagnostic: {error}") from error
        return
    raise BudgetError(f"negative waiver seed {seed} did not fail")


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise BudgetError(f"failed to read {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise BudgetError(f"malformed JSON in {path}: {error}") from error
    if not isinstance(value, dict):
        raise BudgetError(f"{path} root must be an object")
    return value


def require_string(raw: dict[str, Any], field: str, owner: str) -> str:
    value = raw.get(field)
    if not isinstance(value, str) or not value:
        raise BudgetError(f"{owner} field {field} must be a non-empty string")
    return value


def require_string_list(raw: dict[str, Any], field: str, owner: str) -> list[str]:
    value = raw.get(field)
    if not isinstance(value, list) or not value or not all(isinstance(item, str) and item for item in value):
        raise BudgetError(f"{owner} field {field} must be a non-empty string list")
    return value


def require_number(raw: dict[str, Any], field: str, owner: str) -> float:
    value = raw.get(field)
    if not isinstance(value, int | float) or isinstance(value, bool):
        raise BudgetError(f"{owner} field {field} must be numeric")
    return float(value)


def parse_date(value: str, waiver_id: str, field: str) -> date:
    try:
        return date.fromisoformat(value)
    except ValueError as error:
        raise BudgetError(f"waiver {waiver_id} field {field} must be ISO date YYYY-MM-DD") from error


def spike_five_sample_median(result: dict[str, Any]) -> None:
    spike_metric(result, "check-project-004-project-graph", "median_ms")


def spike_twenty_sample_p95(result: dict[str, Any]) -> None:
    spike_metric(result, "interactive-tooling-foundation-002-warm-diagnostics-query", "p95_ms")


def spike_p95_below_threshold(result: dict[str, Any]) -> None:
    force_sample_count(
        result,
        "interactive-tooling-foundation-002-warm-diagnostics-query",
        MIN_P95_SAMPLE_COUNT - 1,
    )
    spike_twenty_sample_p95(result)


def spike_p95_at_threshold(result: dict[str, Any]) -> None:
    force_sample_count(
        result,
        "interactive-tooling-foundation-002-warm-diagnostics-query",
        MIN_P95_SAMPLE_COUNT,
    )
    spike_twenty_sample_p95(result)


def spike_metric(result: dict[str, Any], case_id: str, metric: str) -> None:
    for entry in result["results"]:
        if entry["id"] == case_id:
            entry["metrics"][metric] = 999_999
            return
    raise BudgetError(f"self-test seed did not find {case_id}")


def force_sample_count(result: dict[str, Any], case_id: str, sample_count: int) -> None:
    for entry in result["results"]:
        if entry["id"] == case_id:
            entry["sample_count"] = sample_count
            entry["samples_ms"] = entry["samples_ms"][:sample_count]
            return
    raise BudgetError(f"self-test seed did not find {case_id}")


def make_result_seed(mutator: Any) -> dict[str, Any]:
    result = copy.deepcopy(load_json(DEFAULT_BASELINES))
    mutator(result)
    return result


if __name__ == "__main__":
    raise SystemExit(main())
