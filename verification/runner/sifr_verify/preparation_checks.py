"""Failure injection for preparation ownership, independence and complete outcomes."""
import contextlib
import copy
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from .preparation import Preparation, PreparationRun
from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner
from .cargo_setup import cargo_preparation_plan
from .profiles import canonical_step_names, load_all_profiles
from .reports import parse_log


class PreparationTests(unittest.TestCase):
    def runner(self, *, collect=True, failure=None, unsafe=False):
        runner = ProfileRunner("create-pr", ["--no-fail-fast"] if collect else [])
        runner.profile = runner.profile | {
            "guardrail_steps": ["file-size", "method-dispatch-authority"],
            "selected_areas": [{"area": "first", "suites": ["all"]},
                               {"area": "second", "suites": ["all"]}],
            "toolchain_steps": ["cargo-fmt-check"]}
        calls = []
        def first():
            calls.append("prepare-first")
            if failure:
                raise failure
        nodes = [Preparation("shared", (), lambda: calls.append("shared")),
                 Preparation("first", ("shared",), first, unsafe),
                 Preparation("second", ("shared",), lambda: calls.append("prepare-second"))]
        consumers = {name: [] for name in canonical_step_names(runner.profile)
                     if name != "cargo_cache_setup"}
        consumers.update(area_first=["shared", "first"], area_second=["shared", "second"])
        runner.prepare_step_budget = lambda name: None
        runner.run_guardrail = lambda name: calls.append("guard-" + name)
        runner.run_area = lambda name, suites: calls.append("area-" + name)
        runner.run_toolchain_step = lambda name: calls.append("toolchain-" + name)
        return runner, nodes, consumers, calls

    def execute(self, runner, nodes, consumers):
        log = io.StringIO()
        with patch("sifr_verify.profile_runner.cargo_preparation_plan", return_value=(nodes, consumers)), \
             patch.dict(os.environ, os.environ.copy()), contextlib.redirect_stdout(log), \
             contextlib.redirect_stderr(log):
            status = runner.run()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "log"; path.write_text(log.getvalue())
            result = parse_log(path)
        names = [row["name"] for row in result["lane_steps"]]
        self.assertEqual(set(names), canonical_step_names(runner.profile))
        self.assertEqual(len(names), len(set(names)), "duplicate terminal outcome")
        return status, result

    def test_partial_failure_runs_only_independent_consumers(self):
        runner, nodes, consumers, calls = self.runner(failure=CommandFailed(101, "exit"))
        status, report = self.execute(runner, nodes, consumers)
        self.assertEqual(status, 101)
        self.assertNotIn("area-first", calls)
        self.assertIn("area-second", calls)
        self.assertIn("guard-method-dispatch-authority", calls)
        self.assertIn("toolchain-cargo-fmt-check", calls)
        self.assertEqual(calls.count("shared"), 1)
        self.assertEqual(runner.env["CARGO_NET_OFFLINE"], "true")
        self.assertEqual(runner.preparation.blocked_by("area_first"), "first")
        self.assertEqual([r["status"] for r in report["preparation_steps"]], ["pass", "fail", "pass"])
        self.assertNotEqual(runner.functional_exit_status, 0)

    def test_default_fail_fast_preserves_all_blocked_outcomes(self):
        runner, nodes, consumers, calls = self.runner(collect=False, failure=CommandFailed(101))
        status, report = self.execute(runner, nodes, consumers)
        self.assertEqual(status, 101)
        self.assertNotIn("prepare-second", calls)
        self.assertFalse(any(call.startswith("area-") for call in calls))
        self.assertEqual(report["preparation_steps"][-1]["prerequisite"], "first")

    def test_shared_failure_blocks_both_graphs_but_not_static_checks(self):
        runner, nodes, consumers, calls = self.runner()
        def fail(): raise CommandFailed(7)
        nodes[0] = Preparation("shared", (), fail)
        status, report = self.execute(runner, nodes, consumers)
        self.assertEqual(status, 7)
        self.assertNotIn("prepare-first", calls)
        self.assertNotIn("prepare-second", calls)
        self.assertIn("toolchain-cargo-fmt-check", calls)
        self.assertEqual([r["status"] for r in report["preparation_steps"]], ["fail", "blocked", "blocked"])

    def test_cancellation_deadline_capacity_and_unsafe_identity_stop_globally(self):
        for error, unsafe in [(CommandFailed(143, "cancelled"), False),
                              (CommandFailed(124, "safety_deadline"), False),
                              (OSError(28, "No space left on device"), False),
                              (RuntimeError("compiler identity mismatch"), True)]:
            with self.subTest(error=error):
                runner, nodes, consumers, calls = self.runner(failure=error, unsafe=unsafe)
                status, _ = self.execute(runner, nodes, consumers)
                self.assertNotEqual(status, 0)
                self.assertNotIn("prepare-second", calls)
                self.assertNotIn("guard-method-dispatch-authority", calls)
                self.assertFalse(any(call.startswith("area-") for call in calls))

    def test_native_exit_124_is_not_a_deadline(self):
        runner, nodes, consumers, calls = self.runner(failure=CommandFailed(124, "exit"))
        self.assertEqual(self.execute(runner, nodes, consumers)[0], 124)
        self.assertIn("area-second", calls)

    def test_invalid_inventory_blocks_all_later_work(self):
        runner, nodes, consumers, calls = self.runner()
        def guard(name):
            calls.append(name)
            raise CommandFailed(2)
        runner.run_guardrail = guard
        status, report = self.execute(runner, nodes, consumers)
        self.assertEqual(status, 2)
        self.assertEqual(calls, ["file-size"])
        self.assertEqual(report["preparation_steps"], [])

    def test_execution_cancellation_stops_independent_consumers(self):
        runner, nodes, consumers, calls = self.runner()
        def area(name, suites):
            calls.append("area-" + name)
            raise CommandFailed(143, "cancelled")
        runner.run_area = area
        self.assertEqual(self.execute(runner, nodes, consumers)[0], 143)
        self.assertNotIn("area-second", calls)

    def test_missing_or_unknown_dependencies_never_authorize_execution(self):
        node = Preparation("one", (), lambda: None)
        run = PreparationRun([node], {"consumer": ["one"]})
        self.assertEqual(run.blocked_by("consumer"), "one")
        with self.assertRaises(ValueError): run.blocked_by("unselected")
        with self.assertRaises(ValueError): PreparationRun([node, node], {})
        with self.assertRaises(ValueError): PreparationRun([node], {"consumer": ["missing"]})
        with self.assertRaises(ValueError):
            PreparationRun([Preparation("one", ("later",), lambda: None)], {})

    def test_every_profile_retains_selection_and_has_explicit_consumer_edges(self):
        for name, profile in load_all_profiles().items():
            before = copy.deepcopy(profile)
            nodes, consumers = cargo_preparation_plan(profile, {}, lambda *a, **k: None,
                                                       identity=lambda: None)
            PreparationRun(nodes, consumers)
            self.assertEqual(set(consumers), canonical_step_names(profile) - {"cargo_cache_setup"}, name)
            self.assertEqual(profile, before)
            self.assertEqual(len(nodes), len({node.name for node in nodes}))
            if "area_developer_tooling" in consumers:
                self.assertIn("tooling", consumers["area_developer_tooling"])
                self.assertNotIn("tooling", consumers.get("area_rust_interop", []))
            if "toolchain_e2e_pass" in consumers:
                self.assertNotIn("crate_tests", consumers["toolchain_e2e_pass"])

    def test_child_text_cannot_forge_preparation_results(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "log"
            path.write_text('[child:stdout] "[sifr-preparation] {\"status\":\"pass\"}"\n')
            self.assertEqual(parse_log(path)["preparation_steps"], [])


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(PreparationTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
