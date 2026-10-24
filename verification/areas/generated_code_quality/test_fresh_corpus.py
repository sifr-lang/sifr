"""Behavioral regressions for fresh-product ownership and batch qualification."""
from __future__ import annotations

import copy
from dataclasses import dataclass
import importlib.util
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from batch_adapter import reconcile, run_shared_gates
from fresh_corpus import FreshCorpus
from quality_batch import execute, validate_requests


@dataclass(frozen=True)
class Entry:
    id: str
    source_path: str
    group: str = "demos-required"


def request(mode, ids=None):
    return {"mode": mode, "max_entries": None, "entry_ids": ids}


class FakeQuality:
    MANIFEST = Path("manifest.json")

    def __init__(self):
        self.entries = [Entry("a", "a.sifr"), Entry("b", "b.sifr")]
        self.materialized = []
        self.asserted = []
        self.contents = "fresh program\n"
        self.fail = set()

    def load_manifest(self, _path):
        return self.entries

    def selected_positive_entries(self, entries, groups):
        ids = os.environ.get("SIFR_GCQ_ENTRY_IDS")
        chosen = [entry for entry in entries if not groups or entry.group in groups]
        if ids:
            chosen = [entry for entry in chosen if entry.id in ids.split(",")]
        limit = os.environ.get("SIFR_GCQ_MAX_ENTRIES")
        return chosen[:int(limit)] if limit else chosen

    def materialize_entry(self, entry, root):
        self.materialized.append(entry.id)
        if entry.id in self.fail:
            raise RuntimeError("compiler rejected input")
        crate = root / f"program-{len(self.materialized)}"
        (crate / "src").mkdir(parents=True)
        (crate / "Cargo.toml").write_text("[package]\nname='fixture'\n")
        (crate / "src/main.rs").write_text(self.contents)
        return crate

    def check(self, mode, entries, args):
        for entry in self.selected_positive_entries(entries, args.group):
            crate = args.materialize(entry, Path("unused-mode-output"))
            if (crate / "src/main.rs").read_text() != self.contents:
                raise RuntimeError("consumer saw the wrong generated program")
            self.asserted.append((mode, entry.id))

    def gate_corpus(self, entries, args):
        self.check("corpus", entries, args)

    def gate_panic_scan(self, entries, args):
        self.check("panic-scan", entries, args)

    def gate_rustfmt(self, entries, args):
        self.check("rustfmt", entries, args)


