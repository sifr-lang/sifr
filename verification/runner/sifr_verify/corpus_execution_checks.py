"""Profile isolation, bounded scheduling and incomplete-execution controls."""
from dataclasses import replace
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from .cargo_sysroot_setup_checks import sysroot_module
from .corpus_ownership import CORPUS_CASE
from .errors import VerificationError
from .process_execution import Outcome


class CorpusExecutionTests(unittest.TestCase):
    def setUp(self):
        sysroot_module()
        import corpus_execution
        self.execution = corpus_execution
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.base = self.root / "evidence"
        self.base.mkdir()
        self.artifacts = {"development": {"driver_test": "/prepared/driver"}, "oracle": "/prepared/oracle"}
        self.good = Outcome(0, "exit",
            (f"test {CORPUS_CASE} ... ok\n"
             "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 871 filtered out;\n").encode(),
            b"", False, 1.0)

    def run_profiles(self, env=None):
        self.execution.run_profiles(self.artifacts, self.root, self.root / "snapshot",
                                    self.base, env or {}, "release-version")

    def report(self):
        return json.loads((self.base / "execution.json").read_text())

    def test_budget_environments_and_reverse_completion(self):
        for slots, expected in ((None, 1), ("1", 1), ("2", 2), ("8", 2)):
            env = {"SIFR_DX8_CORPUS_CASES": "partial", "SIFR_DX8_SEMANTIC_TARGET": "partial",
                   "SIFR_DX8_CORPUS_OUTPUT": "old", "RAYON_NUM_THREADS": "3"}
            if slots is not None:
                env["SIFR_VERIFY_PROCESS_SLOTS"] = slots
            original = dict(env)
            def run(commands, **kwargs):
                self.assertEqual(commands, [
                    ["/prepared/driver", "--exact", CORPUS_CASE, "--ignored", "--nocapture"],
                    ["/prepared/oracle", "--fixtures", str(self.root / "crates/sifr/tests/e2e/pass")]])
                self.assertEqual(kwargs["max_parallel"], expected)
                for key in ("SIFR_DX8_CORPUS_CASES", "SIFR_DX8_SEMANTIC_TARGET", "SIFR_DX8_CORPUS_OUTPUT"):
                    self.assertNotIn(key, kwargs["env"])
                if expected == 2:
                    self.assertEqual(kwargs["env"]["RAYON_NUM_THREADS"], "1")
                    self.assertEqual(kwargs["env"]["CARGO_BUILD_JOBS"], "1")
                overrides = kwargs["env_overrides"]
                self.assertEqual(overrides[0]["SIFR_SYSROOT"], str(self.root))
                self.assertEqual(overrides[1]["SIFR_SYSROOT"], str(self.root / "snapshot"))
                self.assertEqual(overrides[1]["SIFR_RELEASE_VERSION"], "release-version")
                self.assertEqual([o["SIFR_DX8_CORPUS_OUTPUT"] for o in overrides],
                                 [str(self.base / n) for n in ("development", "release")])
                kwargs["completed"](1, self.good)
                kwargs["completed"](0, self.good)
            with self.subTest(slots=slots), patch.object(self.execution, "execute_batch", side_effect=run):
                self.run_profiles(env)
            self.assertEqual(env, original)
            self.assertEqual(self.report()["workers"], expected)
            self.assertEqual([r["profile"] for r in self.report()["rows"]], ["development", "release"])

    def test_invalid_budget_cannot_spawn(self):
        with patch.object(self.execution, "execute_batch") as batch:
            for slots in ("0", "-1", "bad"):
                with self.subTest(slots=slots), self.assertRaises((ValueError, VerificationError)):
                    self.run_profiles({"SIFR_VERIFY_PROCESS_SLOTS": slots})
        batch.assert_not_called()

    def test_failure_preserves_output_and_independent_completion(self):
        for change in ({"returncode": 101, "stdout": b"assertion details"},
                       {"cause": "safety_deadline"}, {"truncated": True},
                       {"stdout": b"test result: ok. 0 passed; 0 failed; 0 ignored;"}):
            failed = replace(self.good, **change)
            def run(commands, **kwargs):
                kwargs["completed"](0, failed)
                kwargs["completed"](1, self.good)
            with self.subTest(change=change), patch.object(self.execution, "execute_batch", side_effect=run):
                with self.assertRaises(VerificationError):
                    self.run_profiles()
            self.assertEqual((self.base / "development.stdout").read_bytes(), failed.stdout)
            self.assertEqual([r["status"] for r in self.report()["rows"]], ["fail", "pass"])

    def test_missing_duplicate_unknown_and_spawn_failure_never_pass(self):
        for indices in ([], [0], [0, 0], [2], [True]):
            def run(commands, **kwargs):
                for index in indices:
                    kwargs["completed"](index, self.good)
            with self.subTest(indices=indices), patch.object(self.execution, "execute_batch", side_effect=run):
                with self.assertRaises(VerificationError):
                    self.run_profiles()
            self.assertTrue((self.base / "execution.json").is_file())
        with patch.object(self.execution, "execute_batch", side_effect=OSError("cannot spawn")):
            with self.assertRaises(OSError):
                self.run_profiles()
        self.assertEqual(self.report()["rows"], [])
