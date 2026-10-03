"""Exercise real native observations without claiming Sifr compilation evidence."""
import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from datetime import datetime, timezone, timedelta
import shutil

import generated_program_metrics as programs
from measurement_timer import managed_timer_identity
from benchmark_manifest import BenchmarkError
from sifr_verify.process_execution import execute
import program_evidence


class ProgramTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary=tempfile.TemporaryDirectory(prefix='sifr-program-controls-')
        cls.root=Path(cls.temporary.name)
        cls.binary=cls.root/'native-fixture'
        source=b'#include <stdio.h>\nint main(void) { puts("42"); return 0; }\n'
        result=execute(['cc','-O2','-x','c','-o',str(cls.binary),'-'],cwd=cls.root,input_bytes=source,deadline_seconds=60)
        if result.returncode or result.cause!='exit':
            raise AssertionError(result)
        cls.timer=managed_timer_identity()
        cls.case={'expected_stdout':'42\n','work_units':1,'required_metrics':['startup_lifecycle_ms']}

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def output(self,name):
        return self.root/self.id().split('.')[-1]/name

    def test_real_native_startup_output_size_and_raw_counter_binding(self):
        row=programs.sample(self.binary,self.case,self.output('sample'),self.timer)
        self.assertGreater(row['metrics']['startup_lifecycle_ms'],0)
        self.assertEqual(row['metrics']['binary_size_bytes'],self.binary.stat().st_size)
        self.assertIsNone(row['metrics']['allocation_count'])
        self.assertEqual(row['binary_sha256'],programs.digest(self.binary))
        self.assertEqual({Path(item['path']).name for item in row['raw_files']},
                         {'stdout','stderr','timer.txt','control.json'})
        self.assertEqual(row['observations'],1)
        self.assertEqual(row['inner_timing_samples'],1)

    def test_wrong_output_and_changed_program_fail(self):
        bad=self.case|{'expected_stdout':'wrong\n'}
        with self.assertRaises(BenchmarkError):
            programs.sample(self.binary,bad,self.output('wrong'),self.timer)
        mutable=self.output('mutable');mutable.parent.mkdir(exist_ok=True,parents=True)
        mutable.write_bytes(self.binary.read_bytes());mutable.chmod(0o700)
        original=programs.run_owned_process
        def changed(*args,**kwargs):
            result=original(*args,**kwargs)
            with mutable.open('ab') as stream: stream.write(b'changed bytes')
            return result
        with patch.object(programs,'run_owned_process',changed),self.assertRaises(BenchmarkError):
            programs.sample(mutable,self.case,self.output('drift'),self.timer)

    def test_registered_counts_source_hashes_and_no_allocation_invention(self):
        contract=programs.policy()
        self.assertEqual(contract['levels']['full'],{'warmups':2,'measured':25})
        self.assertEqual(contract['allocation_metrics']['status'],'unavailable')
        with patch.object(programs,'digest',return_value='changed'),self.assertRaises(BenchmarkError):
            programs.policy()

    def test_required_missing_metric_is_not_zero(self):
        case=self.case|{'required_metrics':['allocation_count']}
        with self.assertRaisesRegex(BenchmarkError,'unavailable'):
            programs.sample(self.binary,case,self.output('missing'),self.timer)

    def test_runtime_dependencies_bind_resolved_bytes(self):
        rows=programs.dependencies(self.binary)
        self.assertTrue(rows)
        self.assertTrue(all(Path(row['path']).is_file() and len(row['sha256'])==64 for row in rows))

    def test_actual_wrapper_programs_compile_and_record_commands(self):
        root=self.output('wrappers');root.mkdir(parents=True)
        cargo,rustc,cargo_events,rustc_events=programs.write_wrappers(root)
        environment=os.environ.copy()|{'PROGRAM_CARGO':shutil.which('cargo'),
            'PROGRAM_CARGO_EVENTS':str(cargo_events),'PROGRAM_RUSTC':shutil.which('rustc'),
            'PROGRAM_RUSTC_EVENTS':str(rustc_events)}
        for wrapper in (cargo,rustc):
            result=execute([str(wrapper),'--version'],cwd=root,env=environment,deadline_seconds=30)
            self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual(json.loads(rustc_events.read_text()),['--version'])

    def test_capture_recomputes_raw_counts_summary_freshness_and_drift(self):
        root=self.output('evidence');root.mkdir(parents=True)
        compiler=root/'compiler.json';compiler.write_text(json.dumps({'lane':'test-only'}))
        contract=copy.deepcopy(programs.policy())
        contract['cases']=[self.case|{'id':'startup','source_sha256':'test-only'}]
        observed=programs.sample(self.binary,self.case,root/'startup/0',self.timer)
        observed.update(warmup=False,process_index=0)
        program={'id':'startup','binary':str(self.binary),'binary_sha256':programs.digest(self.binary),
                 'source_sha256':'test-only','runtime_dependencies':programs.dependencies(self.binary)}
        prepared=root/'prepared.json'
        prepared.write_text(json.dumps({'kind':'preparation-output','runtime_assertions':0,'optimization':'release',
            'target_cpu':'generic','compiler_receipt':str(compiler),'compiler_receipt_sha256':programs.digest(compiler),
            'policy_sha256':programs.digest(programs.POLICY),'programs':[program]}))
        duration=observed['metrics']['startup_lifecycle_ms'];now=datetime.now(timezone.utc)
        value={'schema_version':1,'protocol':contract['protocol'],'status':'captured','level':'smoke',
            'numeric_regression_qualification':False,'prepared_path':str(prepared),'prepared_sha256':programs.digest(prepared),
            'policy_sha256':programs.digest(programs.POLICY),'producer':program_evidence.producer_identity(programs.ROOT),
            'timer':self.timer,'started_utc':(now-timedelta(seconds=10)).isoformat(),'finished_utc':now.isoformat(),
            'rows':[{'id':'startup','binary_sha256':program['binary_sha256'],'process_count':1,
                     'observations':[observed],'summary':{'startup_lifecycle_ms':{'median':duration,'empirical_p95':duration}}}]}
        receipt=root/'receipt.json'
        def check(changed):
            receipt.write_text(json.dumps(changed))
            with patch.object(program_evidence,'validate_receipt'):
                return program_evidence.validate_capture(receipt,contract,programs.ROOT)
        self.assertEqual(check(value)['status'],'captured')
        changes=[]
        changed=copy.deepcopy(value);changed['rows'][0]['summary']['startup_lifecycle_ms']['median']+=1;changes.append(changed)
        changed=copy.deepcopy(value);changed['rows'][0]['observations']=[];changes.append(changed)
        changed=copy.deepcopy(value);changed['rows'][0]['observations'][0]['metrics']['binary_size_bytes']+=1;changes.append(changed)
        changed=copy.deepcopy(value);changed['status']='failed';changes.append(changed)
        changed=copy.deepcopy(value);changed['finished_utc']=(now-timedelta(days=2)).isoformat();changes.append(changed)
        for changed in changes:
            with self.assertRaises(BenchmarkError): check(changed)
        (root/'startup/0/stdout').write_text('changed\n')
        with self.assertRaises(BenchmarkError): check(value)


if __name__=='__main__':
    unittest.main()
