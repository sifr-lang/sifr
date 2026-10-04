"""Use actual Cargo/rustc events to control application identity selection."""
import copy
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest

from benchmark_manifest import BenchmarkError
from generated_program_metrics import write_wrappers
from program_artifact_identity import application_events
from program_evidence import target_cpu_generic
from sifr_verify.process_execution import execute


class ActualCargoTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary=tempfile.TemporaryDirectory(prefix='sifr-real-cargo-events-')
        cls.root=Path(cls.temporary.name);(cls.root/'src').mkdir()
        (cls.root/'src/main.rs').write_text('fn main() { println!("42"); }\n')
        cls.target='sifr_output_0123456789abcdef'
        (cls.root/'Cargo.toml').write_text('[package]\nname="sifr_output"\nversion="0.0.0"\nedition="2024"\n'
            '[workspace]\n[profile.release]\noverflow-checks=true\n[[bin]]\nname="'+cls.target+'"\npath="src/main.rs"\n')
        cargo,rustc,events,commands=write_wrappers(cls.root)
        env=os.environ.copy()|{'PROGRAM_CARGO':shutil.which('cargo'),'PROGRAM_RUSTC':shutil.which('rustc'),
            'PROGRAM_CARGO_EVENTS':str(events),'PROGRAM_RUSTC_EVENTS':str(commands),'RUSTC':str(rustc),
            'RUSTFLAGS':'-C target-cpu=generic','CARGO_TARGET_DIR':str(cls.root/'target'),'CARGO_INCREMENTAL':'0'}
        result=execute([str(cargo),'build','--release','--offline','--message-format=json-render-diagnostics'],
                       cwd=cls.root,env=env,deadline_seconds=60,limit_bytes=16*1024**2)
        if result.cause!='exit' or result.returncode or result.truncated: raise AssertionError(result.stderr)
        cls.rows=[json.loads(line) for line in events.read_text().splitlines()]
        cls.commands=[json.loads(line) for line in commands.read_text().splitlines()]
        cls.binary=cls.root/'target/release'/cls.target

    @classmethod
    def tearDownClass(cls): cls.temporary.cleanup()

    def test_actual_hashed_cargo_target_and_rustc_bin(self):
        artifact,command=application_events(self.rows,self.commands,self.binary)
        self.assertEqual(artifact['target']['name'],self.target)
        self.assertEqual(artifact['target']['kind'],['bin'])
        self.assertEqual(artifact['profile']['opt_level'],'3')
        self.assertTrue(artifact['profile']['overflow_checks'])
        self.assertTrue(target_cpu_generic(command))

    def test_ambiguity_and_another_crate_cannot_substitute(self):
        artifact,command=application_events(self.rows,self.commands,self.binary)
        for rows,commands in (([*self.rows,artifact],self.commands),(self.rows,[*self.commands,command]),
                              (self.rows,[['--crate-name','unrelated','--crate-type','bin','-C','target-cpu=generic']])):
            with self.assertRaises(BenchmarkError): application_events(rows,commands,self.binary)
        altered=copy.deepcopy(self.rows)
        for row in altered:
            if row.get('executable'): row['target']['kind']=['custom-build']
        with self.assertRaises(BenchmarkError): application_events(altered,self.commands,self.binary)

    def test_codegen_alias_cannot_hide_a_conflicting_cpu_target(self):
        self.assertTrue(target_cpu_generic(['--codegen=target-cpu=generic']))
        self.assertFalse(target_cpu_generic(['-C','target-cpu=generic','--codegen','target-cpu=native']))


if __name__=='__main__': unittest.main()
