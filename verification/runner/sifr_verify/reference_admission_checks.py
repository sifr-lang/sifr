"""The selected reference is admitted before profile setup mutates caches."""

from __future__ import annotations

import unittest
from unittest.mock import patch

from .profile_runner import ProfileRunner


class ReferenceAdmissionOrderingTests(unittest.TestCase):
    def test_failed_admission_stops_before_cargo_setup(self):
        runner = ProfileRunner("create-pr", [])
        with patch.object(runner, "execute_step", return_value=2) as execute:
            self.assertEqual(runner.run(), 2)
        self.assertEqual(execute.call_count, 1)
        self.assertEqual(execute.call_args.args[0], "performance_reference_admission")


def policy_checks() -> None:
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(ReferenceAdmissionOrderingTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
