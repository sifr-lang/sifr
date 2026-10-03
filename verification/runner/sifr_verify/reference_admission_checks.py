"""Required performance remains blocking without suppressing correctness."""
from __future__ import annotations

import contextlib
import io
import importlib.util
import json
import os
import sys
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner, timed_step
from .paths import REPO_ROOT
from .step_budgets import StepBudgetContext
from .performance_partition import MEASUREMENT_SUITES, combine, result_path
from .profile_results import AreaResultError


class ReferenceAdmissionOrderingTests(unittest.TestCase):
    def exercise(self, profile, *, admission_error=None, area_error=None, functional_error=None, budget=False):
        runner = ProfileRunner(profile, [])
        events = []
        runner.observed_performance_suites = []
        def admission():
            events.append(('admission', 'performance'))
            if admission_error:
                raise CommandFailed(admission_error)
        def area(name, suites):
            events.append(('area', name))
            if name == 'performance':
                phase = runner.performance_result_phase
                runner.observed_performance_suites.append((phase, list(suites)))
                error = functional_error if phase == 'correctness' else area_error
                if error:
                    raise CommandFailed(error)
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()), \
             patch.object(runner, 'prepare_step_budget', side_effect=lambda name:
                StepBudgetContext(name, 1, 'blocking') if budget else None), \
             patch('sifr_verify.profile_runner.combine_performance'), \
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
                    [item['area'] for item in runner.profile['selected_areas']])
                selected = next(item['suites'] for item in runner.profile['selected_areas'] if item['area'] == 'performance')
                self.assertEqual(runner.observed_performance_suites,
                    [('correctness', [name for name in selected if name not in MEASUREMENT_SUITES])])
                self.assertCountEqual([name for kind, name in events if kind == 'guard'], runner.profile['guardrail_steps'])
                self.assertEqual([name for kind, name in events if kind == 'toolchain'], runner.profile['toolchain_steps'])

    def test_performance_execution_failure_keeps_functional_outcome(self):
        runner, status, events = self.exercise('create-pr', area_error=1)
        self.assertEqual(status, 1)
        self.assertEqual((runner.functional_exit_status, runner.performance_exit_status), (0, 1))
        self.assertEqual(events[-2:], [('admission', 'performance'), ('area', 'performance')])

    def test_performance_area_correctness_failure_is_functional(self):
        runner, status, events = self.exercise('create-pr', functional_error=9)
        self.assertEqual(status, 9)
        self.assertEqual((runner.functional_exit_status, runner.performance_exit_status), (9, 0))
        self.assertNotIn(('admission', 'performance'), events)

    def test_blocking_step_timing_does_not_stop_later_correctness(self):
        runner, status, events = self.exercise('create-pr', budget=True)
        self.assertEqual(status, 124)
        self.assertEqual((runner.functional_exit_status, runner.performance_exit_status), (0, 124))
        self.assertEqual([name for kind, name in events if kind == 'toolchain'], runner.profile['toolchain_steps'])
        self.assertEqual([name for kind, name in events if kind == 'area'],
            [item['area'] for item in runner.profile['selected_areas']] + ['performance'])
        selected = next(item['suites'] for item in runner.profile['selected_areas'] if item['area'] == 'performance')
        self.assertCountEqual([name for _, names in runner.observed_performance_suites for name in names], selected)

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


class PerformancePartitionTests(unittest.TestCase):
    def test_combined_step_timing_keeps_prior_correctness_duration(self):
        with patch('sifr_verify.profile_runner.now_ms', side_effect=[1000, 1100]), \
             contextlib.redirect_stdout(io.StringIO()):
            result = timed_step('area_performance', lambda: None, prior_elapsed_ms=500)
        self.assertEqual(result.elapsed_ms, 600)

    def test_functional_area_invocation_ignores_inherited_reference(self):
        spec = importlib.util.spec_from_file_location('independent_performance_adapter',
            REPO_ROOT / 'verification/areas/performance/runner.py')
        adapter = importlib.util.module_from_spec(spec)
        with patch.object(sys, 'path', sys.path.copy()):
            spec.loader.exec_module(adapter)
        suite = {'name': 'frontend-syntax-guardrails', 'blocking': True,
                 'total_variants': 4, 'total_failures': 0}
        with tempfile.TemporaryDirectory() as directory, contextlib.redirect_stdout(io.StringIO()), \
             patch.object(adapter, 'REPO_ROOT', Path(directory)), \
             patch.dict(os.environ, {'SIFR_PERFORMANCE_REFERENCE': 'missing-inherited-reference',
                                    'SIFR_VALIDATION_PROFILE': 'create-pr'}), \
             patch.object(adapter, 'admit_reference') as admission, \
             patch.object(adapter, 'load_profile') as reference, \
             patch.object(adapter, 'run_suite', return_value=suite):
            self.assertEqual(adapter.main(['--suite', 'frontend-syntax-guardrails', '--result-json', 'result.json']), 0)
        admission.assert_not_called()
        reference.assert_not_called()

    def test_complete_partitions_publish_full_selection_and_missing_part_rejects(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selected = ['smoke', 'frontend-syntax-guardrails']
            for phase, suite in [('correctness', selected[1]), ('measurement', selected[0])]:
                path = result_path('fixture', phase, root=root)
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps({'schema_version': 1, 'area': 'performance', 'bless': False,
                    'suites': [{'name': suite, 'blocking': True, 'total_failures': 0, 'total_variants': 2}],
                    'summary': {'blocking_failures': 0, 'total_variants': 2}}))
            combine('fixture', selected, root=root)
            payload = json.loads(result_path('fixture', root=root).read_text())
            self.assertEqual([suite['name'] for suite in payload['suites']], selected)
            self.assertEqual(payload['summary']['total_variants'], 4)
            result_path('fixture', root=root).unlink()
            result_path('fixture', 'measurement', root=root).unlink()
            with self.assertRaises(AreaResultError):
                combine('fixture', selected, root=root)
            self.assertFalse(result_path('fixture', root=root).exists())


def policy_checks() -> None:
    result = unittest.TestResult()
    for case in (ReferenceAdmissionOrderingTests, PerformancePartitionTests):
        unittest.defaultTestLoader.loadTestsFromTestCase(case).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == '__main__':
    unittest.main()
