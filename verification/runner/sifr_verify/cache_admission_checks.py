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

from .cargo_resource_forecast import command_cache_hint, test_cache_hint
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
            self.assertTrue(command_cache_hint(root,{},nested,include_library=True))
            for altered in (["unknown-interpreter",*nested[1:]],nested[:-1]+['invalid-revision'],
                            nested[:4]+['release']+nested[5:],nested+['--extra']):
                self.assertFalse(command_cache_hint(root,{},altered,include_library=True))

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
    for case in (CacheForecastTests, DiskBudgetTests):
        unittest.defaultTestLoader.loadTestsFromTestCase(case).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == '__main__':
    unittest.main()
