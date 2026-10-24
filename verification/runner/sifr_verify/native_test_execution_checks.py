"""Strict native inventory, fail-fast, collection and execution-count controls."""
import contextlib
import io
import json
import unittest
from unittest.mock import patch
from .compiler_configuration_plan import configuration_plan
from .errors import VerificationError
from .native_test_execution import selected_native_cases, run_native_configuration
from .process_execution import Outcome
from .native_test_roles import ADVANCED_DATA_PREPARATION
from .profile_commands import CommandFailed
from .profiles import load_profile
from .native_test_allocation import ALLOCATION_ENV
from .resource_capacity import Capacity

ENV = {ALLOCATION_ENV: json.dumps(dict(workers=2, cargo_jobs=2, rayon_threads=2,
        memory_per_worker_mib=4096, reserved_memory_mib=2048, reserved_cpus=0))}
ROOM = Capacity(8, 16 * 1024**3)

def deliver(results, calls=None, order=None):
    def run(commands, **kwargs):
        for index in range(len(commands)) if order is None else order:
            if calls is not None:
                calls.append(commands[index])
            kwargs["completed"](index, results[index])
    return run

def outcome(text, *, truncated=False):
    return Outcome(0, "exit", text.encode(), b"", truncated, .01)

INVENTORY = f"alpha::first: test\nbeta::second: test\n{ADVANCED_DATA_PREPARATION}: test\n\n3 tests, 0 benchmarks\n"
PASSED = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 81 filtered out; finished in 0.1s"

class NativeExecutionTests(unittest.TestCase):
    def configuration(self):
        return next(c for c in configuration_plan(load_profile("merge"), "full")
                    if c.ids == ("sifr_driver_generated_builds",))

    def test_inventory_rejects_empty_duplicate_truncated_wrong_count_and_encoding(self):
        self.assertEqual(selected_native_cases(outcome(INVENTORY)), ("alpha::first", "beta::second", ADVANCED_DATA_PREPARATION))
        invalid = ["", "0 tests, 0 benchmarks", INVENTORY.replace("3 tests", "4 tests"),
                   INVENTORY.replace("beta::second", "alpha::first"),
                   INVENTORY.replace(": test", ": benchmark"), INVENTORY + "extra"]
        for text in invalid:
            with self.subTest(text=text), self.assertRaises(VerificationError):
                selected_native_cases(outcome(text))
        with self.assertRaises(VerificationError):
            selected_native_cases(outcome(INVENTORY, truncated=True))
        with self.assertRaises(VerificationError):
            selected_native_cases(Outcome(0, "exit", b"\xff", b"", False, .1))

    def setUp(self):
        self.capacity = patch("sifr_verify.native_test_allocation.discover_capacity", return_value=ROOM)
        self.capacity.start()
        self.addCleanup(self.capacity.stop)

    def test_flags_inventory_and_every_exact_case_preserved(self):
        configuration = self.configuration()
        calls = []
        env = dict(ENV, SIFR_VERIFY_SAFETY_DEADLINE_SECONDS="2400")
        stream = io.StringIO()
        with patch("sifr_verify.native_test_execution.run_command", return_value=outcome(INVENTORY)) as inventory, \
             patch("sifr_verify.native_test_execution.run_native_batch", side_effect=deliver(
                 [outcome(PASSED), outcome(PASSED)], calls, [1, 0])) as batch, \
             contextlib.redirect_stdout(stream):
            run_native_configuration(configuration, env=env, no_fail_fast=False, preparation_complete=True)
        self.assertEqual(inventory.call_args.args[0], [*configuration.execution(), "--list"])
        self.assertEqual(batch.call_args.args[0], [
            [*configuration.execution(), "--exact", "alpha::first"],
            [*configuration.execution(), "--exact", "beta::second"]])
        expected_env = dict(env, CARGO_BUILD_JOBS="2", RAYON_NUM_THREADS="2")
        self.assertEqual(inventory.call_args.kwargs, {"env": expected_env})
        self.assertEqual(batch.call_args.kwargs["env"], expected_env)
        self.assertEqual(batch.call_args.kwargs["workers"], 2)
        lines = [line for line in stream.getvalue().splitlines() if line.startswith("[sifr-native-case]")]
        self.assertIn("alpha::first status=pass", lines[0])
        self.assertIn("beta::second status=pass", lines[1])

    def test_first_failure_stops_and_explicit_collection_keeps_failure(self):
        for collect, expected in ((False, 1), (True, 2)):
            calls = []
            with self.subTest(collect=collect), patch(
                "sifr_verify.native_test_execution.run_command", return_value=outcome(INVENTORY)
            ), patch("sifr_verify.native_test_execution.run_native_batch", side_effect=deliver(
                [Outcome(101, "exit", b"", b"", False, .01), outcome(PASSED)], calls)):
                with self.assertRaises(CommandFailed) as caught:
                    run_native_configuration(self.configuration(), env=ENV, no_fail_fast=collect, preparation_complete=True)
                self.assertEqual(caught.exception.returncode, 101)
                self.assertEqual(len(calls), expected)

    def test_invalid_inventory_is_reported_as_failed_profile_step(self):
        from .profile_runner import timed_step
        with patch("sifr_verify.native_test_execution.run_command", return_value=outcome("")):
            result = timed_step("native_inventory", lambda: run_native_configuration(
                self.configuration(), env=ENV, no_fail_fast=False, preparation_complete=True))
        self.assertEqual(result.status, 2)

    def test_success_exit_without_one_executed_case_fails(self):
        for result in (outcome("test result: ok. 0 passed; 0 failed; 0 ignored;"),
                       outcome(PASSED, truncated=True), outcome(PASSED.replace("1 passed", "2 passed"))):
            calls = []
            with self.subTest(result=result), patch(
                "sifr_verify.native_test_execution.run_command", return_value=outcome(INVENTORY),
            ), patch("sifr_verify.native_test_execution.run_native_batch",
                     side_effect=deliver([result, outcome(PASSED)], calls)):
                with self.assertRaises(VerificationError):
                    run_native_configuration(self.configuration(), env=ENV, no_fail_fast=True, preparation_complete=True)
                self.assertEqual(len(calls), 1)

    def test_missing_and_duplicate_results_do_not_pass(self):
        for order in ([0], [0, 0], [2]):
            def incomplete(commands, **kwargs):
                for index in order:
                    kwargs["completed"](index, outcome(PASSED))
            stream = io.StringIO()
            with self.subTest(order=order), patch(
                "sifr_verify.native_test_execution.run_command", return_value=outcome(INVENTORY)
            ), patch("sifr_verify.native_test_execution.run_native_batch", side_effect=incomplete), \
                 contextlib.redirect_stdout(stream), self.assertRaises(VerificationError):
                run_native_configuration(self.configuration(), env=ENV, no_fail_fast=True, preparation_complete=True)
            self.assertIn("status=incomplete", stream.getvalue())

    def test_preparation_must_be_present_and_successful_and_is_not_an_assertion(self):
        for inventory, ready in ((INVENTORY, False),
                                 ("alpha: test\n\n1 test, 0 benchmarks", True),
                                 (f"{ADVANCED_DATA_PREPARATION}: test\n\n1 test, 0 benchmarks", True)):
            with self.subTest(inventory=inventory, ready=ready), patch(
                "sifr_verify.native_test_execution.run_command", return_value=outcome(inventory),
            ) as run, self.assertRaises(VerificationError):
                run_native_configuration(self.configuration(), env=ENV, no_fail_fast=True,
                                         preparation_complete=ready)
            self.assertEqual(run.call_count, 1)

if __name__ == "__main__":
    unittest.main()
