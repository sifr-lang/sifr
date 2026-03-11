#!/usr/bin/env python3

from __future__ import annotations

import unittest
from pathlib import Path

from phase31_leetcode_scorecard import ScorecardPaths, build_scorecard


ROOT = Path(__file__).resolve().parent.parent


class Phase31ScorecardTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scorecard = build_scorecard(
            ScorecardPaths(
                baseline_results=ROOT / "verification/leetcode/phase31_seed_results.json",
                wave_results=ROOT / "verification/leetcode/phase31_seed_results_wave1.json",
                taxonomy=ROOT / "verification/leetcode/phase31_failure_taxonomy.json",
                backlog=ROOT / "verification/leetcode/phase31_remediation_backlog.json",
            )
        )

    def test_totals_match_expected_delta(self) -> None:
        self.assertEqual(self.scorecard["totals"]["case_count"], 50)
        self.assertEqual(self.scorecard["totals"]["delta"]["PASS"], 3)
        self.assertEqual(self.scorecard["totals"]["delta"]["CHECK_ERROR"], -1)
        self.assertEqual(self.scorecard["totals"]["delta"]["RUN_ERROR"], -2)

    def test_fixed_cases_match_wave_one_targets(self) -> None:
        self.assertEqual(
            self.scorecard["fixed_cases"]["case_ids"],
            ["0069", "0151", "2235"],
        )

    def test_unresolved_handoff_count_matches_remaining_buckets(self) -> None:
        self.assertEqual(len(self.scorecard["unresolved_handoff"]), 9)
        self.assertEqual(
            sum(item["remaining_case_count"] for item in self.scorecard["unresolved_handoff"]),
            45,
        )

    def test_phase31_handoff_targets_roll_forward(self) -> None:
        handoff = {
            item["bucket_id"]: item["handoff_target_phase"]
            for item in self.scorecard["unresolved_handoff"]
        }
        self.assertEqual(handoff["type_system.optional_narrowing_and_union_ops"], "phase32")
        self.assertEqual(handoff["lowering.destructuring_target_support"], "phase32")
        self.assertEqual(handoff["stdlib.python_module_surface"], "phase32")


if __name__ == "__main__":
    unittest.main()
