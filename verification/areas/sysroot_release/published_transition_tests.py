"""Independent raw-evidence controls using synthetic records; no native qualification."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

import published_transition as transition
from sifr_verify.errors import VerificationError


class RuntimeEvidenceChecks(unittest.TestCase):
    def setUp(self):
        self.temporary=tempfile.TemporaryDirectory();self.addCleanup(self.temporary.cleanup)
        self.out=Path(self.temporary.name)
        self.candidate={'version':'0.1.0-beta.17','installer':{'path':'/unit/native-installer'}}
        self.report={'runtime_assertions':len(transition.CASES),'cases':[
            {'id':name,'state':'passed','phases':['selected','validated','executed','passed'],
             'execution_kind':'runtime','infrastructure':'none','executed_count':1,'elapsed_seconds':.1}
            for name in transition.CASES],'commands':[]}
        for name,argv in transition.command_inventory(self.candidate,self.out):
            negative=name in {'failed-migration','failed-reinstall'}
            stdout=b'unit fixture'
            if name.endswith('-version'): stdout=b'sifr 0.1.0-beta.17\n'
            if name.endswith('-doctor'): stdout=b'{"status":"ok","package_integrity_verified":true}\n'
            if name in ('published-user-run','published-after-rollback','candidate-user-run','reinstalled-user-run','rollback-user-run'): stdout=b'9\n'
            stderr=b'atomic_generation_switch\nwrite_install_manifest\nrollback_install_transaction\n' if negative else b''
            hashes={}
            for stream,data in [('stdout',stdout),('stderr',stderr)]:
                path=self.out/(name+'.'+stream);path.write_bytes(data);hashes[stream]=transition.digest(path)
            self.report['commands'].append({'id':name,'argv':argv,'cause':'exit','returncode':1 if negative else 0,
                'truncated':False,'expected_success':not negative,'output_sha256':hashes})

    def check(self,report=None):
        transition.check_runtime(self.report if report is None else report,self.out,self.candidate)

    def replace_raw(self,name,stream,data):
        path=self.out/(name+'.'+stream);path.write_bytes(data)
        row=next(row for row in self.report['commands'] if row['id']==name)
        row['output_sha256'][stream]=transition.digest(path)

    def test_complete_coverage_requires_runtime_execution_without_skips(self):
        self.check()
        for mutation in ('missing','skip','compile-only','zero','boolean'):
            report=copy.deepcopy(self.report)
            if mutation=='missing': report['cases'].pop()
            if mutation=='skip': report['cases'][0].update(state='skipped',phases=['selected','skipped'])
            if mutation=='compile-only': report['cases'][0]['execution_kind']='compile'
            if mutation=='zero': report['cases'][0]['executed_count']=0
            if mutation=='boolean': report['cases'][0]['executed_count']=True
            with self.subTest(mutation=mutation),self.assertRaises((ValueError,VerificationError)): self.check(report)

    def test_substituted_commands_and_incomplete_negative_processes_reject(self):
        for mutation in ('argv','deadline','truncation','missing'):
            report=copy.deepcopy(self.report)
            if mutation=='argv': report['commands'][0]['argv']=['true']
            if mutation=='deadline': report['commands'][2]['cause']='safety_deadline'
            if mutation=='truncation': report['commands'][2]['truncated']=True
            if mutation=='missing': report['commands'].pop()
            with self.subTest(mutation=mutation),self.assertRaises(ValueError): self.check(report)

    def test_consistent_hashes_cannot_replace_program_or_integrity_oracle(self):
        self.replace_raw('candidate-user-run','stdout',b'10\n')
        with self.assertRaisesRegex(ValueError,'program result'): self.check()
        self.replace_raw('candidate-user-run','stdout',b'9\n')
        self.replace_raw('upgraded-doctor','stdout',b'{"status":"ok","package_integrity_verified":false}')
        with self.assertRaisesRegex(ValueError,'integrity'): self.check()

    def test_early_failure_cannot_claim_post_switch_transaction_rollback(self):
        self.replace_raw('failed-migration','stderr',b'rollback_install_transaction\n')
        with self.assertRaisesRegex(ValueError,'post-switch'): self.check()

    def test_raw_byte_drift_is_rejected_even_with_passing_case_ledger(self):
        (self.out/'candidate-user-run.stdout').write_bytes(b'9\nextra output\n')
        with self.assertRaisesRegex(ValueError,'raw process output'): self.check()


if __name__=='__main__': unittest.main()
