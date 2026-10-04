"""Physical-storage and real temporary-path controls; no native compiler pass."""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import native_candidate as candidate
import native_candidate_tests as fixtures
import native_preparation_storage as policy
from sifr_verify.process_execution import Outcome
from sifr_verify.resource_admission import Resources, ResourceError, admit


DISK={'storage_device':'/dev/disk1s1','filesystem':'apfs','volume_bus_protocol':'SATA',
      'apfs_backing_stores':[{'DeviceNode':'/dev/disk0s2','BusProtocol':'SATA','Size':300*policy.GIB}]}


def capacity(*,available=9250066432,disk=30*policy.GIB,memory_backed=False):
    return Resources(4,4,14*policy.GIB,available,disk,{},[],
                     {'capacity_authority':'declared-dedicated-darwin-vm-stat',**DISK},memory_backed)


class StorageChecks(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.base=Path(self.tmp.name).resolve()
        self.root=self.base/'source';self.root.mkdir()
        (self.root/'.cargo').mkdir()
        self.config=self.root/'.cargo/config.toml'
        self.config.write_text('[env]\nSYNTAQLITE_SQLITE_VERSION = { value = "3053004", force = true }\n')
        self.output=self.base/'output';self.output.mkdir(mode=0o700)
        self.cargo=self.base/'cargo';self.cargo.mkdir(mode=0o700)
        self.env={'CARGO_HOME':str(self.cargo)}
        self.disk=patch.object(policy,'storage',return_value=copy.deepcopy(DISK));self.disk.start()
        self.addCleanup(self.disk.stop)

    def prepare(self):
        return policy.prepare(self.root,self.output,self.env)

    def check(self,record,admission=None):
        policy.check(record,admission or admit(capacity(),policy.requirements(physical=True)),
                     self.root,self.output,self.env)

    def test_conserved_storage_and_observed_intel_boundary(self):
        new=policy.requirements(physical=True);old=policy.requirements()
        self.assertEqual(new['memory_peak_bytes'],old['memory_peak_bytes'])
        self.assertEqual(new['memory_reserve_bytes'],old['memory_reserve_bytes'])
        self.assertEqual(new['disk_reserve_bytes'],old['disk_reserve_bytes'])
        self.assertEqual(new['disk_growth_bytes']-old['disk_growth_bytes'],policy.GIB)
        self.assertEqual(old['tmpfs_growth_bytes']-new['tmpfs_growth_bytes'],policy.GIB)
        with self.assertRaises(ResourceError): admit(capacity(),old)
        admitted=admit(capacity(),new)
        self.assertEqual(admitted['memory_admitted_bytes'],8*policy.GIB)
        self.assertEqual(admitted['disk_admitted_bytes'],23*policy.GIB//4)
        self.assertEqual(capacity().memory_available_bytes-admitted['memory_admitted_bytes'],660131840)
        for resource in (capacity(available=8*policy.GIB-1),capacity(available=3192209408),
                         capacity(disk=23*policy.GIB//4-1)):
            with self.assertRaises(ResourceError): admit(resource,new)
        self.assertEqual(policy.requirements(recovery=True),dict(disk_growth_bytes=policy.GIB,
            retained_copy_bytes=policy.GIB//4,disk_reserve_bytes=2*policy.GIB,memory_peak_bytes=6*policy.GIB,
            tmpfs_growth_bytes=policy.GIB,memory_reserve_bytes=2*policy.GIB))

    def test_parent_cached_temp_and_real_children_use_private_storage_and_restore(self):
        record=self.prepare();self.check(record)
        foreign=self.base/'foreign';foreign.mkdir()
        old=tempfile.tempdir
        with patch.dict(os.environ,{name:str(foreign) for name in policy.TEMP_VARIABLES}):
            tempfile.tempdir=str(foreign)
            try:
                with self.assertRaisesRegex(RuntimeError,'fixture cleanup'),policy.parent_temporary(record):
                    with tempfile.TemporaryFile() as stream:
                        stream.write(b'parent temporary bytes')
                    with tempfile.NamedTemporaryFile() as stream:
                        self.assertEqual(Path(stream.name).parent,self.output/'temporary')
                    env=os.environ.copy()|record['environment']
                    raw=subprocess.check_output([sys.executable,'-c',
                        'import json,os,tempfile; f=tempfile.NamedTemporaryFile(); '
                        'print(json.dumps({"file":f.name,"vars":dict((n,os.environ[n]) for n in '
                        '["TMPDIR","TMP","TEMP","RUSTC_TMPDIR","SIFR_CACHE_DIR","CLANG_MODULE_CACHE_PATH","CARGO_HOME",'
                        '"CARGO_TARGET_DIR","CARGO_NET_OFFLINE","CARGO_INCREMENTAL","PYTHONDONTWRITEBYTECODE"])}))'],env=env,text=True)
                    result=json.loads(raw)
                    self.assertEqual(Path(result['file']).parent,self.output/'temporary')
                    self.assertEqual(result['vars'],record['environment'])
                    raw=subprocess.check_output(['/bin/sh','-c','mktemp -d "$TMPDIR/package.XXXXXX"'],env=env,text=True)
                    self.assertEqual(Path(raw.strip()).parent,self.output/'temporary')
                    raise RuntimeError('fixture cleanup')
            finally:
                self.assertEqual(tempfile.tempdir,str(foreign))
                self.assertTrue(all(os.environ[name]==str(foreign) for name in policy.TEMP_VARIABLES))
                tempfile.tempdir=old

    def test_known_override_paths_and_unknown_cargo_configs_are_rejected(self):
        for name in ('CARGO_BUILD_BUILD_DIR','CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER','CC',
                     'HOST_CC','CFLAGS_x86_64_apple_darwin','SCCACHE_DIR','CARGO_TARGET_TMPDIR'):
            with self.subTest(name=name),self.assertRaisesRegex(ValueError,'configuration'):
                policy.configuration(self.root,self.cargo,self.env|{name:'/foreign'})
        for path in (self.cargo/'config',self.cargo/'config.toml',self.base/'.cargo/config.toml',
                     self.root/'.cargo/config'):
            path.parent.mkdir(exist_ok=True)
            path.write_text('[build]\ntarget-dir="/foreign"\n')
            try:
                with self.subTest(path=path),self.assertRaisesRegex(ValueError,'configuration'):
                    policy.configuration(self.root,self.cargo,self.env)
            finally: path.unlink()
        self.config.write_text('[env]\nTMPDIR = { value = "/foreign", force = true }\n')
        with self.assertRaisesRegex(ValueError,'configuration'):
            policy.configuration(self.root,self.cargo,self.env)

    def test_unknown_or_different_topology_and_links_reject(self):
        record=self.prepare()
        with patch.object(policy,'storage',side_effect=ValueError('unknown backing store')):
            with self.assertRaisesRegex(ValueError,'unknown'): self.check(record)
        with patch.object(policy,'storage',side_effect=lambda path: DISK if path!=self.cargo else DISK|{'storage_device':'/dev/disk9'}):
            with self.assertRaisesRegex(ValueError,'different physical'): self.check(record)
        link=self.cargo/'registry';link.symlink_to(self.output,target_is_directory=True)
        with self.assertRaisesRegex(ValueError,'escapes'): self.check(record)
        link.unlink()
        link=self.output/'temporary'/'outside';link.symlink_to(self.base,target_is_directory=True)
        with self.assertRaisesRegex(ValueError,'escapes'): self.check(record)

    def test_memory_backed_and_nested_mounts_cannot_authorize_reclassification(self):
        record=self.prepare()
        altered=admit(capacity(available=14*policy.GIB,memory_backed=True),policy.requirements(physical=True))
        with self.assertRaisesRegex(ValueError,'admission differs'): self.check(record,altered)
        original=Path.lstat
        foreign=self.cargo/'registry';foreign.mkdir()
        def changed(path,*args,**kwargs):
            info=original(path,*args,**kwargs)
            if path==foreign:
                fields=list(info);fields[2]+=1
                return os.stat_result(fields)
            return info
        with patch.object(Path,'lstat',changed):
            with self.assertRaisesRegex(ValueError,'escapes'): self.check(record)

    def test_receipt_inventory_environment_config_and_accounting_are_recomputed(self):
        record=self.prepare();self.check(record)
        for field in ('paths','environment','cargo_configuration','cargo_write_paths','temporary_growth_paths','process_custody'):
            altered=copy.deepcopy(record);altered[field]={}
            with self.subTest(field=field),self.assertRaises(ValueError): self.check(altered)
        for change in ('memory','disk','total','topology'):
            admission=admit(capacity(),policy.requirements(physical=True))
            if change=='memory': admission['requirements']['memory_peak_bytes']-=1
            if change=='disk': admission['requirements']['disk_growth_bytes']-=policy.GIB
            if change=='total': admission['memory_admitted_bytes']-=1
            if change=='topology': admission['resources']['diagnostics']['storage_device']='/dev/disk9'
            with self.subTest(change=change),self.assertRaises(ValueError): self.check(record,admission)
        self.config.write_text(self.config.read_text()+'# configuration identity changed\n')
        with self.assertRaisesRegex(ValueError,'evidence differs'): self.check(record)

    def test_nonprivate_output_and_symlinked_roots_reject(self):
        record=self.prepare();self.output.chmod(0o755)
        with self.assertRaisesRegex(ValueError,'owned physical'): self.check(record)
        self.output.chmod(0o700)
        alias=self.base/'alias';alias.symlink_to(self.cargo,target_is_directory=True)
        self.env['CARGO_HOME']=str(alias)
        with self.assertRaisesRegex(ValueError,'canonical'): self.check(record)

    def test_cache_files_are_valid_but_unreadable_inventory_and_special_files_reject(self):
        (self.cargo/'.global-cache').write_bytes(b'fixture cache metadata')
        record=self.prepare();self.check(record)
        def unavailable(*args,**kwargs):
            kwargs['onerror'](PermissionError('fixture unreadable directory'))
        with patch.object(policy.os,'walk',side_effect=unavailable):
            with self.assertRaisesRegex(ValueError,'inventory is unavailable'): self.check(record)
        os.mkfifo(self.output/'temporary'/'unknown-pipe')
        with self.assertRaisesRegex(ValueError,'escapes'): self.check(record)


class CandidateStorageChecks(unittest.TestCase):
    def test_darwin_completed_candidate_independently_requires_storage_evidence(self):
        fixture=fixtures.CandidateChecks();fixture.setUp();self.addCleanup(fixture.doCleanups)
        fixture.out.chmod(0o700)
        old_target=fixture.target;fixture.target='x86_64-apple-darwin'
        fixture.report['target']=fixture.target
        artifact=fixture.report['cargo_artifact']
        artifact['executable']=artifact['executable'].replace(old_target,fixture.target)
        (fixture.out/'cargo.jsonl').write_text(json.dumps(artifact)+'\n')
        fixture.report['cargo_events_sha256']=candidate.digest(fixture.out/'cargo.jsonl')
        previous=Path(fixture.report['archive']['path'])
        archive=previous.with_name(previous.name.replace(old_target,fixture.target))
        previous.rename(archive);Path(str(previous)+'.sha256').rename(Path(str(archive)+'.sha256'))
        fixture.report['archive']['path']=str(archive)
        for command in fixture.report['commands']:
            command['argv']=candidate.commands(fixture.root,fixture.out,fixture.version,fixture.target)[command['id']]
        cargo=fixture.base/'cargo';cargo.mkdir(mode=0o700)
        with patch.dict(os.environ,{'CARGO_HOME':str(cargo)},clear=True),patch.object(policy,'storage',return_value=DISK):
            fixture.report['storage_policy']=policy.prepare(fixture.root,fixture.out,os.environ)
            fixture.report['admission']=admit(capacity(),policy.requirements(physical=True))
            fixture.check()
            for field in ('storage_policy','admission'):
                changed=copy.deepcopy(fixture.report);del changed[field]
                with self.subTest(field=field),self.assertRaisesRegex(ValueError,'physical storage admission'):
                    fixture.check(changed)
            changed=copy.deepcopy(fixture.report)
            changed['storage_policy']['environment']['TMPDIR']='/unaccounted'
            with self.assertRaisesRegex(ValueError,'evidence differs'): fixture.check(changed)
            changed=copy.deepcopy(fixture.report)
            del changed['producer_sha256']['native_preparation_storage.py']
            with self.assertRaisesRegex(ValueError,'identity differs'): fixture.check(changed)

    def test_darwin_recipe_binds_child_and_parent_paths_before_execution(self):
        fixture=fixtures.CandidateChecks();fixture.setUp();self.addCleanup(fixture.doCleanups)
        cargo=fixture.base/'cargo';cargo.mkdir(mode=0o700)
        output=fixture.base/'darwin-preparation'
        env={'CARGO_HOME':str(cargo),'PATH':os.environ['PATH']}
        calls=[]
        def fail_build(argv,**kwargs):
            calls.append(argv)
            self.assertEqual(tempfile.gettempdir(),str(output/'temporary'))
            self.assertEqual(kwargs['env']['RUSTC_TMPDIR'],str(output/'temporary'))
            self.assertEqual(kwargs['env']['SIFR_CACHE_DIR'],str(output/'sifr-cache'))
            self.assertEqual(kwargs['env']['CARGO_TARGET_DIR'],str(output/'target/sysroot_release/cargo-target'))
            self.assertEqual(kwargs['env']['CARGO_BUILD_JOBS'],'2')
            self.assertEqual(kwargs['env']['SIFR_VERIFY_DISK_FLOOR_BYTES'],str(3*policy.GIB))
            return Outcome(1,'exit',b'',b'intentional fixture failure; no compiler build',False,0.01)
        with patch.dict(os.environ,env,clear=True),patch.object(candidate.sys,'platform','darwin'), \
             patch.object(candidate,'current_host_target',return_value='x86_64-apple-darwin'), \
             patch.object(candidate,'tool_identity',return_value={'cargo':{'path':'/fixture/cargo'},'rustc':{'path':'/fixture/rustc'}}), \
             patch.object(candidate.shutil,'which',return_value='/fixture/cargo'), \
             patch.object(candidate,'resources',return_value=capacity()),patch.object(policy,'storage',return_value=DISK), \
             patch.object(candidate,'execute',side_effect=fail_build):
            with self.assertRaisesRegex(ValueError,'build-package failed'): candidate.prepare(fixture.root,output)
        report=json.loads((output/'state.json').read_text())
        self.assertEqual(len(calls),1)
        self.assertEqual(report['admission']['memory_admitted_bytes'],8*policy.GIB)
        self.assertEqual(report['status'],'failed')
        self.assertFalse((output/'receipt.json').exists())

    def test_linux_candidate_rejects_foreign_storage_policy(self):
        fixture=fixtures.CandidateChecks();fixture.setUp();self.addCleanup(fixture.doCleanups)
        fixture.report['storage_policy']={}
        with self.assertRaisesRegex(ValueError,'another target'): fixture.check()


if __name__=='__main__': unittest.main()
