"""Current-invocation SQL aggregation and unchanged cumulative safety controls."""
import copy
import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

from .area_cargo_setup import sql_preparation_commands, sql_runner
from .early_sql import AssertionBudget, DURATION, STEP_DURATION
from .process_execution import SAFETY_DEADLINE_ENV, deadline_environment
from .profile_commands import CommandFailed, run_command
from .sql_partition_results import SqlInvocation


def part_payload(names, *, fail=False):
    adapter = sql_runner()
    suites = adapter.select_suites(json.loads(adapter.MANIFEST_PATH.read_text()), set(names))
    rows = []
    for suite in suites:
        cases = []
        for case in suite['cases']:
            cmd = case['command']
            cases.append(dict(id=case['id'], entry=case['entry'], command=cmd, variants=[dict(
                label=cmd, argv=adapter.COMMANDS[cmd], status='fail' if fail else 'pass',
                mismatches=['unexpected-exit'] if fail else [], expected_exit_code=case['expect_exit_code'],
                actual_exit_code=1 if fail else case['expect_exit_code'], duration_ms=1)]))
        count = len(cases)
        rows.append(dict(name=suite['name'], owner='compiler/sql-platform', blocking=True, runner='sql_platform',
                         cases=cases, failed_cases=count if fail else 0, total_variants=count,
                         total_failures=count if fail else 0))
    count = sum(row['total_variants'] for row in rows)
    failures = count if fail else 0
    return dict(schema_version=1, area='sql_platform', bless=False,
                manifest='verification/areas/sql_platform/manifest.json', suites=rows,
                summary=dict(total_variants=count, total_failures=failures,
                             blocking_failures=failures, non_blocking_failures=0))


def write_part(root, slug, profile, names, *, fail=False):
    path = root/'target/verification/areas'/f'{slug}-{profile}-results.json'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(part_payload(names, fail=fail)))
    return path


