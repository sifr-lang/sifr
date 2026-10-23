"""Fresh materialization coverage, command parity and failure accounting."""
from pathlib import Path
import json
import os
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from . import generated_materialization as batch
from .generated_cargo_setup import quality_module
from .process_execution import Outcome


class Quality:
    def materialization_request(self, entry, root):
        parent = Path(tempfile.mkdtemp(prefix="entry-", dir=root))
        crate = parent / "sifr_output"
        code = (
            "from pathlib import Path; import json,os,sys; "
            "p=Path(sys.argv[1]);p.mkdir(); "
            "(p/'Cargo.toml').write_text(Path(sys.argv[3]).read_text()); "
            "print(json.dumps([os.environ.get('CARGO_BUILD_JOBS'),os.environ.get('RAYON_NUM_THREADS')])); "
            "sys.exit(3 if sys.argv[2]=='fail' else 0)"
        )
        return crate, [sys.executable, "-c", code, str(crate), entry.id, entry.source_path]

    def require_materialized_graph(self, entry, root):
        if not (root / "Cargo.toml").is_file():
            raise RuntimeError(f"{entry.id}: missing Cargo.toml")


class GeneratedMaterializationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / "source"
        self.source.write_text("first input")
        self.entries = [SimpleNamespace(id=name, group="probe", source_path=str(self.source))
                        for name in ["alpha", "beta", "gamma"]]
        self.quality = Quality()

    def outcome(self, **changes):
        values = dict(returncode=0, cause="exit", stdout=b"ok", stderr=b"",
                      truncated=False, elapsed_seconds=.01)
        values.update(changes)
        return Outcome(**values)

    def report(self, root=None):
        paths = list((root or self.root).glob("batch-*/materialization.json"))
        self.assertEqual(len(paths), 1)
        return json.loads(paths[0].read_text())

    def mock_execute(self, commands, *, completed, **kwargs):
        for index in [2, 0, 1]:
            root = Path(commands[index][3]);root.mkdir()
            (root / "Cargo.toml").write_text("graph")
            completed(index, self.outcome())

    def test_real_commands_are_fresh_private_and_bounded_by_outer_budget(self):
        with patch.dict(os.environ, {"SIFR_VERIFY_PROCESS_SLOTS": "2",
                                    "CARGO_BUILD_JOBS": "7", "RAYON_NUM_THREADS": "8"}):
            original = dict(os.environ)
            roots, report = batch.materialize_entries(self.quality, self.entries, self.root / "first")
            self.assertEqual(dict(os.environ), original)
            for root in roots:
                self.assertEqual((root / "Cargo.toml").read_text(), "first input")
                self.assertEqual(json.loads((root.parent / "materialization.stdout").read_text()), ["1", "1"])
            self.source.write_text("changed input")
            second, _ = batch.materialize_entries(self.quality, self.entries, self.root / "second")
        self.assertTrue(set(roots).isdisjoint(second))
        self.assertTrue(all((root / "Cargo.toml").read_text() == "changed input" for root in second))
        data = json.loads(report.read_text())
        self.assertEqual([r["id"] for r in data["rows"]], ["alpha", "beta", "gamma"])
        self.assertEqual((data["status"], data["observed"], data["workers"]), ("pass", 3, 2))

    def test_real_nonzero_does_not_suppress_later_independent_commands(self):
        self.entries[1].id = "fail"
        with patch.dict(os.environ, {"SIFR_VERIFY_PROCESS_SLOTS": "2"}):
            with self.assertRaisesRegex(RuntimeError, "generated materialization failed"):
                batch.materialize_entries(self.quality, self.entries, self.root)
        report = self.report()
        self.assertEqual(report["observed"], 3)
        self.assertEqual([r["status"] for r in report["rows"]], ["pass", "fail", "pass"])
        self.assertEqual(report["rows"][1]["returncode"], 3)
        self.assertNotEqual(report["status"], "pass")

    def test_completion_order_does_not_change_selection_or_return_order(self):
        with patch.object(batch, "execute_batch", side_effect=self.mock_execute):
            roots, report = batch.materialize_entries(self.quality, self.entries, self.root)
        rows = json.loads(report.read_text())["rows"]
        self.assertEqual([r["id"] for r in rows], [e.id for e in self.entries])
        self.assertEqual([r["crate_root"] for r in rows], [str(r) for r in roots])
        self.assertTrue(all(r["status"] == "pass" for r in rows))

    def test_failed_truncated_cancelled_and_missing_graph_outcomes_reject(self):
        for changes in [{"returncode": 1}, {"truncated": True}, {"cause": "cancelled"},
                        {"cause": "safety_deadline"}, {}]:
            with self.subTest(changes=changes), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                def execute(commands, *, completed, **kwargs):
                    for index in range(3):
                        crate = Path(commands[index][3])
                        if index != 1:
                            crate.mkdir();(crate / "Cargo.toml").write_text("graph")
                        completed(index, self.outcome(**(changes if index == 1 else {})))
                with patch.object(batch, "execute_batch", side_effect=execute):
                    with self.assertRaisesRegex(RuntimeError, "generated materialization failed"):
                        batch.materialize_entries(self.quality, self.entries, root)
                report = self.report(root)
                self.assertEqual(report["observed"], 3)
                self.assertEqual([r["status"] for r in report["rows"]], ["pass", "fail", "pass"])

    def test_missing_duplicate_and_unknown_results_never_pass(self):
        for indices, error in [([0, 1], "missing"), ([0, 0], "duplicate"), ([3], "unknown"), ([True], "unknown")]:
            with self.subTest(indices=indices), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                def execute(commands, *, completed, **kwargs):
                    for command in commands:
                        crate = Path(command[3]);crate.mkdir();(crate / "Cargo.toml").write_text("graph")
                    for index in indices:completed(index, self.outcome())
                with patch.object(batch, "execute_batch", side_effect=execute):
                    with self.assertRaisesRegex((ValueError, RuntimeError), error):
                        batch.materialize_entries(self.quality, self.entries, root)
                self.assertNotEqual(self.report(root)["status"], "pass")

    def test_empty_duplicate_and_invalid_budget_reject_before_spawning(self):
        for selection, budget in [([], "2"), (self.entries + self.entries[:1], "2"), (self.entries, "0")]:
            with self.subTest(budget=budget), patch.dict(os.environ, {"SIFR_VERIFY_PROCESS_SLOTS": budget}):
                with patch.object(batch, "execute_batch") as execute:
                    with self.assertRaises((ValueError, RuntimeError)):
                        batch.materialize_entries(self.quality, selection, self.root)
                    execute.assert_not_called()
        self.assertFalse(list(self.root.glob("batch-*")))

    def test_partial_request_setup_preserves_incomplete_evidence(self):
        original = self.quality.materialization_request
        def request(entry, root):
            if entry.id == "beta":raise OSError("injected request setup failure")
            return original(entry, root)
        with patch.object(self.quality, "materialization_request", side_effect=request), \
             patch.object(batch, "execute_batch") as execute:
            with self.assertRaisesRegex(OSError, "injected"):
                batch.materialize_entries(self.quality, self.entries, self.root)
        execute.assert_not_called()
        report = self.report()
        self.assertEqual((report["status"], report["observed"], report["selected"]), ("incomplete", 0, 3))
        self.assertIn("injected", report["error"])

    def test_scalar_and_batch_share_exact_public_command_and_graph_admission(self):
        quality = quality_module()
        entry = quality.Entry("example", "probe", "a source.sifr", "build", "test")
        with patch.object(quality, "sifr_binary", return_value="/owned/sifr"):
            root, command = quality.materialization_request(entry, self.root)
            self.assertEqual(command, ["/owned/sifr", "build", "a source.sifr", "-o",
                                      str(root.parent), "--materialize-only"])
            with self.assertRaisesRegex(RuntimeError, "missing Cargo.toml"):
                quality.require_materialized_graph(entry, root)
            def run(args):
                self.assertEqual(args[0:3], command[0:3])
                crate = Path(args[4]) / "sifr_output";crate.mkdir()
                (crate / "Cargo.toml").write_text("graph")
            with patch.object(quality, "run_command", side_effect=run) as execute:
                other = quality.materialize_entry(entry, self.root)
            self.assertNotEqual(root, other)
            self.assertTrue((other / "Cargo.toml").is_file())
            execute.assert_called_once()


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(GeneratedMaterializationTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError("generated materialization failures: " + repr(result.failures + result.errors))
