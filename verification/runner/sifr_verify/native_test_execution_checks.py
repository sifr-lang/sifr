"""Strict native inventory, fail-fast, collection and execution-count controls."""
import unittest
from unittest.mock import patch
from .compiler_configuration_plan import configuration_plan
from .errors import VerificationError
from .native_test_execution import selected_native_cases, run_native_configuration
from .process_execution import Outcome
from .native_test_roles import ADVANCED_DATA_PREPARATION
from .profile_commands import CommandFailed
from .profiles import load_profile

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

    def test_flags_inventory_and_every_exact_case_preserved(self):
        configuration = self.configuration()
        env = {"SIFR_VERIFY_SAFETY_DEADLINE_SECONDS": "2400"}
        with patch("sifr_verify.native_test_execution.run_command",
                   side_effect=[outcome(INVENTORY), outcome(PASSED), outcome(PASSED)]) as run:
            run_native_configuration(configuration, env=env, no_fail_fast=False, preparation_complete=True)
        self.assertEqual([c.args[0] for c in run.call_args_list], [
            [*configuration.execution(), "--list"],
            [*configuration.execution(), "--exact", "alpha::first"],
            [*configuration.execution(), "--exact", "beta::second"]])
        self.assertTrue(all(c.kwargs == {"env": env} for c in run.call_args_list))

    def test_first_failure_stops_and_explicit_collection_keeps_failure(self):
        for collect, expected in ((False, 2), (True, 3)):
            with self.subTest(collect=collect), patch(
                "sifr_verify.native_test_execution.run_command",
                side_effect=[outcome(INVENTORY), CommandFailed(101), outcome(PASSED)],
            ) as run:
                with self.assertRaises(CommandFailed) as caught:
                    run_native_configuration(self.configuration(), env={}, no_fail_fast=collect, preparation_complete=True)
                self.assertEqual(caught.exception.returncode, 101)
                self.assertEqual(run.call_count, expected)

    def test_invalid_inventory_is_reported_as_failed_profile_step(self):
        from .profile_runner import timed_step
        with patch("sifr_verify.native_test_execution.run_command", return_value=outcome("")):
            result = timed_step("native_inventory", lambda: run_native_configuration(
                self.configuration(), env={}, no_fail_fast=False, preparation_complete=True))
        self.assertEqual(result.status, 2)

    def test_success_exit_without_one_executed_case_fails(self):
        for result in (outcome("test result: ok. 0 passed; 0 failed; 0 ignored;"),
                       outcome(PASSED, truncated=True), outcome(PASSED.replace("1 passed", "2 passed"))):
            with self.subTest(result=result), patch(
                "sifr_verify.native_test_execution.run_command",
                side_effect=[outcome(INVENTORY), result],
            ) as run:
                with self.assertRaises(VerificationError):
                    run_native_configuration(self.configuration(), env={}, no_fail_fast=False, preparation_complete=True)
                self.assertEqual(run.call_count, 2)

    def test_preparation_must_be_present_and_successful_and_is_not_an_assertion(self):
        for inventory, ready in ((INVENTORY, False),
                                 ("alpha: test\n\n1 test, 0 benchmarks", True),
                                 (f"{ADVANCED_DATA_PREPARATION}: test\n\n1 test, 0 benchmarks", True)):
            with self.subTest(inventory=inventory, ready=ready), patch(
                "sifr_verify.native_test_execution.run_command", return_value=outcome(inventory),
            ) as run, self.assertRaises(VerificationError):
                run_native_configuration(self.configuration(), env={}, no_fail_fast=True,
                                         preparation_complete=ready)
            self.assertEqual(run.call_count, 1)

if __name__ == "__main__":
    unittest.main()
