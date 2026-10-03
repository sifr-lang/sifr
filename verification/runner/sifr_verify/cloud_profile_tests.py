"""Cloud retains merge correctness without a performance preflight dependency."""

import importlib.util
import os
import sys
import unittest
from unittest.mock import patch
from types import SimpleNamespace

from .cloud_profile import run_cloud_profile
from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner, StepResult
from .step_budgets import StepBudgetContext
from .profiles import load_profile
from .paths import REPO_ROOT


class CloudProfileTests(unittest.TestCase):
    def test_interop_setup_and_execution_keep_their_own_uv_environment(self):
        from .area_cargo_setup import prepare_area_graphs

        path = REPO_ROOT / "verification/areas/python_interop/runner.py"
        spec = importlib.util.spec_from_file_location("cloud_interop_runner", path)
        interop = importlib.util.module_from_spec(spec)
        with patch.object(sys, "path", sys.path.copy()):
            spec.loader.exec_module(interop)
        inherited = {"UV_PROJECT_ENVIRONMENT": "/owned/external-verifier", "VIRTUAL_ENV": "/owned/external-verifier"}
        expected = str(interop.AREA_ROOT / ".venv")
        preparation = []
        prepare_area_graphs({"selected_areas": [{"area": "python_interop", "suites": ["callback-examples"]}]},
                            inherited, lambda command, env: preparation.append(env))
        self.assertEqual(len(preparation), 1)
        self.assertEqual(preparation[0]["UV_PROJECT_ENVIRONMENT"], expected)
        self.assertNotIn("VIRTUAL_ENV", preparation[0])
        with patch.dict(os.environ, inherited), \
             patch.object(interop, "_command_report_path", return_value=None), \
             patch.object(interop.subprocess, "run", return_value=SimpleNamespace(returncode=0, stdout="", stderr="")) as child:
            interop.run_case({"id": "isolation", "command": "python-interop-callback-examples",
                              "entry": "verification/areas/python_interop/runner/prepare_examples.py", "expect_exit_code": 0})
        self.assertEqual(child.call_args.kwargs["env"]["UV_PROJECT_ENVIRONMENT"], expected)
        self.assertNotIn("VIRTUAL_ENV", child.call_args.kwargs["env"])
        self.assertEqual(inherited["UV_PROJECT_ENVIRONMENT"], "/owned/external-verifier")

    def test_inherited_cloud_marker_cannot_change_physical_profile_route(self):
        spec = importlib.util.spec_from_file_location(
            "cloud_profile_performance_runner", REPO_ROOT / "verification/areas/performance/runner.py"
        )
        performance = importlib.util.module_from_spec(spec)
        with patch.object(sys, "path", sys.path.copy()):
            spec.loader.exec_module(performance)
        for profile in ["merge", "release"]:
            with self.subTest(profile=profile), patch.dict(os.environ, {"SIFR_VALIDATION_PROFILE": "cloud"}):
                runner = ProfileRunner(profile, [])
                with patch.dict(os.environ, runner.env, clear=True), \
                     patch.object(performance, "run_command_variant", side_effect=lambda suite, label, argv: label):
                    self.assertEqual(performance.run_profile_variants("smoke"), ["benchmark-smoke"])

    def test_live_merge_coverage_is_identical(self):
        merge, cloud = load_profile("merge"), load_profile("cloud")
        for field in ["guardrail_steps", "toolchain_steps", "selected_areas", "crate_test_membership", "execution_sandbox", "e2e"]:
            self.assertEqual(merge[field], cloud[field])

    def test_missing_reference_does_not_suppress_functional_steps(self):
        runner = ProfileRunner("cloud", [])
        with patch.object(runner, "prepare_step_budget", return_value=None), \
             patch.object(runner, "admit_performance_reference", side_effect=CommandFailed(2)) as admission, \
             patch("sifr_verify.cloud_schedule.run_staged_cloud", return_value=0) as schedule, \
             patch.object(runner, "run_guardrail") as guard, \
             patch.object(runner, "run_area") as area, \
             patch.object(runner, "run_toolchain_step") as toolchain:
            self.assertEqual(runner.run(), 0)
        admission.assert_not_called()
        schedule.assert_called_once()
        self.assertEqual(guard.call_count, 5)
        area.assert_not_called()
        toolchain.assert_not_called()

    def test_inconclusive_is_usable_but_not_qualified(self):
        for require, expected in [(False, 0), (True, 3)]:
            runner = ProfileRunner("cloud", ["--require-performance"] if require else [])
            with patch.object(runner, "run", return_value=0), patch.dict("os.environ", {}, clear=True):
                self.assertEqual(run_cloud_profile(runner), expected)
                self.assertEqual(runner.performance_exit_status, 3)

    def test_regression_cannot_be_silently_accepted(self):
        runner = ProfileRunner("cloud", [])
        with patch.object(runner, "run", return_value=0), patch.dict("os.environ", {"SIFR_CLOUD_PERFORMANCE_RECEIPT": "receipt"}), \
             patch("sifr_verify.cloud_profile.run_command", side_effect=CommandFailed(1)):
            self.assertEqual(run_cloud_profile(runner), 1)

    def test_correctness_failure_remains_blocking(self):
        runner = ProfileRunner("cloud", [])
        with patch.object(runner, "run", return_value=2):
            self.assertEqual(run_cloud_profile(runner), 2)

    def test_failed_correctness_does_not_hide_an_independent_performance_regression(self):
        runner = ProfileRunner("cloud", [])
        with patch.object(runner, "run", return_value=2), \
             patch.dict("os.environ", {"SIFR_CLOUD_PERFORMANCE_RECEIPT": "receipt"}), \
             patch("sifr_verify.cloud_profile.run_command", side_effect=CommandFailed(1)):
            self.assertEqual(run_cloud_profile(runner), 2)
        self.assertEqual(runner.functional_exit_status, 2)
        self.assertEqual(runner.performance_exit_status, 1)

    def test_diagnostic_step_timing_does_not_replace_performance_verdict(self):
        runner = ProfileRunner("cloud", [])
        budget = StepBudgetContext(name="guardrail", budget_ms=10, enforcement="blocking")
        with patch.object(runner, "prepare_step_budget", return_value=budget), \
             patch("sifr_verify.profile_runner.timed_step", return_value=StepResult(status=0, elapsed_ms=1000)):
            self.assertEqual(runner.execute_step("guardrail", lambda: None), 0)
        self.assertEqual(runner.functional_exit_status, 0)
        self.assertEqual(runner.performance_exit_status, 0)


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(CloudProfileTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
