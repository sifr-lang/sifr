"""Cloud policy detects regressions without requiring quiet raw timings."""

import copy
import math
import random
import unittest

from cloud_contract import evaluate, threshold
from cloud_precision import PAIR_COUNTS
from cloud_statistics import FAMILY_ALPHA, PAIRS, assumption_screens, median_interval, schedule


def corpus(multiplier=1.0):
    manifest = {"cases": [{"id": "case", "kind": "command"}]}
    budgets = {"budgets": [{"benchmark_id": "case", "policy": "command-default",
                           "thresholds": {"median_ms": 1200, "p95_ms": 1500}, "cache": {}}]}
    generator = random.Random(128)
    values = [generator.uniform(600, 1500) for _ in range(PAIRS)]
    rows = []
    for order, value in zip(schedule("case"), values):
        endpoint = {"case_id": "case", "latencies_ms": [value], "peak_rss_bytes": 100000,
                    "cpu_time_ms": 0.0, "cache": {"hits": 0, "misses": 0}}
        candidate = copy.deepcopy(endpoint)
        candidate["latencies_ms"] = [value * multiplier]
        rows.append({"order": order, "baseline": endpoint, "candidate": candidate})
    return manifest, budgets, {"case": rows}


class CloudPolicyTests(unittest.TestCase):
    def test_same_source_under_large_shared_noise_passes(self):
        result = evaluate(*corpus())
        self.assertEqual(result["status"], "pass")
        self.assertFalse(result["p95_qualified"])

    def test_real_relative_regression_detected(self):
        self.assertEqual(evaluate(*corpus(1.5))["status"], "regression")

    def test_ambiguous_evidence_is_not_pass(self):
        manifest, budgets, pairs = corpus()
        generator = random.Random(99)
        excesses = [-50, 50] * 16
        generator.shuffle(excesses)
        for row, excess in zip(pairs["case"], excesses):
            row["candidate"]["latencies_ms"] = [threshold(budgets["budgets"][0], row["baseline"]["latencies_ms"][0]) + excess]
        self.assertEqual(evaluate(manifest, budgets, pairs)["status"], "inconclusive")

    def test_incomplete_or_reordered_pairs_rejected(self):
        for mutation in [lambda rows: rows.pop(), lambda rows: rows.reverse()]:
            manifest, budgets, pairs = corpus()
            mutation(pairs["case"])
            with self.assertRaises(ValueError):
                evaluate(manifest, budgets, pairs)

    def test_missing_counters_or_extra_internal_samples_rejected(self):
        for key, value in [("cpu_time_ms", None), ("peak_rss_bytes", 0), ("latencies_ms", [1, 2]), ("latencies_ms", [float('nan')])]:
            manifest, budgets, pairs = corpus()
            pairs["case"][0]["candidate"][key] = value
            with self.assertRaises(ValueError):
                evaluate(manifest, budgets, pairs)

    def test_empirical_editor_p95_breach_remains_strict(self):
        manifest, budgets, pairs = corpus()
        budgets["budgets"][0]["policy"] = "lsp-query"
        pairs["case"][0]["candidate"]["latencies_ms"] = [2000]
        pairs["case"][1]["candidate"]["latencies_ms"] = [2000]
        result = evaluate(manifest, budgets, pairs)
        self.assertEqual(result["status"], "regression")
        self.assertIn("observed-empirical-p95-budget", result["results"][0]["hard_failures"])

    def test_isolated_spike_does_not_change_p95_into_maximum(self):
        manifest, budgets, pairs = corpus()
        budgets["budgets"][0]["policy"] = "lsp-query"
        for row in pairs["case"]:
            for endpoint in ("baseline", "candidate"):
                row[endpoint]["latencies_ms"] = [100]
        pairs["case"][0]["candidate"]["latencies_ms"] = [2000]
        result = evaluate(manifest, budgets, pairs)
        self.assertEqual(result["status"], "pass")
        self.assertEqual(result["results"][0]["descriptive_p95_ms"], 100)
        self.assertFalse(result["p95_qualified"])

    def test_frozen_precision_plan_and_balanced_schedules(self):
        self.assertEqual(len(PAIR_COUNTS), 65)
        self.assertEqual(sum(PAIR_COUNTS.values()), 5120)
        for case_id, count in PAIR_COUNTS.items():
            orders = schedule(case_id)
            self.assertEqual(len(orders), count)
            self.assertEqual(orders.count("AB"), count // 2)
            self.assertEqual(orders, schedule(case_id))

    def test_heterogeneous_count_is_mandatory_without_internal_pseudoreplication(self):
        manifest, budgets, pairs = corpus()
        case_id = "build-project-003-cargo-manifest"
        manifest["cases"][0]["id"] = case_id
        budgets["budgets"][0]["benchmark_id"] = case_id
        template = pairs["case"][0]
        rows = []
        for order in schedule(case_id):
            row = copy.deepcopy(template)
            row["order"] = order
            for endpoint in ("baseline", "candidate"):
                row[endpoint]["case_id"] = case_id
                row[endpoint]["latencies_ms"] = [1000]
            rows.append(row)
        result = evaluate(manifest, budgets, {case_id: rows})
        self.assertEqual(result["status"], "pass")
        self.assertEqual(result["pairs_by_case"], {case_id: 192})
        rows.pop()
        with self.assertRaises(ValueError):
            evaluate(manifest, budgets, {case_id: rows})

    def test_large_count_interval_keeps_exact_family_error_allocation(self):
        for count in (64, 192, 928):
            lower, upper = median_interval(list(range(count)), 65, count)
            probability = sum(math.comb(count, j) for j in range(lower + 1)) / 2 ** count
            next_probability = probability + math.comb(count, lower + 1) / 2 ** count
            self.assertLessEqual(probability, FAMILY_ALPHA / (2 * 65))
            self.assertGreater(next_probability, FAMILY_ALPHA / (2 * 65))
            self.assertEqual(upper, count - lower - 1)

    def test_real_rss_and_cache_breaches_rejected(self):
        for key, value in [("peak_rss_bytes", 100000000), ("cache", {"hits": 0, "misses": 1})]:
            manifest, budgets, pairs = corpus()
            budgets["budgets"][0]["cache"] = {"max_misses": 0}
            pairs["case"][0]["candidate"][key] = value
            self.assertEqual(evaluate(manifest, budgets, pairs)["status"], "regression")

    def test_multiple_case_family_widens_interval(self):
        narrow = median_interval(list(range(PAIRS)), 1)
        wide = median_interval(list(range(PAIRS)), 65)
        self.assertLess(wide[0], narrow[0])
        self.assertGreater(wide[1], narrow[1])

    def test_order_drift_and_serial_screens(self):
        orders = schedule("case")
        for values, screen in [(list(range(PAIRS)), "temporal-drift"),
                               ([1 if order == "AB" else -1 for order in orders], "execution-order"),
                               ([-1] * 16 + [1] * 16, "serial-dependence")]:
            self.assertTrue(assumption_screens(values, orders, 1)[screen]["rejected"])

    def test_case_and_family_coverage_rejected(self):
        manifest, budgets, pairs = corpus()
        with self.assertRaises(ValueError):
            evaluate(manifest, budgets, {})
        budgets["budgets"].append(budgets["budgets"][0])
        with self.assertRaises(ValueError):
            evaluate(manifest, budgets, pairs)

    def test_internal_query_observations_do_not_inflate_pair_count(self):
        manifest, budgets, pairs = corpus()
        manifest["cases"][0].update(kind="frontend-query", measured=5)
        budgets["budgets"][0]["policy"] = "frontend-query-edit-loop"
        for row in pairs["case"]:
            for key in ("baseline", "candidate"):
                row[key]["latencies_ms"] *= 5
        result = evaluate(manifest, budgets, pairs)
        self.assertEqual(result["pairs_by_case"], {"case": PAIRS})
        self.assertEqual(result["status"], "pass")


if __name__ == "__main__":
    unittest.main()
