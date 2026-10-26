"""Crate collection must preserve selection while retaining terminal failures."""
import contextlib
import io
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from .compiler_configuration_plan import configuration_plan
from .errors import VerificationError
from .corpus_ownership import corpus_owner_selected, corpus_assertion_owners, CORPUS_CASE, CORPUS_OWNER
from .native_test_execution import NATIVE_SUITES, run_native_configuration
from .native_test_execution_checks import outcome, PASSED, ENV, ROOM, deliver
from .process_execution import Outcome
from .native_test_roles import ADVANCED_DATA_PREPARATION
from .preparation import PreparationRun
from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner
from .profiles import load_profile


class CrateExecutionTests(unittest.TestCase):
    def runner(self, collect):
        runner = ProfileRunner.__new__(ProfileRunner)
        runner.profile = load_profile("merge")
        runner.env = {"CARGO_TARGET_DIR": "/owned/target", "RUST_TEST_THREADS": "1"}
        runner.no_fail_fast = collect
        runner.preparation = None
        return runner

    def groups(self):
        commands = [
            ["cargo", "test", "--locked", "--offline", "-p", "first", "-p", "second"],
            ["cargo", "test", "--locked", "--offline", "-p", "third", "--", "--ignored"],
            ["cargo", "test", "--locked", "--offline", "-p", "last", "selected_test"],
        ]
        return [SimpleNamespace(ids=(str(i),), execution=lambda c=c: list(c))
                for i, c in enumerate(commands)]

    def test_default_stops_at_first_failure_and_reports_it(self):
        runner = self.runner(False)
        failure = CommandFailed(101, "exit")
        stream = io.StringIO()
        with patch("sifr_verify.profile_runner.configuration_plan", return_value=self.groups()), \
             patch("sifr_verify.profile_runner.run_command", side_effect=failure) as run, \
             contextlib.redirect_stdout(stream), self.assertRaises(CommandFailed) as caught:
            runner.run_crate_tests("full")
        self.assertIs(caught.exception, failure)
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.args[0], self.groups()[0].execution())
        self.assertIn("case=0", stream.getvalue())
        self.assertIn("status=fail", stream.getvalue())
        self.assertNotIn("case=1", stream.getvalue())

    def test_collection_runs_all_configurations_and_keeps_first_failure(self):
        for failure in (CommandFailed(101), CommandFailed(1, "exit"), VerificationError("assertion")):
            runner = self.runner(True)
            stream = io.StringIO()
            with self.subTest(failure=failure), \
                 patch("sifr_verify.profile_runner.configuration_plan", return_value=self.groups()), \
                 patch("sifr_verify.profile_runner.run_command", side_effect=[failure, None, CommandFailed(2)]) as run, \
                 contextlib.redirect_stdout(stream), self.assertRaises(type(failure)) as caught:
                runner.run_crate_tests("full")
            self.assertIs(caught.exception, failure)
            self.assertEqual(run.call_count, 3)
            lines = stream.getvalue().splitlines()
            self.assertEqual(len(lines), 3)
            for i, status in enumerate(("fail", "pass", "fail")):
                self.assertIn(f"case={i}", lines[i])
                self.assertIn(f"status={status}", lines[i])

    def test_cargo_flags_precede_harness_and_preserve_environment(self):
        runner = self.runner(True)
        groups = self.groups()
        with patch("sifr_verify.profile_runner.configuration_plan", return_value=groups), \
             patch("sifr_verify.profile_runner.run_command") as run, \
             contextlib.redirect_stdout(io.StringIO()):
            runner.run_crate_tests("full")
        for group, call in zip(groups, run.call_args_list, strict=True):
            original = group.execution()
            self.assertEqual(call.args[0], [*original[:2], "--no-fail-fast", *original[2:]])
            self.assertIs(call.kwargs["env"], runner.env)

    def test_every_canonical_plan_preserves_selection_and_native_dispatch(self):
        for profile in ("create-pr", "merge", "nightly", "release"):
            for mode in ("smoke", "full"):
                for collect in (False, True):
                    runner = self.runner(collect)
                    runner.profile = load_profile(profile)
                    plan = configuration_plan(runner.profile, mode)
                    with self.subTest(profile=profile, mode=mode, collect=collect), \
                         patch("sifr_verify.profile_runner.run_command") as ordinary, \
                         patch("sifr_verify.profile_runner.run_native_configuration") as native, \
                         contextlib.redirect_stdout(io.StringIO()):
                        runner.run_crate_tests(mode)
                    selected = [g for g in plan if not any(s in NATIVE_SUITES for s in g.ids)]
                    self.assertEqual(len(selected), ordinary.call_count)
                    for group, call in zip(selected, ordinary.call_args_list, strict=True):
                        expected = group.execution()
                        if collect:
                            expected.insert(2, "--no-fail-fast")
                        self.assertEqual(call.args[0], expected)
                        self.assertEqual(call.kwargs, {"env": runner.env})
                    selected = [g for g in plan if any(s in NATIVE_SUITES for s in g.ids)]
                    self.assertEqual(len(selected), native.call_count)
                    for group, call in zip(selected, native.call_args_list, strict=True):
                        self.assertEqual(call.args, (group,))
                        self.assertEqual(call.kwargs, {"env": runner.env, "no_fail_fast": collect, "preparation_complete": False,
                                                      "corpus_owner_selected": corpus_owner_selected(runner.profile)})

    def test_only_the_exact_selected_corpus_suite_can_own_the_assertion(self):
        from .profiles import build_profile_plan, compare_plans
        import json
        import tempfile
        from pathlib import Path
        for areas in ([], [{"area": "sysroot_release", "suites": ["metadata-structural"]}],
                      [{"area": "other", "suites": ["metadata-corpus"]}]):
            self.assertFalse(corpus_owner_selected({"selected_areas": areas}))
            self.assertEqual(corpus_assertion_owners({"selected_areas": areas}), {})
        for name in ("create-pr", "merge", "nightly", "release"):
            profile = load_profile(name)
            plan = build_profile_plan(name)
            expected = {CORPUS_CASE: CORPUS_OWNER} if corpus_owner_selected(profile) else {}
            self.assertEqual(plan["assertion_owners"], expected)
        plan = build_profile_plan("merge", correctness_only=True)
        self.assertEqual(plan["assertion_owners"], {CORPUS_CASE: CORPUS_OWNER})
        with tempfile.TemporaryDirectory() as directory, \
             contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            left, right = Path(directory) / "local.json", Path(directory) / "ci.json"
            left.write_text(json.dumps(plan))
            right.write_text(json.dumps(plan | {"assertion_owners": {}}))
            self.assertEqual(compare_plans(str(left), str(right)), 1)

    def test_failed_or_blocked_corpus_owner_prevents_a_passing_profile(self):
        for blocked in (False, True):
            runner = ProfileRunner("merge", ["--no-fail-fast"], correctness_only=True)
            self.assertTrue(corpus_owner_selected(runner.profile))
            if blocked:
                runner.preparation = SimpleNamespace(
                    fatal_reason=None,
                    blocked_by=lambda step: "sysroot_package" if step == "area_sysroot_release" else "")
            def area(name, suites):
                if name == "sysroot_release":
                    self.assertIn("metadata-corpus", suites)
                    raise CommandFailed(9)
            stream = io.StringIO()
            with self.subTest(blocked=blocked), \
                 patch.object(runner, "prepare_step_budget", return_value=None), \
                 patch.object(runner, "prepare_cargo_cache"), \
                 patch.object(runner, "run_guardrail"), \
                 patch.object(runner, "run_toolchain_step"), \
                 patch.object(runner, "run_area", side_effect=area), \
                 patch("sifr_verify.profile_runner.enable_profile_offline_cargo"), \
                 contextlib.redirect_stdout(stream):
                self.assertEqual(runner.run(), 2 if blocked else 9)
            self.assertNotEqual(runner.functional_exit_status, 0)
            self.assertIn("name=area_sysroot_release", stream.getvalue())
            self.assertIn("status=blocked" if blocked else "status=fail", stream.getvalue())

    def test_terminal_failure_takes_precedence_and_stops_collection(self):
        for terminal in (CommandFailed(-15), CommandFailed(124, "deadline"),
                         CommandFailed(130, "cancelled"), CommandFailed(2, "spawn-error")):
            runner = self.runner(True)
            with self.subTest(terminal=terminal), \
                 patch("sifr_verify.profile_runner.configuration_plan", return_value=self.groups()), \
                 patch("sifr_verify.profile_runner.run_command", side_effect=[CommandFailed(101), terminal, None]) as run, \
                 contextlib.redirect_stdout(io.StringIO()), self.assertRaises(CommandFailed) as caught:
                runner.run_crate_tests("full")
            self.assertIs(caught.exception, terminal)
            self.assertEqual(run.call_count, 2)

    def test_native_failure_collection_preserves_terminal_precedence(self):
        configuration = next(g for g in configuration_plan(load_profile("merge"), "full")
                             if g.ids == ("sifr_driver_generated_builds",))
        inventory = f"alpha: test\nbeta: test\ngamma: test\n{ADVANCED_DATA_PREPARATION}: test\n\n4 tests, 0 benchmarks\n"
        for terminal in (Outcome(-15, "exit", b"", b"", False, .01),
                         Outcome(124, "deadline", b"", b"", False, .01),
                         Outcome(130, "cancelled", b"", b"", False, .01)):
            calls = []
            with self.subTest(terminal=terminal), patch(
                "sifr_verify.native_test_execution.run_command", return_value=outcome(inventory)
            ), patch("sifr_verify.native_test_allocation.discover_capacity", return_value=ROOM), \
                 patch("sifr_verify.native_test_execution.run_native_batch", side_effect=deliver(
                    [Outcome(101, "exit", b"", b"", False, .01), terminal, outcome(PASSED)], calls)), \
                 contextlib.redirect_stdout(io.StringIO()), self.assertRaises(CommandFailed) as caught:
                run_native_configuration(configuration, env=ENV, no_fail_fast=True, preparation_complete=True)
            self.assertEqual(caught.exception.returncode, terminal.returncode)
            self.assertEqual(caught.exception.cause, terminal.cause)
            self.assertEqual(len(calls), 2)

    def test_native_failure_continues_to_independent_configuration(self):
        runner = self.runner(True)
        native = next(g for g in configuration_plan(runner.profile, "full") if g.ids == ("sifr_driver_generated_builds",))
        failure = VerificationError("native assertion count")
        with patch("sifr_verify.profile_runner.configuration_plan", return_value=[native, self.groups()[0]]), \
             patch("sifr_verify.profile_runner.run_native_configuration", side_effect=failure), \
             patch("sifr_verify.profile_runner.run_command") as ordinary, \
             contextlib.redirect_stdout(io.StringIO()), self.assertRaises(VerificationError) as caught:
            runner.run_crate_tests("full")
        self.assertIs(caught.exception, failure)
        self.assertEqual(ordinary.call_count, 1)

    def test_unexpected_exception_is_not_aggregated(self):
        runner = self.runner(True)
        failure = RuntimeError("unexpected infrastructure error")
        with patch("sifr_verify.profile_runner.configuration_plan", return_value=self.groups()), \
             patch("sifr_verify.profile_runner.run_command", side_effect=[CommandFailed(101), failure, None]) as run, \
             contextlib.redirect_stdout(io.StringIO()), self.assertRaises(RuntimeError) as caught:
            runner.run_crate_tests("full")
        self.assertIs(caught.exception, failure)
        self.assertEqual(run.call_count, 2)

    def test_native_preparation_uses_current_run_success_only(self):
        for status, fatal, expected in (("pass", None, True), ("fail", None, False),
                                        ("blocked", None, False), (None, None, False),
                                        ("pass", "workspace", False)):
            runner = self.runner(True)
            runner.preparation = PreparationRun([], {})
            runner.preparation.results = {"crate_tests": {"status": status}}
            runner.preparation.fatal_reason = fatal
            native = next(g for g in configuration_plan(runner.profile, "full")
                          if g.ids == ("sifr_driver_generated_builds",))
            with self.subTest(status=status, fatal=fatal), patch(
                "sifr_verify.profile_runner.configuration_plan", return_value=[native]
            ), patch("sifr_verify.profile_runner.run_native_configuration") as run:
                runner.run_crate_tests("full")
            self.assertIs(run.call_args.kwargs["preparation_complete"], expected)


if __name__ == "__main__":
    unittest.main()
