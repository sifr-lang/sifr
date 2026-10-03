"""Required performance remains blocking without suppressing correctness."""
from __future__ import annotations

import contextlib
import io
import unittest
from unittest.mock import patch

from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner
from .step_budgets import StepBudgetContext


class ReferenceAdmissionOrderingTests(unittest.TestCase):
    def exercise(self, profile, *, admission_error=None, area_error=None, budget=False):
        runner = ProfileRunner(profile, [])
        events = []
        def admission():
            events.append(('admission', 'performance'))
            if admission_error:
                raise CommandFailed(admission_error)
        def area(name, suites):
            events.append(('area', name))
            if name == 'performance' and area_error:
                raise CommandFailed(area_error)
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()), \
             patch.object(runner, 'prepare_step_budget', side_effect=lambda name:
                StepBudgetContext(name, 1, 'blocking') if budget else None), \
             patch('sifr_verify.profile_runner.enforce_prepared_step_budget',
                   side_effect=lambda context, elapsed: 124 if budget else 0), \
             patch.object(runner, 'admit_performance_reference', side_effect=admission), \
             patch.object(runner, 'prepare_cargo_cache', side_effect=lambda: events.append(('setup', 'cargo'))), \
             patch.object(runner, 'run_guardrail', side_effect=lambda name: events.append(('guard', name))), \
             patch.object(runner, 'run_area', side_effect=area), \
             patch.object(runner, 'run_toolchain_step', side_effect=lambda name: events.append(('toolchain', name))):
            status = runner.run()
        return runner, status, events

    def test_failed_admission_runs_every_functional_selection_before_blocking(self):
        for profile in ('create-pr', 'merge', 'nightly', 'release'):
            with self.subTest(profile=profile):
                runner, status, events = self.exercise(profile, admission_error=2)
                self.assertEqual(status, 2)
                self.assertEqual(runner.functional_exit_status, 0)
                self.assertEqual(runner.performance_exit_status, 2)
                self.assertEqual(events[-1], ('admission', 'performance'))
                self.assertEqual([name for kind, name in events if kind == 'area'],
                    [item['area'] for item in runner.profile['selected_areas'] if item['area'] != 'performance'])
                self.assertCountEqual([name for kind, name in events if kind == 'guard'], runner.profile['guardrail_steps'])
                self.assertEqual([name for kind, name in events if kind == 'toolchain'], runner.profile['toolchain_steps'])

    def test_performance_execution_failure_keeps_functional_outcome(self):
        runner, status, events = self.exercise('create-pr', area_error=1)
        self.assertEqual(status, 1)
        self.assertEqual((runner.functional_exit_status, runner.performance_exit_status), (0, 1))
        self.assertEqual(events[-2:], [('admission', 'performance'), ('area', 'performance')])

    def test_blocking_step_timing_does_not_stop_later_correctness(self):
        runner, status, events = self.exercise('create-pr', budget=True)
        self.assertEqual(status, 124)
        self.assertEqual((runner.functional_exit_status, runner.performance_exit_status), (0, 124))
        self.assertEqual([name for kind, name in events if kind == 'toolchain'], runner.profile['toolchain_steps'])
        self.assertEqual([name for kind, name in events if kind == 'area'],
            [item['area'] for item in runner.profile['selected_areas'] if item['area'] != 'performance'] + ['performance'])

    def test_functional_failure_remains_fail_fast(self):
        runner = ProfileRunner('create-pr', [])
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()), \
             patch.object(runner, 'prepare_step_budget', return_value=None), \
             patch.object(runner, 'run_guardrail', side_effect=CommandFailed(9)), \
             patch.object(runner, 'prepare_cargo_cache') as setup, \
             patch.object(runner, 'admit_performance_reference') as admission:
            self.assertEqual(runner.run(), 9)
        self.assertEqual(runner.functional_exit_status, 9)
        setup.assert_not_called()
        admission.assert_not_called()


def policy_checks() -> None:
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(ReferenceAdmissionOrderingTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == '__main__':
    unittest.main()