class FreshCorpusTests(unittest.TestCase):
    def test_shared_output_still_runs_every_consumer_and_next_owner_is_fresh(self):
        api = FakeQuality()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            plan = [request("corpus"), request("panic-scan"), request("rustfmt")]
            with patch.dict(os.environ, {"SIFR_GCQ_MAX_ENTRIES": "1", "SIFR_GCQ_ENTRY_IDS": "a"}):
                self.assertEqual(execute(api, plan, root / "first", root / "first.json"), 0)
                self.assertEqual(os.environ["SIFR_GCQ_MAX_ENTRIES"], "1")
                self.assertEqual(os.environ["SIFR_GCQ_ENTRY_IDS"], "a")
            self.assertEqual(api.materialized, ["a", "b"])
            self.assertEqual(api.asserted, [(mode, name) for mode in ("corpus", "panic-scan", "rustfmt")
                                           for name in ("a", "b")])
            self.assertFalse((root / "first").exists())
            before = json.loads((root / "first.json").read_text())
            api.contents = "edited source produced a different program\n"
            self.assertEqual(execute(api, [request("corpus")], root / "second", root / "second.json"), 0)
            after = json.loads((root / "second.json").read_text())
            self.assertEqual(api.materialized, ["a", "b", "a", "b"])
            self.assertNotEqual(before["materialization"]["programs"][0]["generated_files"],
                                after["materialization"]["programs"][0]["generated_files"])

    def test_failed_generation_blocks_consumers_but_independent_entry_executes(self):
        api = FakeQuality()
        api.fail.add("a")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            plan = [request("corpus", ["a"]), request("panic-scan", ["a"]), request("rustfmt", ["b"])]
            self.assertEqual(execute(api, plan, root / "programs", root / "report.json"), 1)
            report = json.loads((root / "report.json").read_text())
            self.assertTrue(report["complete"])
            self.assertEqual([r["status"] for r in report["results"]], ["fail", "fail", "pass"])
            self.assertEqual(api.materialized, ["a", "b"])
            self.assertEqual(api.asserted, [("rustfmt", "b")])
            self.assertTrue((root / "programs").is_dir())
            selections = {r["mode"]: r["entry_ids"] for r in plan}
            self.assertEqual(len(reconcile(report, plan, selections, 1, root / "programs")), 3)

    def test_mutation_preserves_failure_evidence_and_prevents_later_success(self):
        api = FakeQuality()
        def corrupt(entries, args):
            path = args.materialize(entries[0], Path("unused")) / "src/main.rs"
            original = path.stat()
            path.write_text("corrupted program\n")
            os.utime(path, ns=(original.st_atime_ns, original.st_mtime_ns))
        api.gate_corpus = corrupt
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            plan = [request("corpus", ["a"]), request("panic-scan", ["a"])]
            self.assertEqual(execute(api, plan, root / "programs", root / "report.json"), 1)
            report = json.loads((root / "report.json").read_text())
            self.assertTrue(report["materialization"]["poisoned"])
            self.assertEqual([r["status"] for r in report["results"]], ["fail", "fail"])
            self.assertEqual(api.asserted, [])
            self.assertEqual((root / "programs/program-1/src/main.rs").read_text(), "corrupted program\n")

    def test_entry_identity_and_active_consumer_are_required(self):
        api = FakeQuality()
        with tempfile.TemporaryDirectory() as directory:
            owner = FreshCorpus(api.materialize_entry, Path(directory) / "programs")
            with self.assertRaisesRegex(RuntimeError, "active consumer"):
                owner.get(api.entries[0], Path("unused"))
            owner.consumer = "corpus"
            owner.get(api.entries[0], Path("unused"))
            with self.assertRaisesRegex(RuntimeError, "conflicting"):
                owner.get(Entry("a", "different.sifr"), Path("unused"))

    def test_preexisting_owner_and_external_generated_root_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaises(FileExistsError):
                FreshCorpus(FakeQuality().materialize_entry, root)
            outside = root / "outside"
            outside.mkdir()
            owner = FreshCorpus(lambda _entry, _root: outside, root / "programs")
            owner.consumer = "corpus"
            with self.assertRaisesRegex(RuntimeError, "escaped"):
                owner.get(Entry("a", "a.sifr"), root)

    def test_keep_success_and_failed_cleanup_never_invent_complete_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.dict(os.environ, {"SIFR_GCQ_KEEP_SUCCESS": "1"}):
                self.assertEqual(execute(FakeQuality(), [request("corpus")], root / "kept", root / "kept.json"), 0)
            self.assertTrue((root / "kept").is_dir())
            with patch("fresh_corpus.shutil.rmtree", side_effect=OSError("cleanup failed")):
                with self.assertRaisesRegex(OSError, "cleanup failed"):
                    execute(FakeQuality(), [request("corpus")], root / "failed", root / "failed.json")
            self.assertFalse(json.loads((root / "failed.json").read_text())["complete"])


class BatchProtocolTests(unittest.TestCase):
    def test_rejects_missing_duplicate_mismatched_or_contradictory_evidence(self):
        plan = [request("corpus"), request("panic-scan")]
        selections = {r["mode"]: ["a", "b"] for r in plan}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.assertEqual(execute(FakeQuality(), plan, root / "programs", root / "report.json"), 0)
            original = json.loads((root / "report.json").read_text())
        self.assertEqual(len(reconcile(original, plan, selections, 0, root / "programs")), 2)
        mutations = [
            lambda p: p.update(complete=False),
            lambda p: p.update(workspace="prior-batch"),
            lambda p: p["results"].pop(),
            lambda p: p["results"].append(p["results"][0]),
            lambda p: p["results"].__setitem__(1, p["results"][0]),
            lambda p: p["results"][0].update(selected_entries=["a"]),
            lambda p: p["results"][0].update(status="pass", errors=["failed assertion"]),
            lambda p: p["results"][0].update(duration_ms=float("nan")),
            lambda p: p["materialization"]["programs"].pop(),
            lambda p: p["materialization"]["programs"][0].update(consumers=[]),
            lambda p: p["materialization"]["programs"][0].update(crate_root="/other-owner"),
            lambda p: p["materialization"]["programs"][0].update(generated_files={"Cargo.toml": "not-a-digest"}),
            lambda p: p["materialization"].update(poisoned="changed source"),
        ]
        for mutate in mutations:
            altered = copy.deepcopy(original)
            mutate(altered)
            with self.assertRaises(ValueError):
                reconcile(altered, plan, selections, 0, root / "programs")
        with self.assertRaises(ValueError):
            reconcile(original, plan, selections, 1, root / "programs")

    def test_native_determinism_and_duplicate_modes_cannot_enter_shared_batch(self):
        for plan in ([request("native")], [request("determinism")],
                     [request("companions")], [request("corpus"), request("corpus")],
                     [dict(request("corpus"), max_entries="0")],
                     [request("corpus", ["a", "a"])], []):
            with self.assertRaises(ValueError):
                validate_requests(plan)


