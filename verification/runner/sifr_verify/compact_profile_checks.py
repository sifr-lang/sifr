"""Compact execution preserves assertions and keeps cleanup session scoped."""
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import uuid
from unittest.mock import patch

from .compact_profile import prepare_compact
from .cloud_schedule import load_schedule
from .graph_retirement import GraphLease, require_owned_path
from .profile_runner import ProfileRunner, ProfileRunnerError
from .resource_admission import ResourceError
from .sysroot_preparation import producer


class CompactChecks(unittest.TestCase):
    def test_explicit_configuration_preserves_every_profile_selection(self):
        with patch.dict(os.environ, {}, clear=True):
            for name in ('create-pr', 'merge', 'nightly', 'cloud'):
                standard = ProfileRunner(name, [])
                compact = ProfileRunner(name, ['--compact-resources', '--run-jobs', '1'])
                self.assertEqual(compact.profile, standard.profile)
                self.assertEqual(compact.forward_args, ['--run-jobs', '1'])
                self.assertEqual(compact.env['CARGO_PROFILE_DEV_DEBUG'], '0')
                self.assertEqual(compact.env['CARGO_INCREMENTAL'], '0')
            with self.assertRaises(ProfileRunnerError):
                ProfileRunner('release', ['--compact-resources'])
        for environment in ({'SIFR_VERIFY_RESOURCE_POLICY':'compact'},
                            {'SIFR_VERIFY_RESOURCE_POLICY':'cloud'},
                            {'SIFR_VERIFY_RESOURCE_POLICY':''},
                            {'SIFR_VERIFY_SYSROOT_GRAPH_SESSION':str(uuid.uuid4())}):
            with patch.dict(os.environ, environment, clear=True):
                with self.assertRaises(ProfileRunnerError):
                    ProfileRunner('cloud', [])
                if environment.get('SIFR_VERIFY_RESOURCE_POLICY') == 'compact':
                    self.assertTrue(ProfileRunner('cloud', ['--compact-resources']).compact_resources)
                else:
                    with self.assertRaises(ProfileRunnerError):
                        ProfileRunner('cloud', ['--compact-resources'])
        with patch.dict(os.environ, {'CARGO_PROFILE_DEV_DEBUG': '2'}):
            with self.assertRaises(ProfileRunnerError):
                ProfileRunner('merge', ['--compact-resources'])
        self.assertEqual(load_schedule()['stages']['sysroot-source']['disk_reserve_bytes'], 8*1024**3)
        self.assertEqual(load_schedule(mode='compact')['stages']['sysroot-source']['disk_reserve_bytes'], 2*1024**3)

    def test_producer_and_consumer_share_private_paths_and_only_owned_graph_retires(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first, second = str(uuid.uuid4()), str(uuid.uuid4())
            source, package = producer('source_build'), producer('package_build')
            env = {'SIFR_VERIFY_SYSROOT_GRAPH_SESSION': first}
            _, source_env, binary = source.source_build_configuration(root, env)
            _, package_env = package.package_build_configuration(root, env, 'x86_64-unknown-linux-gnu', root/'archive')
            self.assertEqual(Path(source_env['CARGO_TARGET_DIR']), root/'target/sysroot_release/graphs'/first/'source-cargo-target')
            self.assertEqual(Path(package_env['CARGO_TARGET_DIR']), root/'target/sysroot_release/graphs'/first/'cargo-target')
            shared = root/'target/debug/unrelated'
            shared.parent.mkdir(parents=True)
            shared.write_bytes(b'shared-cache')
            leases = [GraphLease(root, f'target/sysroot_release/graphs/{owner}/source-cargo-target', owner).acquire({'compile'})
                      for owner in (first, second)]
            for lease in leases:
                self.addCleanup(lease.close)
                item = lease.path/'debug/sifr'
                item.parent.mkdir()
                item.write_bytes(b'compiler')
            self.assertNotEqual(leases[0].marker, leases[1].marker)
            leases[0].passed_consumer('compile')
            with patch('sifr_verify.graph_retirement.active_builds', return_value=[]):
                record = leases[0].retire(retained=root/'retained', protected=[binary], env={},
                    command_runner=lambda argv, env: shutil.rmtree(Path(argv[-1])))
            self.assertTrue(record['retired'])
            self.assertEqual(shared.read_bytes(), b'shared-cache')
            self.assertTrue((leases[1].path/'debug/sifr').exists())
            for invalid in ('target/debug', f'target/sysroot_release/graphs/{first}/../../debug',
                            'target/sysroot_release/graphs/arbitrary/source-cargo-target'):
                with self.assertRaises(ResourceError):
                    require_owned_path(root, invalid)
            env['SIFR_VERIFY_SYSROOT_GRAPH_SESSION'] = '../escape'
            with self.assertRaises(ValueError):
                source.source_build_configuration(root, env)

    def test_compact_order_runs_exact_selected_sysroot_once_and_blocks_on_failure(self):
        for name in ('create-pr', 'merge'):
            for fail in (False, True):
                events = []
                runner = ProfileRunner(name, ['--compact-resources'])
                selected = next(item for item in runner.profile['selected_areas'] if item['area']=='sysroot_release')
                class FakeSchedule:
                    owner = graph_owner = str(uuid.uuid4())
                    journal = Path('/test/journal')
                    def __init__(self, runner): pass
                    def record(self, *args, **kwargs): pass
                    def prepare_command(self, *args, **kwargs): pass
                    def step(self, step, callback, **kwargs):
                        events.append(step)
                        if fail and step == 'preparation_sysroot_source':
                            return 2
                        callback()
                        return 0
                with patch('sifr_verify.compact_profile.Schedule', FakeSchedule), \
                     patch('sifr_verify.compact_profile.acquire_cargo_dependencies'), \
                     patch('sifr_verify.compact_profile.run_command') as command, \
                     patch('sifr_verify.compact_profile.prepare_remaining_graphs') as remaining, \
                     patch.object(runner, 'run_area') as area:
                    status = prepare_compact(runner)
                if fail:
                    self.assertEqual(status, 2)
                    area.assert_not_called()
                    remaining.assert_not_called()
                    self.assertFalse(getattr(runner, 'compact_completed_areas', set()))
                    continue
                self.assertEqual(status, 0)
                area.assert_called_once_with('sysroot_release', selected['suites'])
                self.assertFalse(remaining.call_args.kwargs['include_sysroot'])
                self.assertLess(events.index('area_sysroot_release'), events.index('cargo_cache_setup'))
                metadata = [call.args[0][-1] for call in command.call_args_list if '--metadata-suite' in call.args[0]]
                self.assertEqual(set(metadata), set(selected['suites']) & {'metadata-structural','metadata-corpus'})

    def test_full_runner_keeps_remaining_assertions_and_quantitative_measurement_last(self):
        runner = ProfileRunner('create-pr', ['--compact-resources'])
        events = []
        def prepare(current):
            selection = next(item for item in current.profile['selected_areas'] if item['area']=='sysroot_release')
            current.run_area('sysroot_release', selection['suites'])
            current.compact_completed_areas = {'sysroot_release'}
            return 0
        def execute(name, callback, **kwargs):
            events.append(name)
            callback()
            return 0
        with patch('sifr_verify.compact_profile.prepare_compact', side_effect=prepare), \
             patch.object(runner, 'execute_step', side_effect=execute), \
             patch.object(runner, 'run_guardrail') as guards, \
             patch.object(runner, 'run_area') as areas, \
             patch.object(runner, 'run_toolchain_step') as tools, \
             patch.object(runner, 'run_performance_part'), \
             patch.object(runner, 'admit_performance_reference'), \
             patch('sifr_verify.profile_runner.combine_performance'):
            self.assertEqual(runner.run(), 0)
        expected = [(item['area'], item['suites']) for item in runner.profile['selected_areas'] if item['area']!='performance']
        self.assertCountEqual([call.args for call in areas.call_args_list], expected)
        self.assertEqual(guards.call_count, len(runner.profile['guardrail_steps']))
        self.assertEqual(tools.call_count, len(runner.profile['toolchain_steps']))
        self.assertEqual(events[-1], 'area_performance')
        self.assertEqual(len(events), len(set(events)))

    def test_foundation_isolates_mock_profiles_and_restores_outer_scheduler_environment(self):
        from . import selftest
        environment = {'SIFR_VERIFY_RESOURCE_POLICY':'compact',
                       'SIFR_VERIFY_SYSROOT_GRAPH_SESSION':str(uuid.uuid4())}
        def foundation():
            self.assertNotIn('SIFR_VERIFY_RESOURCE_POLICY', os.environ)
            self.assertNotIn('SIFR_VERIFY_SYSROOT_GRAPH_SESSION', os.environ)
            return ['passed']
        with patch.dict(os.environ, environment, clear=True):
            with patch.object(selftest, '_run_all', side_effect=foundation):
                self.assertEqual(selftest.run_all(), ['passed'])
            self.assertEqual(dict(os.environ), environment)
            with patch.object(selftest, '_run_all', side_effect=RuntimeError('failure')):
                with self.assertRaises(RuntimeError): selftest.run_all()
            self.assertEqual(dict(os.environ), environment)

    def test_metadata_preparation_does_not_compile_unselected_corpus(self):
        package = producer('package_build')
        commands = []
        with patch.object(package, 'prepare_source_snapshot') as snapshot:
            package.prepare_metadata(Path('/test'), {}, lambda argv, **kwargs: commands.append(argv), suite='metadata-structural')
        snapshot.assert_not_called()
        self.assertEqual(len(commands), 1)
        self.assertEqual(commands[0][-1], 'metadata_structural_')
        with self.assertRaises(ValueError):
            package.prepare_metadata(Path('/test'), {}, suite='unknown')

    def test_only_structural_selection_uses_smaller_assertion_allocation(self):
        from .resource_admission import Resources, admit
        for suites in (['metadata-structural'], ['metadata-corpus'],
                       ['metadata-structural','boundary-equivalence'], ['future-suite']):
            runner=ProfileRunner('create-pr',['--compact-resources'])
            selection=next(row for row in runner.profile['selected_areas'] if row['area']=='sysroot_release')
            selection['suites']=suites
            allocations={}
            class FakeSchedule:
                owner=graph_owner=str(uuid.uuid4())
                journal=Path('/test/journal')
                def __init__(self,runner): pass
                def record(self,*args,**kwargs): pass
                def prepare_command(self,*args,**kwargs): pass
                def step(self,name,callback,**kwargs):
                    allocations[name]=kwargs['allocation'];callback();return 0
            with patch('sifr_verify.compact_profile.Schedule',FakeSchedule), \
                 patch('sifr_verify.compact_profile.acquire_cargo_dependencies'), \
                 patch('sifr_verify.compact_profile.run_command'), \
                 patch('sifr_verify.compact_profile.prepare_remaining_graphs'), \
                 patch.object(runner,'run_area') as area:
                self.assertEqual(prepare_compact(runner),0)
            area.assert_called_once_with('sysroot_release',suites)
            expected='sysroot-structural-assertions' if suites==['metadata-structural'] else 'sysroot-assertions'
            self.assertEqual(allocations['area_sysroot_release'],expected)
        stages=load_schedule(mode='compact')['stages']
        # Capacity of the failed actual compact run. Keep the shared-memory
        # accounting and reserve; the selected workload estimate changes.
        capacity=Resources(5,4,16*1024**3,11520053248,6*1024**3,{},[],{},True)
        admit(capacity,stages['sysroot-structural-assertions'])
        with self.assertRaises(ResourceError): admit(capacity,stages['sysroot-assertions'])


def policy_checks():
    result = unittest.TextTestRunner().run(unittest.defaultTestLoader.loadTestsFromTestCase(CompactChecks))
    if not result.wasSuccessful():
        raise AssertionError('compact resource execution controls failed')


if __name__ == '__main__':
    unittest.main()
