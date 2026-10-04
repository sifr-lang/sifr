"""SQL clean builds retain their own envelope, assertions and caller containment."""
from __future__ import annotations

import contextlib
import copy
import io
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

from .assertion_resource_forecast import SQL_BUILD_ALLOCATION, assertion_allocation
from .cloud_schedule import Schedule, load_schedule, run_staged_cloud
from .process_disk_budget import DiskBudget, FLOOR_VARIABLE, PATH_VARIABLE
from .profile_runner import ProfileRunner
from .resource_admission import Resources

GIB = 1024**3


class SqlResourceTests(unittest.TestCase):
    def test_exact_selection_preserves_canonical_profiles_and_rejects_unknowns(self):
        for profile in ('create-pr', 'merge', 'cloud'):
            runner = ProfileRunner(profile, ['--compact-resources'])
            original = copy.deepcopy(runner.profile)
            self.assertEqual(assertion_allocation('area_sql_platform', runner.profile), SQL_BUILD_ALLOCATION)
            self.assertEqual(runner.profile, original)
        valid = {'selected_areas': [{'area': 'sql_platform', 'suites': ['build-qualification']}]}
        self.assertEqual(assertion_allocation('area_sql_platform', valid), SQL_BUILD_ALLOCATION)
        for name in ('area_sql', 'area_sql_platform_extra', 'guardrail_sql_platform', 'area_other'):
            self.assertEqual(assertion_allocation(name, valid), 'remaining-assertions')
        for rows in ([], valid['selected_areas'] * 2,
                     [{'area': 'other', 'suites': ['build-qualification']}],
                     *([{'area': 'sql_platform', 'suites': suites}] for suites in (
                         [], ['contracts'], ['unknown'], ['build-qualification', 'unknown'],
                         ['build-qualification'] * 2, 'build-qualification', [None]))):
            with self.subTest(rows=rows):
                self.assertEqual(assertion_allocation('area_sql_platform', {'selected_areas': rows}),
                                 'remaining-assertions')

    def test_compact_dispatch_keeps_every_callback_and_generic_allocation(self):
        runner = ProfileRunner('create-pr', ['--compact-resources'])
        calls = []
        runner.compact_schedule = SimpleNamespace(step=lambda name, callback, **kw:
            (calls.append((name, kw)), callback(), 0)[-1])
        callback = Mock()
        runner.execute_assertion('area_sql_platform', callback)
        runner.execute_assertion('area_other', callback)
        next(row for row in runner.profile['selected_areas'] if row['area'] == 'sql_platform')['suites'] = ['contracts']
        runner.execute_assertion('area_sql_platform', callback)
        self.assertEqual(callback.call_count, 3)
        self.assertEqual([kw for _, kw in calls], [
            {'allocation': SQL_BUILD_ALLOCATION, 'monitor_disk': True},
            {'allocation': 'remaining-assertions', 'monitor_disk': True},
            {'allocation': 'remaining-assertions', 'monitor_disk': True}])

    def test_cloud_dispatch_monitors_only_selected_sql_and_preserves_all_suites(self):
        for compact in (False, True):
            for build in (False, True):
                with self.subTest(compact=compact, build=build), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    result = root/'target/verification/areas/sysroot-release-cloud-results.json'
                    result.parent.mkdir(parents=True)
                    result.write_text('{"passed":true}')
                    runner = ProfileRunner('cloud', ['--compact-resources'] if compact else [])
                    if not build:
                        next(row for row in runner.profile['selected_areas'] if row['area'] == 'sql_platform')['suites'].remove('build-qualification')
                    original = copy.deepcopy(runner.profile)
                    events = {}
                    class FakeSchedule:
                        owner = graph_owner = 'test'
                        journal = root/'journal'
                        def __init__(self, runner): pass
                        def record(self, *args): pass
                        def prepare_command(self, *args, **kwargs): pass
                        def step(self, name, callback, **kwargs):
                            events[name] = kwargs
                            callback()
                            return 0
                    with patch('sifr_verify.cloud_schedule.REPO_ROOT', root), \
                         patch('sifr_verify.cloud_schedule.Schedule', FakeSchedule), \
                         patch('sifr_verify.cloud_schedule.run_command'), \
                         patch('sifr_verify.cloud_schedule.acquire_cargo_dependencies'), \
                         patch('sifr_verify.cloud_schedule.prepare_remaining_graphs'), \
                         patch.object(runner, 'run_guardrail'), \
                         patch.object(runner, 'run_toolchain_step'), \
                         patch.object(runner, 'run_area') as areas:
                        self.assertEqual(run_staged_cloud(runner, set()), 0)
                    self.assertEqual(runner.profile, original)
                    self.assertEqual(areas.call_count, len(original['selected_areas']))
                    self.assertEqual(events['area_sql_platform'], {
                        'allocation': SQL_BUILD_ALLOCATION if build else 'remaining-assertions',
                        'monitor_disk': build})
                    for name, options in events.items():
                        if name.startswith('area_') and name not in ('area_sql_platform', 'area_sysroot_release'):
                            self.assertEqual(options, {'allocation': 'remaining-assertions', 'monitor_disk': False})
                    self.assertEqual([call.args for call in areas.call_args_list],
                        [(row['area'], row['suites']) for row in original['selected_areas']
                         if row['area'] == 'sysroot_release'] +
                        [(row['area'], row['suites']) for row in original['selected_areas']
                         if row['area'] != 'sysroot_release'])

    def test_policy_preserves_mode_reserves_and_memory(self):
        for mode, reserve in (('compact', 2), ('cloud', 8)):
            policy = load_schedule(mode=mode)
            self.assertEqual(policy['stages'][SQL_BUILD_ALLOCATION], dict(
                disk_growth_bytes=6*GIB, disk_reserve_bytes=reserve*GIB, retained_copy_bytes=0,
                memory_peak_bytes=6*GIB, memory_reserve_bytes=2*GIB, tmpfs_growth_bytes=0))
            self.assertEqual(policy['assertion_command_deadline_seconds'], 2400)
            self.assertEqual(policy['cold_preparation_deadline_seconds'], 7200)

    def test_admission_monitored_floor_failure_classification_and_restoration(self):
        for mode, available, inherited, free, expected in (
                ('compact', 8*GIB-1, None, None, 2),
                ('compact', 8*GIB, None, 3*GIB, 0),
                ('compact', 8*GIB, None, 3*GIB-1, 2),
                ('compact', 12*GIB, None, 6*GIB-1, 2),
                ('compact', 8*GIB, 7*GIB, 6*GIB, 2),
                ('compact', 8*GIB, 7*GIB, 7*GIB, 0),
                ('cloud', 14*GIB-1, None, None, 2),
                ('cloud', 14*GIB, None, 9*GIB, 0),
                ('cloud', 14*GIB, None, 9*GIB-1, 2)):
            with self.subTest(mode=mode, available=available, inherited=inherited, free=free), \
                 tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                runner = ProfileRunner('create-pr', ['--compact-resources'])
                runner.prepare_step_budget = lambda name: None
                runner.env['SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'] = '17'
                if inherited is not None:
                    runner.env.update({FLOOR_VARIABLE: str(inherited), PATH_VARIABLE: str(root)})
                before = {field: runner.env.get(field) for field in (FLOOR_VARIABLE, PATH_VARIABLE)}
                schedule = object.__new__(Schedule)
                schedule.runner, schedule.root, schedule.policy = runner, root, load_schedule(mode=mode)
                schedule.key = {'inputs': {'source': []}}
                records = []
                schedule.record = lambda name, payload: records.append((name, payload))
                runner.compact_schedule = schedule
                capacity = Resources(5, 4, 32*GIB, 32*GIB, available, {}, [], {})
                calls = []
                def native():
                    calls.append(1)
                    self.assertEqual(runner.env['SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'], '17.0')
                    self.assertNotIn('SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS', runner.env)
                    with patch('sifr_verify.process_disk_budget.shutil.disk_usage',
                               return_value=SimpleNamespace(free=free)):
                        DiskBudget.from_environment(runner.env).check()
                with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()), \
                     patch('sifr_verify.cloud_schedule.inventory', return_value=[]), \
                     patch('sifr_verify.cloud_schedule.discover', return_value=capacity):
                    status = runner.execute_assertion('area_sql_platform', native)
                self.assertEqual(status, expected)
                self.assertEqual(len(calls), 0 if free is None else 1)
                self.assertEqual({field: runner.env.get(field) for field in before}, before)
                self.assertEqual(runner.env['SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'], '17')
                if expected:
                    self.assertTrue(any(row.get('classification') == 'enospc' for _, row in records))
                if calls:
                    floor = next(row['floor_bytes'] for name, row in records if name.endswith('-disk-floor'))
                    reserve = (2 if mode == 'compact' else 8)*GIB
                    self.assertEqual(floor, max(available-6*GIB, reserve+GIB, inherited or 0))


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(SqlResourceTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == '__main__':
    unittest.main()