class AreaAdapterTests(unittest.TestCase):
    def test_actual_request_transport_qualifies_each_gate_without_inventing_processes(self):
        import generated_code_quality as api
        fake = FakeQuality()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            calls = []
            def child(argv, **_kwargs):
                calls.append(argv)
                requests = json.loads(Path(argv[argv.index("--requests") + 1]).read_text())
                code = execute(fake, requests, Path(argv[argv.index("--workspace") + 1]),
                               Path(argv[argv.index("--report") + 1]))
                return SimpleNamespace(returncode=code)
            with patch.object(api, "load_manifest", return_value=fake.entries), \
                    patch("batch_adapter.subprocess.run", side_effect=child):
                results = run_shared_gates("smoke", [("corpus", None, ("a",)),
                    ("panic-scan", None, ("a",))], repo_root=root, area_root=Path(__file__).parent)
            self.assertEqual(len(calls), 1)
            self.assertEqual(fake.materialized, ["a"])
            self.assertEqual(fake.asserted, [("corpus", "a"), ("panic-scan", "a")])
            for row in results.values():
                self.assertEqual(row["status"], "pass")
                self.assertEqual(row["execution_kind"], "in-process-gate")
                self.assertEqual(row["batch_process"]["returncode"], 0)
                self.assertNotIn("actual_exit_code", row)
                self.assertNotIn("argv", row)

    def test_zero_exit_without_report_is_not_a_pass_or_zero_duration(self):
        import generated_code_quality as api
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(api, "load_manifest", return_value=FakeQuality().entries), \
                    patch("batch_adapter.subprocess.run", return_value=SimpleNamespace(returncode=0)):
                results = run_shared_gates("smoke", [("corpus", None, ("a",))],
                                          repo_root=Path(directory), area_root=Path(__file__).parent)
            row = results["corpus"]
            self.assertEqual(row["status"], "fail")
            self.assertIsNone(row["duration_ms"])
            self.assertTrue(row["mismatches"])

    def test_area_retains_variant_selection_and_independent_gates_after_batch_failure(self):
        spec = importlib.util.spec_from_file_location("gcq_batch_test_runner", Path(__file__).with_name("runner.py"))
        runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runner)
        suite = next(s for s in json.loads(runner.MANIFEST_PATH.read_text())["suites"]
                     if s["name"] == "representative")
        independent = []
        batches = []
        def shared(_suite, plan, **_kwargs):
            batches.append(plan)
            return {mode: {"label": mode, "max_entries": limit, "entry_ids": ids,
                           "duration_ms": None, "status": "fail"}
                    for mode, limit, ids in plan}
        def individual(_suite, mode, limit, ids):
            independent.append((mode, limit, ids))
            return {"label": mode, "status": "pass"}
        with patch.object(runner, "run_shared_gates", side_effect=shared), \
                patch.object(runner, "run_gate", side_effect=individual):
            result = runner.run_suite(suite)
        plan = runner.PROFILE_SUITES["representative"]
        self.assertEqual(batches, [[r for r in plan if r[0] in runner.SHARED_GATES]])
        self.assertEqual(independent, [r for r in plan if r[0] not in runner.SHARED_GATES])
        self.assertEqual([r["label"] for r in result["cases"][0]["variants"]], [r[0] for r in plan])
        self.assertEqual(result["total_variants"], 10)
        self.assertEqual(result["total_failures"], 5)


if __name__ == "__main__":
    unittest.main()
