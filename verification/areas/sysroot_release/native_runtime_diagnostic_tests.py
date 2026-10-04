"""Small independent diagnostic receipt and workflow mutation controls."""
import copy
from dataclasses import asdict
import hashlib
import json
import os
import subprocess
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import native_runtime_diagnostic as diagnostic
import native_runtime_observer as observer
from native_runtime_observer_tests import PAGES, raw
from native_runtime_storage_tests import TOPOLOGY
from sifr_verify.resource_admission import Resources, ResourceError, admit
sys.path.insert(0, str(diagnostic.ROOT/'scripts'))
from check_local_first_workflow import validate_runtime_diagnostic
from check_uv_toolchain import parse_workflows


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.output = Path(self.tmp.name).resolve(); self.evidence = self.output/'evidence'; self.evidence.mkdir()
        self.identity = {'commit': 'a'*40, 'producer_sha256': {'unit.py': 'b'*64}}
        self.source = patch.object(diagnostic, 'source_identity', return_value=self.identity)
        self.source.start(); self.addCleanup(self.source.stop)
        policy = json.loads(diagnostic.POLICY.read_text())
        asset = diagnostic.select(policy, diagnostic.TARGET, '0.1.0-beta.1300')
        capacity = Resources(3, 3., 7*1024**3, 3*1024**3, 40*1024**3, {}, [],
            TOPOLOGY | {'capacity_authority': 'declared-dedicated-darwin-vm-stat'}, False)
        requirements = diagnostic.storage.requirements(asset)
        self.report = {'schema_version': 1, 'protocol': diagnostic.PROTOCOL, 'status': 'observed',
            'runtime_assertions': 0, 'native_qualification': False, 'target': diagnostic.TARGET,
            'source': self.identity, 'publication': policy, 'asset': asset,
            'version': '0.1.0-beta.1300', 'specification': observer.SPEC,
            'requirements': requirements, 'capacity': asdict(capacity),
            'inherited_disk_floor_bytes': 8*1024**3, 'deadline': 3000.,
            'admission': admit(capacity, requirements), 'original_output': str(self.output),
            'source_root': str(diagnostic.ROOT), 'finished': 'unit completion',
            'storage': {'environment': diagnostic.storage.bindings(self.output),
                        'paths': {name: {'storage': TOPOLOGY} for name in ('.', *diagnostic.storage.NAMES)}},
            'tools': {name: {'path': '/unit/'+name, 'sha256': 'c'*64} for name in ('python', 'rustup')},
            'commands': []}
        plan = diagnostic.command_plan(self.output, '/unit/rustup', '/unit/python')
        rows = []
        for index, phase in enumerate(diagnostic.PHASES):
            sample = {'phase': phase, 'started': float(index), 'finished': index+.1,
                      'pgid': 1, 'driver_pid': 2, 'collector_pid': 3,
                      'ps': raw(1, 1)+raw(2, 2)+raw(3, 3), 'vm_stat': PAGES, 'stop': None}
            sample['observation'] = observer.interpret(sample, capacity.memory_limit_bytes); rows.append(sample)
            env = diagnostic.storage.bindings(self.output)
            env['SIFR_VERIFY_DISK_FLOOR_BYTES'] = str(capacity.disk_available_bytes-requirements['disk_growth_bytes']-requirements['retained_copy_bytes'])
            env['CARGO_NET_OFFLINE'] = 'false' if phase in ('toolchain', 'fetch') else 'true'
            env['SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC'] = '3000.0'
            row = {'id': phase, 'argv': plan[phase], 'cwd': str(self.output/'project' if phase == 'run' else self.output),
                   'environment': env, 'admission': admit(capacity, dict(requirements, disk_growth_bytes=0, retained_copy_bytes=0)),
                   'status': 'finished', 'cause': 'exit', 'returncode': 0, 'truncated': False,
                   'observation': {'samples': 1, 'sampled_maximum_rss_bytes': 300*1024, 'stop': None}}
            self.report['commands'].append(row)
            for stream in ('stdout', 'stderr'): (self.evidence/(phase+'.'+stream)).write_bytes(b'')
        (self.evidence/'samples.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in rows))
        (self.evidence/'run.stdout').write_text('9\n')
        (self.evidence/'version.stdout').write_text('sifr '+policy['version']+'\n')
        (self.evidence/'tool-identity.stdout').write_text('release: 1.98.1\nhost: aarch64-apple-darwin\n')
        (self.evidence/'sdk.stdout').write_text('/unit/sdk\n'); (self.evidence/'linker.stdout').write_text('clang unit\n')
        for name in diagnostic.dependencies.INPUTS:
            path = self.evidence/'package-inputs'/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('[features]\nmath=[]\n' if name.endswith('sifr_stdlib/Cargo.toml') else 'unit input')
        package_files = diagnostic.dependencies.inputs(self.evidence/'package-inputs') | {'bin/sifr': 'd'*64}
        (self.evidence/'package.json').write_text(json.dumps({'asset': asset, 'publication_source': policy['source_commit'],
            'rows': [{'path': name, 'sha256': sha} for name, sha in package_files.items()],
            'binary_sha256': package_files['bin/sifr']}))
        (self.evidence/'dependency-inputs.json').write_text(json.dumps({name: package_files[name] for name in diagnostic.dependencies.INPUTS}))
        for name in diagnostic.USER_FILES:
            path = self.evidence/'user-inputs'/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(diagnostic.SOURCE if name == 'src/main.sifr' else 'unit input')
        (self.evidence/'user-inputs.json').write_text(json.dumps(diagnostic.user_inputs(self.evidence/'user-inputs')))
        (self.evidence/'cold-cache.json').write_text(json.dumps({name: [] for name in diagnostic.COLD_PATHS}))
        manifest = diagnostic.dependencies.manifest(self.evidence/'package-inputs').replace(
            json.dumps(str(self.evidence/'package-inputs/crates/sifr_stdlib')),
            json.dumps(str(self.output/'package/crates/sifr_stdlib')))
        (self.evidence/'dependency.Cargo.toml').write_text(manifest)
        (self.evidence/'dependency.Cargo.lock').write_text('unit lock')
        next(row for row in self.report['commands'] if row['id'] == 'fetch-offline')['lock_sha256'] = diagnostic.digest(self.evidence/'dependency.Cargo.lock')
        self.seal()

    def seal(self):
        self.report['files'] = diagnostic.file_evidence(self.evidence)
        (self.evidence/'state.json').write_text(json.dumps(self.report))

    def check(self):
        return diagnostic.check_retained(self.evidence, 'a'*40)

    def test_complete_observation_is_still_zero_qualification(self):
        self.assertEqual(self.check()['runtime_assertions'], 0)
        for key, value in (('runtime_assertions', 1), ('runtime_assertions', False),
                           ('native_qualification', True), ('target', 'x86_64-apple-darwin')):
            original = self.report[key]; self.report[key] = value; self.seal()
            with self.subTest(key=key), self.assertRaises(ValueError): self.check()
            self.report[key] = original

    def test_rehashed_command_environment_allocation_and_summary_tamper(self):
        original = copy.deepcopy(self.report)
        mutations = [lambda r: r['commands'].pop(),
                     lambda r: r['commands'][0]['argv'].append('--foreign'),
                     lambda r: r['commands'][0]['environment'].update(CARGO_BUILD_JOBS='2'),
                     lambda r: r['commands'][0]['environment'].update(SIFR_VERIFY_DISK_FLOOR_BYTES='1'),
                     lambda r: r['commands'][0]['observation'].update(sampled_maximum_rss_bytes=1),
                     lambda r: r['requirements'].update(memory_reserve_bytes=1),
                     lambda r: r['capacity'].update(disk_memory_backed=True),
                     lambda r: r.update(source={'commit': 'a'*40, 'producer_sha256': {}})]
        for mutate in mutations:
            self.report = copy.deepcopy(original); mutate(self.report); self.seal()
            with self.assertRaises((ValueError, KeyError, ResourceError)): self.check()

    def test_raw_tamper_and_rehashed_peak_tamper_reject(self):
        path = self.evidence/'samples.jsonl'
        text = path.read_text()
        path.write_text(text.replace('"rss_bytes": 307200', '"rss_bytes": 1', 1))
        with self.assertRaises(ValueError): self.check()
        self.seal()
        with self.assertRaises(ValueError): self.check()

    def test_no_environment_credentials_are_serialized(self):
        env = diagnostic.storage.bindings(self.output) | {'ACTIONS_RUNTIME_TOKEN': 'private', 'SECRET': 'private'}
        self.assertNotIn('private', json.dumps(diagnostic.recorded_environment(env, self.output)))

    def test_worker_interpreter_preserves_canonical_dependency_context(self):
        import packaging
        selected = diagnostic.python_identity()
        result = subprocess.run([selected['path'], '-B', '-c',
            'import sys,json,packaging; print(json.dumps([sys.prefix,packaging.__file__]))'],
            capture_output=True, text=True, check=True, timeout=5)
        self.assertEqual(json.loads(result.stdout), [sys.prefix, packaging.__file__])
        self.assertEqual(selected['path'], str(Path(sys.executable).absolute()))
        self.assertEqual(selected['sha256'], diagnostic.digest(Path(sys.executable).resolve()))

    def test_inherited_absolute_deadline_is_preserved(self):
        limit = time.monotonic()+2
        env, actual = diagnostic.deadline_environment({'SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC': repr(limit)}, diagnostic.DEADLINE)
        self.assertEqual(actual, limit)
        self.assertEqual(env['SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC'], repr(limit))

    def test_rehashed_workload_and_dependency_selection_reject(self):
        for name, content in (('user-inputs/src/main.sifr', 'print(9)'),
                              ('dependency.Cargo.toml', '[dependencies]'),
                              ('cold-cache.json', '{"target":["warm"]}')):
            path = self.evidence/name; original = path.read_bytes()
            path.write_text(content); self.seal()
            with self.subTest(name=name), self.assertRaises(ValueError): self.check()
            path.write_bytes(original); self.seal()

    def test_admission_refusal_starts_no_workload(self):
        output = self.output/'refused'; evidence = output/'evidence'
        proof = self.report['storage']
        def prepare(root, _):
            evidence.mkdir(parents=True)
            return diagnostic.storage.bindings(root), proof
        capacity = Resources(3, 3., 7*1024**3, 2*1024**3, 40*1024**3, {}, [],
            TOPOLOGY | {'capacity_authority': 'declared-dedicated-darwin-vm-stat'}, False)
        with patch.object(diagnostic.platform, 'system', return_value='Darwin'), \
             patch.object(diagnostic.platform, 'machine', return_value='arm64'), \
             patch.dict(os.environ, {'SIFR_NATIVE_HOST_KIND': 'dedicated-darwin'}), \
             patch.object(diagnostic.storage, 'prepare', side_effect=prepare), \
             patch.object(diagnostic.shutil, 'which', return_value=sys.executable), \
             patch.object(diagnostic, 'digest', return_value='c'*64), \
             patch.object(diagnostic, 'resources', return_value=capacity), \
             patch.object(diagnostic, 'execute') as execute:
            with self.assertRaises(ResourceError): diagnostic.observe(diagnostic.ROOT, output)
            execute.assert_not_called()
        report = json.loads((evidence/'state.json').read_text())
        self.assertEqual(report['status'], 'stopped')
        self.assertEqual(report['commands'], [])

    def test_foreign_architecture_and_symlink_evidence_reject(self):
        with patch('metadata_artifact.host_target', return_value=diagnostic.TARGET):
            fake = self.output/'binary'; fake.write_bytes(bytes.fromhex('cffaedfe')+(0x01000007).to_bytes(4, 'little')+bytes(24))
            with self.assertRaises(ValueError): diagnostic.require_native_binary(fake, False)
        (self.evidence/'foreign').symlink_to(self.output/'binary')
        with self.assertRaises(ValueError): self.seal()

    def test_workflow_fixed_standard_arm_and_blocking_checks(self):
        name = '.github/workflows/native-runtime-diagnostic.yml'
        workflow = parse_workflows({name: (diagnostic.ROOT/name).read_text()})[name]
        self.assertFalse(validate_runtime_diagnostic(workflow))
        for mutate in (lambda w: w['jobs']['observe'].update({'runs-on': 'macos-15-xlarge'}),
                       lambda w: w['permissions'].update(contents='write'),
                       lambda w: w['jobs']['observe']['steps'][3].update({'continue-on-error': True}),
                       lambda w: w['jobs']['observe']['steps'][4].update(run='echo pass'),
                       lambda w: w['jobs']['observe']['steps'][0]['with'].update(ref='main')):
            changed = copy.deepcopy(workflow); mutate(changed)
            self.assertTrue(validate_runtime_diagnostic(changed))


if __name__ == '__main__': unittest.main()
