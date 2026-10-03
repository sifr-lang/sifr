"""Prospective shared-cloud budgets and complete paired-receipt evaluation."""

from __future__ import annotations

import math
import statistics

from cloud_precision import pairs_for_case
from cloud_statistics import POLICY_VERSION, median_decision, schedule


def threshold(budget: dict, baseline: float) -> float:
    policy = budget["policy"]
    cap = budget["thresholds"]["median_ms"]
    if policy == "command-default":
        return max(baseline * 1.10, baseline + 25)
    if policy == "formatter-command-default":
        return min(cap, max(baseline * 1.10, baseline + 25))
    if policy == "frontend-query-edit-loop":
        return max(baseline * 1.05, baseline + 2)
    if policy == "lsp-query":
        if budget["benchmark_id"] == "lsp-query-001-request-families":
            return cap
        return min(cap, max(baseline * 3, baseline + 5))
    raise ValueError("unknown shared-cloud budget policy")


def validate_endpoint(value: dict, case_id: str) -> None:
    if value.get("case_id") != case_id:
        raise ValueError("paired case identity mismatch")
    values = value.get("latencies_ms")
    if (not isinstance(values, list) or not values or
            any(isinstance(x, bool) or not isinstance(x, (float, int)) or not math.isfinite(x) or x <= 0 for x in values)):
        raise ValueError("cloud endpoint requires real positive latency samples")
    rss, cpu = value.get("peak_rss_bytes"), value.get("cpu_time_ms")
    if not isinstance(rss, int) or isinstance(rss, bool) or rss <= 0 or not isinstance(cpu, (float, int)) or isinstance(cpu, bool) or not math.isfinite(cpu) or cpu < 0:
        raise ValueError("cloud endpoint requires actual per-process RSS and CPU")
    for key in ("hits", "misses"):
        count = value.get("cache", {}).get(key)
        if not isinstance(count, int) or isinstance(count, bool) or count < 0:
            raise ValueError("invalid cloud cache evidence")


def empirical_p95(values: list[float]) -> float:
    """Descriptive finite-data quantile; no independence or population claim."""
    return sorted(values)[math.ceil(.95 * len(values)) - 1]


def p95_threshold(budget: dict, baseline: float) -> float:
    policy = budget["policy"]
    cap = budget["thresholds"]["p95_ms"]
    if policy == "formatter-command-default":
        return min(cap, max(baseline * 1.15, baseline + 50))
    if policy == "frontend-query-edit-loop":
        return max(baseline * 1.10, baseline + 5)
    if policy == "lsp-query":
        if budget["benchmark_id"] == "lsp-query-001-request-families":
            return cap
        return min(cap, max(baseline * 4, baseline + 10))
    raise ValueError("unknown shared-cloud empirical p95 policy")


def evaluate(manifest: dict, budgets: dict, pairs: dict) -> dict:
    expected = {case["id"] for case in manifest["cases"]}
    if set(pairs) != expected:
        raise ValueError("shared-cloud evidence must cover every manifest case exactly")
    by_budget = {row["benchmark_id"]: row for row in budgets["budgets"]}
    if set(by_budget) != expected or len(by_budget) != len(budgets["budgets"]):
        raise ValueError("cloud budget family differs from complete manifest")
    rows = []
    for case in sorted(manifest["cases"], key=lambda row: row["id"]):
        case_id = case["id"]
        observations = pairs[case_id]
        orders = schedule(case_id)
        count = pairs_for_case(case_id)
        if len(observations) != count or [row["order"] for row in observations] != orders:
            raise ValueError("cloud evidence is incomplete, reordered or selectively combined")
        excesses, candidate_values, baseline_values, baseline_rss, candidate_rss = [], [], [], [], []
        errors = []
        budget = by_budget[case_id]
        for observation in observations:
            a, b = observation["baseline"], observation["candidate"]
            for endpoint in (a, b):
                validate_endpoint(endpoint, case_id)
                expected_samples = 1 if case["kind"] == "command" else case["measured"]
                if len(endpoint["latencies_ms"]) != expected_samples:
                    raise ValueError("endpoint query observations differ from manifest; internal samples are not independent pairs")
            excesses.append(statistics.median(b["latencies_ms"]) - threshold(budget, statistics.median(a["latencies_ms"])))
            candidate_values.extend(b["latencies_ms"])
            baseline_values.extend(a["latencies_ms"])
            baseline_rss.append(a["peak_rss_bytes"])
            candidate_rss.append(b["peak_rss_bytes"])
            if b["cache"]["hits"] < budget.get("cache", {}).get("min_hits", 0):
                errors.append("cache-hit-budget")
            if b["cache"]["misses"] > budget.get("cache", {}).get("max_misses", math.inf):
                errors.append("cache-miss-budget")
        rss_cap = max(max(baseline_rss) * 1.10, max(baseline_rss) + 32 * 1024 * 1024)
        if max(candidate_rss) > rss_cap:
            errors.append("observed-rss-budget")
        tail_cap = None
        observed_p95 = empirical_p95(candidate_values)
        if budget["policy"] in {"lsp-query", "frontend-query-edit-loop", "formatter-command-default"}:
            tail_cap = p95_threshold(budget, empirical_p95(baseline_values))
            if observed_p95 > tail_cap:
                errors.append("observed-empirical-p95-budget")
        decision = median_decision(excesses, orders, len(expected), count)
        if errors:
            decision["status"] = "regression"
        rows.append({"case_id": case_id, "pairs": count, **decision, "hard_failures": sorted(set(errors)),
                     "observed_peak_rss_bytes": max(candidate_rss), "rss_ceiling_bytes": rss_cap,
                     "descriptive_p95_ms": observed_p95, "empirical_p95_ceiling_ms": tail_cap,
                     "p95_qualified": False})
    status = "regression" if any(row["status"] == "regression" for row in rows) else "inconclusive" if any(row["status"] == "inconclusive" for row in rows) else "pass"
    return {"policy": POLICY_VERSION, "status": status, "family_size": len(expected),
            "pairs_by_case": {case_id: pairs_for_case(case_id) for case_id in sorted(expected)}, "p95_qualified": False,
            "inference_assumptions": "stationary independent pair summaries; diagnostic screens do not prove independence",
            "results": rows}
