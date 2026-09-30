"""H02h2 current-tree CPython/callback partition admission (no runtime audit)."""
import copy
import json
from pathlib import Path
import unittest

import check_unsafe_abi_contracts as unsafe
from rust_policy_sites import Source
from unsafe_policy_contracts import FAMILY_RULES, FIELDS, operation_family
from unsafe_policy_segments import read_segments, segment_for, validate_segments


class UnsafePythonCoreTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root = Path(__file__).resolve().parent.parent
        # Discover the entire native Python namespace, including parent module
        # allowances and resource sources; select by source partition afterwards.
        paths = [cls.root / 'crates/sifr_runtime/src/python.rs',
                 *sorted((cls.root / 'crates/sifr_runtime/src/python').rglob('*.rs'))]
        cls.sources = [Source(p.relative_to(cls.root).as_posix(), p.read_text()) for p in paths]
        cls.sites = [site for source in cls.sources for site in unsafe.discover(source)
                     if segment_for(site) == 'python_core']
        cls.inventory = read_segments(cls.root, selected=('python_core',))['python_core']
        cls.by_key = {s.key: s for s in cls.sites}

    def one_record_errors(self, record):
        site = self.by_key[record['site']]
        return unsafe.validate([site], {'sites': [record]}, sources=self.sources)

    def test_current_core_segment_has_complete_local_contracts(self):
        self.assertGreater(len(self.sites), 0)
        self.assertEqual(set(self.by_key), {r['site'] for r in self.inventory['sites']})
        self.assertEqual(len(self.sites), len(self.inventory['sites']))
        errors = validate_segments(self.sites, {'python_core': self.inventory},
                                   selected=('python_core',), sources=self.sources)
        self.assertEqual(errors, [], '\n'.join(errors))
        schema = json.loads((self.root / 'verification/policy/schemas/unsafe_abi_segment.schema.json').read_text())
        properties = schema['properties']['sites']['items']['properties']
        self.assertEqual(set(properties['operation_family']['enum']), set(FAMILY_RULES))
        for field, index in zip(FIELDS, range(4)):
            self.assertEqual(set(properties['obligations']['properties'][field]['properties']['rule']['enum']),
                             {rules[index] for rules in FAMILY_RULES.values()})
        families = {operation_family(s) for s in self.sites}
        self.assertTrue({'current-target-erasure', 'target-erasure', 'future-erasure',
                         'callback-erasure', 'thread-marker', 'cpython-config-clear',
                         'cpython-observation', 'cpython-detach'} <= families)
        for record in self.inventory['sites']:
            with self.subTest(site=record['site']):
                self.assertEqual(record['owner'], 'H02d0' if '/callbacks/' in record['site'] else 'H02d1')
                for field in FIELDS:
                    changed = copy.deepcopy(record)
                    del changed['contract'][field]
                    self.assertTrue(any('missing local' in e for e in self.one_record_errors(changed)))
        record = self.inventory['sites'][0]
        for field, value, expected in [('fingerprint', '0' * 64, 'changed fingerprint'),
                                       ('operation', 'unsafe { unrelated(); }', 'operation binding'),
                                       ('source_evidence', '// other proof', 'SAFETY evidence')]:
            changed = copy.deepcopy(record)
            changed[field] = value
            self.assertTrue(any(expected in e for e in self.one_record_errors(changed)))
        clear = next(r for r in self.inventory['sites'] if r['operation_family'] == 'cpython-config-clear')
        changed = copy.deepcopy(clear)
        changed['obligations']['ownership']['rule'] = 'borrow-only'
        self.assertTrue(any('invalid ownership obligation' in e for e in self.one_record_errors(changed)))

    def test_target_and_future_erasure_require_lifetime_and_revocation_proof(self):
        records = [r for r in self.inventory['sites'] if r['operation_family'] in
                   ('current-target-erasure', 'target-erasure', 'future-erasure', 'callback-erasure')]
        self.assertGreater(len(records), 0)
        for original in records:
            self.assertEqual(self.one_record_errors(original), [])
            changed = copy.deepcopy(original)
            # Claim the rejected non-retention effect in the structured lifetime
            # obligation; the original operation and every proof stay bound.
            changed['contract']['lifetime'] = 'Borrowed captures are not retained by Python.'
            changed['obligations']['lifetime']['rule'] = 'capture-not-retained'
            self.assertTrue(any('invalid lifetime obligation' in e for e in self.one_record_errors(changed)))
            for field in ('lifetime', 'alias', 'ownership'):
                changed = copy.deepcopy(original)
                obligation = changed['obligations'][field]
                # Remove external admission/drain/revocation evidence while
                # retaining the valid local operation proof and its fingerprint.
                obligation['proofs'] = [p for p in obligation['proofs'] if p['kind'] != 'code']
                self.assertTrue(obligation['proofs'])
                self.assertTrue(any(f'{field} requires source admission/teardown proof' in e
                                    for e in self.one_record_errors(changed)))
                self.assertEqual(changed['fingerprint'], original['fingerprint'])
                self.assertEqual(changed['source_evidence'], original['source_evidence'])
        # A change to the actual teardown invalidates an otherwise unchanged
        # erasure record: full proof-context fingerprints bind the revocation.
        future = next(r for r in records if r['operation_family'] == 'future-erasure')
        changed_sources = [Source(s.path, s.text.replace('self.drain.revoke();', 'skip_revocation();'))
                           if s.path.endswith('/callbacks/asyncio.rs') else s for s in self.sources]
        self.assertTrue(any('obligation/proof' in e for e in unsafe.validate(
            [self.by_key[future['site']]], {'sites': [future]}, sources=changed_sources)))

    def test_callback_owner_template_with_distinct_suffixes_is_rejected(self):
        originals = [r for r in self.inventory['sites'] if r['operation_family'] in
                     ('target-erasure', 'future-erasure')][:2]
        self.assertEqual(len(originals), 2)
        records = copy.deepcopy(originals)
        sites = [self.by_key[r['site']] for r in records]
        for record in records:
            record.pop('contract_ref', None)
            record['contract'] = {field: 'All callback operations share the owner template. '
                + f"site={record['site']}; fingerprint={record['fingerprint']}; "
                + f"operation={record['operation']}; "
                + (f"evidence={record['source_evidence']}" if record['source_evidence'] else '')
                for field in FIELDS}
        errors = unsafe.validate(sites, {'sites': records}, sources=self.sources)
        self.assertTrue(any('repeated owner-level contract lacks local obligations' in e for e in errors))
        self.assertFalse(any('fingerprint' in e or 'stale local' in e for e in errors))
