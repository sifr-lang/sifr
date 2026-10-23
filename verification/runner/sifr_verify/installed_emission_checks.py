"""Installed corpus ownership, completeness and byte-level rejection controls."""
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import tempfile
import unittest
from unittest.mock import patch

from .errors import VerificationError
from .paths import REPO_ROOT
from .process_execution import Outcome

spec = importlib.util.spec_from_file_location(
    "installed_emission", REPO_ROOT / "verification/areas/sysroot_release/installed_emission.py")
emission = importlib.util.module_from_spec(spec)
spec.loader.exec_module(emission)


class Qualification:
    def __init__(self, root):
        self.workspace = root / 'work'; self.workspace.mkdir()
        self.corpus = root / 'corpus'; self.corpus.mkdir()
        self.rows = [{'id': 'initial-integrity'}]
        self.report = {'rows': self.rows}
        self.env = {'SIFR_VERIFY_PROCESS_SLOTS': '4', 'RAYON_NUM_THREADS': '4', 'CARGO_BUILD_JOBS': '4'}
        self.saved = root / 'report.json'
    def record(self, name, command, result):
        self.rows.append({'id': name, 'command': command, 'returncode': result.returncode})
        self.save()
    def save(self):
        self.saved.write_text(json.dumps(self.report))


class InstalledEmissionTests(unittest.TestCase):
    def fixture(self, root):
        owner = Qualification(root)
        expected = b'fn main() {}\n'
        rows = []
        for name in ['alpha', 'beta', 'gamma']:
            source = b'def main() -> None:\n    pass\n'
            (owner.corpus / (name + '.sifr')).write_bytes(source)
            (owner.corpus / (name + '.source.rs')).write_bytes(expected)
            rows.append({'fixture': name, 'source_sha256': hashlib.sha256(source).hexdigest(),
                         'final_newline': True, 'preamble_start': None, 'preamble_end': None,
                         'generated_source_maps': [{'path': name + '.rs', 'source_bytes': len(expected),
                                                   'source_sha256': hashlib.sha256(expected).hexdigest()}]})
        return owner, rows, expected

    def outcome(self, expected, **changes):
        values = dict(returncode=0, cause='exit', stdout=expected, stderr=b'', truncated=False, elapsed_seconds=.1)
        values.update(changes)
        return Outcome(**values)

    def test_all_commands_execute_and_report_order_is_canonical(self):
        with tempfile.TemporaryDirectory() as directory:
            owner, rows, expected = self.fixture(Path(directory)); original_env = dict(owner.env)
            def execute(commands, **kwargs):
                self.assertEqual(len(commands), 3)
                self.assertEqual(kwargs['max_parallel'], 4)
                self.assertEqual(kwargs['env']['RAYON_NUM_THREADS'], '1')
                self.assertEqual(kwargs['env']['CARGO_BUILD_JOBS'], '1')
                for index in [2, 0, 1]:
                    self.assertEqual(commands[index], ['/installed/bin/sifr', 'emit',
                                     str(owner.workspace / (rows[index]['fixture'] + '.sifr'))])
                    kwargs['completed'](index, self.outcome(expected))
            with patch.object(emission, 'execute_batch', side_effect=execute):
                emission.verify_public_emission(owner, Path('/installed/bin/sifr'), rows)
            self.assertEqual(owner.env, original_env)
            self.assertEqual([row['id'] for row in owner.rows],
                             ['initial-integrity', 'emit-alpha', 'emit-beta', 'emit-gamma'])
            self.assertEqual(owner.report['public_emit_execution']['observed'], 3)
            self.assertEqual(owner.report['public_emit_execution']['failures'], [])

    def test_byte_and_process_failures_do_not_hide_independent_cases(self):
        for changes in [{'stdout': b'wrong'}, {'returncode': 1}, {'cause': 'safety_deadline'}, {'truncated': True}]:
            with self.subTest(changes=changes), tempfile.TemporaryDirectory() as directory:
                owner, rows, expected = self.fixture(Path(directory)); calls = []
                def execute(commands, **kwargs):
                    for index in range(3):
                        calls.append(index)
                        kwargs['completed'](index, self.outcome(expected, **(changes if index == 1 else {})))
                with patch.object(emission, 'execute_batch', side_effect=execute):
                    with self.assertRaisesRegex(AssertionError, 'public emit qualification failed'):
                        emission.verify_public_emission(owner, Path('/installed/bin/sifr'), rows)
                self.assertEqual(calls, [0, 1, 2])
                self.assertEqual(owner.report['public_emit_execution']['failures'][0][0], 'beta')

    def test_every_map_newline_and_preamble_assertion_remains_authoritative(self):
        mutations = [lambda row: row.update(final_newline=False),
                     lambda row: row.update(preamble_start=0),
                     lambda row: row.update(preamble_end=0),
                     lambda row: row['generated_source_maps'][0].update(source_bytes=1),
                     lambda row: row['generated_source_maps'][0].update(source_sha256='0'*64),
                     lambda row: row['generated_source_maps'][0].update(path='case#stdlib-preamble')]
        for mutate in mutations:
            with tempfile.TemporaryDirectory() as directory:
                owner, rows, expected = self.fixture(Path(directory)); mutate(rows[0])
                with self.assertRaises(AssertionError):
                    emission.assert_emission(rows[0], expected, expected)

    def test_missing_duplicate_and_unknown_outcomes_reject(self):
        for sequence, message in [([0, 1], 'missing'), ([0, 0], 'duplicate'), ([3], 'unknown')]:
            with self.subTest(sequence=sequence), tempfile.TemporaryDirectory() as directory:
                owner, rows, expected = self.fixture(Path(directory))
                def execute(commands, **kwargs):
                    for index in sequence: kwargs['completed'](index, self.outcome(expected))
                with patch.object(emission, 'execute_batch', side_effect=execute):
                    with self.assertRaisesRegex(AssertionError, message):
                        emission.verify_public_emission(owner, Path('/installed/bin/sifr'), rows)

    def test_profile_owns_batch_slots_instead_of_inheriting_an_unbounded_value(self):
        from .profile_runner import ProfileRunner
        with patch.dict(os.environ, {'SIFR_VERIFY_PROCESS_SLOTS': '99999'}):
            runner = ProfileRunner('merge', [], correctness_only=True)
        self.assertEqual(runner.env['SIFR_VERIFY_PROCESS_SLOTS'], str(runner.build_allocation.slot_limit))

    def test_empty_duplicate_source_drift_and_invalid_budget_reject_before_execution(self):
        for mode in ['empty', 'duplicate', 'source', 'budget']:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                owner, rows, _ = self.fixture(Path(directory))
                if mode == 'empty': rows = []
                if mode == 'duplicate': rows = rows + [rows[0]]
                if mode == 'source': rows[0]['source_sha256'] = '0' * 64
                if mode == 'budget': owner.env['SIFR_VERIFY_PROCESS_SLOTS'] = '0'
                with patch.object(emission, 'execute_batch', side_effect=AssertionError('spawned')) as execute:
                    with self.assertRaises((AssertionError, ValueError, VerificationError)):
                        emission.verify_public_emission(owner, Path('/installed/bin/sifr'), rows)
                    execute.assert_not_called()


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(InstalledEmissionTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError('installed emission failures: ' + repr(result.failures + result.errors))
