"""Real Cargo collector control; CPP identity is an explicitly stubbed boundary."""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import generated_program_allocations as allocations
from benchmark_manifest import BenchmarkError


class CollectorTests(unittest.TestCase):
    def test_real_cargo_collection_and_independent_rejection_controls(self):
        with tempfile.TemporaryDirectory(dir=allocations.ROOT.parent) as temporary:
            root=Path(temporary)
            project=root/'original';(project/'src').mkdir(parents=True)
            source=project/'src/main.rs'
            source.write_text('fn main() { println!("42"); }\n')
            target='sifr_output_1234567890abcdef'
            manifest=project/'Cargo.toml'
            manifest.write_text('[package]\nname="sifr_output"\nversion="0.1.0"\nedition="2024"\n'
                '[workspace]\n[[bin]]\nname="'+target+'"\npath="src/main.rs"\n'
                '[profile.release]\noverflow-checks=true\n')
            events=root/'rustc-invocations.jsonl'
            wrapper=root/'original-rustc'
            wrapper.write_text('#!'+sys.executable+'\n'+'''import json,os,sys
with open(os.environ['CONTROL_EVENTS'],'a') as output:
    output.write(json.dumps(sys.argv[2:])+'\\n')
os.execv(sys.argv[1],sys.argv[1:])
''');wrapper.chmod(0o700)
            env=os.environ|{'RUSTFLAGS':'-C target-cpu=generic','RUSTC_WRAPPER':str(wrapper),
                            'CONTROL_EVENTS':str(events),'CARGO_TARGET_DIR':str(root/'original-target')}
            subprocess.run(['cargo','generate-lockfile','--offline','--manifest-path',str(manifest)],
                           env=env,check=True,capture_output=True,timeout=30)
            result=subprocess.run(['cargo','build','--locked','--offline','--release','--manifest-path',str(manifest),
                                   '--message-format=json-render-diagnostics'],env=env,check=True,capture_output=True,timeout=60)
            cargo=root/'cargo-artifacts.jsonl';cargo.write_bytes(result.stdout)
            binary=root/'original-target/release'/target
            artifact,invocation=allocations.application_events([json.loads(line) for line in result.stdout.decode().splitlines() if line.startswith('{')],
                [json.loads(line) for line in events.read_text().splitlines()],binary)
            sifr=root/'control.sifr';sifr.write_text('print(42)\n')
            # This fixture tests collection and checking, not Sifr compilation.
            compiler=root/'compiler.json';compiler.write_text('{"lane":"control-boundary"}')
            case={'id':'control','source_sha256':allocations.digest(sifr),'expected_stdout':'42\n'}
            program={'id':'control','binary':str(binary),'binary_sha256':allocations.digest(binary),
                     'source_sha256':case['source_sha256'],'compiled_source_path':str(sifr),
                     'generated_rust_sha256':allocations.digest(source),'cargo_artifact':artifact,'rustc_command':invocation,
                     'cargo_events_range':[0,cargo.stat().st_size],'rustc_events_range':[0,events.stat().st_size]}
            prepared=root/'prepared.json'
            prepared.write_text(json.dumps({'schema_version':1,'kind':'preparation-output','runtime_assertions':0,
                'optimization':'release','target_cpu':'generic','policy_sha256':allocations.digest(allocations.POLICY),
                'compiler_receipt':str(compiler),'compiler_receipt_sha256':allocations.digest(compiler),
                'programs':[program],'cargo_events_sha256':allocations.digest(cargo),'rustc_events_sha256':allocations.digest(events)}))
            with patch.object(allocations,'require_clean_source'), patch.object(allocations,'validate_receipt'), \
                 patch.object(allocations,'policy',return_value={'cases':[case]}):
                output=root/'capture'
                captured=allocations.collect(prepared,output)
                receipt=output/'receipt.json'
                self.assertEqual(captured['status'],'captured')
                self.assertIs(captured['numeric_regression_qualification'],False)
                self.assertGreater(captured['rows'][0]['counts']['allocation_calls'],0)
                self.assertEqual(allocations.check(receipt)['status'],'captured')
                original_receipt=receipt.read_bytes()
                for field in ('status','counts','artifact','source','raw'):
                    altered=copy.deepcopy(captured)
                    if field=='status': altered['status']='failed'
                    elif field=='counts': altered['rows'][0]['counts']['allocation_calls']+=1
                    elif field=='artifact': altered['rows'][0]['binary_sha256']='0'*64
                    elif field=='source': altered['rows'][0]['instrumented_rust_sha256']='0'*64
                    else: altered['rows'][0]['observation_process']['output']['stdout']['sha256']='0'*64
                    receipt.write_text(json.dumps(altered))
                    with self.assertRaises(BenchmarkError): allocations.check(receipt)
                receipt.write_bytes(original_receipt)
                generated=output/'control/project/src/main.rs'
                saved=generated.read_bytes();generated.write_bytes(saved+b'// altered\n')
                with self.assertRaises(BenchmarkError): allocations.check(receipt)
                generated.write_bytes(saved)
                counts=output/'control/counts.json'
                saved=counts.read_bytes();counts.write_text('{}')
                with self.assertRaises(BenchmarkError): allocations.check(receipt)
                counts.write_bytes(saved)
                with patch.object(allocations,'checked',side_effect=BenchmarkError('controlled incomplete process')):
                    failed_output=root/'failed'
                    with self.assertRaises(BenchmarkError): allocations.collect(prepared,failed_output)
                self.assertFalse((failed_output/'receipt.json').exists())
                failed=json.loads((failed_output/'state.json').read_text())
                self.assertEqual(failed['status'],'failed')
                self.assertIs(failed['numeric_regression_qualification'],False)


if __name__=='__main__': unittest.main()
