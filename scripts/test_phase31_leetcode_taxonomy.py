#!/usr/bin/env python3

from __future__ import annotations

import unittest

from phase31_leetcode_lib import REPO_ROOT, load_json
from phase31_leetcode_taxonomy import (
    build_spot_audit,
    build_taxonomy,
    classify_failure,
    load_manifest_cases,
    load_results,
)


class Phase31LeetCodeTaxonomyTests(unittest.TestCase):
    def test_classify_recursive_tree_type_gap(self) -> None:
        result = {
            "status": "CHECK_ERROR",
            "stages": [{"stderr": "type error: unknown type: 'TreeNode'"}],
        }
        classification = classify_failure(result)
        self.assertEqual(
            classification["bucket_id"], "type_system.recursive_node_forward_reference"
        )

    def test_classify_mutable_binding_codegen_gap(self) -> None:
        result = {
            "status": "RUN_ERROR",
            "stages": [{"stderr": "error[E0384]: cannot assign twice to immutable variable"}],
        }
        classification = classify_failure(result)
        self.assertEqual(classification["bucket_id"], "codegen.mutable_binding_emission")

    def test_taxonomy_builds_from_seed_results(self) -> None:
        results = load_results(REPO_ROOT / "verification" / "leetcode" / "phase31_seed_results.json")
        manifest_cases = load_manifest_cases(
            REPO_ROOT / "verification" / "leetcode" / "phase31_seed_corpus.json"
        )
        payload = build_taxonomy(results, manifest_cases)
        self.assertEqual(payload["summary"]["classified_case_count"], 48)
        self.assertGreaterEqual(payload["summary"]["bucket_count"], 8)

    def test_spot_audit_threshold_holds(self) -> None:
        results = load_results(REPO_ROOT / "verification" / "leetcode" / "phase31_seed_results.json")
        manifest_cases = load_manifest_cases(
            REPO_ROOT / "verification" / "leetcode" / "phase31_seed_corpus.json"
        )
        payload = build_taxonomy(results, manifest_cases)
        spot_audit = build_spot_audit(payload)
        self.assertTrue(spot_audit["passed"])
        self.assertGreaterEqual(spot_audit["accuracy"], 0.9)


if __name__ == "__main__":
    unittest.main()
