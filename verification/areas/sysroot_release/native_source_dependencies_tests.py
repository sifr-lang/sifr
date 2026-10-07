"""Fail-closed controls for cold native source dependency preparation."""
import copy,json,os,tempfile,unittest
from contextlib import ExitStack
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import native_source_dependencies as deps

class SourceDependenciesTests(unittest.TestCase):
    def fixture(self,mode=None):
        stack=ExitStack();self.addCleanup(stack.close)
        root=Path(stack.enter_context(tempfile.TemporaryDirectory(prefix='sifr-native-source-deps-')))
        source=root/'source';source.mkdir()
        for name in deps.INPUTS:
            path=source/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text('original '+name+'\n')
        tools={'cargo':{'path':'/canonical/cargo'},'rustc':{'path':'/canonical/rustc'}}
        stack.enter_context(patch.object(deps,'ROOT',source))
        stack.enter_context(patch.object(deps.native_candidate,'clean_source',return_value='e'*40))
        stack.enter_context(patch.object(deps.native_candidate,'tool_identity',return_value=tools))
        stack.enter_context(patch.object(deps,'resources',return_value='cache-resources'))
        stack.enter_context(patch.dict(os.environ,{'CARGO_HOME':str(root/'cache')}))
        admission=stack.enter_context(patch.object(deps,'admit',return_value={'admitted':True}))
        calls=[]
        def execute(argv,**kwargs):
            calls.append((argv,kwargs))
            if mode=='input-drift':(source/'Cargo.lock').write_text('drift\n')
            return SimpleNamespace(cause='safety_deadline' if mode=='timeout' else 'exit',
                returncode=101 if mode=='nonzero' else 0,truncated=(mode=='truncated'),stdout=b'raw stdout\n',stderr=b'raw stderr\n')
        stack.enter_context(patch.object(deps,'execute',side_effect=execute))
        return source,root/'output',calls,admission

    def test_real_command_order_bounds_and_zero_assertion_identity(self):
        source,out,calls,admission=self.fixture();report=deps.prepare(source,out)
        self.assertEqual(deps.check(out/'receipt.json'),report)
        self.assertEqual(report['runtime_assertions'],0)
        self.assertEqual([argv for argv,_ in calls],deps.commands('/canonical/cargo'))
        for _,kwargs in calls:
            self.assertEqual(kwargs['cwd'],source)
            self.assertEqual((kwargs['deadline_seconds'],kwargs['limit_bytes']),(900,16*1024**2))
            self.assertEqual(kwargs['env']['CARGO_NET_OFFLINE'],'false')
            self.assertEqual(kwargs['env']['SIFR_VERIFY_DISK_FLOOR_BYTES'],str(3*1024**3))
        self.assertEqual(admission.call_args.args[1]['disk_reserve_bytes'],2*1024**3)
        self.assertEqual(admission.call_args.args[1]['memory_reserve_bytes'],2*1024**3)

    def test_failed_processes_and_input_drift_never_publish(self):
        for mode in ('timeout','nonzero','truncated','input-drift'):
            with self.subTest(mode=mode):
                source,out,_,_=self.fixture(mode)
                with self.assertRaises(ValueError):deps.prepare(source,out)
                self.assertFalse((out/'receipt.json').exists())
                report=json.loads((out/'state.json').read_text());self.assertEqual(report['status'],'failed')
                self.assertEqual(report['runtime_assertions'],0)
                self.assertEqual(len(report['commands']),1)
                self.assertEqual((out/'0.stderr').read_bytes(),b'raw stderr\n')

    def test_resealed_incomplete_or_forged_reports_are_rejected(self):
        source,out,_,_=self.fixture();valid=deps.prepare(source,out)
        mutations=[lambda r:r.update(runtime_assertions=False),lambda r:r['commands'][0].update(returncode=False),
                   lambda r:r['commands'].pop(),lambda r:r.update(source_commit='f'*40),
                   lambda r:r.update(producer_sha256='0'*64),lambda r:r['tools']['cargo'].update(path='/foreign/cargo')]
        for mutation in mutations:
            report=copy.deepcopy(valid);mutation(report)
            for name in ('state.json','receipt.json'):(out/name).write_text(json.dumps(report))
            with self.assertRaises(ValueError):deps.check(out/'receipt.json')

    def test_raw_bytes_and_source_inputs_must_stay_unchanged(self):
        source,out,_,_=self.fixture();deps.prepare(source,out)
        (out/'0.stdout').write_bytes(b'substituted')
        with self.assertRaises(ValueError):deps.check(out/'receipt.json')
        (out/'0.stdout').write_bytes(b'raw stdout\n');(source/'Cargo.toml').write_text('drift')
        with self.assertRaises(ValueError):deps.check(out/'receipt.json')

    def test_admission_failure_is_preserved_without_success(self):
        source,out,_,admission=self.fixture();admission.side_effect=ValueError('capacity unavailable')
        with self.assertRaises(ValueError):deps.prepare(source,out)
        self.assertEqual(json.loads((out/'state.json').read_text())['status'],'failed')
        self.assertFalse((out/'receipt.json').exists())

if __name__=='__main__':unittest.main()
