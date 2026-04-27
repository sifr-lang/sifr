#!/usr/bin/env python3

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

from phase31_leetcode_lib import (
    REPO_ROOT,
    build_inventory,
    build_runner_summary,
    detect_oracle_mode,
    load_seed_manifest,
    run_case,
    validate_seed_manifest,
)


class Phase31LeetCodeTests(unittest.TestCase):
    def test_detect_oracle_mode(self) -> None:
        self.assertEqual(detect_oracle_mode("assert value == 1\n"), "embedded_asserts")
        self.assertEqual(
            detect_oracle_mode("def main():\n    value = 1\n"),
            "no_oracle",
        )

    def test_seed_manifest_is_balanced(self) -> None:
        manifest = load_seed_manifest(
            REPO_ROOT / "verification" / "leetcode" / "phase31_seed_corpus.json"
        )
        summary = validate_seed_manifest(manifest)
        self.assertGreaterEqual(summary["seed_problem_count"], 50)
        self.assertTrue(all(count > 0 for count in summary["topic_counts"].values()))
        self.assertTrue(all(count > 0 for count in summary["difficulty_counts"].values()))

    def test_inventory_matches_fixture_directory(self) -> None:
        manifest = load_seed_manifest(
            REPO_ROOT / "verification" / "leetcode" / "phase31_seed_corpus.json"
        )
        inventory = build_inventory({entry["id"] for entry in manifest})
        self.assertEqual(len(inventory), 411)
        self.assertEqual(
            sum(1 for entry in inventory if entry["oracle_mode"] == "embedded_asserts"),
            411,
        )
        self.assertEqual(
            sum(1 for entry in inventory if entry["oracle_mode"] == "no_oracle"),
            0,
        )

    def test_runner_classifies_check_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            helper = Path(tmpdir) / "fake_sifr.py"
            helper.write_text(
                "import sys\n"
                "if sys.argv[1] == 'check':\n"
                "    print('check failed', file=sys.stderr)\n"
                "    raise SystemExit(1)\n"
                "raise SystemExit(0)\n"
            )
            result = run_case(
                {
                    "id": "9999",
                    "fixture_slug": "fake_case",
                    "sifr_path": "audits/leetcode/src/0001_two_sum.sifr",
                    "primary_topic": "arrays",
                    "difficulty": "easy",
                    "scope_classification": "in_scope",
                    "oracle": {"mode": "embedded_asserts"},
                },
                [sys.executable, str(helper)],
                1,
            )
        self.assertEqual(result["status"], "CHECK_ERROR")
        self.assertEqual(result["failure_stage"], "check")

    def test_runner_classifies_no_oracle_success(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            helper = Path(tmpdir) / "fake_sifr.py"
            helper.write_text(
                "import sys\n"
                "raise SystemExit(0)\n"
            )
            result = run_case(
                {
                    "id": "9998",
                    "fixture_slug": "fake_case_no_oracle",
                    "sifr_path": "audits/leetcode/src/0100_same_tree.sifr",
                    "primary_topic": "trees",
                    "difficulty": "easy",
                    "scope_classification": "in_scope",
                    "oracle": {"mode": "no_oracle"},
                },
                [sys.executable, str(helper)],
                1,
            )
        self.assertEqual(result["status"], "NO_ORACLE")
        self.assertIsNone(result["failure_stage"])

    def test_runner_classifies_timeout(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            helper = Path(tmpdir) / "fake_sifr.py"
            helper.write_text(
                "import sys\n"
                "import time\n"
                "time.sleep(2)\n"
                "raise SystemExit(0)\n"
            )
            result = run_case(
                {
                    "id": "9997",
                    "fixture_slug": "fake_case_timeout",
                    "sifr_path": "audits/leetcode/src/0001_two_sum.sifr",
                    "primary_topic": "arrays",
                    "difficulty": "easy",
                    "scope_classification": "in_scope",
                    "oracle": {"mode": "embedded_asserts"},
                },
                [sys.executable, str(helper)],
                1,
            )
        self.assertEqual(result["status"], "TIMEOUT")
        self.assertEqual(result["failure_stage"], "check")

    def test_runner_summary_counts(self) -> None:
        summary = build_runner_summary(
            [
                {
                    "status": "PASS",
                    "primary_topic": "arrays",
                    "difficulty": "easy",
                    "scope_classification": "in_scope",
                },
                {
                    "status": "NO_ORACLE",
                    "primary_topic": "trees",
                    "difficulty": "medium",
                    "scope_classification": "in_scope",
                },
            ]
        )
        self.assertEqual(summary["case_count"], 2)
        self.assertEqual(summary["status_counts"]["PASS"], 1)
        self.assertEqual(summary["status_counts"]["NO_ORACLE"], 1)

    def test_runner_script_prefers_existing_release_binary(self) -> None:
        source = (REPO_ROOT / "scripts" / "run_phase31_leetcode.py").read_text()
        self.assertIn('target" / "release" / "sifr', source)
        self.assertIn("--build-release-if-missing", source)


if __name__ == "__main__":
    unittest.main()
