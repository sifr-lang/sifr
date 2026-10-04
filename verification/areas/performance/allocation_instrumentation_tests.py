"""Real allocator events and report failure cannot be inferred from RSS."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from allocation_instrumentation import FIELDS, instrument, validate_counts


class AllocationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def build(self, source):
        path = self.root/'main.rs'
        path.write_bytes(instrument(source.encode()))
        binary = self.root/'program'
        subprocess.run(['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=generic',
                        str(path),'-o',str(binary)],check=True,capture_output=True,timeout=60)
        return binary

    def test_actual_alloc_zero_realloc_dealloc_events_are_exact(self):
        source = '''fn main() {
    use std::alloc::{GlobalAlloc, Layout, System};
    let layout = Layout::from_size_align(8, 8).unwrap();
    unsafe {
        let first = std::alloc::alloc(layout);
        assert!(!first.is_null());
        let larger = std::alloc::realloc(first, layout, 32);
        assert!(!larger.is_null());
        std::hint::black_box(larger);
        std::alloc::dealloc(larger, Layout::from_size_align(32,8).unwrap());
        let zeroed = std::alloc::alloc_zeroed(layout);
        assert!(!zeroed.is_null());
        std::hint::black_box(zeroed);
        std::alloc::dealloc(zeroed, layout);
    }
}'''
        binary = self.build(source)
        receipt = self.root/'counts.json'
        subprocess.run([str(binary)],env=os.environ|{'SIFR_ALLOCATION_RECEIPT':str(receipt)},check=True,timeout=30)
        self.assertEqual(validate_counts(json.loads(receipt.read_text())),dict(
            allocation_calls=2,allocation_requested_bytes=16,reallocation_calls=1,
            reallocation_requested_bytes=32,deallocation_calls=2))

    def test_zero_events_and_unwritable_existing_report_fail_closed(self):
        binary = self.build('fn main() { std::hint::black_box(42); }')
        receipt = self.root/'counts.json'
        env=os.environ|{'SIFR_ALLOCATION_RECEIPT':str(receipt)}
        subprocess.run([str(binary)],env=env,check=True,timeout=30)
        self.assertEqual(set(validate_counts(json.loads(receipt.read_text())).values()),{0})
        self.assertEqual(subprocess.run([str(binary)],env=env,timeout=30).returncode,2)
        env.pop('SIFR_ALLOCATION_RECEIPT')
        self.assertEqual(subprocess.run([str(binary)],env=env,timeout=30).returncode,2)

    def test_ambiguous_entrypoint_and_invalid_reports_are_rejected(self):
        for source in (b'fn other() {}',b'fn main() {}\nfn main() {}',b'fn main() {} // global_allocator'):
            with self.assertRaises(ValueError): instrument(source)
        valid={'schema_version':1,**dict.fromkeys(FIELDS,0)}
        for payload in ({}, valid|{'schema_version':True}, valid|{'allocation_calls':False},
                        valid|{'allocation_calls':-1}, valid|{'allocation_requested_bytes':1},
                        valid|{'reallocation_calls':1}, valid|{'deallocation_calls':2**64}):
            with self.assertRaises(ValueError): validate_counts(payload)


if __name__ == '__main__': unittest.main()
