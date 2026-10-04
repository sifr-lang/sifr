"""Bounded cache estimates cannot skip native checks or leak owned processes."""
from __future__ import annotations
import errno
import contextlib
import io
import os
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from .cargo_resource_forecast import command_cache_hint, generated_preparation, test_cache_hint
from .cloud_schedule import Schedule, load_schedule
from .process_disk_budget import DiskBudget, FLOOR_VARIABLE, PATH_VARIABLE
from .process_execution import execute
from . import process_recovery_checks as recovery_checks
from .profile_runner import ProfileRunner
from .resource_admission import Resources


class CacheForecastTests(unittest.TestCase):
    def test_coordination_does_not_cap_child_growth_and_keeps_caller_floor(self):
        for caller_floor,expected_status in ((3*1024**3,0),(6*1024**3,2)):
            with self.subTest(caller_floor=caller_floor),tempfile.TemporaryDirectory() as directory:
                root=Path(directory);runner=ProfileRunner('create-pr',['--compact-resources'])
                runner.prepare_step_budget=lambda name:None
                runner.env.update({FLOOR_VARIABLE:str(caller_floor),PATH_VARIABLE:str(root)})
                schedule=object.__new__(Schedule)
                schedule.runner,schedule.root,schedule.policy=runner,root,load_schedule(mode='compact')
                schedule.key={'inputs':{'source':[]}};schedule.index=0
                def record(name,payload): schedule.index+=1
                schedule.record=record
                capacity=Resources(5,4,32*1024**3,32*1024**3,7*1024**3,{},[],{})
                floors=[]
                def native(argv,*,env):
                    budget=DiskBudget.from_environment(env);floors.append(budget.floor)
                    # The child may consume more than the coordinator's 256MiB
                    # while remaining within its own prospectively admitted 2GiB.
                    with patch('sifr_verify.process_disk_budget.shutil.disk_usage',return_value=SimpleNamespace(free=5*1024**3+512*1024**2)):
                        budget.check()
                with contextlib.redirect_stdout(io.StringIO()),contextlib.redirect_stderr(io.StringIO()), \
                     patch('sifr_verify.cloud_schedule.inventory',return_value=[]), \
                     patch('sifr_verify.cloud_schedule.discover',return_value=capacity), \
                     patch('sifr_verify.cloud_schedule.command_cache_hint',return_value=False), \
                     patch('sifr_verify.cloud_schedule.run_command',side_effect=native):
                    status=schedule.step('cargo_cache_setup',lambda:schedule.prepare_command(
                        ['cargo','build','-p','fixture'],env=runner.env),allocation='preparation-coordination',preparation=True)
                self.assertEqual(status,expected_status)
                self.assertEqual(floors,[max(5*1024**3,caller_floor)])
                self.assertEqual(runner.env[FLOOR_VARIABLE],str(caller_floor))

    def infrastructure_failure(self, *, drift=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runner = ProfileRunner('cloud', [])
            runner.prepare_step_budget = lambda name: None
            schedule = object.__new__(Schedule)
            schedule.runner, schedule.root, schedule.policy = runner, root, load_schedule()
            schedule.key = {'inputs': {'source': []}}
            schedule.index = 0
            records = []
            def record(name, payload):
                schedule.index += 1; records.append((name, payload))
            schedule.record = record
            resources = Resources(5, 4, 32*1024**3, 32*1024**3,
                                  (20 if drift else 10)*1024**3, {}, [], {})
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()), \
                 patch('sifr_verify.cloud_schedule.inventory', side_effect=[[], [], ['drift']] if drift else None,
                       return_value=[]), \
                 patch('sifr_verify.cloud_schedule.discover', return_value=resources), \
                 patch('sifr_verify.cloud_schedule.command_cache_hint', return_value=False), \
                 patch('sifr_verify.cloud_schedule.run_command') as native:
                status = schedule.step('cargo_cache_setup', lambda: schedule.prepare_command(
                    ['cargo', 'test', '--no-run', '-p', 'fixture'], env=runner.env),
                    allocation='preparation-coordination', preparation=True)
            self.assertEqual(status, 2)
            if drift:
                native.assert_called_once()
            else:
                native.assert_not_called()
            failure = [payload for name, payload in records if name == 'cargo_cache_setup' and payload['state'] == 'failed']
            self.assertEqual(len(failure), 1)
            self.assertEqual(failure[0]['classification'], 'unavailable' if drift else 'enospc')

    def test_inner_admission_refusal_preserves_outer_infrastructure_classification(self):
        self.infrastructure_failure()

    def test_post_command_source_drift_preserves_outer_infrastructure_classification(self):
        self.infrastructure_failure(drift=True)

    def test_present_cache_is_only_a_hint_and_unknown_targets_stay_cold(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            debug = root / 'target/debug'
            binary = debug / 'deps/sifr_driver-abc'
            fingerprint = debug / '.fingerprint/sifr_driver-abc/test-lib-sifr_driver'
            binary.parent.mkdir(parents=True)
            fingerprint.parent.mkdir(parents=True)
            binary.write_bytes(b'native')
            fingerprint.write_bytes(b'fingerprint')
            command = ['cargo', 'test', '--no-run', '-p', 'sifr_driver']
            self.assertTrue(command_cache_hint(root, {}, command))
            self.assertFalse(command_cache_hint(root, {}, command + ['--target', 'foreign']))
            self.assertFalse(command_cache_hint(root, {}, command + ['--release']))
            self.assertFalse(command_cache_hint(root, {}, ['python', 'build.py']))
            self.assertFalse(command_cache_hint(root, {'CARGO_TARGET_DIR': 'other'}, command))
            binary.unlink(); binary.symlink_to(fingerprint)
            self.assertFalse(test_cache_hint(root, {}, 'sifr_driver'))

    def test_partial_libraries_only_inform_explicit_compact_forecasts(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            debug = root/'target/debug'
            for package in ('first','second'):
                library=debug/'deps'/('lib'+package+'-abc.rlib')
                fingerprint=debug/'.fingerprint'/(package+'-abc')/('lib-'+package)
                library.parent.mkdir(parents=True,exist_ok=True)
                fingerprint.parent.mkdir(parents=True)
                library.write_bytes(b'compiled-library-presence')
                fingerprint.write_bytes(b'fingerprint-presence')
            command=['cargo','test','--no-run','-p','first','-p','second']
            self.assertFalse(command_cache_hint(root,{},command))
            self.assertTrue(command_cache_hint(root,{},command,include_library=True))
            (debug/'deps/libsecond-abc.rlib').unlink()
            self.assertFalse(command_cache_hint(root,{},command,include_library=True))
            (debug/'deps/libfirst-abc.rlib').unlink()
            (debug/'deps/libfirst-abc.rlib').symlink_to(debug/'.fingerprint/first-abc/lib-first')
            self.assertFalse(test_cache_hint(root,{},'first',include_library=True))
            (debug/'sifr').write_bytes(b'compiler-presence')
            nested=[sys.executable,'-m','sifr_verify.generated_cargo_setup','--profile','create-pr','--revision','a'*40]
            self.assertFalse(command_cache_hint(root,{},nested))
            self.assertFalse(command_cache_hint(root,{},nested,include_library=True))
            self.assertTrue(generated_preparation(nested))
            for altered in (["unknown-interpreter",*nested[1:]],nested[:-1]+['invalid-revision'],
                            nested[:4]+['release']+nested[5:],nested+['--extra']):
                self.assertFalse(command_cache_hint(root,{},altered,include_library=True))
                self.assertFalse(generated_preparation(altered))

    def test_sequential_commands_are_admitted_individually_and_always_executed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runner = ProfileRunner('cloud', [])
            runner.execute_step = lambda name, callback: (callback(), 0)[1]
            schedule = object.__new__(Schedule)
            schedule.runner, schedule.root, schedule.policy = runner, root, load_schedule()
            schedule.key = {'inputs': {'source': []}}
            schedule.index = 0
            records = []
            def record(name, payload):
                schedule.index += 1; records.append((name, payload))
            schedule.record = record
            resources = Resources(5, 4, 32*1024**3, 32*1024**3, 20*1024**3, {}, [], {})
            calls = []
            env = {'CARGO_BUILD_JOBS': '99', 'RAYON_NUM_THREADS': '99'}
            with patch('sifr_verify.cloud_schedule.inventory', return_value=[]), \
                 patch('sifr_verify.cloud_schedule.discover', return_value=resources), \
                 patch('sifr_verify.cloud_schedule.command_cache_hint', return_value=True), \
                 patch('sifr_verify.cloud_schedule.run_command', side_effect=lambda argv, env: calls.append((argv, env.copy()))):
                for package in ['first', 'second']:
                    schedule.prepare_command(['cargo', 'test', '--no-run', '-p', package], env=env)
            self.assertEqual([argv[-1] for argv, _ in calls], ['first', 'second'])
            self.assertTrue(all(item[FLOOR_VARIABLE] == str(18*1024**3) for _, item in calls))
            self.assertTrue(all(item['CARGO_BUILD_JOBS'] == '4' for _, item in calls))
            self.assertNotIn(FLOOR_VARIABLE, env)
            self.assertNotIn(PATH_VARIABLE, env)
            forecasts = [payload for name, payload in records if name == 'preparation-command']
            self.assertEqual(len(forecasts), 2)
            self.assertTrue(all(payload['assertion_reuse'] is False for payload in forecasts))


class GeneratedPreparationForecastTests(unittest.TestCase):
    def command(self, profile='create-pr', revision='a'*40):
        return [sys.executable, '-m', 'sifr_verify.generated_cargo_setup',
                '--profile', profile, '--revision', revision]

    def schedule(self, root, mode):
        runner = ProfileRunner('create-pr', ['--compact-resources'])
        runner.prepare_step_budget = lambda name: None
        runner.env['SIFR_VERIFY_RESOURCE_POLICY'] = mode
        schedule = object.__new__(Schedule)
        schedule.runner, schedule.root, schedule.policy = runner, root, load_schedule(mode=mode)
        schedule.key = {'inputs': {'source': []}}
        schedule.index = 0
        records = []
        def record(name, payload):
            schedule.index += 1
            records.append((name, payload))
        schedule.record = record
        return schedule, records

    def test_finite_wrapper_requires_exact_interpreter_profile_revision_and_argv(self):
        for profile in ('create-pr', 'merge', 'nightly', 'cloud'):
            self.assertTrue(generated_preparation(self.command(profile)))
        command = self.command()
        for altered in ([], command[:-1], command + ['--extra'],
                        ['python', *command[1:]], command[:1] + ['-I'] + command[1:],
                        command[:2] + ['sifr_verify.generated_cargo_setup_other'] + command[3:],
                        self.command('release'), self.command(revision='HEAD'),
                        self.command(revision='a'*39), self.command(revision='A'*40),
                        command[:3] + ['--revision', 'a'*40, '--profile', 'create-pr']):
            with self.subTest(argv=altered):
                self.assertFalse(generated_preparation(altered))

    def test_outer_cli_presence_cannot_discount_revision_sources_or_native_growth(self):
        for mode, growth, reserve in (('compact', 4, 2), ('cloud', 6, 8)):
            for warm in (False, True):
                with self.subTest(mode=mode, warm=warm), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    if warm:
                        binary = root/'target/debug/sifr'
                        binary.parent.mkdir(parents=True)
                        binary.write_bytes(b'outer-compiler-without-revision-checkout')
                    schedule, records = self.schedule(root, mode)
                    capacity = Resources(5, 4, 32*1024**3, 32*1024**3, 20*1024**3, {}, [], {})
                    calls = []
                    def native(argv, *, env):
                        calls.append((argv, env.copy()))
                    with patch('sifr_verify.cloud_schedule.inventory', return_value=[]), \
                         patch('sifr_verify.cloud_schedule.discover', return_value=capacity), \
                         patch('sifr_verify.cloud_schedule.run_command', side_effect=native):
                        # A different exact revision always needs the same closure,
                        # even when the outer CLI or an earlier revision was present.
                        for revision in ('a'*40, 'b'*40):
                            schedule.prepare_command(self.command(revision=revision), env=schedule.runner.env)
                    self.assertEqual([argv for argv, _ in calls], [self.command(revision=r*40) for r in ('a', 'b')])
                    forecasts = [p for name, p in records if name == 'preparation-command']
                    self.assertEqual([p['allocation'] for p in forecasts], ['generated-preparation']*2)
                    self.assertTrue(all(not p['cache_presence_hint'] and not p['assertion_reuse'] for p in forecasts))
                    for _, env in calls:
                        self.assertEqual(int(env[FLOOR_VARIABLE]), (20-growth)*1024**3)
                        self.assertEqual(float(env['SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS']), 7200)
                    allocation = schedule.policy['stages']['generated-preparation']
                    self.assertEqual(allocation, dict(disk_growth_bytes=growth*1024**3,
                        disk_reserve_bytes=reserve*1024**3, retained_copy_bytes=0,
                        memory_peak_bytes=6*1024**3, memory_reserve_bytes=2*1024**3, tmpfs_growth_bytes=0))
                    self.assertNotIn(FLOOR_VARIABLE, schedule.runner.env)

    def test_generated_admission_and_growth_keep_reserve_and_caller_floor(self):
        # First refuses before execution; the others exercise the actual disk
        # budget during a command rather than merely asserting a policy number.
        for available, caller_floor, observed_free, expected in (
                (6*1024**3-1, None, None, 2),
                (9*1024**3, None, 7*1024**3, 0),
                (9*1024**3, None, 5*1024**3-1, 2),
                (9*1024**3, 8*1024**3, 7*1024**3, 2)):
            with self.subTest(available=available, caller_floor=caller_floor, free=observed_free), \
                 tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                schedule, records = self.schedule(root, 'compact')
                env = schedule.runner.env
                if caller_floor is not None:
                    env.update({FLOOR_VARIABLE: str(caller_floor), PATH_VARIABLE: str(root)})
                capacity = Resources(5, 4, 32*1024**3, 32*1024**3, available, {}, [], {})
                def native(argv, *, env):
                    with patch('sifr_verify.process_disk_budget.shutil.disk_usage',
                               return_value=SimpleNamespace(free=observed_free)):
                        DiskBudget.from_environment(env).check()
                with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()), \
                     patch('sifr_verify.cloud_schedule.inventory', return_value=[]), \
                     patch('sifr_verify.cloud_schedule.discover', return_value=capacity), \
                     patch('sifr_verify.cloud_schedule.run_command', side_effect=native) as execute:
                    status = schedule.step('cargo_cache_setup', lambda: schedule.prepare_command(
                        self.command(), env=env), allocation='preparation-coordination', preparation=True)
                self.assertEqual(status, expected)
                self.assertEqual(execute.call_count, 0 if observed_free is None else 1)
                if expected:
                    failures = [p for _, p in records if p.get('state') in {'failed', 'infrastructure-failure'}]
                    self.assertTrue(failures)
                    self.assertTrue(all(p['classification'] == 'enospc' for p in failures))
                self.assertEqual(env.get(FLOOR_VARIABLE), None if caller_floor is None else str(caller_floor))


class DiskBudgetTests(unittest.TestCase):
    def test_invalid_partial_relative_or_nonpositive_budget_rejects(self):
        self.assertIsNone(DiskBudget.from_environment({}))
        for env in [{FLOOR_VARIABLE: '1'}, {PATH_VARIABLE: '/tmp'},
                    {FLOOR_VARIABLE: '0', PATH_VARIABLE: '/tmp'},
                    {FLOOR_VARIABLE: '-1', PATH_VARIABLE: '/tmp'},
                    {FLOOR_VARIABLE: '1.5', PATH_VARIABLE: '/tmp'},
                    {FLOOR_VARIABLE: '1', PATH_VARIABLE: '.'}]:
            with self.subTest(env=env), self.assertRaises(ValueError):
                DiskBudget.from_environment(env)

    def test_exhausted_budget_rejects_before_spawn(self):
        env = dict(os.environ, **{FLOOR_VARIABLE: str(2**100), PATH_VARIABLE: str(Path.cwd().resolve())})
        with patch('sifr_verify.process_execution.subprocess.Popen') as spawn:
            with self.assertRaises(OSError) as failure:
                execute([sys.executable, '-c', 'pass'], cwd=Path.cwd(), env=env)
        self.assertEqual(failure.exception.errno, errno.ENOSPC)
        spawn.assert_not_called()

    @unittest.skipUnless(sys.platform.startswith('linux'), 'Linux owned descendant contract')
    def test_growth_abort_reaps_detached_term_ignoring_descendant(self):
        with tempfile.TemporaryDirectory() as directory:
            pidfile = Path(directory) / 'pid'
            recovery = recovery_checks.RecoveryTests()
            env = dict(os.environ, **{FLOOR_VARIABLE: '1', PATH_VARIABLE: str(Path.cwd().resolve())})
            def check(budget):
                if pidfile.exists():
                    raise OSError(errno.ENOSPC, 'injected growth budget exhausted')
            with patch.object(DiskBudget, 'check', check):
                with self.assertRaises(OSError) as failure:
                    execute([sys.executable, '-c', recovery.program(pidfile, wait=True)],
                            cwd=Path.cwd(), env=env, deadline_seconds=3)
            self.assertEqual(failure.exception.errno, errno.ENOSPC)
            recovery.assert_reaped(pidfile)


def policy_checks():
    result = unittest.TestResult()
    for case in (CacheForecastTests, GeneratedPreparationForecastTests, DiskBudgetTests):
        unittest.defaultTestLoader.loadTestsFromTestCase(case).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == '__main__':
    unittest.main()
