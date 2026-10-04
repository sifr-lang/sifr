"""Synthetic custody controls; never claim paired measurement execution."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch
from . import cloud_receipt_reuse as reuse
from .errors import VerificationError
from .profile_commands import CommandFailed


class ReuseChecks(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='sifr-cloud-reuse-control-')
        self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.source=self.root/'source';self.source.mkdir()
        self.env=os.environ.copy()
        self.git(self.source,'init','-q')
        self.git(self.source,'config','user.name','Control')
        self.git(self.source,'config','user.email','control@example.invalid')
        (self.source/'AGENTS.md').write_text('old docs\n')
        (self.source/'.gitignore').write_text('target/\n')
        (self.source/'crates').mkdir();(self.source/'crates/compiler.rs').write_text('source\n')
        self.git(self.source,'add','.');self.git(self.source,'commit','-qm','fixture')
        self.observed=self.git(self.source,'rev-parse','HEAD')
        self.candidate=self.root/'candidate'
        self.git(self.source,'worktree','add','-qb','candidate',str(self.candidate))
        (self.candidate/'AGENTS.md').write_text('new docs\n')
        self.git(self.candidate,'commit','-qam','docs')
        self.compiler=self.root/'compiler'
        self.identity={'identity_kind':'product','compiler_build_id':'a'*64}
        self.compiler.write_text('#!'+sys.executable+'\nimport json\nprint('+repr(json.dumps(self.identity))+')\n')
        self.compiler.chmod(0o700)
        self.context={'interpreter_sha256':'fixture','ambient_inputs':{}}
        self.receipt=self.root/'paired-receipt.json'
        endpoint={'cloud_repo':str(self.source),'cloud_source':self.observed,
                  'embedded_compatibility_identity':self.identity,
                  'artifact':{'sha256':reuse.digest(self.compiler)},
                  'cloud_identity':{'execution':{'cloud_python_context':self.context}}}
        self.receipt.write_text(json.dumps({'specification':{'endpoints':{'candidate':endpoint}}}))

    def git(self,root,*args):
        return subprocess.check_output(['git',*args],cwd=root,stderr=subprocess.PIPE,text=True).strip()

    def consume(self,checker=lambda *a,**k:None,context=None):
        original=reuse.command
        def command(argv,root,env,**kwargs):
            if argv[:2]==[sys.executable,'-c']:
                return json.dumps(self.context if context is None else context)
            return original(argv,root,env,**kwargs)
        with patch.object(reuse,'command',side_effect=command):
            return reuse.consume(self.receipt,self.source,self.candidate,self.compiler,self.env,checker)

    def test_valid_non_measurement_reuse_keeps_original_bytes_and_commit(self):
        before=self.receipt.read_bytes();calls=[]
        result=self.consume(lambda argv,**kwargs:calls.append(argv))
        self.assertEqual(self.receipt.read_bytes(),before)
        self.assertEqual(result['proof']['observed_commit'],self.observed)
        self.assertEqual(result['proof']['candidate_commit'],self.git(self.candidate,'rev-parse','HEAD'))
        self.assertEqual(result['runtime_assertions_executed'],0)
        self.assertEqual(len(calls),1)
        self.assertEqual(calls[0][1],str(self.source/'verification/areas/performance/cloud_benchmarks.py'))
        self.assertEqual(calls[0][-1],str(self.receipt))

    def test_unknown_core_harness_lock_and_submodule_changes_are_rejected(self):
        for name in ('crates/compiler.rs','verification/areas/performance/new.py','Cargo.lock','.gitmodules','\nAGENTS.md'):
            with self.subTest(name=name):
                path=self.candidate/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text('drift')
                self.git(self.candidate,'add','.');self.git(self.candidate,'commit','-qm','drift')
                with self.assertRaisesRegex(VerificationError,'dependency closure changed'):
                    self.consume()
                self.git(self.candidate,'revert','--no-edit','HEAD')

    def test_ignored_runtime_source_drift_and_sparse_trees_are_rejected(self):
        (self.source/'.git/info/exclude').write_text('crates/ignored.rs\n')
        (self.candidate/'crates/ignored.rs').write_text('untracked runtime drift')
        with self.assertRaisesRegex(VerificationError,'live compiler/runtime/corpus'):
            self.consume()
        (self.candidate/'crates/ignored.rs').unlink()
        self.git(self.candidate,'config','core.sparseCheckout','true')
        with self.assertRaisesRegex(VerificationError,'full source worktrees'):
            self.consume()

    def test_compiler_bytes_identity_and_python_context_are_bound(self):
        with self.assertRaisesRegex(VerificationError,'Python context'):
            self.consume(context={'interpreter_sha256':'other'})
        old=self.compiler.read_text();self.compiler.write_text(old+'# changed bytes\n')
        with self.assertRaisesRegex(VerificationError,'compiler bytes'):
            self.consume()
        self.compiler.write_text(old.replace('a'*64,'b'*64))
        with self.assertRaisesRegex(VerificationError,'build identity'):
            self.consume()

    def test_original_checker_regression_and_drift_preserve_failure(self):
        def reject(*args,**kwargs):raise CommandFailed(1)
        with self.assertRaises(CommandFailed):self.consume(reject)
        def drift(*args,**kwargs):(self.candidate/'AGENTS.md').write_text('drift after checking')
        with self.assertRaisesRegex(VerificationError,'clean source'):
            self.consume(drift)
        states=list((self.candidate/'target/validation_lane_reports/cloud-performance-reuse').glob('*/failure.json'))
        self.assertEqual(len(states),2)
        self.assertTrue(all(json.loads(p.read_text())['status']=='failed' for p in states))
        self.assertFalse(any(p.with_name('receipt.json').exists() for p in states))

    def test_foreign_or_self_source_is_rejected(self):
        with self.assertRaisesRegex(VerificationError,'observed source worktree'):
            reuse.equivalence(self.source,self.source,self.observed,self.env)
        foreign=self.root/'foreign';foreign.mkdir();self.git(foreign,'init','-q')
        self.git(foreign,'config','user.name','Control');self.git(foreign,'config','user.email','control@example.invalid')
        (foreign/'file').write_text('fixture');self.git(foreign,'add','.');self.git(foreign,'commit','-qm','fixture')
        with self.assertRaisesRegex(VerificationError,'same owned Git repository'):
            reuse.equivalence(self.source,foreign,self.observed,self.env)

    def test_explicit_profile_route_keeps_required_failure_outcomes(self):
        from .cloud_profile import run_cloud_profile
        variables={'SIFR_CLOUD_PERFORMANCE_RECEIPT':str(self.receipt),
                   'SIFR_CLOUD_PERFORMANCE_SOURCE_WORKTREE':str(self.source)}
        for error,expected in ((None,0),(VerificationError('rejected'),3),(CommandFailed(1),1)):
            runner=SimpleNamespace(run=lambda:0,env={'SIFR_GCQ_BIN':str(self.compiler)},
                                   require_performance=True,performance_exit_status=0)
            with self.subTest(error=error),patch.dict(os.environ,variables,clear=True), \
                 patch.object(reuse,'consume',side_effect=error) as consumer:
                self.assertEqual(run_cloud_profile(runner),expected)
                consumer.assert_called_once()


def policy_checks():
    result=unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(ReuseChecks).run(result)
    if not result.wasSuccessful():raise AssertionError(result.errors+result.failures)


if __name__=='__main__':unittest.main()
