"""Pure actual-algorithm adapters and hostile retained-receipt controls; no builds."""
from collections import namedtuple
from contextlib import ExitStack, nullcontext
import copy
import errno
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

import native_teardown_diagnostic as diagnostic
import native_teardown_process as fixture
from native_teardown_checker import check_case
from native_runtime_storage_tests import TOPOLOGY
from sifr_verify import process_execution as production
from sifr_verify.resource_admission import Resources


class Controls(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.output = Path(self.tmp.name)
        (self.output/'temporary').mkdir(); (self.output/'evidence').mkdir()
        self.pid = 900001
        self.child = 900002
        self.driver = {'pid': os.getpid(), 'uid': os.geteuid(), 'ruid': os.getuid()}
        self.now = 100.

    def clock(self):
        self.now += .001
        return self.now

    def raw(self, state=None, child=False):
        def row(pid, parent, group, status):
            return f'{pid} {parent} {group} {self.driver["uid"]} {self.driver["ruid"]} {status} Mon Oct  5 00:00:00 2026\n'
        result = row(self.driver['pid'], 1, self.driver['pid'], 'S')
        if state is not None:
            result += row(self.pid, self.driver['pid'], self.pid, state)
        if child:
            result += row(self.child, 1, self.pid, 'S')
        return result

    def simulate(self, kind, signal_error=None):
        owner = self
        state = {'leader': 'S' if kind == 'deadline-term' else 'Z', 'child': kind == 'resistant-pipe'}
        class Proc:
            pid = owner.pid
            returncode = None
            def wait(self, *args, **kwargs):
                self.returncode = -signal.SIGTERM if kind == 'deadline-term' else (0 if kind == 'resistant-pipe' else 7)
                state['leader'] = None
                return self.returncode
        proc = Proc()
        def ready(argv):
            fixture.publish_ready(argv[-1], {'kind': kind, 'leader': self.pid,
                'children': [self.child] if kind == 'resistant-pipe' else []})
        def launch(argv, **kwargs):
            ready(argv)
            return proc
        def snapshot(deadline):
            return production._darwin_process_rows(self.raw(state['leader'], state['child']))
        def killpg(pid, number):
            self.assertEqual(pid, self.pid)
            if number == signal.SIGTERM:
                state['leader'] = 'Z'
            if number == signal.SIGKILL:
                state['child'] = False
            if signal_error is not None:
                raise OSError(signal_error, 'retained race')
        def execute(argv, **kwargs):
            ready(argv)
            group = production._DarwinGroup(proc)
            group.cleanup()
            proc.wait()
            group.cleanup()
            return production.Outcome(124 if kind == 'deadline-term' else (0 if kind == 'resistant-pipe' else 7),
                'safety_deadline' if kind == 'deadline-term' else 'exit', fixture.OUTPUT, fixture.ERROR_OUTPUT, False, .3)
        def sleep(seconds):
            self.now += seconds
        info = namedtuple('info', 'si_pid si_uid si_signo si_status si_code')
        with ExitStack() as stack:
            for target, name, value in (
                (fixture.time, 'monotonic', self.clock), (fixture.time, 'sleep', sleep),
                (production, '_darwin_child_exited', lambda p: state['leader'] == 'Z'),
                (production, '_darwin_snapshot', snapshot), (production, 'execute', execute),
                (fixture.os, 'killpg', killpg), (fixture.subprocess, 'Popen', launch),
                (fixture, 'capture', lambda: self.raw(state['leader'], state['child'])),
                (fixture.os, 'waitid', lambda *a: info(self.pid, 0, 20, 7, 1)),
                (fixture.os, 'WEXITED', 4), (fixture.os, 'WNOHANG', 1), (fixture.os, 'WNOWAIT', 32),
            ):
                stack.enter_context(patch.object(target, name, value))
            case = fixture.observe_case(kind, self.output/'temporary', sys.executable, lambda: None, {})
        report = self.report([case])
        self.assertIsNone(case['failure'], case['failure'])
        return case, report

    def report(self, cases):
        return {'tools': {'python': {'path': sys.executable}}, 'source_root': str(diagnostic.ROOT),
                'original_output': str(self.output), 'driver': self.driver, 'finished': self.now+1,
                'cases': cases}

    def test_all_four_exact_cases(self):
        for kind in fixture.KINDS:
            with self.subTest(kind=kind):
                case, report = self.simulate(kind)
                self.assertTrue(check_case(case, report))

    def test_deadline_is_expected_but_pipe_deadline_is_failure(self):
        case, report = self.simulate('resistant-pipe')
        case['outcome'].update(cause='safety_deadline', returncode=124)
        with self.assertRaisesRegex(ValueError, 'Outcome'):
            check_case(case, report)

    def test_output_and_truncation_mutations(self):
        original, report = self.simulate('natural-output')
        for change in ({'stdout': ''}, {'stderr': ''}, {'truncated': True}, {'returncode': 0}):
            case = copy.deepcopy(original); case['outcome'].update(change)
            with self.assertRaises(ValueError): check_case(case, report)

    def test_ready_identity_and_boolean_leader_rejected(self):
        original, report = self.simulate('natural-output')
        for change in ({'leader': True}, {'leader': self.pid+1}, {'kind': 'deadline-term'}, {'children': [self.child]}):
            case = copy.deepcopy(original); case['ready'].update(change)
            with self.assertRaises(ValueError): check_case(case, report)

    def test_missing_kill_rejected(self):
        case, report = self.simulate('resistant-pipe')
        case['events'] = [e for e in case['events'] if not (e['kind'] in ('send', 'signal-start', 'signal-end')
                                                          and e['signal'] == signal.SIGKILL)]
        with self.assertRaises(ValueError): check_case(case, report)

    def test_wait_and_absence_are_both_required(self):
        original, report = self.simulate('natural-output')
        for omitted in ('wait-end', 'independent-snapshot', 'members'):
            case = copy.deepcopy(original); case['events'] = [e for e in case['events'] if e['kind'] != omitted]
            with self.assertRaises(ValueError): check_case(case, report)

    def test_foreign_members_and_changed_identity_rejected(self):
        original, report = self.simulate('resistant-pipe')
        for find, replace in ((str(self.child), str(self.child+1)), ('Mon Oct', 'Tue Oct')):
            case = copy.deepcopy(original)
            event = next(e for e in case['events'] if e['kind'] == 'snapshot')
            event['raw'] = event['raw'].replace(find, replace)
            with self.assertRaises(ValueError): check_case(case, report)

    def test_waitid_requires_repeat_retention_and_status(self):
        original, report = self.simulate('waitid-capability')
        for change in ('flag', 'status', 'reaped', 'missing-repeat'):
            case = copy.deepcopy(original)
            event = next(e for e in case['events'] if e['kind'] == 'waitid')
            if change == 'flag': event['flags'] = 5
            if change == 'status': event['value']['si_status'] = 0
            if change == 'reaped': event['returncode'] = 7
            if change == 'missing-repeat': case['events'].remove(event)
            with self.assertRaises(ValueError): check_case(case, report)

    def test_darwin_zero_uid_is_data_not_ownership_authority(self):
        original, report = self.simulate('waitid-capability')
        values = [e['value'] for e in original['events'] if e['kind'] == 'waitid']
        self.assertEqual([v['si_uid'] for v in values], [0, 0])
        self.assertTrue(check_case(original, report))
        for field, value in (('si_uid', 501), ('si_uid', False), ('si_pid', self.child)):
            case = copy.deepcopy(original)
            event = next(e for e in case['events'] if e['kind'] == 'waitid')
            event['value'][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                check_case(case, report)

    def test_zero_waitid_uid_does_not_waive_inventory_uid_or_ruid(self):
        original, report = self.simulate('waitid-capability')
        for column in (3, 4):
            case = copy.deepcopy(original)
            event = next(e for e in case['events'] if e['kind'] == 'independent-snapshot')
            lines = event['raw'].splitlines()
            fields = lines[-1].split()
            self.assertEqual(int(fields[0]), self.pid)
            fields[column] = str(int(fields[column])+1)
            lines[-1] = ' '.join(fields)
            event['raw'] = '\n'.join(lines)+'\n'
            with self.subTest(column=column), self.assertRaisesRegex(ValueError, 'owned identity'):
                check_case(case, report)

    def test_signal_error_retained_after_dead_only_race(self):
        case, report = self.simulate('deadline-term', errno.EPERM)
        self.assertTrue(check_case(case, report))
        ends = [e for e in case['events'] if e['kind'] == 'cleanup-end']
        self.assertEqual(ends[0]['signal_errors'][0]['errno'], errno.EPERM)
        for end in ends: end['signal_errors'] = []
        with self.assertRaises(ValueError): check_case(case, report)

    def test_signal_error_with_live_member_never_passes(self):
        case, report = self.simulate('resistant-pipe')
        event = next(e for e in case['events'] if e['kind'] == 'signal-end')
        event['error'] = fixture.failure(OSError(errno.EPERM, 'not discharged'))
        with self.assertRaisesRegex(ValueError, 'live members'):
            check_case(case, report)

    def test_partial_failure_is_retained_not_observed(self):
        case, report = self.simulate('natural-output')
        case['failure'] = fixture.failure(RuntimeError('actual cleanup failure'))
        self.assertFalse(check_case(case, report))
        self.assertEqual(diagnostic.status([case], report), 'stopped')

    def test_forwarding_restores_every_adapter_on_error(self):
        group, parser, kill = production._DarwinGroup, production._darwin_process_rows, os.killpg
        with self.assertRaisesRegex(RuntimeError, 'sentinel'):
            with fixture.observe_production(fixture.Events()):
                raise RuntimeError('sentinel')
        self.assertIs(production._DarwinGroup, group)
        self.assertIs(production._darwin_process_rows, parser)
        self.assertIs(os.killpg, kill)

    def test_signal_forwarded_once_same_error_object(self):
        error = OSError(errno.EPERM, 'sentinel')
        events = fixture.Events()
        with patch.object(os, 'killpg', side_effect=error) as actual:
            with fixture.observe_production(events):
                with self.assertRaises(OSError) as raised: os.killpg(self.pid, signal.SIGTERM)
            self.assertIs(raised.exception, error)
            actual.assert_called_once_with(self.pid, signal.SIGTERM)
        self.assertEqual(events.rows[-1]['error']['errno'], errno.EPERM)

    def test_snapshot_forwarded_once_unchanged(self):
        raw = self.raw('Z')
        sentinel = object()
        with patch.object(production, '_darwin_process_rows', return_value=sentinel) as actual:
            with fixture.observe_production(fixture.Events()):
                self.assertIs(production._darwin_process_rows(raw), sentinel)
            actual.assert_called_once_with(raw)

    def test_observation_bound(self):
        events = fixture.Events()
        with self.assertRaisesRegex(ValueError, 'overflow'): events.add('snapshot', raw='x'*fixture.MAX_EVENTS)
        self.assertEqual(events.rows, [])

    def test_atomic_ready_preserves_original(self):
        path = self.output/'ready'
        fixture.publish_ready(path, {'leader': 123})
        with self.assertRaises(ValueError): fixture.publish_ready(path, {'leader': 456})
        self.assertEqual(json.loads(path.read_text()), {'leader': 123})

    def test_fixed_workflows_and_negatives(self):
        for name in diagnostic.WORKFLOWS:
            doc = json.loads((diagnostic.ROOT/name).read_text())
            self.assertEqual(diagnostic.validate_workflow(doc), [])
            for change in ('runner', 'python', 'check', 'permissions', 'trigger'):
                bad = copy.deepcopy(doc)
                if change == 'runner': bad['jobs']['observe']['runs-on'] = 'macos-15-large'
                if change == 'python': bad['jobs']['observe']['steps'][3]['run'] = 'python3 harness.py'
                if change == 'check': bad['jobs']['observe']['steps'][4].pop('if')
                if change == 'permissions': bad['permissions']['contents'] = 'write'
                if change == 'trigger': bad['on'] = {'push': None}
                self.assertTrue(diagnostic.validate_workflow(bad))

    def test_exact_production_hash_binding(self):
        with patch.object(diagnostic.common, 'source_identity', return_value={'producer_sha256': {}}):
            self.assertEqual(diagnostic.identity()['production']['commit'], diagnostic.PRODUCTION_COMMIT)
            with patch.object(diagnostic.common, 'digest', return_value='wrong'):
                with self.assertRaisesRegex(ValueError, 'production runtime'): diagnostic.identity()

    def test_initial_capacity_refusal(self):
        resource = Resources(2, 2., 7*1024**3, 2*1024**3, 40*1024**3, {}, [], TOPOLOGY, False)
        with self.assertRaises(Exception): diagnostic.common.admit(resource, diagnostic.REQUIREMENTS)

    def full_report(self):
        cases = [self.simulate(kind)[0] for kind in fixture.KINDS]
        resource = Resources(2, 2., 7*1024**3, 3*1024**3, 40*1024**3, {}, [],
            TOPOLOGY | {'capacity_authority': 'declared-dedicated-darwin-vm-stat'}, False)
        admission = diagnostic.common.admit(resource, diagnostic.REQUIREMENTS)
        for case in cases:
            case['admission'] = diagnostic.common.admit(resource, dict(diagnostic.REQUIREMENTS,
                disk_growth_bytes=0, retained_copy_bytes=0))
            case['validation_error'] = None
        source = {'commit': 'a'*40, 'producer_sha256': {'fixture': 'b'*64},
                  'production': {'commit': diagnostic.PRODUCTION_COMMIT,
                                 'runtime_sha256': diagnostic.PRODUCTION_RUNTIME}}
        proof = {'environment': diagnostic.common.storage.bindings(self.output),
                 'paths': {name: {'storage': diagnostic.common.storage.topology(resource.diagnostics)}
                           for name in ('.', *diagnostic.common.storage.NAMES)}}
        env = diagnostic.common.storage.bindings(self.output) | {
            'SIFR_VERIFY_DISK_FLOOR_BYTES': str(resource.disk_available_bytes-24*1024**2),
            'SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC': repr(160.)}
        report = self.report(cases) | {'protocol': diagnostic.PROTOCOL, 'schema_version': 1,
            'source': source, 'specification': diagnostic.SPEC, 'requirements': diagnostic.REQUIREMENTS,
            'runtime_assertions': 0, 'native_qualification': False, 'target': diagnostic.common.TARGET,
            'status': 'observed', 'host': {'system': 'Darwin', 'machine': 'arm64',
                'uname': ['Darwin', 'owned-runner', '24.6.0', 'retained-kernel', 'arm64']},
            'storage': proof, 'workflow': {'GITHUB_SHA': source['commit']}, 'admission': admission,
            'deadline': 160., 'inherited_disk_floor_bytes': 8*1024**3,
            'environment': diagnostic.common.recorded_environment(env, self.output), 'files': {}}
        return report

    def retained(self, report):
        (self.output/'evidence/state.json').write_text(json.dumps(report))
        with patch.object(diagnostic, 'identity', return_value=report['source']):
            return diagnostic.check_retained(self.output/'evidence', report['source']['commit'])

    def test_complete_retained_receipt_and_rehashed_negatives(self):
        original = self.full_report()
        self.assertEqual(self.retained(original)['status'], 'observed')
        for change in ('cause', 'qualification', 'admission', 'ready', 'files'):
            report = copy.deepcopy(original)
            if change == 'cause': report['cases'][-1]['outcome']['cause'] = 'safety_deadline'
            if change == 'qualification': report['native_qualification'] = True
            if change == 'admission': report['cases'][-1]['admission']['memory_admitted_bytes'] = 1
            if change == 'ready': report['cases'][-1]['ready']['leader'] = True
            if change == 'files': report['files']['missing'] = {'sha256': '0'*64, 'size': 1}
            with self.subTest(change=change), self.assertRaises(ValueError): self.retained(report)

    def test_unexpected_actual_outcome_can_be_valid_failed_evidence(self):
        report = self.full_report()
        case = report['cases'][-1]
        case['outcome'].update(cause='safety_deadline', returncode=124)
        passed, case['validation_error'] = diagnostic.case_validation(case, report)
        self.assertFalse(passed)
        report.update(status='stopped', failure='fixture actual Outcome differed')
        self.assertEqual(self.retained(report)['status'], 'stopped')
        report['status'] = 'observed'
        with self.assertRaises(ValueError): self.retained(report)

    def test_finite_grace_and_no_signal_after_reap(self):
        original, report = self.simulate('resistant-pipe')
        case = copy.deepcopy(original)
        start = next(e for e in case['events'] if e['kind'] == 'signal-start' and e['signal'] == signal.SIGKILL)
        start['time'] = next(e['time'] for e in case['events'] if e['kind'] == 'signal-end')
        with self.assertRaises(ValueError): check_case(case, report)
        case = copy.deepcopy(original)
        end = next(e for e in case['events'] if e['kind'] == 'signal-start')
        case['events'].append(dict(end, time=report['finished']))
        with self.assertRaises(ValueError): check_case(case, report)

    def test_tighter_deadline_and_per_case_admission_refuse_spawn(self):
        for mode in ('deadline', 'per-case'):
            resource = Resources(2, 2., 7*1024**3, 3*1024**3, 40*1024**3, {}, [],
                TOPOLOGY | {'capacity_authority': 'declared-dedicated-darwin-vm-stat'}, False)
            env = diagnostic.common.storage.bindings(self.output)
            proof = {'environment': env}
            budget = SimpleNamespace(floor=8*1024**3, check=lambda: None)
            def deadline_environment(selected, seconds):
                deadline = fixture.time.monotonic()+(1 if mode == 'deadline' else 60)
                return selected | {'SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC': repr(deadline)}, deadline
            with ExitStack() as stack:
                for obj, name, value in (
                    (diagnostic.platform, 'system', lambda: 'Darwin'),
                    (diagnostic.platform, 'machine', lambda: 'arm64'),
                    (diagnostic, 'identity', lambda: {'commit': 'a'*40}),
                    (diagnostic.common.storage, 'prepare', lambda *a: (env, proof)),
                    (diagnostic.common.storage, 'check', lambda *a: None),
                    (diagnostic.common, 'parent_temporary', lambda *a: nullcontext()),
                    (diagnostic.common, 'deadline_environment', deadline_environment),
                    (diagnostic.DiskBudget, 'from_environment', lambda *a: budget),
                ):
                    stack.enter_context(patch.object(obj, name, value))
                stack.enter_context(patch.dict(os.environ, {'SIFR_NATIVE_HOST_KIND': 'dedicated-darwin'}))
                resources = stack.enter_context(patch.object(diagnostic.common, 'resources',
                    side_effect=[resource, ValueError('per-case resource refusal')]))
                invoke = stack.enter_context(patch.object(fixture, 'observe_case'))
                result = diagnostic.observe(self.output)
                self.assertEqual(result['status'], 'stopped')
                self.assertIn('deadline' if mode == 'deadline' else 'per-case', result['failure'])
                invoke.assert_not_called()
                self.assertEqual(resources.call_count, 1 if mode == 'deadline' else 2)


if __name__ == '__main__':
    unittest.main()
