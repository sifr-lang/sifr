"""Deterministic fixtures for the performance budget checker."""

from __future__ import annotations

import copy
import io
import statistics
from contextlib import redirect_stderr
from typing import Any

import check_budgets as checker


def run_self_test() -> None:
    manifest = checker.load_json(checker.DEFAULT_MANIFEST)
    budgets = governed_budgets()
    waivers = checker.load_json(checker.DEFAULT_WAIVERS)
    baselines = checker.load_json(checker.DEFAULT_BASELINES)
    checker.check_budgets(manifest, budgets, waivers, baselines)

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
    assert_budget_fails(
        "budget_timeout_result.json", "timeout regression", allow_subset=True
    )
    assert_budget_fails("budget_missing_result.json", "missing benchmark ids")
    assert_budget_fails(
        "budget_unknown_id_result.json", "unknown benchmark id", allow_subset=True
    )
    assert_budget_fails(
        "budget_malformed_result.json", "missing metric", allow_subset=True
    )
    assert_budget_fails(
        "unstable_result.json", "coefficient_variation regression", allow_subset=True
    )
    assert_waiver_fails("expired_waiver.json", "expired")
    assert_waiver_fails("malformed_waiver.json", "owner")
    assert_waiver_fails("correctness_waiver.json", "non-performance")

    assert_result_fails(
        make_result_seed(spike_five_sample_median),
        "median_ms regression",
        allow_subset=True,
    )
    checker.check_budgets(
        manifest,
        budgets,
        checker.EMPTY_WAIVERS,
        make_result_seed(spike_p95_below_threshold),
    )
    assert_result_fails(
        make_result_seed(spike_p95_at_threshold),
        "p95_ms regression",
        allow_subset=True,
    )

    active_waiver = checker.load_json(
        checker.NEGATIVE_ROOT / "active_median_waiver.json"
    )
    median_regression = checker.load_json(
        checker.NEGATIVE_ROOT / "budget_median_regression_result.json"
    )
    checker.check_budgets(
        manifest, budgets, active_waiver, median_regression, allow_subset=True
    )

    stale_result = copy.deepcopy(baselines)
    stale_result["invocation_id"] = "prior-invocation"
    assert_result_fails(
        stale_result,
        "invocation_id does not match",
        expected_invocation_id="current-invocation",
    )
    assert_result_fails(
        make_instruction_seed(baselines, budgets, regress=True),
        "median_instructions regression",
    )
    note_stream = io.StringIO()
    with redirect_stderr(note_stream):
        checker.check_budgets(
            manifest,
            budgets,
            checker.EMPTY_WAIVERS,
            make_instruction_boundary_noise_seed(baselines, budgets),
        )
    note = note_stream.getvalue()
    for field in ["case=", "measured=", "threshold=", "mad=", "uncertainty="]:
        if field not in note:
            raise checker.BudgetError(
                f"instruction boundary-noise self-test note is missing {field}"
            )
    assert_result_fails(
        make_instruction_capped_noise_seed(baselines, budgets),
        "median_instructions regression",
    )
    assert_result_fails(
        make_work_rss_seed(baselines, budgets),
        "peak_rss_bytes regression",
    )
    checker.check_budgets(
        manifest,
        budgets,
        checker.EMPTY_WAIVERS,
        make_instruction_seed(baselines, budgets, regress=False),
    )


def assert_budget_fails(
    seed: str, expected: str, *, allow_subset: bool = False
) -> None:
    result = checker.load_json(checker.NEGATIVE_ROOT / seed)
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
        checker.check_budgets(
            checker.load_json(checker.DEFAULT_MANIFEST),
            governed_budgets(),
            checker.EMPTY_WAIVERS,
            result,
            allow_subset=allow_subset,
            expected_invocation_id=expected_invocation_id,
        )
    except checker.BudgetError as error:
        if expected not in str(error):
            raise checker.BudgetError(
                f"negative budget seed {seed} failed with wrong diagnostic: {error}"
            ) from error
        return
    raise checker.BudgetError(f"negative budget seed {seed} did not fail")


def assert_waiver_fails(seed: str, expected: str) -> None:
    try:
        checker.check_budgets(
            checker.load_json(checker.DEFAULT_MANIFEST),
            governed_budgets(),
            checker.load_json(checker.NEGATIVE_ROOT / seed),
            checker.load_json(checker.DEFAULT_BASELINES),
        )
    except checker.BudgetError as error:
        if expected not in str(error):
            raise checker.BudgetError(
                f"negative waiver seed {seed} failed with wrong diagnostic: {error}"
            ) from error
        return
    raise checker.BudgetError(f"negative waiver seed {seed} did not fail")


def governed_budgets() -> dict[str, Any]:
    return checker.apply_work_budgets(
        checker.load_json(checker.DEFAULT_BUDGETS),
        checker.load_json(checker.DEFAULT_WORK_BUDGETS),
    )