class PartitionChecks(unittest.TestCase):
    def test_build_suite_has_no_test_preparation_and_partition_preserves_every_command(self):
        from .profiles import load_profile
        for profile in ('create-pr', 'merge', 'cloud'):
            suites = next(row['suites'] for row in load_profile(profile)['selected_areas'] if row['area']=='sql_platform')
            self.assertEqual(sql_preparation_commands(['build-qualification']), [])
            rest = [suite for suite in suites if suite != 'build-qualification']
            self.assertEqual(sql_preparation_commands(rest), sql_preparation_commands(suites))
            self.assertEqual(len(sql_preparation_commands(rest)), 31)
            self.assertEqual(sum(cmd[:3] == ["cargo", "test", "--no-run"]
                                 for cmd in sql_preparation_commands(rest)), 30)

    def test_complete_result_has_canonical_order_counts_and_fresh_part_paths(self):
        from .profiles import load_profile
        suites = next(row['suites'] for row in load_profile('create-pr')['selected_areas'] if row['area']=='sql_platform')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = SqlInvocation('create-pr', suites, root=root)
            other = SqlInvocation('create-pr', suites, root=root)
            self.assertNotEqual(first.part_path('build'), other.part_path('build'))
            for label, selected in (('build', ['build-qualification']),
                                    ('remaining', [s for s in suites if s != 'build-qualification'])):
                write_part(root, first.part_slug(label), 'create-pr', selected)
                self.assertEqual(first.accept(label, selected, 0), 0)
            payload = first.finish(missing_reason='not run')
            expected = part_payload(suites)
            self.assertEqual(payload, expected)
            self.assertEqual(payload['summary']['total_variants'], 66)
            self.assertEqual(other.accept('build', ['build-qualification'], 0), 2)

    def test_missing_duplicate_corrupt_and_false_process_success_never_qualify(self):
        mutations = [lambda p:p['suites'].append(copy.deepcopy(p['suites'][0])),
                     lambda p:p['suites'][0]['cases'].clear(),
                     lambda p:p['suites'][0]['cases'][0].update(id='unknown'),
                     lambda p:p['suites'][0]['cases'][0]['variants'][0].update(argv=['wrong']),
                     lambda p:p['suites'][0]['cases'][0]['variants'].append({}),
                     lambda p:p['summary'].update(total_variants=True),
                     lambda p:p.update(area='other'), lambda p:p.update(bless=True)]
        for mutation in mutations:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                item = SqlInvocation('test', ['build-qualification'], root=Path(directory))
                path = write_part(item.root, item.part_slug('build'), 'test', ['build-qualification'])
                payload = json.loads(path.read_text()); mutation(payload); path.write_text(json.dumps(payload))
                self.assertEqual(item.accept('build', ['build-qualification'], 0), 2)
                result = item.finish(missing_reason='not run')
                self.assertEqual(result['summary']['blocking_failures'], 1)
                self.assertEqual(result['suites'][0]['cases'][0]['variants'][0]['status'], 'blocked')
        for code in (0, 124):
            with tempfile.TemporaryDirectory() as directory:
                item = SqlInvocation('test', ['build-qualification'], root=Path(directory))
                write_part(item.root, item.part_slug('build'), 'test', ['build-qualification'])
                self.assertEqual(item.accept('build', ['build-qualification'], code), code)
                self.assertEqual(item.accept('build', ['build-qualification'], 0), 2)
                if code:
                    self.assertEqual(item.finish(missing_reason='timeout')['summary']['blocking_failures'], 1)

    def test_actual_failure_retains_failed_case_and_unexecuted_selection_is_blocked(self):
        with tempfile.TemporaryDirectory() as directory:
            item = SqlInvocation('test', ['contracts', 'build-qualification'], root=Path(directory))
            write_part(item.root, item.part_slug('build'), 'test', ['build-qualification'], fail=True)
            self.assertEqual(item.accept('build', ['build-qualification'], 1), 1)
            result = item.finish(missing_reason='fail-fast after build')
            self.assertEqual(result['suites'][0]['name'], 'contracts')
            self.assertEqual(result['suites'][0]['cases'][0]['variants'][0]['status'], 'blocked')
            self.assertEqual(result['suites'][1]['cases'][0]['variants'][0]['status'], 'fail')
            self.assertEqual(result['summary']['blocking_failures'], result['summary']['total_variants'])

    def test_run_command_cannot_prefer_a_longer_step_duration_over_remaining_time(self):
        for duration, step in (('2400', '9000'), ('2400', '17'), ('9', '17')):
            original = {DURATION:duration, STEP_DURATION:step, SAFETY_DEADLINE_ENV:'1005'}
            runner = SimpleNamespace(env=original.copy())
            def execute(argv, **kwargs):
                selected.append(float(kwargs['deadline_seconds']))
                with patch('sifr_verify.process_execution.time.monotonic', return_value=1000):
                    self.assertLessEqual(deadline_environment(kwargs['env'], kwargs['deadline_seconds'])[1], 1005)
                return SimpleNamespace(returncode=0)
            schedule = SimpleNamespace(policy={'assertion_command_deadline_seconds':2400},
                                       step=lambda name, callback, **kwargs:(callback(),0)[1])
            budget = AssertionBudget(runner, schedule)
            selected = []
            with patch('sifr_verify.profile_commands.execute', side_effect=execute), \
                 patch('sifr_verify.early_sql.time.monotonic', side_effect=[10,12,20,22]):
                budget.run(schedule, 'build', lambda:run_command(['fixture'], env=runner.env), allocation='sql-build-qualification')
                budget.run(schedule, 'remaining', lambda:run_command(['fixture'], env=runner.env), allocation='remaining-assertions')
            limit = min(float(duration), float(step), 2400)
            self.assertEqual(selected, [limit, limit-2])
            self.assertEqual(runner.env, original)

    def test_combined_duration_excludes_preparation_and_keeps_absolute_deadline(self):
        for inherited in (None, '17'):
            env = {SAFETY_DEADLINE_ENV: '5000'}
            if inherited:
                env[DURATION] = inherited
            runner = SimpleNamespace(env=env)
            durations, deadlines = [], []
            def step(name, callback, **kwargs):
                durations.append(float(env[DURATION])); callback(); return 0
            schedule = SimpleNamespace(policy={'assertion_command_deadline_seconds':2400}, step=step)
            budget = AssertionBudget(runner, schedule)
            clock = [100.0]
            def assertion():
                with patch('sifr_verify.process_execution.time.monotonic', return_value=4995):
                    deadlines.append(deadline_environment(env, env[DURATION])[1])
                clock[0] += 5
            with patch('sifr_verify.early_sql.time.monotonic', side_effect=lambda:clock[0]):
                budget.run(schedule, 'first', assertion, allocation='sql-build-qualification')
                clock[0] += 10000  # Deliberately much longer than assertion limit: preparation.
                budget.run(schedule, 'rest', assertion, allocation='remaining-assertions')
            limit = 17 if inherited else 2400
            self.assertEqual(durations, [limit, limit-5])
            self.assertEqual(budget.spent, 10)
            self.assertEqual(deadlines, [5000, 5000])
            self.assertEqual(env.get(DURATION), inherited)
            self.assertEqual(env[SAFETY_DEADLINE_ENV], '5000')
            budget.spent = limit
            child = Mock()
            self.assertEqual(budget.run(schedule, 'exhausted', child, allocation='remaining-assertions'), 124)
            self.assertIn('timeout', budget.failure_detail)
            child.assert_not_called()
            self.assertEqual(env.get(DURATION), inherited)


if __name__ == '__main__':
    unittest.main()
