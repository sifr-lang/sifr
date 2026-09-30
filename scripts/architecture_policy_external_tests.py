"""H02h4 exact unsafe union and source-owned external contracts; no runtime audit."""
import copy
import json
from pathlib import Path
import unittest

import check_unsafe_abi_contracts as unsafe
from rust_policy_sites import Source, rust_sources
from unsafe_policy_contracts import FAMILY_RULES, FIELDS, operation_family, proof_reference
from unsafe_policy_segments import SEGMENTS, read_segments, segment_for, validate_segments


class UnsafeExternalOwnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root = Path(__file__).resolve().parent.parent
        # Independent full first-party discovery: no stored site counts, path
        # list or inventory owner labels select the source union.
        cls.sources = list(rust_sources(cls.root))
        cls.sites = [site for source in cls.sources for site in unsafe.discover(source)]
        cls.by_key = {site.key: site for site in cls.sites}
        cls.segments = read_segments(cls.root)
        cls.inventory = cls.segments['external']

    def one_record_errors(self, record):
        return unsafe.validate([self.by_key[record['site']]], {'sites': [record]},
                               sources=self.sources)

    def test_current_union_has_complete_disjoint_site_coverage(self):
        self.assertGreater(len(self.sites), 0)
        recorded = [record for name in SEGMENTS for record in self.segments[name]['sites']]
        self.assertEqual(len(self.by_key), len(self.sites))
        self.assertEqual(len(recorded), len(self.sites))
        self.assertEqual({r['site'] for r in recorded}, set(self.by_key))
        for name in SEGMENTS:
            actual = {s.key for s in self.sites if segment_for(s) == name}
            self.assertGreater(len(actual), 0)
            self.assertEqual(actual, {r['site'] for r in self.segments[name]['sites']})
        errors = validate_segments(self.sites, self.segments, sources=self.sources)
        self.assertEqual(errors, [], '\n'.join(errors))
        schema = json.loads((self.root / 'verification/policy/schemas/unsafe_abi_segment.schema.json').read_text())
        props = schema['properties']['sites']['items']['properties']
        self.assertEqual(set(props['operation_family']['enum']), set(FAMILY_RULES))
        for index, field in enumerate(FIELDS):
            self.assertEqual(set(props['obligations']['properties'][field]['properties']['rule']['enum']),
                             {rules[index] for rules in FAMILY_RULES.values()})
        # Bounded negatives use the actual validator with the complete explicit
        # segment interface, while avoiding redundant exhaustive pair comparison.
        sample = self.inventory['sites'][0]
        site = self.by_key[sample['site']]
        segments = {name: dict(schema_version=1, segment=name, sites=[]) for name in SEGMENTS}
        segments['external']['sites'] = [copy.deepcopy(sample)]
        self.assertEqual(validate_segments([site], segments, sources=self.sources), [])
        for mutation, expected in [('missing-segment', 'missing required'),
                                   ('missing-site', 'new/unclassified'),
                                   ('duplicate', 'duplicate or invalid'),
                                   ('stale', 'stale site'),
                                   ('wrong-partition', 'wrong source partition')]:
            changed = copy.deepcopy(segments)
            if mutation == 'missing-segment':
                del changed['external']
            elif mutation == 'missing-site':
                changed['external']['sites'].clear()
            elif mutation == 'duplicate':
                changed['external']['sites'].append(copy.deepcopy(sample))
            elif mutation == 'stale':
                changed['external']['sites'][0]['site'] += '-removed'
            else:
                changed['python_core']['sites'] = changed['external']['sites']
                changed['external']['sites'] = []
            self.assertTrue(any(expected in e for e in validate_segments(
                [site], changed, sources=self.sources)), mutation)
        new_source = Source('crates/new_native_owner/src/lib.rs',
                            'fn new_operation() { unsafe { observe(pointer); } }')
        new_sites = unsafe.discover(new_source)
        self.assertTrue(any('new/unclassified' in e for e in validate_segments(
            [site, *new_sites], segments, sources=[*self.sources, new_source])))
        for field, value, message in [('fingerprint', '0' * 64, 'changed fingerprint'),
                                      ('operation', 'unsafe { unrelated(); }', 'operation binding'),
                                      ('source_evidence', '// neighboring proof', 'SAFETY evidence')]:
            changed = copy.deepcopy(sample)
            changed[field] = value
            self.assertTrue(any(message in e for e in self.one_record_errors(changed)))
        for record in self.inventory['sites']:
            for field in FIELDS:
                changed = copy.deepcopy(record)
                del changed['contract'][field]
                self.assertTrue(any('missing local' in e for e in self.one_record_errors(changed)))

    def test_windows_acl_contract_cannot_claim_python_attachment(self):
        records = [r for r in self.inventory['sites']
                   if 'windows_storage_security.rs::test_grant_world::unsafe-block' in r['site']]
        self.assertEqual(len(records), 3)
        for original in records:
            self.assertEqual(original['operation_family'], 'windows-security')
            self.assertEqual(original['owner'], 'cache-owner')
            self.assertEqual(self.one_record_errors(original), [])
            changed = copy.deepcopy(original)
            changed['contract']['thread'] = 'Hold supported Python attachment while reading the pointer.'
            changed['obligations']['thread']['rule'] = 'supported-gil'
            self.assertTrue(any('invalid thread obligation' in e for e in self.one_record_errors(changed)))
            self.assertEqual(changed['fingerprint'], original['fingerprint'])
            self.assertEqual(changed['source_evidence'], original['source_evidence'])
            self.assertEqual(changed['obligations']['thread']['proofs'], original['obligations']['thread']['proofs'])
            changed = copy.deepcopy(original)
            changed['obligations']['ownership']['rule'] = 'borrow-only'
            self.assertTrue(any('invalid ownership obligation' in e for e in self.one_record_errors(changed)))
            changed = copy.deepcopy(original)
            for field in FIELDS:
                changed['obligations'][field]['proofs'] = [p for p in changed['obligations'][field]['proofs'] if p['kind'] != 'code']
            self.assertTrue(any('requires source admission/teardown proof' in e for e in self.one_record_errors(changed)))
        # LocalFree cleanup is part of the Windows owner proof, not Python
        # attachment. Removing it invalidates descriptor/DACL records.
        original = records[-1]
        modified = [Source(s.path, s.text.replace('LocalFree(self.0)', 'retain_descriptor(self.0)'))
                    if s.path.endswith('windows_storage_security.rs') else s for s in self.sources]
        self.assertTrue(any('obligation/proof' in e for e in unsafe.validate(
            [self.by_key[original['site']]], {'sites': [original]}, sources=modified)))

    def test_external_and_generated_operations_keep_their_actual_owners(self):
        groups = [('sifr_sql_', 'SQL-owner', 'SQL ownership changed'),
                  ('sifr_cache_storage/', 'cache-owner', 'cache ownership changed'),
                  ('sifr_driver/', 'driver-owner', 'driver ownership changed'),
                  ('sifr_codegen/', 'X02', 'generated Rust must remain with X02')]
        for path_fragment, owner, diagnostic in groups:
            records = [r for r in self.inventory['sites'] if path_fragment in r['site']]
            self.assertGreater(len(records), 0)
            for original in records:
                self.assertEqual(original['owner'], owner)
                self.assertEqual(self.one_record_errors(original), [])
                for new_owner in ('runtime-owner', 'test-owner', 'Python-bridge-owner'):
                    changed = copy.deepcopy(original)
                    changed['owner'] = new_owner
                    self.assertTrue(any(diagnostic in e for e in self.one_record_errors(changed)))
                    self.assertEqual(changed['fingerprint'], original['fingerprint'])
        generated = [r for r in self.inventory['sites'] if r['owner'] == 'X02']
        for original in generated:
            changed = copy.deepcopy(original)
            changed['operation_family'] = 'abi-declaration'
            for field, rule in zip(FIELDS, FAMILY_RULES['abi-declaration']):
                changed['obligations'][field]['rule'] = rule
            self.assertTrue(any('operation family mismatch' in e for e in self.one_record_errors(changed)))
        # Existing core/resource records keep their exact source partitions and
        # established acceptance owners when the external effects are consumed.
        for name in ('python_core', 'python_resources'):
            for record in self.segments[name]['sites']:
                self.assertEqual(segment_for(self.by_key[record['site']]), name)
                self.assertIn(record['owner'], ('H02d0', 'H02d1') if name == 'python_core' else ('H02e', 'H02f', 'H02g'))
        # Acquire/release/transfer/state effects cannot be substituted with the
        # generic borrowed-pointer ownership rule on unchanged live records.
        effects = [r for r in self.inventory['sites'] if r['operation_family'].startswith('external-')]
        self.assertGreater(len(effects), 0)
        for original in effects:
            changed = copy.deepcopy(original)
            changed['obligations']['ownership']['rule'] = 'borrow-only'
            self.assertTrue(any('invalid ownership obligation' in e for e in self.one_record_errors(changed)))
        # A neighboring observation does not conceal the stronger effect.
        for body, family in [('libc::geteuid(); CloseHandle(handle);', 'external-owner-release'),
                             ('libc::geteuid(); CreateJobObjectW(name, attrs);', 'external-owner-acquire'),
                             ('libc::geteuid(); File::from_raw_handle(handle);', 'external-owner-transfer'),
                             ('libc::geteuid(); libc::fcntl(fd, flags, 0);', 'external-state-mutation'),
                             ('libc::geteuid(); libc::statvfs(path, output);', 'external-output-write')]:
            source = Source('crates/sifr_driver/src/mixed_external_fixture.rs', 'fn mixed() { unsafe { '+body+' } }')
            sites = unsafe.discover(source)
            self.assertEqual([operation_family(s) for s in sites], [family])
