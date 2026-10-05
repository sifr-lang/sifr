"""Early SQL scheduling preserves full assertions, preparation and failure custody."""
from __future__ import annotations

import contextlib
import copy
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from .area_cargo_setup import prepare_area_graphs, sql_preparation_commands
from .cargo_setup import prepare_remaining_graphs
from .cargo_cli_command import ordinary_cli_build_command
from .cloud_schedule import Schedule, load_schedule
from .early_sql import run_early_sql
from .process_disk_budget import FLOOR_VARIABLE, PATH_VARIABLE
from .profile_commands import CommandFailed
from .profile_runner import ProfileRunner
from .reports import parse_log
from .sql_partition_checks import write_part
from .resource_admission import Resources


class EarlySqlChecks(unittest.TestCase):
    def test_unselected_unknown_and_nonbuild_do_not_create_early_steps(self):
        for rows in ([], [{'area': 'other', 'suites': ['build-qualification']}],
                     [{'area': 'sql_platform', 'suites': ['contracts']}],
                     [{'area': 'sql_platform', 'suites': ['build-qualification', 'unknown']}]):
            runner = Mock(profile={'selected_areas': rows})
            schedule = Mock()
            outcome = run_early_sql(runner, schedule)
            self.assertFalse(outcome.selected)
            self.assertEqual(outcome.status, 0)
            self.assertIsNone(outcome.assertion_status)
            schedule.assert_not_called()
            self.assertEqual(schedule.mock_calls, [])
            runner.run_area.assert_not_called()

    def test_declared_preparation_and_other_areas_are_unchanged(self):
        profile = ProfileRunner('create-pr', ['--compact-resources']).profile
        selection = next(row for row in profile['selected_areas'] if row['area'] == 'sql_platform')
        manifest = json.loads((Path(__file__).resolve().parents[2]/'areas/sql_platform/manifest.json').read_text())
        selected = [suite for suite in manifest['suites'] if suite['name'] in selection['suites']]
        self.assertEqual((len(selected), sum(len(suite['cases']) for suite in selected)), (19, 66))
        commands = sql_preparation_commands(selection['suites'])
        self.assertEqual(len(commands), 31)
        self.assertEqual(sum(cmd[:3] == ["cargo", "test", "--no-run"] for cmd in commands), 30)
        self.assertEqual(commands[-1], ordinary_cli_build_command())
        calls = []
        env = {'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
        prepare_area_graphs(profile, env, lambda cmd, **kw: calls.append((cmd, kw['env'].copy())))
        later = []
        prepare_area_graphs(profile, env, lambda cmd, **kw: later.append((cmd, kw['env'].copy())),
                            sql_preparation_handled=True)
        self.assertEqual(later, [(cmd, active) for cmd, active in calls if cmd not in commands])
        self.assertEqual([cmd for cmd, _ in calls if cmd in commands], commands)
        self.assertEqual(profile, ProfileRunner('create-pr', ['--compact-resources']).profile)

    def test_remaining_preparation_preserves_all_other_callbacks_and_offline_env(self):
        names = ['prepare_generated_inputs', 'prepare_crate_test_binaries', 'prepare_authoring_test_binaries',
                 'prepare_tooling_test_binaries', 'prepare_performance_binaries', 'prepare_generated_oracle_binary',
                 'prepare_area_graphs', 'prepare_maintained_demo_cache']
        profile = {'selected_areas': []}
        command = Mock()
        for handled in (False, True):
            events = []
            with contextlib.ExitStack() as stack:
                mocks = {}
                for name in names:
                    def record(*args, name=name, **kwargs):
                        events.append((name, args, kwargs))
                    mocks[name] = stack.enter_context(patch('sifr_verify.cargo_setup.' + name, side_effect=record))
                source = stack.enter_context(patch('sifr_verify.cargo_setup.prepare_sysroot_source_binary'))
                package = stack.enter_context(patch('sifr_verify.cargo_setup.prepare_sysroot_package_binary'))
                prepare_remaining_graphs(profile, {'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0'},
                    command, include_sysroot=False, sql_preparation_handled=handled)
            self.assertEqual([name for name, _, _ in events], names)
            self.assertTrue(all(args[0] is profile and args[2] is command for _, args, _ in events))
            self.assertEqual(mocks['prepare_area_graphs'].call_args.kwargs, {'sql_preparation_handled': handled})
            self.assertEqual(mocks['prepare_area_graphs'].call_args.args[1]['CARGO_NET_OFFLINE'], 'true')
            source.assert_not_called()
            package.assert_not_called()

    def route(self, profile_name, *, no_fail_fast=False, failure=None, build=True, build_only=False,
              inherited_floor=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = ['--compact-resources'] + (['--no-fail-fast'] if no_fail_fast else [])
            runner = ProfileRunner(profile_name, args)
            runner.prepare_step_budget = lambda name: None
            runner.env['SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'] = '17'
            runner.env.pop(FLOOR_VARIABLE, None)
            runner.env.pop(PATH_VARIABLE, None)
            if inherited_floor:
                runner.env.update({FLOOR_VARIABLE: str(63*1024**3), PATH_VARIABLE: str(root)})
            original_floor = {key: runner.env.get(key) for key in (FLOOR_VARIABLE, PATH_VARIABLE)}
            if not build:
                next(row for row in runner.profile['selected_areas'] if row['area'] == 'sql_platform')['suites'].remove('build-qualification')
            if build_only:
                next(row for row in runner.profile['selected_areas'] if row['area'] == 'sql_platform')['suites'] = ['build-qualification']
            original = copy.deepcopy(runner.profile)
            commands, events, areas, policies = [], [], [], []
            area_workers = []
            failed_once = False
            disk_free = [64*1024**3]
            def native(command, *, env):
                nonlocal failed_once
                commands.append((list(command), env.copy()))
                events.append('command:' + ' '.join(command))
                if failure == 'sql-cli-preparation' and command == ordinary_cli_build_command():
                    raise CommandFailed(101)
                if (failure == 'sql-preparation' and not failed_once
                        and command[:3] == ['cargo', 'test', '--no-run']):
                    failed_once = True
                    raise CommandFailed(101)
            def area(name, suites, *, result_slug=None):
                areas.append((name, list(suites)))
                area_workers.append((name, list(suites), runner.env['CARGO_BUILD_JOBS']))
                if name == 'sysroot_release':
                    result = root/'target/verification/areas/sysroot-release-cloud-results.json'
                    result.parent.mkdir(parents=True, exist_ok=True)
                    result.write_text('{"passed":true}')
                if name == 'sysroot_release' and failure == 'sysroot-area':
                    raise CommandFailed(5)
                if name == 'sql_platform' and result_slug and 'build-qualification' in suites and failure == 'sql-build-infrastructure':
                    raise CommandFailed(124, 'timeout')
                if name == 'sql_platform' and result_slug:
                    write_part(root, result_slug, profile_name, suites, fail=failure == 'sql-area')
                if name == 'sql_platform' and failure == 'sql-area':
                    raise CommandFailed(7)
            def remaining(profile, env, run, **kwargs):
                self.assertEqual({key: env.get(key) for key in original_floor}, original_floor)
                self.assertEqual(env['SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'], '17')
                events.append('generated-preparation')
                if failure == 'remaining-preparation':
                    raise CommandFailed(9)
                prepare_area_graphs(profile, env, run,
                                    sql_preparation_handled=kwargs['sql_preparation_handled'])
            def schedule(current):
                active = object.__new__(Schedule)
                active.runner, active.root, active.policy = current, root, load_schedule(mode='compact')
                active.owner = active.graph_owner = 'session'
                active.journal = root/'journal'
                active.key = {'inputs': {'source': []}}
                active.index = 0
                def record(name, payload):
                    active.index += 1
                    if failure == 'remaining-admission' and name == 'preparation_sql_platform' and payload.get('state') == 'completed':
                        disk_free[0] = 2*1024**3
                    if payload.get('state') == 'admitted':
                        events.append(name)
                        policies.append((name, payload['requirements']))
                active.record = record
                return active
            output = io.StringIO()
            with contextlib.ExitStack() as stack:
                stack.enter_context(contextlib.redirect_stdout(output))
                stack.enter_context(contextlib.redirect_stderr(output))
                for module in ('compact_profile', 'cloud_schedule'):
                    stack.enter_context(patch(f'sifr_verify.{module}.Schedule', side_effect=schedule))
                    stack.enter_context(patch(f'sifr_verify.{module}.acquire_cargo_dependencies'))
                    stack.enter_context(patch(f'sifr_verify.{module}.run_command', side_effect=native))
                    stack.enter_context(patch(f'sifr_verify.{module}.prepare_remaining_graphs', side_effect=remaining))
                stack.enter_context(patch('sifr_verify.cloud_schedule.REPO_ROOT', root))
                stack.enter_context(patch('sifr_verify.cloud_schedule.inventory', return_value=[]))
                stack.enter_context(patch('sifr_verify.early_sql.inventory', return_value=[]))
                stack.enter_context(patch('sifr_verify.cloud_schedule.discover', side_effect=lambda **kwargs:
                    Resources(5, 4, 64*1024**3, 64*1024**3, disk_free[0], {}, [], {})))
                for method in ('print_header', 'run_guardrail', 'run_toolchain_step',
                               'run_performance_part', 'admit_performance_reference'):
                    stack.enter_context(patch.object(runner, method))
                stack.enter_context(patch.object(runner, 'run_area', side_effect=area))
                stack.enter_context(patch('sifr_verify.profile_runner.combine_performance'))
                stack.enter_context(patch('sifr_verify.profile_runner.performance_result_path', return_value=root/'performance'))
                status = runner.run()
            canonical = root/'target/verification/areas'/f'sql-platform-{profile_name}-results.json'
            payload = json.loads(canonical.read_text()) if canonical.exists() else None
            log = root/'lane.log'
            log.write_text(output.getvalue())
            parsed = parse_log(log)
            self.assertEqual(runner.profile, original)
            self.assertEqual({key: runner.env.get(key) for key in original_floor}, original_floor)
            self.assertEqual(runner.env['SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'], '17')
            self.assertEqual(runner.env['CARGO_BUILD_JOBS'], '1')
            return dict(status=status, functional=runner.functional_exit_status, events=events, commands=commands,
                        areas=areas, steps=parsed['lane_steps'], profile=original, policies=policies, result=payload,
                        area_workers=area_workers)

    def test_both_routes_execute_complete_sql_once_before_unrelated_preparation(self):
        for profile in ('create-pr', 'merge', 'cloud'):
            with self.subTest(profile=profile):
                result = self.route(profile)
                self.assertEqual(result['status'], 0)
                events = result['events']
                self.assertLess(events.index('preparation_dependencies'), events.index('area_sysroot_release'))
                ordered = ['area_sysroot_release', 'sql_build_assertions', 'preparation_sql_platform',
                           'sql_remaining_assertions', 'generated-preparation', 'area_python_interop']
                self.assertEqual([events.index(name) for name in ordered], sorted(events.index(name) for name in ordered))
                sql = next(row for row in result['profile']['selected_areas'] if row['area'] == 'sql_platform')
                expected = [(row['area'], row['suites']) for row in result['profile']['selected_areas']
                            if row['area'] != 'sql_platform' and (profile == 'cloud' or row['area'] != 'performance')]
                expected += [('sql_platform', ['build-qualification']),
                             ('sql_platform', [s for s in sql['suites'] if s != 'build-qualification'])]
                self.assertCountEqual(result['areas'], expected)
                self.assertEqual(result['result']['summary']['total_variants'], 66)
                self.assertEqual(result['result']['summary']['blocking_failures'], 0)
                expected_commands = sql_preparation_commands(sql['suites'])
                self.assertEqual([cmd for cmd, _ in result['commands'] if cmd in expected_commands],
                                 expected_commands)
                cli = 'command:' + ' '.join(ordinary_cli_build_command())
                self.assertLess(events.index('preparation_sql_platform'), events.index(cli))
                self.assertLess(events.index(cli), events.index('sql_remaining_assertions'))
                self.assertEqual([step['status'] for step in result['steps'] if step['name'] == 'area_sql_platform'], ['pass'])
                self.assertTrue(all(env['CARGO_NET_OFFLINE'] == 'true' and env['CARGO_INCREMENTAL'] == '0'
                                    for cmd, env in result['commands'] if cmd in expected_commands))
                self.assertTrue(all(env['CARGO_BUILD_JOBS'] == '1' for _, env in result['commands']))
                self.assertEqual([(name, suites) for name, suites, jobs in result['area_workers'] if jobs != '1'],
                                 [('sql_platform', ['build-qualification'])])
                self.assertTrue(all(jobs == '2' for name, suites, jobs in result['area_workers']
                                    if name == 'sql_platform' and suites == ['build-qualification']))

    def test_stricter_caller_floor_is_restored_before_independent_work(self):
        for profile in ('create-pr', 'cloud'):
            for failure in (None, 'sql-preparation', 'sql-area'):
                with self.subTest(profile=profile, failure=failure):
                    self.route(profile, no_fail_fast=True, failure=failure, inherited_floor=True)

    def test_failed_sysroot_does_not_move_cloud_sql_ahead_of_its_prerequisite(self):
        result = self.route('cloud', no_fail_fast=True, failure='sysroot-area')
        self.assertEqual(result['status'], 5)
        self.assertNotIn('preparation_sql_platform', result['events'])
        self.assertLess(result['events'].index('generated-preparation'), result['events'].index('area_sql_platform'))
        self.assertEqual([step['status'] for step in result['steps'] if step['name'] == 'area_sql_platform'], ['pass'])

    def test_build_infrastructure_failure_controls_continuation_without_false_case_results(self):
        for profile in ('create-pr', 'cloud'):
            for no_fail_fast in (False, True):
                result = self.route(profile, no_fail_fast=no_fail_fast, failure='sql-build-infrastructure')
                self.assertEqual(result['status'], 124)
                self.assertEqual('preparation_sql_platform' in result['events'], no_fail_fast)
                self.assertEqual(result['result']['summary']['total_variants'], 66)
                self.assertEqual(result['result']['summary']['blocking_failures'], 1 if no_fail_fast else 66)
                build = next(s for s in result['result']['suites'] if s['name']=='build-qualification')
                self.assertEqual(build['cases'][0]['variants'][0]['status'], 'blocked')
                self.assertIn('timeout', build['cases'][0]['variants'][0]['reason'])

    def test_remainder_budget_refusal_blocks_exact_unexecuted_cases_with_cause(self):
        for profile in ('create-pr', 'cloud'):
            result = self.route(profile, failure='remaining-admission')
            self.assertEqual(result['status'], 2)
            self.assertEqual(result['result']['summary']['total_variants'], 66)
            self.assertEqual(result['result']['summary']['blocking_failures'], 65)
            for suite in result['result']['suites']:
                for case in suite['cases']:
                    variant = case['variants'][0]
                    if suite['name'] == 'build-qualification':
                        self.assertEqual(variant['status'], 'pass')
                    else:
                        self.assertEqual(variant['status'], 'blocked')
                        self.assertIn('enospc', variant['reason'])
            self.assertEqual([step['status'] for step in result['steps'] if step['name']=='area_sql_platform'], ['fail'])

    def test_nonbuild_selection_keeps_later_sql_preparation_and_assertion(self):
        for profile in ('create-pr', 'cloud'):
            result = self.route(profile, build=False)
            self.assertEqual(result['status'], 0)
            self.assertNotIn('preparation_sql_platform', result['events'])
            self.assertLess(result['events'].index('generated-preparation'), result['events'].index('area_sql_platform'))
            self.assertEqual([step['status'] for step in result['steps'] if step['name'] == 'area_sql_platform'], ['pass'])
            self.assertEqual(len([cmd for cmd, _ in result['commands'] if cmd[:3] == ['cargo', 'test', '--no-run']]), 30)
            self.assertTrue(all(jobs == '1' for _, _, jobs in result['area_workers']))
            self.assertTrue(all(env['CARGO_BUILD_JOBS'] == '1' for _, env in result['commands']))

    def test_build_only_selection_uses_named_workers_without_adding_preparation(self):
        for profile in ('create-pr', 'cloud'):
            result = self.route(profile, build_only=True)
            self.assertEqual(result['status'], 0)
            self.assertEqual(result['result']['summary']['total_variants'], 1)
            self.assertNotIn('preparation_sql_platform', result['events'])
            self.assertFalse(any(cmd[:3] == ['cargo', 'test', '--no-run'] for cmd, _ in result['commands']))
            self.assertEqual([row for row in result['area_workers'] if row[0] == 'sql_platform'],
                             [('sql_platform', ['build-qualification'], '2')])

    def test_failures_preserve_exactly_one_status_and_no_fail_fast_independence(self):
        for profile in ('create-pr', 'cloud'):
            for no_fail_fast in (False, True):
                for failure, code, sql_status in (('sql-preparation', 101, 'fail'),
                                                  ('sql-cli-preparation', 101, 'fail'),
                                                  ('sql-area', 7, 'fail'),
                                                  ('remaining-preparation', 9, 'pass')):
                    with self.subTest(profile=profile, no_fail_fast=no_fail_fast, failure=failure):
                        result = self.route(profile, no_fail_fast=no_fail_fast, failure=failure)
                        self.assertEqual(result['status'], code)
                        self.assertEqual(result['functional'], code)
                        self.assertEqual([step['status'] for step in result['steps'] if step['name'] == 'area_sql_platform'], [sql_status])
                        if failure == 'sql-preparation':
                            self.assertNotIn(ordinary_cli_build_command(), [cmd for cmd, _ in result['commands']])
                        if failure == 'sql-cli-preparation':
                            self.assertEqual(sum(cmd[:3] == ['cargo', 'test', '--no-run']
                                                 for cmd, _ in result['commands']), 30)
                            self.assertEqual(result['result']['summary']['blocking_failures'], 65)
                        count = 1 if failure in ('sql-preparation', 'sql-cli-preparation') else 2
                        self.assertEqual(sum(name == 'sql_platform' for name, _ in result['areas']), count)
                        if failure != 'remaining-preparation':
                            self.assertEqual('generated-preparation' in result['events'], no_fail_fast)
                            self.assertEqual(any(name == 'python_interop' for name, _ in result['areas']), no_fail_fast)
                        if failure == 'remaining-preparation':
                            self.assertFalse(any(name == 'python_interop' for name, _ in result['areas']))
                            self.assertEqual([step['status'] for step in result['steps'] if step['name'] == 'cargo_cache_setup'], ['fail'])


if __name__ == '__main__':
    unittest.main()
