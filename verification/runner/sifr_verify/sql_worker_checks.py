"""Named SQL build workers preserve admission, ordinary workers and failure custody."""
from __future__ import annotations

import contextlib
import errno
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from .cloud_schedule import Schedule, load_schedule
from .process_disk_budget import FLOOR_VARIABLE, PATH_VARIABLE, DiskBudget
from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner
from .resource_admission import ResourceError, Resources

GIB = 1024**3


class SqlWorkerChecks(unittest.TestCase):
    @contextlib.contextmanager
    def setup_schedule(self, *, mode='compact', cpus=4, memory=64*GIB, disk=64*GIB,
                       inventories=None):
        with tempfile.TemporaryDirectory() as directory, contextlib.ExitStack() as stack:
            root = Path(directory)
            runner = ProfileRunner('merge', ['--compact-resources'])
            runner.execute_step = lambda name, callback: (callback(), 0)[1]
            runner.env['SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'] = '17'
            runner.env[FLOOR_VARIABLE] = str(12*GIB if mode == 'cloud' else 4*GIB)
            runner.env[PATH_VARIABLE] = str(root)
            schedule = object.__new__(Schedule)
            schedule.runner, schedule.root, schedule.policy = runner, root, load_schedule(mode=mode)
            schedule.key = {'inputs': {'source': []}}
            records = []
            schedule.record = lambda name, payload: records.append((name, payload))
            resources = Resources(5, cpus, 64*GIB, memory, disk, {}, [], {})
            stack.enter_context(patch('sifr_verify.cloud_schedule.discover', return_value=resources))
            stack.enter_context(patch('sifr_verify.cloud_schedule.inventory',
                                      side_effect=inventories or (lambda root: [])))
            yield runner, schedule, records

    def run_build(self, schedule, callback, **kwargs):
        return schedule.step('sql_build_assertions', callback, allocation='sql-build-qualification',
                             monitor_disk=True, **kwargs)

    def test_both_policies_clamp_and_record_actual_workers_without_changing_e2e(self):
        for mode in ('compact', 'cloud'):
            for cpus, expected in ((1.5, 1), (2, 2), (4, 2)):
                with self.subTest(mode=mode, cpus=cpus), self.setup_schedule(mode=mode, cpus=cpus) as (runner, schedule, records):
                    original = runner.env.copy()
                    seen = []
                    def callback():
                        seen.append(runner.env['CARGO_BUILD_JOBS'])
                        self.assertEqual(runner.e2e_worker_limits['cargo_build_jobs'], 1)
                        # Merge retains its four-thread Rayon policy; only Cargo gets the SQL limit.
                        self.assertEqual(runner.env['RAYON_NUM_THREADS'], str(min(4, int(cpus))))
                        floor = DiskBudget.from_environment(runner.env).floor
                        self.assertEqual(floor, 58*GIB)
                        self.assertEqual(runner.env['SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'], '17.0')
                    self.assertEqual(self.run_build(schedule, callback), 0)
                    self.assertEqual(seen, [str(expected)])
                    self.assertEqual(runner.env['CARGO_BUILD_JOBS'], original['CARGO_BUILD_JOBS'])
                    for key in (FLOOR_VARIABLE, PATH_VARIABLE, 'SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'):
                        self.assertEqual(runner.env[key], original[key])
                    terminal = [row for _, row in records if row.get('state') in ('admitted', 'completed')]
                    self.assertEqual(len(terminal), 2)
                    for row in terminal:
                        self.assertEqual(row['cargo_workers'], {'scope':'sql-build-qualification',
                            'requested':2, 'effective':expected, 'ordinary':'1'})
                    admitted = terminal[0]
                    self.assertEqual(admitted['memory_admitted_bytes'], 8*GIB)
                    self.assertEqual(admitted['requirements']['disk_reserve_bytes'], (2 if mode == 'compact' else 8)*GIB)

    def test_every_failure_restores_workers_and_preserves_its_classification(self):
        for error, classification in ((CommandFailed(1), 'assertion'),
                (CommandFailed(124, 'safety_deadline'), 'timeout'),
                (OSError(errno.ENOSPC, 'full'), 'enospc'),
                (KeyboardInterrupt(), 'cancelled'), (RuntimeError('unexpected'), 'assertion')):
            with self.subTest(error=type(error).__name__, kind=classification), self.setup_schedule() as (runner, schedule, records):
                before = runner.env.copy()
                def callback():
                    self.assertEqual(runner.env['CARGO_BUILD_JOBS'], '2')
                    raise error
                with self.assertRaises(type(error)):
                    self.run_build(schedule, callback)
                self.assertEqual(runner.env, before)
                failed = records[-1][1]
                self.assertEqual((failed['state'], failed['classification']), ('failed', classification))
                self.assertEqual(failed['cargo_workers']['effective'], 2)

    def test_area_command_receives_override_then_e2e_and_remaining_receive_ordinary_workers(self):
        with self.setup_schedule() as (runner, schedule, records):
            calls = []
            with patch('sifr_verify.profile_runner.run_selected_area',
                       side_effect=lambda **kwargs: kwargs['command_runner'](['selected-area'])), \
                 patch('sifr_verify.profile_runner.run_command',
                       side_effect=lambda argv, env: calls.append((argv, env.copy()))):
                self.run_build(schedule, lambda: runner.run_area('sql_platform', ['build-qualification']))
                runner.run_area('sql_platform', ['contracts'])
                runner.run_e2e_pass_suite()
            self.assertEqual([env['CARGO_BUILD_JOBS'] for _, env in calls], ['2', '1', '1'])
            argv = calls[-1][0]
            self.assertEqual(argv[argv.index('--cargo-build-jobs') + 1], '1')
            self.assertEqual(records[-1][1]['cargo_workers']['effective'], 2)

    def test_post_callback_validation_failure_restores_workers(self):
        with self.setup_schedule(inventories=[[], ['changed']]) as (runner, schedule, records):
            original = runner.env.copy()
            with self.assertRaisesRegex(ResourceError, 'inputs changed during'):
                self.run_build(schedule, lambda: self.assertEqual(runner.env['CARGO_BUILD_JOBS'], '2'))
            self.assertEqual(runner.env, original)
            self.assertEqual(records[-1][1]['classification'], 'unavailable')
            self.assertEqual(records[-1][1]['cargo_workers']['effective'], 2)

    def test_insufficient_memory_or_disk_never_installs_override_or_spawns_callback(self):
        for mode, capacity, classification in (
                ('compact', {'memory':8*GIB-1}, 'admission'),
                ('compact', {'disk':8*GIB-1}, 'enospc'),
                ('cloud', {'memory':8*GIB-1}, 'admission'),
                ('cloud', {'disk':14*GIB-1}, 'enospc')):
            with self.subTest(mode=mode, capacity=capacity), self.setup_schedule(mode=mode, **capacity) as (runner, schedule, records):
                original = runner.env.copy()
                with self.assertRaises(ResourceError):
                    self.run_build(schedule, lambda: self.fail('unadmitted callback'))
                self.assertEqual(runner.env, original)
                self.assertEqual(records[-1][1]['classification'], classification)
                self.assertNotIn('cargo_workers', records[-1][1])

    def test_wrong_step_or_allocation_does_not_override_ordinary_workers(self):
        for name, allocation in (('area_sql_platform', 'sql-build-qualification'),
                ('sql_remaining_assertions', 'remaining-assertions'),
                ('sql_build_assertions', 'remaining-assertions'),
                ('unknown', 'sql-build-qualification')):
            with self.subTest(name=name, allocation=allocation), self.setup_schedule() as (runner, schedule, records):
                schedule.step(name, lambda: self.assertEqual(runner.env['CARGO_BUILD_JOBS'], '1'),
                              allocation=allocation, monitor_disk=True)
                self.assertTrue(all('cargo_workers' not in row for _, row in records))

    def test_named_override_rejects_preparation_and_any_alternate_environment(self):
        for kind in ('preparation', 'copy', 'same'):
            with self.subTest(kind=kind), self.setup_schedule() as (runner, schedule, records):
                original = runner.env.copy()
                alternate = runner.env.copy() if kind == 'copy' else runner.env
                kwargs = {'preparation':True} if kind == 'preparation' else {'command_env':alternate}
                with self.assertRaisesRegex(ResourceError, 'assertion runner environment'):
                    self.run_build(schedule, lambda: self.fail('invalid-context callback'), **kwargs)
                self.assertEqual(runner.env, original)
                self.assertEqual(alternate, original)
                self.assertEqual(records[-1][1]['classification'], 'unavailable')
