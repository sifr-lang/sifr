"""The selected LSP consumer gets its ordinary CLI before SQL assertions."""
import copy
import json
import unittest
from unittest.mock import Mock, patch

from .area_cargo_setup import prepare_area_graphs, sql_preparation_commands, sql_runner
from .cargo_cli_command import ordinary_cli_build_command
from .cargo_setup import prepare_performance_binaries
from .generated_cargo_setup import prepare_generated_graphs
from .profile_commands import CommandFailed


class SqlCliPreparationChecks(unittest.TestCase):
    def test_selected_consumer_appends_cli_once_after_unchanged_test_commands(self):
        adapter = sql_runner()
        manifest = json.loads(adapter.MANIFEST_PATH.read_text())
        suites = [row['name'] for row in manifest['suites']]
        expected = []
        for suite in adapter.select_suites(manifest, set(suites)):
            for case in suite['cases']:
                command = adapter.COMMANDS[case['command']]
                if command[:2] == ['cargo', 'test']:
                    prepared = [*command[:2], '--no-run', *command[2:]]
                    if prepared not in expected:
                        expected.append(prepared)
        self.assertEqual(len(expected), 30)
        self.assertEqual(sql_preparation_commands(suites), expected + [ordinary_cli_build_command()])
        self.assertEqual(sql_preparation_commands(['incremental-editor'] * 2)[-1], ordinary_cli_build_command())
        self.assertEqual(sql_preparation_commands(['incremental-editor'] * 2).count(ordinary_cli_build_command()), 1)

    def test_other_selections_and_command_name_lookalikes_do_not_prepare_cli(self):
        adapter = sql_runner()
        manifest = json.loads(adapter.MANIFEST_PATH.read_text())
        other = [row['name'] for row in manifest['suites'] if row['name'] != 'incremental-editor']
        for suites in (['build-qualification'], other):
            self.assertNotIn(ordinary_cli_build_command(), sql_preparation_commands(suites))
        # The SQL adapter's existing empty selection means all suites.
        self.assertEqual(sql_preparation_commands([])[-1], ordinary_cli_build_command())
        renamed = copy.deepcopy(manifest)
        for suite in renamed['suites']:
            for case in suite['cases']:
                if case['command'] == 'sql-incremental-editor-tests':
                    case['command'] += '-lookalike'
        adapter.COMMANDS['sql-incremental-editor-tests-lookalike'] = adapter.COMMANDS['sql-incremental-editor-tests']
        with patch('sifr_verify.area_cargo_setup.sql_runner', return_value=adapter), \
             patch('sifr_verify.area_cargo_setup.json.loads', return_value=renamed):
            self.assertNotIn(ordinary_cli_build_command(), sql_preparation_commands(['incremental-editor']))

    def test_same_environment_and_cli_failure_propagate_without_later_preparation(self):
        profile = {'selected_areas': [dict(area='sql_platform', suites=['incremental-editor']),
                                     dict(area='fuzz_property', suites=['fuzz-smoke'])]}
        env = dict(CARGO_TARGET_DIR='/owned/target', CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0',
                   CARGO_PROFILE_DEV_DEBUG='0', CARGO_NET_OFFLINE='true', CARGO_HOME='/owned/cargo',
                   RUSTUP_TOOLCHAIN='1.98.1', RUSTFLAGS='inherited')
        before = env.copy()
        calls = []
        def run(command, *, env):
            calls.append((command, env))
            if command == ordinary_cli_build_command():
                raise CommandFailed(101)
        with self.assertRaises(CommandFailed) as failed:
            prepare_area_graphs(profile, env, run)
        self.assertEqual(failed.exception.returncode, 101)
        self.assertEqual([cmd for cmd, _ in calls], sql_preparation_commands(['incremental-editor']))
        self.assertTrue(all(active is env for _, active in calls))
        self.assertEqual(env, before)

    def test_existing_performance_and_generated_consumers_keep_exact_recipe(self):
        expected = ['cargo', 'build', '--locked', '--offline', '-p', 'sifr']
        env = {'CARGO_NET_OFFLINE': 'true'}
        calls = []
        prepare_performance_binaries({'selected_areas': [dict(area='performance', suites=['smoke'])]},
                                     env, lambda cmd, **kw: calls.append((cmd, kw['env'])))
        self.assertEqual(calls[0], (expected, env))
        quality = Mock()
        with patch('sifr_verify.generated_cargo_setup.os.environ', {}), \
             patch('sifr_verify.generated_cargo_setup.quality_module', return_value=quality), \
             patch('sifr_verify.generated_cargo_setup.run_command', side_effect=CommandFailed(101)) as run:
            with self.assertRaises(CommandFailed):
                prepare_generated_graphs({'selected_areas': []}, 'a' * 40)
        run.assert_called_once_with(expected, env={'CARGO_NET_OFFLINE': 'true'})
        quality.shared_artifact_root.assert_not_called()
        command = ordinary_cli_build_command()
        command.append('--release')
        self.assertEqual(ordinary_cli_build_command(), expected)


if __name__ == '__main__':
    unittest.main()
