"""The selected reference is admitted before profile setup mutates caches."""

from __future__ import annotations

import unittest
from unittest.mock import patch

from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner


class ReferenceAdmissionOrderingTests(unittest.TestCase):
    def test_failed_admission_stops_before_cargo_setup(self):
        runner = ProfileRunner("create-pr", [])
        with patch.object(runner, "prepare_step_budget", return_value=None), \
             patch.object(runner, "admit_performance_reference", side_effect=CommandFailed(2)) as admission, \
             patch.object(runner, "prepare_cargo_cache") as setup, \
             patch.object(runner, "run_guardrail") as guard:
            self.assertEqual(runner.run(), 2)
        admission.assert_called_once_with()
        setup.assert_not_called()
        guard.assert_not_called()


def policy_checks() -> None:
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(ReferenceAdmissionOrderingTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
