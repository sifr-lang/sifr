"""The Rust semantic-property manifest has a one-run contract."""

from __future__ import annotations

import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

from .property_and_fuzz import run_property_suite, run_rust_property


class RustPropertyContractTests(unittest.TestCase):
    def entry(self) -> dict:
        test_name = "semantic_property_tests::normalization_idempotent"
        return {
            "id": "PROP-TYPE-0001", "kind": "rust-test", "crate": "sifr_type_system",
            "test_name": test_name, "note": "generated normalization",
            "reproduction_command": ["cargo", "test", "-p", "sifr_type_system",
                                     "--lib", test_name, "--", "--exact"],
        }

    def test_declared_repeat_runs_is_rejected_without_execution(self) -> None:
        entry = self.entry()
        entry["repeat_runs"] = 2
        with patch("subprocess.run") as run:
            result = run_rust_property(entry=entry, repo_root=Path("/tmp"))
        run.assert_not_called()
        self.assertEqual(result["variants"][0]["mismatches"], ["repeat_runs.unused"])

    def test_rust_property_runs_exactly_once(self) -> None:
        with patch("subprocess.run", return_value=subprocess.CompletedProcess([], 0, "ok", "")) as run:
            result = run_rust_property(entry=self.entry(), repo_root=Path("/tmp"))
        run.assert_called_once()
        self.assertEqual(result["variants"][0]["label"], "run-1")
        self.assertEqual(result["variants"][0]["status"], "pass")

    def test_suite_counts_one_rust_invocation(self) -> None:
        suite = {"name": "property", "index": "property_manifest.json"}
        with (patch("verification.runner.sifr_verify.hardening.property_and_fuzz.load_index",
                    return_value=[self.entry()]),
              patch("verification.runner.sifr_verify.hardening.property_and_fuzz.load_known_targets",
                    return_value={}),
              patch("subprocess.run", return_value=subprocess.CompletedProcess([], 0, "ok", "")) as run):
            result = run_property_suite(suite=suite, repo_root=Path("/tmp"))
        run.assert_called_once()
        self.assertEqual(result["total_variants"], 1)
        self.assertEqual(result["total_failures"], 0)
