"""R1 admission, override, preparation and selection regression controls."""
from __future__ import annotations

from copy import deepcopy
import io
import os
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from .build_allocation import allocate, extract_overrides
from .errors import VerificationError
from .profiles import load_all_profiles, load_profile
from .resource_capacity import Capacity, linux_capacity

GIB = 1024 ** 3
ROOM = Capacity(8, 12 * GIB)


class BuildAllocationTests(unittest.TestCase):
    def test_compiler_budget_is_independent_of_native_inner_jobs(self):
        profile = load_profile('merge')
        result = allocate(profile, {}, {}, ROOM)
        self.assertEqual(result.compiler_jobs, 4)
        self.assertEqual((result.rust_jobs, result.cargo_build_jobs), (3, 1))
        profile['e2e']['cargo_build_jobs'] = 2
        result = allocate(profile, {}, {}, ROOM)
        self.assertEqual(result.compiler_jobs, 4)
        self.assertEqual((result.rust_jobs, result.cargo_build_jobs), (2, 2))

    def test_profile_env_cli_precedence_and_capacity_bound(self):
        profile = load_profile('merge')
        env = {'CARGO_BUILD_JOBS': '2', 'SIFR_E2E_RUST_JOBS': '9',
               'SIFR_E2E_CARGO_BUILD_JOBS': '3'}
        overrides, args = extract_overrides(['--no-cache', '--cargo-build-jobs', '2',
                                             '--rust-jobs=3', '--max-group-fixtures', '7'])
        result = allocate(profile, env, overrides, ROOM)
        self.assertEqual(result.compiler_jobs, 2)
        self.assertEqual((result.rust_jobs, result.cargo_build_jobs), (2, 2))
        self.assertEqual(result.requested['rust_jobs'], 3)
        self.assertEqual(result.sources['rust_jobs'], 'CLI')
        self.assertEqual(result.sources['compiler_jobs'], 'CARGO_BUILD_JOBS')
        self.assertEqual(args, ['--no-cache', '--max-group-fixtures', '7'])
        for cpus in (1, 2, 4, 16):
            for memory in (4, 6, 16):
                result = allocate(profile, env, overrides, Capacity(cpus, memory * GIB))
                self.assertLessEqual(result.compiler_jobs, result.slot_limit)
                self.assertLessEqual(result.rust_jobs * result.cargo_build_jobs, result.slot_limit)
                self.assertLessEqual(result.sifr_jobs, result.slot_limit)
                self.assertLessEqual(result.run_jobs, result.slot_limit)
                self.assertLessEqual(result.slot_limit, max(1, cpus - 1))
                self.assertLessEqual(result.slot_limit, (memory - 2) // 2)
        with self.assertRaisesRegex(VerificationError, 'insufficient memory'):
            allocate(profile, {}, {}, Capacity(8, 3 * GIB))

    def test_invalid_profile_allocations_fail(self):
        for key in ('max_parallel', 'compiler_build_jobs', 'memory_per_build_job_mib'):
            profile = load_profile('merge')
            profile['resource_policy'][key] = 0
            with self.assertRaises(VerificationError):
                allocate(profile, {}, {}, ROOM)
        for key in ('reserved_cpus', 'reserved_memory_mib'):
            profile = load_profile('merge')
            profile['resource_policy'][key] = -1
            with self.assertRaises(VerificationError):
                allocate(profile, {}, {}, ROOM)

    def test_invalid_overrides_fail_before_execution(self):
        for value in ('0', '-1', 'auto', '1.5', '', ' 2', '٢'):
            with self.assertRaises(VerificationError):
                allocate(load_profile('merge'), {'CARGO_BUILD_JOBS': value}, {}, ROOM)
            with self.assertRaises(VerificationError):
                extract_overrides(['--rust-jobs', value])
        with self.assertRaises(VerificationError):
            extract_overrides(['--sifr-jobs'])

    def test_immutable_selection_and_configuration(self):
        from .compiler_configuration_plan import configuration_plan
        for profile in load_all_profiles().values():
            original = deepcopy(profile)
            modes = ('smoke', 'full') if profile.get('crate_test_membership', {}).get('suites') else ()
            plans = [configuration_plan(profile, mode) for mode in modes]
            allocate(profile, {}, {}, ROOM)
            self.assertEqual(profile, original)
            self.assertEqual(plans, [configuration_plan(profile, mode) for mode in modes])

    def test_runner_preparation_and_execution_keep_compiler_env(self):
        from . import profile_runner
        from .cargo_crate_setup import prepare_crate_test_binaries
        with patch('sifr_verify.build_allocation.discover_capacity', return_value=ROOM), \
             patch.dict('os.environ', {'CARGO_BUILD_JOBS': '3'}, clear=True):
            runner = profile_runner.ProfileRunner('merge', ['--cargo-build-jobs', '2'])
        preparations = []
        prepare_crate_test_binaries(runner.profile, runner.env,
                                   lambda cmd, **kw: preparations.append((cmd, kw['env'].copy())))
        executions = []
        with patch.object(profile_runner, 'run_command',
                          side_effect=lambda cmd, **kw: executions.append((cmd, kw['env'].copy()))):
            runner.run_e2e_pass_suite()
        self.assertTrue(preparations)
        self.assertEqual({env['CARGO_BUILD_JOBS'] for _, env in preparations + executions}, {'3'})
        command, env = executions[0]
        self.assertEqual(command.count('--cargo-build-jobs'), 1)
        self.assertEqual(command[command.index('--cargo-build-jobs') + 1], '2')
        self.assertEqual(command[command.index('--rust-jobs') + 1], '2')
        self.assertEqual(env['SIFR_E2E_CARGO_BUILD_JOBS'], '2')
        self.assertIn('"compiler_jobs": 3', runner.build_allocation.report())
        self.assertIn('"requested"', runner.build_allocation.report())

    def test_shell_preserves_compiler_jobs_and_separates_native_override(self):
        from .paths import REPO_ROOT
        with TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for name, body in {
                'uv': '#!/bin/sh\necho merge\n',
                'cargo': '#!/bin/sh\nprintf "compiler=%s native=%s\\n" "$CARGO_BUILD_JOBS" "$SIFR_E2E_CARGO_BUILD_JOBS"\n',
            }.items():
                path = directory / name
                path.write_text(body)
                path.chmod(0o755)
            env = dict(os.environ, PATH=str(directory) + os.pathsep + os.environ['PATH'],
                       CARGO_BUILD_JOBS='4', SIFR_E2E_CARGO_BUILD_JOBS='1')
            result = subprocess.run(['bash', str(REPO_ROOT / 'verification/runner/e2e/run_e2e_pass.sh'),
                                     '--profile', 'merge', '--cargo-build-jobs', '2'],
                                    env=env, capture_output=True, text=True, check=True, timeout=10)
            self.assertIn('compiler=4 native=2', result.stdout)

    def test_linux_nested_cgroup_parent_limits_and_affinity(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc, cg = root / 'proc', root / 'cgroup'
            (proc / 'self').mkdir(parents=True)
            (cg / 'parent/child').mkdir(parents=True)
            (proc / 'meminfo').write_text(f'MemAvailable: {16 * 1024 ** 2} kB\n')
            (proc / 'self/cgroup').write_text('0::/container/parent/child\n')
            (proc / 'self/mountinfo').write_text('1 0 0:1 /container /sys/fs/cgroup rw - cgroup2 cgroup rw\n')
            (cg / 'parent/cpu.max').write_text('250000 100000')
            (cg / 'parent/memory.max').write_text(str(8 * GIB))
            (cg / 'parent/memory.current').write_text(str(3 * GIB))
            (cg / 'parent/child/cpu.max').write_text('max 100000')
            (cg / 'parent/child/memory.max').write_text('max')
            with patch('os.sched_getaffinity', return_value=set(range(8)), create=True):
                self.assertEqual(linux_capacity(proc, cg), Capacity(2, 5 * GIB))
            with patch('os.sched_getaffinity', return_value={0}, create=True):
                self.assertEqual(linux_capacity(proc, cg).cpu_count, 1)
            (cg / 'parent/memory.current').write_text(str(9 * GIB))
            with patch('os.sched_getaffinity', return_value={0}, create=True):
                self.assertEqual(linux_capacity(proc, cg).available_memory_bytes, 0)


def policy_checks() -> None:
    result = unittest.TextTestRunner(stream=io.StringIO()).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(BuildAllocationTests))
    if not result.wasSuccessful():
        raise AssertionError(f'R1 allocation checks failed: {result.failures} {result.errors}')


if __name__ == '__main__':
    unittest.main()
