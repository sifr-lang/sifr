"""Crate collection must preserve selection while retaining terminal failures."""
import contextlib
import io
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from .compiler_configuration_plan import configuration_plan
from .errors import VerificationError
from .native_test_execution import NATIVE_SUITES, run_native_configuration
from .native_test_execution_checks import outcome, PASSED
from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner
from .profiles import load_profile


class CrateExecutionTests(unittest.TestCase):
    def runner(self, collect):
        runner = ProfileRunner.__new__(ProfileRunner)
        runner.profile = load_profile("merge")
        runner.env = {"CARGO_TARGET_DIR": "/owned/target", "RUST_TEST_THREADS": "1"}
        runner.no_fail_fast = collect
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
                        self.assertEqual(call.kwargs, {"env": runner.env, "no_fail_fast": collect})

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
        inventory = "alpha: test\nbeta: test\ngamma: test\n\n3 tests, 0 benchmarks\n"
        for terminal in (CommandFailed(-15), CommandFailed(124, "deadline"), CommandFailed(130, "cancelled")):
            with self.subTest(terminal=terminal), patch(
                "sifr_verify.native_test_execution.run_command",
                side_effect=[outcome(inventory), CommandFailed(101), terminal, outcome(PASSED)],
            ) as run, contextlib.redirect_stdout(io.StringIO()), self.assertRaises(CommandFailed) as caught:
                run_native_configuration(configuration, env={}, no_fail_fast=True)
            self.assertIs(caught.exception, terminal)
            self.assertEqual(run.call_count, 3)

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


if __name__ == "__main__":
    unittest.main()
