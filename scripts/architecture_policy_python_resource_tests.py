"""H02h3 resource policy admission; native runtime source remains read-only."""
import copy
import json
from pathlib import Path
import unittest

import check_unsafe_abi_contracts as unsafe
from rust_policy_sites import Source
from unsafe_policy_contracts import FAMILY_RULES, FIELDS, operation_family
from unsafe_policy_segments import read_segments, segment_for, validate_segments


class UnsafePythonResourceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root = Path(__file__).resolve().parent.parent
        paths = [cls.root / 'crates/sifr_runtime/src/python.rs',
                 *sorted((cls.root / 'crates/sifr_runtime/src/python').rglob('*.rs'))]
        cls.sources = [Source(p.relative_to(cls.root).as_posix(), p.read_text()) for p in paths]
        cls.all_sites = [site for source in cls.sources for site in unsafe.discover(source)]
        cls.sites = [s for s in cls.all_sites if segment_for(s) == 'python_resources']
        cls.inventory = read_segments(cls.root, selected=('python_resources',))['python_resources']
        cls.by_key = {s.key: s for s in cls.sites}

    def one_record_errors(self, record):
        return unsafe.validate([self.by_key[record['site']]], {'sites': [record]},
                               sources=self.sources)

    def test_current_resource_segment_has_complete_local_contracts(self):
        self.assertGreater(len(self.sites), 0)
        self.assertEqual(set(self.by_key), {r['site'] for r in self.inventory['sites']})
        self.assertEqual(len(self.sites), len(self.inventory['sites']))
        # Revalidate predecessor and current partitions together, explicitly
        # bounded to native Python. The external segment is not claimed ready.
        segments = read_segments(self.root, selected=('python_core', 'python_resources'))
        errors = validate_segments(self.all_sites, segments,
                                   selected=('python_core', 'python_resources'), sources=self.sources)
        self.assertEqual(errors, [], '\n'.join(errors))
        parent = next(s for s in self.sites if s.path.endswith('/python.rs'))
        self.assertEqual(parent.kind, 'unsafe-allowance')
        self.assertIn('mod arrow_ops', parent.text)
        self.assertTrue(any(s.test_only for s in self.sites))
        schema = json.loads((self.root / 'verification/policy/schemas/unsafe_abi_segment.schema.json').read_text())
        props = schema['properties']['sites']['items']['properties']
        self.assertEqual(set(props['operation_family']['enum']), set(FAMILY_RULES))
        for index, field in enumerate(FIELDS):
            self.assertEqual(set(props['obligations']['properties'][field]['properties']['rule']['enum']),
                             {rules[index] for rules in FAMILY_RULES.values()})
        for record in self.inventory['sites']:
            with self.subTest(site=record['site']):
                expected = 'H02e' if '/buffer_ops' in record['site'] else 'H02g' if '/dlpack_ops' in record['site'] else 'H02f'
                self.assertEqual(record['owner'], expected)
                for field in FIELDS:
                    changed = copy.deepcopy(record)
                    del changed['contract'][field]
                    self.assertTrue(any('missing local' in e for e in self.one_record_errors(changed)))
        for field, value, message in [('fingerprint', '0' * 64, 'changed fingerprint'),
                                       ('operation', 'unsafe { unrelated(); }', 'operation binding'),
                                       ('source_evidence', '// neighboring proof', 'SAFETY evidence')]:
            changed = copy.deepcopy(self.inventory['sites'][0])
            changed[field] = value
            self.assertTrue(any(message in e for e in self.one_record_errors(changed)))
        # Local code proofs are full-context bound, including the release
        # marker that protects explicit release followed by Drop.
        release = next(r for r in self.inventory['sites'] if r['operation_family'] == 'exporter-release')
        changed_sources = [Source(s.path, s.text.replace('raw.obj = ptr::null_mut();', 'retain_obj();'))
                           if s.path.endswith('/buffer_ops/raw.rs') else s for s in self.sources]
        self.assertTrue(any('obligation/proof' in e for e in unsafe.validate(
            [self.by_key[release['site']]], {'sites': [release]}, sources=changed_sources)))

    def test_buffer_release_cannot_claim_borrow_only_ownership(self):
        records = [r for r in self.inventory['sites'] if r['operation_family'] == 'exporter-release']
        self.assertGreater(len(records), 0)
        for original in records:
            self.assertEqual(self.one_record_errors(original), [])
            changed = copy.deepcopy(original)
            changed['contract']['ownership'] = 'Borrow the view without consuming its exporter reference.'
            changed['obligations']['ownership']['rule'] = 'borrow-only'
            self.assertTrue(any('invalid ownership obligation' in e for e in self.one_record_errors(changed)))
            self.assertEqual(changed['fingerprint'], original['fingerprint'])
            self.assertEqual(changed['source_evidence'], original['source_evidence'])
            self.assertEqual(changed['obligations']['ownership']['proofs'], original['obligations']['ownership']['proofs'])

    def test_arrow_and_dlpack_transfer_cannot_reuse_pointer_read_contract(self):
        effects = [r for r in self.inventory['sites'] if r['operation_family'] in
                   ('resource-release', 'resource-capsule-transfer', 'resource-header-initialize',
                    'resource-reference-acquire')]
        self.assertTrue(any('/arrow_ops' in r['site'] for r in effects))
        self.assertTrue(any('/dlpack_ops' in r['site'] for r in effects))
        for original in effects:
            self.assertEqual(self.one_record_errors(original), [])
            changed = copy.deepcopy(original)
            changed['operation_family'] = 'raw-access'
            changed['contract'] = {f: 'Observe an initialized raw pointer without mutation or ownership transfer.' for f in FIELDS}
            for field, rule in zip(FIELDS, FAMILY_RULES['raw-access']):
                changed['obligations'][field]['rule'] = rule
            errors = self.one_record_errors(changed)
            self.assertTrue(any('operation family mismatch' in e for e in errors))
            self.assertTrue(any('invalid ownership obligation' in e for e in errors))
            self.assertEqual(changed['fingerprint'], original['fingerprint'])
            self.assertEqual(changed['source_evidence'], original['source_evidence'])
            changed = copy.deepcopy(original)
            for obligation in changed['obligations'].values():
                obligation['proofs'] = [p for p in obligation['proofs'] if p['kind'] != 'code']
            self.assertTrue(any('requires source admission/teardown proof' in e for e in self.one_record_errors(changed)))
        # Renaming a selected release callback cannot turn its invocation into
        # raw access: lexical origin is the actual release field, not its name.
        from unsafe_policy_contracts import proof_reference
        from architecture_policy_test_fixtures import unsafe_records
        for name in ('schema_release', 'array_release', 'arbitrary_selected_callback'):
            body = f'let {name} = unsafe {{ pointer.as_ref() }}.release.expect("present"); unsafe {{ {name}(pointer.as_ptr()) }};'
            source = Source('crates/sifr_runtime/src/python/arrow_ops/release_fixture.rs',
                            'fn consume() { ' + body + ' }')
            sites = unsafe.discover(source)
            self.assertEqual([operation_family(s) for s in sites], ['raw-access', 'resource-release'])
            records = unsafe_records(sites)
            invoked = records['sites'][1]
            for obligation in invoked['obligations'].values():
                obligation['proofs'].append(proof_reference(source, 'consume', body))
            self.assertEqual(unsafe.validate(sites, records, sources=[source]), [])
            invoked['operation_family'] = 'raw-access'
            invoked['obligations']['ownership']['rule'] = 'borrow-only'
            self.assertTrue(any('invalid ownership obligation' in e for e in unsafe.validate(sites, records, sources=[source])))
        # The live consuming fixture records themselves are admitted as release,
        # independent of the set used to choose mutation subjects above.
        for ordinal in (2, 4):
            record = next(r for r in self.inventory['sites'] if r['site'].endswith(
                f'arrow_ops/tests.rs::consume_argument_pair::unsafe-block::{ordinal}'))
            self.assertEqual(record['operation_family'], 'resource-release')
        for suffix in ('buffer_ops/h02_contract_tests.rs::__getbuffer__::unsafe-block::1',
                       'buffer_ops/release_evidence_tests.rs::__getbuffer__::unsafe-block::1',
                       'buffer_ops/typed_access_evidence_tests.rs::__getbuffer__::unsafe-block::1',
                       'buffer_ops/tests.rs::indirect_byte_memoryview::unsafe-block::1'):
            record = next(r for r in self.inventory['sites'] if r['site'].endswith(suffix))
            self.assertEqual(record['operation_family'], 'resource-reference-acquire')
        # A CPython observation cannot conceal a deleter/free or capsule rename
        # in the same resource block (H02h2 refinement consumption regression).
        from architecture_policy_test_fixtures import unsafe_records
        from unsafe_policy_contracts import proof_reference
        for body, family in [('ffi::PyGILState_Check(); deleter(pointer);', 'resource-release'),
                             ('ffi::PyGILState_Check(); Box::from_raw(pointer);', 'resource-release'),
                             ('ffi::PyGILState_Check(); ffi::PyCapsule_SetName(capsule, used);', 'resource-capsule-transfer')]:
            source = Source('crates/sifr_runtime/src/python/dlpack_ops/mixed_fixture.rs',
                            'fn mixed() { unsafe { ' + body + ' } }')
            sites = unsafe.discover(source)
            self.assertEqual([operation_family(s) for s in sites], [family])
            records = unsafe_records(sites)
            for obligation in records['sites'][0]['obligations'].values():
                obligation['proofs'].append(proof_reference(source, 'mixed', body))
            self.assertEqual(unsafe.validate(sites, records, sources=[source]), [])
            records['sites'][0]['obligations']['ownership']['rule'] = 'borrow-only'
            self.assertTrue(any('invalid ownership obligation' in e for e in unsafe.validate(sites, records, sources=[source])))
