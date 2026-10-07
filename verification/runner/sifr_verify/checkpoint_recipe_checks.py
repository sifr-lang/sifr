"""Actual isolated guard execution plus negative audited-recipe controls.

Capacity is a fixture here; this does not qualify the constrained workload.
"""
from __future__ import annotations

import copy
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from . import checkpoint_recipes as recipes
from .checkpoint_runtime import stdlib_identity
from .execution_identity import EvidenceError
from .process_execution import execute


class RecipeTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        for relative in ("scripts/check_hir_maintainability_guardrails.py",
                         "internal_docs/hir_maintainability_guardrails.md",
                         "verification/policy/correctness_checkpoints.json"):
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(recipes.REPO_ROOT / relative, destination)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)
        subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                        "commit", "-qm", "fixture"], cwd=self.root, check=True)
        self.calls = 0
        self.env = os.environ.copy()
        self.patches = [
            patch("sifr_verify.execution_identity.runtime_identity", return_value={"fixture": "runtime-v1"}),
            patch.object(recipes.shutil, "disk_usage", return_value=shutil._ntuple_diskusage(64*1024**3, 0, 64*1024**3)),
        ]
        for item in self.patches:
            item.start()
            self.addCleanup(item.stop)

    def command(self, arguments, *, env):
        self.calls += 1
        result = execute(arguments, cwd=self.root, env=env)
        if result.returncode:
            from .profile_commands import CommandFailed
            error = CommandFailed(result.returncode, result.cause)
            error.outcome = result
            raise error
        return result

    def run_guard(self):
        with unittest.mock.patch("sys.stdout", new=io.StringIO()):
            return recipes.run_guard("hir-maintainability", root=self.root,
                                     env=self.env, command_runner=self.command)

    def test_actual_matching_guard_reuses_one_executed_validation(self):
        first = self.run_guard()
        second = self.run_guard()
        self.assertEqual((first.mode, second.mode, self.calls), ("executed", "reused", 1))
        row = second.evidence["records"][0]
        self.assertEqual((row["execution_kind"], row["executed_count"]), ("validation", 1))
        self.assertEqual(row["phases"], ["selected", "executed", "passed"])

    def test_consumed_document_drift_cannot_reuse_or_pass(self):
        from .profile_commands import CommandFailed
        first = self.run_guard()
        document = self.root / recipes.REQUIRED_PATHS["hir-maintainability"][0]
        original = document.read_bytes()
        document.write_text("unrelated document")
        with self.assertRaises(CommandFailed):
            self.run_guard()
        failures = list((self.root / "target").rglob("failure.json"))
        self.assertEqual(len(failures), 1)
        self.assertTrue(first.path.exists())
        document.write_bytes(original)
        restored = self.run_guard()
        self.assertEqual(restored.mode, "reused")
        self.assertEqual(self.calls, 2)
        self.assertTrue(failures[0].exists(), "restoration erased failed evidence")

    def test_untracked_forbidden_path_invalidates_passing_evidence(self):
        from .profile_commands import CommandFailed
        self.run_guard()
        banned = self.root / "crates/sifr_lowering/src/lower.rs"
        banned.parent.mkdir(parents=True)
        banned.write_text("// forbidden monolith\n")
        with self.assertRaises(CommandFailed):
            self.run_guard()
        self.assertEqual(self.calls, 2)

    def test_changed_unaudited_script_executes_fresh(self):
        self.run_guard()
        script = self.root / recipes.SUPPORTED["hir-maintainability"]
        script.write_text(script.read_text() + "\n# changed recipe needs audit\n")
        self.assertIsNone(self.run_guard())
        self.assertEqual(self.calls, 2)

    def test_capacity_cannot_suppress_actual_guard(self):
        with patch.object(recipes.shutil, "disk_usage", return_value=shutil._ntuple_diskusage(1, 1, 0)):
            self.assertIsNone(self.run_guard())
        self.assertEqual(self.calls, 1)
        self.assertFalse((self.root / "target").exists())

    def test_policy_cannot_omit_a_consumed_document(self):
        policy = recipes.load_policy()
        policy["recipes"][0]["consumed_paths"] = policy["recipes"][0]["consumed_paths"][1:]
        path = self.root / "bad-policy.json"
        path.write_text(json.dumps(policy))
        with self.assertRaises(EvidenceError):
            recipes.load_policy(path)

    def test_probe_records_actual_stdlib_and_loaded_native_bytes(self):
        result = stdlib_identity(recipes.SUPPORTED["hir-maintainability"], self.env, cwd=self.root)
        self.assertTrue(result["stdlib_files"])
        self.assertTrue(result["native_dependencies"])
        self.assertTrue(all(len(row["sha256"]) == 64 for row in result["native_dependencies"]))
        self.assertFalse(any("__pycache__" in row["path"] for row in result["stdlib_files"]))


def policy_checks():
    result = unittest.TextTestRunner(stream=io.StringIO()).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(RecipeTests))
    if not result.wasSuccessful():
        raise AssertionError(str(result.errors + result.failures))


if __name__ == "__main__":
    unittest.main()
