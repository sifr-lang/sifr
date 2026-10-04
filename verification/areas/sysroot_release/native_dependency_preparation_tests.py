"""Control preparation claims and immutable dependency evidence independently."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
import native_dependency_preparation as dependencies
from published_predecessor import digest


class DependencyEvidence(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        root=Path(self.tmp.name);self.package=root/'package';self.package.mkdir();self.out=root/'preparation';self.out.mkdir()
        for name in dependencies.INPUTS:
            path=self.package/name;path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text('[features]\nmath=[]\n' if name.endswith('sifr_stdlib/Cargo.toml') else 'unit input')
        (self.out/'Cargo.toml').write_text(dependencies.manifest(self.package));(self.out/'Cargo.lock').write_text('unit prepared lock')
        self.report=dict(kind='preparation-output',status='prepared',runtime_assertions=0,
            published_inputs_sha256=dependencies.inputs(self.package),manifest_sha256=digest(self.out/'Cargo.toml'),
            lock_sha256=digest(self.out/'Cargo.lock'),commands=[])
        for index,argv in enumerate(dependencies.commands(self.out)):
            raw={}
            for stream in ('stdout','stderr'):
                path=self.out/(str(index)+'.'+stream);path.write_text('unit raw output');raw[stream]=digest(path)
            self.report['commands'].append(dict(argv=argv,cause='exit',returncode=0,truncated=False,output_sha256=raw))

    def check(self,report=None):
        report=self.report if report is None else report
        (self.out/'state.json').write_text(json.dumps(report))
        dependencies.check(report,self.out,self.package,self.package)

    def test_complete_preparation_can_never_be_runtime_qualification(self):
        self.check()
        for field,value in [('status','incomplete'),('kind','runtime'),('runtime_assertions',1),('runtime_assertions',False)]:
            r=copy.deepcopy(self.report);r[field]=value
            with self.subTest(field=field),self.assertRaises(ValueError):self.check(r)

    def test_missing_substituted_or_unfinished_commands_reject(self):
        for change in ('missing','argv','cause','exit','truncated','boolean-exit'):
            r=copy.deepcopy(self.report)
            if change=='missing':r['commands'].pop()
            if change=='argv':r['commands'][0]['argv']=['true']
            if change=='cause':r['commands'][0]['cause']='safety_deadline'
            if change=='exit':r['commands'][0]['returncode']=1
            if change=='boolean-exit':r['commands'][0]['returncode']=False
            if change=='truncated':r['commands'][0]['truncated']=True
            with self.subTest(change=change),self.assertRaises(ValueError):self.check(r)

    def test_published_input_prepared_lock_and_raw_byte_drift_reject(self):
        for path in (self.package/'Cargo.lock',self.out/'Cargo.lock',self.out/'0.stdout'):
            before=path.read_bytes();path.write_text('changed')
            with self.subTest(path=path),self.assertRaises(ValueError):self.check()
            path.write_bytes(before)

    def test_rehashed_substitute_manifest_still_rejects(self):
        (self.out/'Cargo.toml').write_text('different dependencies')
        self.report['manifest_sha256']=digest(self.out/'Cargo.toml')
        with self.assertRaises(ValueError):self.check()


if __name__=='__main__':unittest.main()
