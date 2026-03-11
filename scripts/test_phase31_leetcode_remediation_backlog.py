#!/usr/bin/env python3

from __future__ import annotations

import unittest

from phase31_leetcode_remediation import (
    PRIORITY_ORDER,
    VALID_ITEM_TYPES,
    build_backlog,
    load_taxonomy_buckets,
)


class Phase31LeetCodeRemediationBacklogTests(unittest.TestCase):
    def test_every_bucket_has_backlog_entry(self) -> None:
        backlog = build_backlog()
        backlog_bucket_ids = {entry["bucket_id"] for entry in backlog["entries"]}
        taxonomy_bucket_ids = {bucket["bucket_id"] for bucket in load_taxonomy_buckets()}
        self.assertEqual(backlog_bucket_ids, taxonomy_bucket_ids)

    def test_entries_have_valid_types_and_policy(self) -> None:
        backlog = build_backlog()
        self.assertEqual(backlog["stale_blocker_policy"]["threshold_days"], 14)
        for entry in backlog["entries"]:
            self.assertIn(entry["item_type"], VALID_ITEM_TYPES)
            self.assertTrue(entry["acceptance_criteria"])
            self.assertIsNotNone(entry["owner"])

    def test_entries_are_priority_sorted(self) -> None:
        backlog = build_backlog()
        priorities = [PRIORITY_ORDER[entry["priority"]] for entry in backlog["entries"]]
        self.assertEqual(priorities, sorted(priorities))

    def test_dependencies_reference_known_buckets(self) -> None:
        backlog = build_backlog()
        known = {entry["bucket_id"] for entry in backlog["entries"]}
        for entry in backlog["entries"]:
            for dependency in entry["dependencies"]:
                self.assertIn(dependency, known)


if __name__ == "__main__":
    unittest.main()
