"""Finite shared-VM inference; guarantees require stationary independent pairs."""

from __future__ import annotations

import math
import random
import statistics

from cloud_precision import pairs_for_case

PAIRS = 32
FAMILY_ALPHA = 0.05
SCREEN_PERMUTATIONS = 4095
POLICY_VERSION = "shared-cloud-median-v2"


def schedule(case_id: str) -> list[str]:
    order = ["AB", "BA"] * (pairs_for_case(case_id) // 2)
    random.Random(POLICY_VERSION + case_id).shuffle(order)
    return order


def median_interval(values: list[float], family_size: int, count: int = PAIRS) -> tuple[float, float]:
    if (isinstance(count, bool) or not isinstance(count, int) or count < PAIRS or count % 2
            or len(values) != count or family_size < 1 or any(not math.isfinite(x) for x in values)):
        raise ValueError("cloud inference requires its declared finite pair count and family")
    tail = FAMILY_ALPHA / (2 * family_size)
    cutoff = -1
    cumulative, term = 0, 1
    for rank in range(count // 2):
        cumulative += term
        probability = cumulative / 2 ** count
        if probability <= tail:
            cutoff = rank
        term = term * (count - rank) // (rank + 1)
    if cutoff < 0:
        return -math.inf, math.inf
    ordered = sorted(values)
    return ordered[cutoff], ordered[-cutoff - 1]


def ranks(values: list[float]) -> list[float]:
    ordered = sorted(set(values))
    mapped = {}
    position = 0
    for value in ordered:
        count = values.count(value)
        mapped[value] = position + (count - 1) / 2
        position += count
    mean = (len(values) - 1) / 2
    return [mapped[value] - mean for value in values]


def assumption_screens(values: list[float], orders: list[str], family_size: int, count: int = PAIRS) -> dict:
    """Permutation diagnostics support, but do not prove, inference assumptions."""
    if len(values) != count or len(orders) != count:
        raise ValueError("incomplete cloud diagnostic data")
    ranked = ranks(values)
    times = [i - (count - 1) / 2 for i in range(count)]
    signs = [1 if order == "AB" else -1 for order in orders]

    def statistics_for(items):
        return [abs(sum(a * b for a, b in zip(items, times))),
                abs(sum(a * b for a, b in zip(items, signs))),
                abs(sum(a * b for a, b in zip(items, items[1:]))) ]

    observed = statistics_for(ranked)
    exceedances = [1, 1, 1]
    generator = random.Random(POLICY_VERSION + "assumption-screens")
    for _ in range(SCREEN_PERMUTATIONS):
        permuted = ranked.copy()
        generator.shuffle(permuted)
        for i, value in enumerate(statistics_for(permuted)):
            exceedances[i] += value >= observed[i]
    limit = FAMILY_ALPHA / (3 * family_size)
    return {name: {"p_value": count / (SCREEN_PERMUTATIONS + 1),
                   "rejected": count / (SCREEN_PERMUTATIONS + 1) <= limit}
            for name, count in zip(["temporal-drift", "execution-order", "serial-dependence"], exceedances)}


def median_decision(excesses: list[float], orders: list[str], family_size: int, count: int = PAIRS) -> dict:
    lower, upper = median_interval(excesses, family_size, count)
    screens = assumption_screens(excesses, orders, family_size, count)
    status = "pass" if upper <= 0 else "regression" if lower > 0 else "inconclusive"
    if any(screen["rejected"] for screen in screens.values()):
        status = "inconclusive"
    return {"status": status, "median_excess_ms": statistics.median(excesses),
            "interval_ms": [lower, upper], "assumption_screens": screens}
