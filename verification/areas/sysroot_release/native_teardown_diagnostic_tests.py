"""Tiny ownership, retained-byte, workflow and real harmless fixture controls."""
import copy
import errno
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import native_teardown_diagnostic as diagnostic
import native_teardown_process as process
from native_runtime_storage_tests import TOPOLOGY
from sifr_verify.resource_admission import Resources
sys.path.insert(0, str(diagnostic.ROOT/'scripts'))
from check_uv_toolchain import parse_workflows


def row(pid, parent, group, state='S', uid=None, start='Mon Oct  5 00:00:00 2026'):
    uid = os.getuid() if uid is None else uid
    return f'{pid} {parent} {group} {uid} {uid} 1024 {state} {start} Python\n'


class Controls(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.output = Path(self.tmp.name)
        self.evidence = self.output/'evidence'; self.evidence.mkdir()
        self.temporary = self.output/'temporary'; self.temporary.mkdir()
        self.leader = 900001
        self.ready = {'kind': 'ordinary', 'leader': self.leader, 'children': []}
        self.driver = {'pid': os.getpid(), 'uid': os.geteuid(), 'ruid': os.getuid()}
        self.initial = row(self.driver['pid'], 1, self.driver['pid'])+row(self.leader, self.driver['pid'], self.leader)

    def simulated_case(self, kill_errno=None, foreign=False, kind='ordinary'):
        counter = 0
        ready = self.ready | {'kind': kind, 'children': [900002, 900003] if kind == 'multi-child' else []}
        children = ''.join(row(pid, self.leader, self.leader) for pid in ready['children'])
        def snapshot(label):
            nonlocal counter
            raw = self.initial+children
            if 'after-term' in label or 'after-kill' in label or 'cleanup-before-reap' in label:
                raw = row(self.driver['pid'], 1, self.driver['pid'])+row(self.leader, self.driver['pid'], self.leader, 'Z')+children
                if foreign: raw += row(900002, 1, self.leader)
            if 'after-reap' in label: raw = row(self.driver['pid'], 1, self.driver['pid'])
            name = kind+'-'+str(counter)+'.ps'; counter += 1
            path = self.evidence/name; path.write_text(raw)
            now = time.monotonic()
            return {'file': name, 'sha256': diagnostic.common.digest(path), 'raw': raw,
                    'started': now, 'finished': now}
        class Child:
            pid = self.leader
            def wait(this, timeout): return -signal.SIGTERM
        def launch(*args, **kwargs):
            (self.temporary/(kind+'-ready.json')).write_text(json.dumps(ready))
            return Child()
        def kill(pid, number):
            self.assertEqual(pid, self.leader)
            if number == signal.SIGKILL and kill_errno: raise OSError(kill_errno, 'fixture errno')
        with patch.object(process.subprocess, 'Popen', side_effect=launch), \
             patch.object(process.os, 'killpg', side_effect=kill) as signals, \
             patch.object(process.os, 'getsid', return_value=self.leader):
            case = process.observe_case(kind, self.temporary, sys.executable, snapshot, lambda: None, {})
        return case, signals.call_args_list

    def report(self, case):
        capacity = Resources(3, 3., 7*1024**3, 3*1024**3, 40*1024**3, {}, [],
            TOPOLOGY | {'capacity_authority': 'declared-dedicated-darwin-vm-stat'}, False)
        case['admission'] = diagnostic.common.admit(capacity, dict(diagnostic.REQUIREMENTS,
            disk_growth_bytes=0, retained_copy_bytes=0))
        deadline = time.monotonic()+60
        proof = {'environment': diagnostic.common.storage.bindings(self.output),
                 'paths': {name: {'storage': diagnostic.common.storage.topology(capacity.diagnostics)}
                           for name in ('.', *diagnostic.common.storage.NAMES)}}
        env = diagnostic.common.storage.bindings(self.output) | {
            'SIFR_VERIFY_DISK_FLOOR_BYTES': str(capacity.disk_available_bytes-
                diagnostic.REQUIREMENTS['disk_growth_bytes']-diagnostic.REQUIREMENTS['retained_copy_bytes']),
            'SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC': repr(deadline)}
        return {'protocol': diagnostic.PROTOCOL, 'schema_version': 1,
            'source': {'commit': 'a'*40, 'producer_sha256': {}}, 'specification': diagnostic.SPEC,
            'requirements': diagnostic.REQUIREMENTS, 'runtime_assertions': 0, 'native_qualification': False,
            'target': diagnostic.common.TARGET, 'status': 'stopped', 'original_output': str(self.output),
            'source_root': str(diagnostic.ROOT), 'driver': self.driver,
            'host': {'system': 'Darwin', 'machine': 'arm64', 'uname': ['Darwin', 'unit', '24.6.0', 'unit kernel', 'arm64']},
            'tools': {'python': {'path': sys.executable}}, 'storage': proof, 'cases': [case], 'workflow': {},
            'admission': diagnostic.common.admit(capacity, diagnostic.REQUIREMENTS),
            'environment': diagnostic.common.recorded_environment(env, self.output), 'deadline': deadline,
            'inherited_disk_floor_bytes': 8*1024**3, 'finished': time.monotonic()}

    def seal(self, report):
        report['files'] = diagnostic.common.file_evidence(self.evidence)
        (self.evidence/'state.json').write_text(json.dumps(report))

    def check(self, report):
        self.seal(report)
        with patch.object(diagnostic, 'identity', return_value=report['source']):
            return diagnostic.check_retained(self.evidence, 'a'*40)

    def test_parser_and_owned_pid_zero_or_duplicate_rejection(self):
        rows = process.parse(row(0, 0, 0)+self.initial)
        identities = process.owned(rows, self.ready, **{'driver': self.driver['pid'], 'uid': self.driver['uid'], 'ruid': self.driver['ruid']})
        self.assertTrue(process.eligible(rows, identities, self.leader))
        for raw in ('bad row', self.initial+self.initial):
            with self.assertRaises(ValueError): process.parse(raw)
        for value in (0, True, -1):
            with self.assertRaises(ValueError): process.owned(rows, self.ready | {'leader': value}, self.driver['pid'], self.driver['uid'], self.driver['ruid'])

    def test_changed_start_uid_group_unknown_member_and_missing_anchor_reject(self):
        rows = process.parse(self.initial)
        identities = process.owned(rows, self.ready, self.driver['pid'], self.driver['uid'], self.driver['ruid'])
        for key, value in (('start', 'other start'), ('uid', 9876), ('pgid', 900002)):
            changed = copy.deepcopy(rows); changed[self.leader][key] = value
            with self.assertRaises(ValueError): process.eligible(changed, identities, self.leader)
        unknown = process.parse(self.initial+row(900002, self.leader, self.leader))
        with self.assertRaises(ValueError): process.eligible(unknown, identities, self.leader)
        del unknown[self.leader]
        with self.assertRaises(ValueError): process.eligible(unknown, identities, self.leader)
        self.assertFalse(process.eligible({}, identities, self.leader))

    def test_eperm_is_retained_even_when_later_absence_is_verified(self):
        case, calls = self.simulated_case(errno.EPERM)
        self.assertTrue(case['cleanup_verified'])
        self.assertEqual([args.args[1] for args in calls], [signal.SIGTERM, signal.SIGKILL, signal.SIGKILL])
        self.assertEqual(diagnostic.status([case, case]), 'signal_failure')
        self.check(self.report(case))

    def test_complete_two_fixture_signal_failure_cannot_be_reclassified(self):
        ordinary, _ = self.simulated_case()
        multi, _ = self.simulated_case(errno.EPERM, kind='multi-child')
        report = self.report(ordinary)
        multi['admission'] = copy.deepcopy(ordinary['admission'])
        report['cases'].append(multi)
        report['status'] = 'signal_failure'
        self.assertEqual(self.check(report)['status'], 'signal_failure')
        report['status'] = 'observed'
        with self.assertRaises(ValueError): self.check(report)

    def test_complete_receipt_binds_ready_leader_kind_and_actual_term(self):
        ordinary, _ = self.simulated_case()
        multi, _ = self.simulated_case(kind='multi-child')
        original = self.report(ordinary)
        multi['admission'] = copy.deepcopy(ordinary['admission'])
        original['cases'].append(multi)
        original['status'] = 'observed'
        self.assertEqual(self.check(original)['status'], 'observed')
        for mutate in (lambda c: c.update(leader=900099, session=900099),
                       lambda c: c['ready'].update(leader=True),
                       lambda c: c['ready'].update(kind='multi-child')):
            changed = copy.deepcopy(original); mutate(changed['cases'][0])
            with self.subTest(mutation=mutate), self.assertRaisesRegex(ValueError, 'ready fixture identity differs'):
                self.check(changed)
        forged = copy.deepcopy(original)
        case = forged['cases'][0]
        case.update(leader=900099, session=900099)
        for event in case['events']:
            if event['kind'] == 'signal':
                event.update(attempted=False, reason='group absent')
        with self.assertRaisesRegex(ValueError, 'ready fixture identity differs'):
            self.check(forged)
        omitted = copy.deepcopy(original)
        term = next(event for event in omitted['cases'][0]['events']
                    if event['kind'] == 'signal' and event['label'] == 'term')
        term.update(attempted=False, reason='group absent')
        with self.assertRaisesRegex(ValueError, 'signal authorization differs'):
            self.check(omitted)

    def test_foreign_member_never_receives_group_kill(self):
        case, calls = self.simulated_case(foreign=True)
        self.assertEqual([args.args[1] for args in calls], [signal.SIGTERM])
        self.assertIn('unknown', case['failure'])
        self.assertEqual(diagnostic.status([case, case]), 'stopped')

    def test_rehashed_signal_reap_identity_environment_and_claim_mutations_reject(self):
        case, _ = self.simulated_case()
        original = self.report(case); self.check(original)
        mutations = [lambda r: r.update(runtime_assertions=1),
            lambda r: r.update(native_qualification=True), lambda r: r.update(status='observed'),
            lambda r: r['environment'].update(SIFR_VERIFY_DISK_FLOOR_BYTES='1'),
            lambda r: r['cases'][0]['argv'].append('--foreign'),
            lambda r: r['cases'][0]['identities'][str(self.leader)].update(uid=9999),
            lambda r: r['cases'][0]['events'][1].update(signal=int(signal.SIGKILL)),
            lambda r: r['cases'][0]['events'].insert(-1, {'kind': 'signal', 'label': 'kill', 'signal': 9, 'attempted': True}),
            lambda r: r['cases'][0]['events'].pop(),
            lambda r: r['cases'][0]['events'].__setitem__(1, {'kind': 'signal', 'label': 'term', 'signal': 15, 'attempted': False, 'reason': 'group absent'})]
        for mutate in mutations:
            changed = copy.deepcopy(original); mutate(changed)
            with self.subTest(mutation=mutate), self.assertRaises((ValueError, KeyError)):
                self.check(changed)

    def test_raw_inventory_mutation_rejects_even_with_rehashed_file_map(self):
        case, _ = self.simulated_case(); report = self.report(case)
        self.check(report)
        path = self.evidence/case['events'][0]['record']['file']
        path.write_text(path.read_text().replace('Python', 'other'))
        with self.assertRaises(ValueError): self.check(report)

    def test_canonical_venv_command_and_both_fixed_workflow_paths(self):
        data = {name: (diagnostic.ROOT/name).read_text() for name in diagnostic.WORKFLOWS}
        self.assertEqual(*data.values())
        for name, document in parse_workflows(data).items():
            with self.subTest(path=name):
                self.assertFalse(diagnostic.validate_workflow(document))
                for mutate in (lambda w: w['jobs']['observe'].update({'runs-on': 'macos-15-xlarge'}),
                    lambda w: w['jobs']['observe']['steps'][4].update(run='uv run python3 check'),
                    lambda w: w['jobs']['observe']['steps'][3].update({'continue-on-error': True}),
                    lambda w: w['jobs']['observe']['steps'][0]['with'].update(ref='main'),
                    lambda w: w['on'].update(push=None), lambda w: w['permissions'].update(contents='write')):
                    changed = copy.deepcopy(document); mutate(changed)
                    self.assertTrue(diagnostic.validate_workflow(changed))

    def test_insufficient_capacity_never_launches_fixture(self):
        output = self.output/'admission-refusal'; output.mkdir(); (output/'evidence').mkdir()
        capacity = Resources(3, 3., 7*1024**3, 2*1024**3, 40*1024**3, {}, [],
            TOPOLOGY | {'capacity_authority': 'declared-dedicated-darwin-vm-stat'}, False)
        with patch.object(diagnostic.platform, 'system', return_value='Darwin'), \
             patch.object(diagnostic.platform, 'machine', return_value='arm64'), \
             patch.dict(os.environ, {'SIFR_NATIVE_HOST_KIND': 'dedicated-darwin'}), \
             patch.object(diagnostic, 'identity', return_value={'commit': 'a'*40}), \
             patch.object(diagnostic.common.storage, 'prepare', return_value=(diagnostic.common.storage.bindings(output), {})), \
             patch.object(diagnostic.common, 'resources', return_value=capacity), \
             patch.object(diagnostic.process, 'observe_case') as observe_case:
            report = diagnostic.observe(output)
        self.assertEqual(report['status'], 'stopped')
        self.assertEqual(report['cases'], [])
        self.assertIn('memory', report['failure'])
        observe_case.assert_not_called()

    def test_tighter_inherited_deadline_refuses_fixture_before_spawn(self):
        output = self.output/'deadline-refusal'; output.mkdir(); (output/'evidence').mkdir()
        capacity = Resources(3, 3., 7*1024**3, 3*1024**3, 40*1024**3, {}, [],
            TOPOLOGY | {'capacity_authority': 'declared-dedicated-darwin-vm-stat'}, False)
        deadline = time.monotonic()+1
        env = diagnostic.common.storage.bindings(output) | {'SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC': repr(deadline)}
        with patch.object(diagnostic.platform, 'system', return_value='Darwin'), \
             patch.object(diagnostic.platform, 'machine', return_value='arm64'), \
             patch.dict(os.environ, {'SIFR_NATIVE_HOST_KIND': 'dedicated-darwin'}), \
             patch.object(diagnostic, 'identity', return_value={'commit': 'a'*40}), \
             patch.object(diagnostic.common.storage, 'prepare', return_value=(env, {})), \
             patch.object(diagnostic.common, 'resources', return_value=capacity), \
             patch.object(diagnostic.process, 'observe_case') as observe_case:
            report = diagnostic.observe(output)
        self.assertEqual(report['deadline'], deadline)
        self.assertIn('cleanup window', report['failure'])
        observe_case.assert_not_called()

    def test_multi_child_readiness_requires_exact_two_owned_children(self):
        ready = {'kind': 'multi-child', 'leader': self.leader, 'children': [900002, 900003]}
        rows = process.parse(self.initial+row(900002, self.leader, self.leader)+row(900003, self.leader, self.leader))
        identities = process.owned(rows, ready, self.driver['pid'], self.driver['uid'], self.driver['ruid'])
        self.assertEqual(len(identities), 3)
        for children in ([900002], [900002, 900002], [900002, True]):
            with self.assertRaises(ValueError):
                process.owned(rows, ready | {'children': children}, self.driver['pid'], self.driver['uid'], self.driver['ruid'])

    def test_atomic_ready_publication_and_existing_destination_rejection(self):
        path = self.temporary/'atomic-ready.json'
        process.publish_ready(path, self.ready)
        self.assertEqual(json.loads(path.read_text()), self.ready)
        self.assertFalse(path.with_name(path.name+'.writing').exists())
        with self.assertRaisesRegex(ValueError, 'already exists'):
            process.publish_ready(path, {'foreign': True})
        self.assertEqual(json.loads(path.read_text()), self.ready)

    def test_partial_ready_write_never_exposes_destination(self):
        path = self.temporary/'partial-ready.json'
        def interrupted(payload, stream):
            stream.write('{'); stream.flush()
            self.assertFalse(path.exists())
            raise OSError('fixture interrupted before atomic publication')
        with patch.object(process.json, 'dump', side_effect=interrupted):
            with self.assertRaises(OSError): process.publish_ready(path, self.ready)
        self.assertFalse(path.exists())
        self.assertEqual(path.with_name(path.name+'.writing').read_text(), '{')

    def test_real_ordinary_fixture_has_owned_session_and_bounded_term_exit(self):
        path = self.temporary/'real-ready.json'
        proc = subprocess.Popen([sys.executable, '-I', '-S', '-B', process.__file__, 'ordinary', str(path)],
                                start_new_session=True, stdin=subprocess.DEVNULL,
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            until = time.monotonic()+2
            while not path.exists() and time.monotonic() < until: time.sleep(.01)
            ready = json.loads(path.read_text())
            self.assertEqual(ready['leader'], proc.pid)
            self.assertEqual(os.getpgid(proc.pid), proc.pid)
            self.assertEqual(os.getsid(proc.pid), proc.pid)
            # The direct owned child is unreaped: its group identifier is reserved.
            os.killpg(proc.pid, signal.SIGTERM)
            self.assertEqual(proc.wait(timeout=2), -signal.SIGTERM)
        finally:
            if proc.poll() is None: proc.kill()
            proc.wait()


if __name__ == '__main__': unittest.main()