def make_instruction_seed(
    baselines: dict[str, Any], budgets: dict[str, Any], *, regress: bool
) -> dict[str, Any]:
    result = copy.deepcopy(baselines)
    budget = budgets["budgets"][0]
    case_id = budget["benchmark_id"]
    threshold = int(budget["work_thresholds"]["median_instructions"])
    baseline = int(budget["work_baseline"]["median_instructions"])
    for entry in result["results"]:
        if entry["id"] != case_id:
            continue
        measured = threshold + 1 if regress else baseline
        sample_count = int(entry["sample_count"])
        entry["samples_instructions"] = [measured] * sample_count
        entry["control"] = {"mode": "work"}
        entry["metrics"].update(
            {
                "median_instructions": measured,
                "p95_instructions": measured,
                "instructions_mad": 0,
                "instructions_coefficient_variation": 0.0,
                "median_cycles_per_instruction": 0.5,
                "peak_rss_bytes": budget["work_baseline"]["peak_rss_bytes"],
            }
        )
        if not regress:
            entry["metrics"]["median_ms"] = (
                float(budget["thresholds"]["median_ms"]) + 1.0
            )
            entry["metrics"]["p95_ms"] = float(budget["thresholds"]["p95_ms"]) + 1.0
        return result
    raise checker.BudgetError(
        f"instruction self-test could not find benchmark {case_id}"
    )


def make_work_rss_seed(
    baselines: dict[str, Any], budgets: dict[str, Any]
) -> dict[str, Any]:
    result = make_instruction_seed(baselines, budgets, regress=False)
    budget = budgets["budgets"][0]
    case_id = budget["benchmark_id"]
    for entry in result["results"]:
        if entry["id"] == case_id:
            entry["metrics"]["peak_rss_bytes"] = (
                int(budget["work_thresholds"]["peak_rss_bytes"]) + 1
            )
            return result
    raise checker.BudgetError(f"work RSS self-test could not find benchmark {case_id}")


def make_instruction_boundary_noise_seed(
    baselines: dict[str, Any], budgets: dict[str, Any]
) -> dict[str, Any]:
    result = make_instruction_seed(baselines, budgets, regress=True)
    budget = budgets["budgets"][0]
    threshold = int(budget["work_thresholds"]["median_instructions"])
    for entry in result["results"]:
        if entry["id"] == budget["benchmark_id"]:
            measured = threshold + 100_000
            set_instruction_samples(
                entry,
                [
                    measured - 100_000,
                    measured - 50_000,
                    measured,
                    measured + 50_000,
                    measured + 100_000,
                ],
            )
            return result
    raise checker.BudgetError("instruction boundary-noise self-test lost its benchmark")


def make_instruction_capped_noise_seed(
    baselines: dict[str, Any], budgets: dict[str, Any]
) -> dict[str, Any]:
    result = make_instruction_seed(baselines, budgets, regress=True)
    budget = budgets["budgets"][0]
    threshold = int(budget["work_thresholds"]["median_instructions"])
    for entry in result["results"]:
        if entry["id"] == budget["benchmark_id"]:
            measured = threshold + max(1_000_000, threshold // 100)
            spread = max(1, measured // 100)
            set_instruction_samples(
                entry,
                [
                    measured - 2 * spread,
                    measured - spread,
                    measured,
                    measured + spread,
                    measured + 2 * spread,
                ],
            )
            return result
    raise checker.BudgetError("instruction capped-noise self-test lost its benchmark")


def set_instruction_samples(entry: dict[str, Any], samples: list[int]) -> None:
    if int(entry["sample_count"]) != len(samples):
        raise checker.BudgetError(
            "instruction uncertainty self-test requires five-sample baseline data"
        )
    median = statistics.median(samples)
    mad = statistics.median(abs(sample - median) for sample in samples)
    mean = statistics.mean(samples)
    cv = statistics.pstdev(samples) / mean
    entry["samples_instructions"] = samples
    entry["metrics"]["median_instructions"] = round(median)
    entry["metrics"]["p95_instructions"] = max(samples)
    entry["metrics"]["instructions_mad"] = round(mad)
    entry["metrics"]["instructions_coefficient_variation"] = round(cv, 6)


def spike_five_sample_median(result: dict[str, Any]) -> None:
    spike_metric(result, "check-project-004-project-graph", "median_ms")


def spike_twenty_sample_p95(result: dict[str, Any]) -> None:
    spike_metric(
        result,
        "interactive-tooling-foundation-002-warm-diagnostics-query",
        "p95_ms",
    )


def spike_p95_below_threshold(result: dict[str, Any]) -> None:
    force_sample_count(
        result,
        "interactive-tooling-foundation-002-warm-diagnostics-query",
        checker.MIN_P95_SAMPLE_COUNT - 1,
    )
    spike_twenty_sample_p95(result)


def spike_p95_at_threshold(result: dict[str, Any]) -> None:
    force_sample_count(
        result,
        "interactive-tooling-foundation-002-warm-diagnostics-query",
        checker.MIN_P95_SAMPLE_COUNT,
    )
    spike_twenty_sample_p95(result)


def spike_metric(result: dict[str, Any], case_id: str, metric: str) -> None:
    for entry in result["results"]:
        if entry["id"] == case_id:
            entry["metrics"][metric] = 999_999
            return
    raise checker.BudgetError(f"self-test seed did not find {case_id}")


def force_sample_count(result: dict[str, Any], case_id: str, sample_count: int) -> None:
    for entry in result["results"]:
        if entry["id"] == case_id:
            entry["sample_count"] = sample_count
            entry["samples_ms"] = entry["samples_ms"][:sample_count]
            return
    raise checker.BudgetError(f"self-test seed did not find {case_id}")


def make_result_seed(mutator: Any) -> dict[str, Any]:
    result = copy.deepcopy(checker.load_json(checker.DEFAULT_BASELINES))
    mutator(result)
    return result
