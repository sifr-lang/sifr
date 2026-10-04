"""Negative preparation-custody controls; fixture bytes are not native qualification."""
import copy
from datetime import datetime,timezone
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import native_candidate as candidate


class CandidateChecks(unittest.TestCase):
    def setUp(self):
        self.temporary=tempfile.TemporaryDirectory();self.addCleanup(self.temporary.cleanup)
        self.base=Path(self.temporary.name);self.root=self.base/'source';self.root.mkdir()
        (self.root/'crates/sifr/src').mkdir(parents=True)
        (self.root/'crates/sifr/src/main.rs').write_text('fn main() {}\n')
        (self.root/'crates/sifr/Cargo.toml').write_text('[package]\nname="sifr"\nversion="0.0.0"\n')
        (self.root/'Cargo.lock').write_text('fixture locked input\n')
        producer=self.root/'verification/areas/sysroot_release';producer.mkdir(parents=True)
        for name in ('native_candidate.py','native_capacity.py','native_recovery.py'):
            shutil.copy2(Path(candidate.__file__).parent/name,producer/name)
        shutil.copy2(Path(candidate.__file__).parent/'package_build.py',producer/'package_build.py')
        def git(*argv):
            return subprocess.check_output(['git',*argv],cwd=self.root,stderr=subprocess.PIPE,text=True).strip()
        git('init','-q');git('add','.')
        git('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','commit.gpgsign=false','commit','-qm','fixture')
        self.source=git('rev-parse','HEAD');self.out=self.base/'preparation';self.out.mkdir()
        self.target='x86_64-unknown-linux-gnu';self.version='0.1.0-beta.1300'
        archive=self.out/'artifacts'/f'sifr-{self.version}-{self.target}.tar.gz';archive.parent.mkdir()
        binary=b'unit fixture only: no native compiler execution'
        manifest=f'built-by-compiler-commit="{self.source}"\ncargo-lock-sha256="{candidate.digest(self.root/"Cargo.lock")}"\n'
        with tarfile.open(archive,'w:gz') as stream:
            for name,data in [('sysroot.toml',manifest.encode()),('bin/sifr',binary)]:
                item=tarfile.TarInfo(name);item.size=len(data);stream.addfile(item,io.BytesIO(data))
        Path(str(archive)+'.sha256').write_text(candidate.digest(archive)+'\n')
        installer=self.out/f'sifr-installer-{self.version}';installer.write_bytes(b'unit fixture installer')
        artifact={'reason':'compiler-artifact','manifest_path':str(self.root/'crates/sifr/Cargo.toml'),
            'target':{'name':'sifr','kind':['bin'],'crate_types':['bin'],'src_path':str(self.root/'crates/sifr/src/main.rs')},
            'profile':{'opt_level':'3','test':False,'debug_assertions':False,'overflow_checks':False},
            'executable':str(self.out/'target/sysroot_release/cargo-target'/self.target/'release/sifr')}
        events=self.out/'cargo.jsonl';events.write_text(json.dumps(artifact)+'\n')
        now=datetime.now(timezone.utc).isoformat()
        self.tools={'fixture':'explicit unit tool identity'}
        commands=[]
        for label,argv in candidate.commands(self.root,self.out,self.version,self.target).items():
            raw={}
            for stream in ('stdout','stderr'):
                path=self.out/(label+'.'+stream);path.write_bytes(b'unit fixture output')
                raw[stream]={'path':str(path),'sha256':candidate.digest(path)}
            commands.append({'id':label,'argv':argv,'cause':'exit','returncode':0,'truncated':False,'output':raw})
        self.report={'schema_version':1,'protocol':'native-candidate-preparation-v1','kind':'preparation-output',
            'status':'prepared','runtime_assertions':0,'source_root':str(self.root),'source_commit':self.source,
            'version':self.version,'version_role':'qualification-only','source_package_version':'0.0.0',
            'qualification_version_sha256':candidate.digest(producer/'package_build.py'),
            'target':self.target,'tools':self.tools,
            'producer_sha256':{name:candidate.digest(producer/name) for name in ('native_candidate.py','native_capacity.py','native_recovery.py')},
            'cargo_lock_sha256':candidate.digest(self.root/'Cargo.lock'),'started_utc':now,'finished_utc':now,
            'commands':commands,'cargo_artifact':artifact,'cargo_events_sha256':candidate.digest(events),
            'archive':{'path':str(archive),'sha256':candidate.digest(archive)},
            'installer':{'path':str(installer),'sha256':candidate.digest(installer)},
            'binary_sha256':hashlib.sha256(binary).hexdigest()}
        self.path=self.out/'candidate.json'

    def check(self,report=None):
        self.path.write_text(json.dumps(self.report if report is None else report))
        # Archive layout is covered by verify_release_archive's existing real
        # bundle controls. These deliberately minimal fixtures test custody.
        with patch.object(candidate,'verify_archive'),patch.object(candidate,'tool_identity',return_value=self.tools), \
             patch.object(candidate,'current_host_target',return_value=self.target):
            return candidate.check(self.path,_pending=True)

    def test_source_placeholder_is_separate_from_canonical_qualification_version(self):
        self.assertEqual(candidate.qualification_version(self.root),self.version)
        self.assertEqual(self.check()['source_package_version'],'0.0.0')
        for field,value in [('version','0.0.0'),('source_package_version',self.version),
                            ('version_role','published'),('qualification_version_sha256','0'*64)]:
            changed=copy.deepcopy(self.report);changed[field]=value
            with self.subTest(field=field),self.assertRaises(ValueError): self.check(changed)
        declaration=self.root/'verification/areas/sysroot_release/package_build.py'
        for value in ["'0.0.0'", "'0.1.0-beta.16'", "str('0.1.0-beta.1300')"]:
            declaration.write_text('RELEASE_VERSION = '+value+'\n')
            with self.subTest(value=value),self.assertRaises(ValueError): candidate.qualification_version(self.root)

    def test_preparation_never_claims_runtime_assertions(self):
        self.assertEqual(self.check()['runtime_assertions'],0)
        changed=copy.deepcopy(self.report);changed['runtime_assertions']=1
        with self.assertRaises(ValueError): self.check(changed)

    def test_unpublished_or_failed_state_cannot_be_consumed(self):
        self.check()
        with self.assertRaises(ValueError): candidate.check(self.path)
        receipt=self.out/'receipt.json';receipt.write_text(json.dumps(self.report))
        failed=copy.deepcopy(self.report);failed['status']='failed'
        (self.out/'state.json').write_text(json.dumps(failed))
        with self.assertRaises(ValueError): candidate.check(receipt)
        receipt.write_text(json.dumps(failed))
        with self.assertRaises(ValueError): candidate.check(receipt)

    def test_failed_build_preserves_state_and_canonical_release_flags(self):
        from sifr_verify.process_execution import Outcome
        from sifr_verify.resource_admission import Resources
        output=self.base/'failed-native-attempt'
        capacity=Resources(4,4,100*1024**3,100*1024**3,100*1024**3,{},[],{},False)
        tools={'cargo':{'path':shutil.which('cargo')},'rustc':{'path':shutil.which('rustc')}}
        def fail_build(argv,**kwargs):
            self.assertEqual(argv,candidate.commands(self.root,output,self.version,self.target)['build-package'])
            self.assertNotIn('RUSTFLAGS',kwargs['env'])
            self.assertEqual(kwargs['env']['RUSTC'],tools['rustc']['path'])
            return Outcome(1,'exit',b'',b'explicit unit build failure',False,0.01)
        with patch.object(candidate,'resources',return_value=capacity),patch.object(candidate,'tool_identity',return_value=tools), \
             patch.object(candidate,'current_host_target',return_value=self.target),patch.object(candidate,'execute',side_effect=fail_build):
            with self.assertRaises(ValueError): candidate.prepare(self.root,output)
        self.assertEqual(json.loads((output/'state.json').read_text())['status'],'failed')
        self.assertFalse((output/'receipt.json').exists())

    def test_command_or_profile_or_target_substitution_is_rejected(self):
        for change in ('command','optimization','manifest','target-kind','source'):
            changed=copy.deepcopy(self.report)
            if change=='command': changed['commands'][0]['argv']=['true']
            else:
                artifact=changed['cargo_artifact']
                if change=='optimization': artifact['profile']['opt_level']='1'
                if change=='manifest': artifact['manifest_path']=str(self.root/'other/Cargo.toml')
                if change=='target-kind': artifact['target']['kind']=['lib']
                if change=='source': artifact['target']['src_path']=str(self.root/'other.rs')
                events=self.out/'cargo.jsonl';events.write_text(json.dumps(artifact)+'\n')
                changed['cargo_events_sha256']=candidate.digest(events)
            with self.subTest(change=change),self.assertRaises(ValueError): self.check(changed)

    def test_source_tools_and_input_drift_are_rejected(self):
        changed=copy.deepcopy(self.report);changed['tools']={'fixture':'different'}
        with self.assertRaises(ValueError): self.check(changed)
        (self.root/'Cargo.lock').write_text('different locked input')
        with self.assertRaises(ValueError): self.check()

    def test_checksum_raw_output_installer_and_incomplete_process_are_rejected(self):
        for field in ('checksum','raw-output','installer','process'):
            with self.subTest(field=field):
                if field=='process':
                    changed=copy.deepcopy(self.report);changed['commands'][0]['cause']='deadline'
                    with self.assertRaises(ValueError): self.check(changed)
                    continue
                path={'checksum':Path(self.report['archive']['path']+'.sha256'),
                      'raw-output':self.out/'build-package.stdout','installer':Path(self.report['installer']['path'])}[field]
                old=path.read_bytes();path.write_bytes(b'substituted')
                try:
                    with self.assertRaises(ValueError): self.check()
                finally: path.write_bytes(old)


if __name__=='__main__': unittest.main()
