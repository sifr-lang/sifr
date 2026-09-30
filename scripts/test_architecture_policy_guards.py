"""Bounded positive/negative fixtures for H02h; runnable without repository state."""
from __future__ import annotations

import copy
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

import check_method_dispatch_authority as methods
import check_unsafe_abi_contracts as unsafe
from rust_policy_sites import Source, code_tokens
from architecture_policy_python_core_tests import UnsafePythonCoreTests
from architecture_policy_python_resource_tests import UnsafePythonResourceTests


from architecture_policy_test_fixtures import (
    MethodTests, UnsafeTests, method_records, unsafe_records,
)


class PolicyFoundationTests(unittest.TestCase):
    @staticmethod
    def pair_segments(sites, records):
        from unsafe_policy_segments import SEGMENTS, segment_for
        segments = {name: dict(schema_version=1, segment=name, sites=[]) for name in SEGMENTS}
        by_key = {s.key: s for s in sites}
        for record in records['sites']:
            segments[segment_for(by_key[record['site']])]['sites'].append(record)
        if records.get('shared_contracts'):
            segments['python_core']['shared_contracts'] = records['shared_contracts']
        return segments

    def assert_repeated_union(self, sites, records, sources):
        from unsafe_policy_segments import validate_segments
        errors = validate_segments(sites, self.pair_segments(sites, records), sources=sources)
        self.assertTrue(errors)
        self.assertTrue(all('repeated owner-level' in error for error in errors), errors)
        return errors

    def test_pair_scoped_comparison_rejects_binding_collision_templates(self):
        # Search genuine source contexts; no forged fingerprints or line bindings.
        paths = ('crates/sifr_runtime/src/python/callbacks/one.rs',
                 'crates/sifr_runtime/src/python/buffer_ops/two.rs')
        sources = [Source(paths[0], f'fn reads() {{ let seed = {seed}; unsafe {{ read(first); }} }}')
                   for seed in range(200)]
        first_source = next(source for source in sources
                            if unsafe.discover(source)[0].fingerprint.startswith('a'))
        second_source = next(source for source in (
            Source(paths[1], '\n\nfn reads() { let seed = ' + str(seed)
                   + '; unsafe { read(filler); }\nunsafe { read(second); } }')
            for seed in range(200))
            if all(not site.fingerprint.startswith('a') for site in unsafe.discover(source)))
        sources = [first_source, second_source]
        second = unsafe.discover(second_source)
        sites = [unsafe.discover(first_source)[0], second[1], second[0]]
        self.assertNotEqual(sites[0].ordinal, sites[1].ordinal)
        self.assertEqual(sum(site.fingerprint.startswith('a') for site in sites), 1)
        self.assertNotEqual(sites[0].line, sites[1].line)
        # The retained case also covers distinct ordinals in one source scope.
        self.test_actual_binding_values_cannot_distinguish_repeated_contracts()
        suffixes = [lambda s: '', lambda s: s.key, lambda s: s.fingerprint[:12]]
        for binding in (lambda s: s.key, lambda s: s.path,
                        lambda s: f'{s.path}:{s.line}', lambda s: s.scope,
                        lambda s: str(s.ordinal), lambda s: str(s.line)):
            for punctuation in ('.', ':', ','):
                suffixes.append(lambda s, binding=binding, punctuation=punctuation:
                                binding(s) + punctuation)
        for word in ('a', str(sites[1].line)):
            for suffix in suffixes:
                with self.subTest(word=word, suffix=suffix(sites[0])):
                    records = unsafe_records(sites)
                    for site, record in zip(sites, records['sites']):
                        record['contract'] = dict.fromkeys(unsafe.CONTRACT_FIELDS,
                            f'Owner keeps {word} leases on the allocation. ' + suffix(site))
                    errors = self.assert_repeated_union(sites, records, sources)
                    self.assertEqual(errors, ['repeated owner-level contract lacks local obligations: '
                                              + ', '.join(sorted(site.key for site in sites))])

    def test_third_site_binding_references_preserve_distinct_obligations(self):
        from itertools import permutations
        from unsafe_policy_contracts import normalized_obligations
        from unsafe_policy_segments import validate_segments
        sources = [Source('crates/sifr_runtime/src/python/callbacks/pair.rs',
                          '\n' * 9 + 'fn alpha() { unsafe { read(first); } }\n'
                          + '\n' * 9 + 'fn beta() { unsafe { read(second); } }'),
                   Source('crates/sifr_runtime/src/python/buffer_ops/release.rs',
                          '\n' * 51 + 'fn release() { unsafe { read(third); } }'),
                   Source('crates/sifr_runtime/src/python/drop.rs',
                          '\n' * 152 + 'fn drop() { unsafe { read(fourth); } }')]
        # Reference ordinals 2/3 must belong to neither compared site: each
        # independently discovered reference source has multiple operations.
        sources += [Source('crates/sifr_runtime/src/python/buffer_ops/frames.rs',
                           'fn references() { unsafe { read(five); } '
                           'unsafe { read(six); } unsafe { read(seven); } }')]
        discovered = [unsafe.discover(source) for source in sources]
        for pair in ((discovered[0][0], discovered[1][0]),
                     (discovered[0][0], discovered[0][1])):
            # Ordinal example uses cross-file ordinal-1 pair; same-file pair
            # retains a third ordinal using values 3/4096.
            examples = [('ordinal', '3', '4096', 'Owner allows {} frames.'),
                        ('line', '153', '4096', 'Owner allows at most {} bytes.'),
                        ('scope', 'drop', 'writer_guard', 'Owner pins through {}.'),
                        ('path', discovered[3][0].path, 'writers/independent.rs',
                         'Owner serializes with the writer in {}.')]
            if pair[0].ordinal == pair[1].ordinal == 1:
                examples += [('frames', '2', '3', 'Owner allows {} frames.')]
            if all(site.line not in (52, 153) for site in pair):
                examples += [('bytes', '52', '153', 'Owner allows at most {} bytes.')]
            if all(site.scope not in ('release', 'drop') for site in pair):
                examples += [('scopes', 'release', 'drop', 'Owner pins through {}.')]
            if all(site.path not in (discovered[2][0].path, discovered[3][0].path) for site in pair):
                examples += [('writers', discovered[2][0].path, discovered[3][0].path,
                              'Owner serializes with the writer in {}.')]
            selected = list(pair) + [s for group in discovered for s in group if s not in pair]
            for name, left, right, prose in examples:
                with self.subTest(name=name, pair=[s.key for s in pair]):
                    records = unsafe_records(selected)
                    for index, record in enumerate(records['sites']):
                        # Third sites are independently valid and distinct.
                        record['contract'] = dict.fromkeys(unsafe.CONTRACT_FIELDS,
                            f'Reference allocation has independent guard label_{index}_end.')
                    for index, value in enumerate((left, right)):
                        record = records['sites'][index]
                        record['contract'] = dict.fromkeys(unsafe.CONTRACT_FIELDS, prose.format(value))
                        actual = normalized_obligations(record['contract'], pair[index], record,
                                                        comparison_sites=pair)
                        self.assertEqual(actual, (prose.format(value),) * 4)
                    for order in permutations(range(3)):
                        ordered = [selected[i] for i in order] + selected[3:]
                        shuffled = {'sites': [records['sites'][i] for i in order] + records['sites'][3:]}
                        segments = self.pair_segments(ordered, shuffled)
                        segments = dict(reversed(list(segments.items())))
                        errors = validate_segments(ordered, segments, sources=list(reversed(sources)),
                                                   selected=tuple(segments))
                        self.assertFalse(errors, errors)

    def test_pair_repetition_groups_keep_exact_reviewed_sharing(self):
        from itertools import permutations
        from unsafe_policy_contracts import normalized_obligations, repeated_contract_groups
        from unsafe_policy_segments import validate_segments
        sources = [Source('crates/sifr_runtime/src/python/callbacks/one.rs',
                          'fn release() { unsafe { read(first); } }'),
                   Source('crates/sifr_runtime/src/python/buffer_ops/two.rs',
                          'fn drop() { unsafe { read(second); } }'),
                   Source('crates/sifr_runtime/src/python/callbacks/three.rs',
                          'fn other() { unsafe { read(third); } }')]
        sites = [unsafe.discover(source)[0] for source in sources]
        records = unsafe_records(sites)
        for record, scope in zip(records['sites'], ('drop', 'release', 'release')):
            record['contract'] = dict.fromkeys(unsafe.CONTRACT_FIELDS, f'Owner pins through {scope}.')
        def equal(a, b):
            pair = (sites[a], sites[b])
            return normalized_obligations(records['sites'][a]['contract'], sites[a], records['sites'][a],
                                          comparison_sites=pair) == normalized_obligations(
                records['sites'][b]['contract'], sites[b], records['sites'][b], comparison_sites=pair)
        self.assertTrue(equal(0, 1))
        self.assertTrue(equal(1, 2))
        self.assertFalse(equal(0, 2))
        expected = sorted(s.key for s in sites)
        for order in permutations(range(3)):
            ordered = [sites[i] for i in order]
            shuffled = {'sites': [records['sites'][i] for i in order]}
            groups = repeated_contract_groups(ordered, shuffled)
            self.assertEqual([[r['site'] for r in group] for group in groups], [expected])
            errors = validate_segments(ordered, self.pair_segments(ordered, shuffled),
                                      sources=list(reversed(sources)),
                                      selected=('external', 'python_resources', 'python_core'))
            self.assertEqual(errors, ['repeated owner-level contract lacks local obligations: '
                                      + ', '.join(expected)])
        # Changing unrelated C's bindings cannot change the A/B edge.
        changed_source = Source(sources[2].path + '.different.rs',
                                '\n' * 10 + 'fn different() { unsafe { read(changed); } }')
        changed_sites = sites[:2] + unsafe.discover(changed_source)
        changed_records = unsafe_records(changed_sites)
        changed_records['sites'][:2] = copy.deepcopy(records['sites'][:2])
        changed_records['sites'][2]['contract'] = dict.fromkeys(unsafe.CONTRACT_FIELDS,
                                                               'Independent third admission remains guarded.')
        errors = self.assert_repeated_union(changed_sites, changed_records, sources[:2] + [changed_source])
        self.assertEqual(errors, ['repeated owner-level contract lacks local obligations: '
                                  + ', '.join(sorted(s.key for s in sites[:2]))])
        shared = dict.fromkeys(unsafe.CONTRACT_FIELDS, 'Owner pins allocations until every admission drains.')
        for record in records['sites']:
            record['contract'], record['contract_ref'] = shared.copy(), 'reviewed'
        records['shared_contracts'] = {'reviewed': dict(contract=shared,
            review_rationale='The fixture allocations share one admitted teardown.',
            bindings={s.key: s.fingerprint for s in sites})}
        self.assertFalse(validate_segments(sites, self.pair_segments(sites, records), sources=sources))
        for mutation in ('remove', 'change', 'add', 'reference', 'missing-rationale',
                         'empty-rationale', 'whitespace-rationale', 'raw-binding', 'new-duplicate'):
            with self.subTest(mutation=mutation):
                changed = copy.deepcopy(records)
                definition = changed['shared_contracts']['reviewed']
                bounded, inputs = sites, sources
                if mutation == 'remove':
                    definition['bindings'].pop(sites[1].key)
                elif mutation == 'change':
                    definition['bindings'][sites[1].key] = '0' * 64
                elif mutation == 'add':
                    definition['bindings']['unknown'] = sites[0].fingerprint
                elif mutation == 'reference':
                    changed['sites'][1]['contract_ref'] = 'different'
                elif mutation == 'missing-rationale':
                    definition.pop('review_rationale')
                elif mutation in ('empty-rationale', 'whitespace-rationale'):
                    definition['review_rationale'] = '' if mutation == 'empty-rationale' else '  '
                elif mutation == 'raw-binding':
                    changed['sites'][1]['contract'] = {k: v + ' ' + sites[1].key for k, v in shared.items()}
                else:
                    extra = Source('crates/sifr_runtime/src/python/buffer_ops/four.rs',
                                   'fn fourth() { unsafe { read(fourth); } }')
                    found = unsafe.discover(extra)
                    extra_record = unsafe_records(found)['sites'][0]
                    extra_record['contract'], extra_record['contract_ref'] = shared.copy(), 'reviewed'
                    changed['sites'].append(extra_record)
                    bounded, inputs = sites + found, sources + [extra]
                self.assert_repeated_union(bounded, changed, inputs)

    def test_actual_binding_values_cannot_distinguish_repeated_contracts(self):
        sources = [Source('crates/example/src/one.rs', '''fn reads(x: X, y: X) {
// SAFETY: first allocation stays pinned.
// First admission remains held.
unsafe { read(x); }
// SAFETY: second allocation stays pinned.
// Second admission remains held.
unsafe { read(y); }
}'''), Source('crates/example/src/two.rs', '''

fn reads(z: X) {
// SAFETY: third allocation stays pinned.
// Third admission remains held.
unsafe { read(z); }
}''')]
        first = unsafe.discover(sources[0])
        other = unsafe.discover(sources[1])
        self.assertEqual(first[0].scope, first[1].scope)
        self.assertNotEqual(first[0].ordinal, first[1].ordinal)
        self.assertNotEqual(first[0].line, first[1].line)
        self.assertNotEqual(first[0].path, other[0].path)
        variants = {
            'kind-space': lambda s: f'unsafe block {s.ordinal}',
            'kind-hyphen': lambda s: f'unsafe-block {s.ordinal}',
            'kind-underscore': lambda s: f'unsafe_block {s.ordinal}',
            'kind-hash': lambda s: f'unsafe block #{s.ordinal}',
            'site': lambda s: f'site {s.ordinal}',
            'ordinal-colon': lambda s: f'ordinal: {s.ordinal}',
            'ordinal-equals': lambda s: f'ordinal={s.ordinal}',
            'bare': lambda s: str(s.ordinal),
            'parenthesized': lambda s: f'({s.ordinal})',
            'hash': lambda s: f'#{s.ordinal}',
            'line': lambda s: f'line {s.line}',
            'bare-line': lambda s: str(s.line),
            'path': lambda s: s.path,
            'path-line': lambda s: f'{s.path}:{s.line}',
            'key-period': lambda s: s.key + '.',
            'path-period': lambda s: s.path + '.',
            'path-line-period': lambda s: f'{s.path}:{s.line}.',
            'scope-period': lambda s: s.scope + '.',
            'bare-ordinal-period': lambda s: f'{s.ordinal}.',
            'bare-ordinal-colon': lambda s: f'{s.ordinal}:',
            'bare-line-period': lambda s: f'{s.line}.',
            'bare-line-colon': lambda s: f'{s.line}:',
        }
        for length in (1, 6, 12, 63, 64):
            for uppercase in (False, True):
                variants[f'fingerprint-{length}-{uppercase}'] = (
                    lambda s, n=length, upper=uppercase:
                    s.fingerprint[:n].upper() if upper else s.fingerprint[:n])
        for sites in (first, [first[0], other[0]]):
            for name, suffix_for in variants.items():
                for placement in ('beginning', 'middle', 'end'):
                    for combined in (False, True):
                        with self.subTest(name=name, placement=placement, combined=combined,
                                          paths=[s.path for s in sites]):
                            records = unsafe_records(sites)
                            for site, record in zip(sites, records['sites']):
                                suffix = suffix_for(site)
                                if combined:
                                    suffix += (' scope: ' + site.scope + ' site: ' + site.key
                                               + ' operation: ' + ''.join(t.value for t in code_tokens(site.text))
                                               + ' evidence: ' + site.source_evidence.replace('// ', ''))
                                prose = {
                                    'beginning': f'{suffix} Owner allocation stays live.',
                                    'middle': f'Owner allocation {suffix} stays live.',
                                    'end': f'Owner allocation stays live. {suffix}',
                                }[placement]
                                record['contract'] = dict.fromkeys(unsafe.CONTRACT_FIELDS, prose)
                            errors = unsafe.validate(sites, records, sources=sources)
                            self.assertTrue(any('repeated owner-level' in e for e in errors), errors)
                            self.assertTrue(all('repeated owner-level' in e for e in errors), errors)
        # Exact identical prose cannot diverge when a meaningful word/number
        # happens to be metadata for only one discovered site. The same must
        # hold after adding distinct actual bindings to that shared template.
        sites = unsafe.discover(Source('crates/example/src/lib.rs',
            'fn reads() { let seed = 20; unsafe { read(first); } unsafe { read(second); } }'))
        self.assertTrue(sites[0].fingerprint.startswith('a'))
        self.assertFalse(sites[1].fingerprint.startswith('a'))
        self.assertEqual([s.ordinal for s in sites], [1, 2])
        self.assertEqual([s.line for s in sites], [1, 1])
        for prose in ('Owner keeps a lease on the allocation.', 'Owner keeps 2 leases on the allocation.'):
            for suffix_for in (lambda s: '', lambda s: s.key, lambda s: s.fingerprint[:12]):
                with self.subTest(prose=prose, suffix=suffix_for(sites[0])):
                    records = unsafe_records(sites)
                    for site, record in zip(sites, records['sites']):
                        record['contract'] = dict.fromkeys(unsafe.CONTRACT_FIELDS, prose + ' ' + suffix_for(site))
                    self.assertTrue(any('repeated owner-level' in e for e in unsafe.validate(sites, records)))

    def test_binding_normalization_preserves_semantic_obligations(self):
        from unsafe_policy_contracts import normalized_obligations
        from unsafe_policy_segments import SEGMENTS, segment_for, validate_segments
        sources = [Source('crates/sifr_runtime/src/python/callbacks/example.rs',
                          'fn reads() { unsafe { read(first); } }'),
                   Source('crates/sifr_runtime/src/python/buffer_ops/example.rs',
                          'fn reads() { unsafe { read(second); } }')]
        sites = [unsafe.discover(source)[0] for source in sources]
        records = unsafe_records(sites)
        records['sites'][0]['contract']['lifetime'] = 'First allocation stays pinned through the admitted read.'
        records['sites'][1]['contract']['lifetime'] = 'Second allocation remains retained until its owner drains.'
        records['sites'][1]['contract']['ownership'] = 'The second owner retains storage through teardown.'
        self.assertFalse(unsafe.validate(sites, records, sources=sources))
        site = sites[0]
        other_hash = ('0' if site.fingerprint[0] != '0' else '1') + site.fingerprint[1:]
        preserved = ('4096', 'buffer4096', 'different/reads/file.rs', other_hash,
                     site.fingerprint + '0', 'unsafe_blockish', 'prereads',
                     site.path.upper(), site.path + '/different.rs', site.path + '.bak',
                     site.path + 'X', 'different/1.rs', 'different/a/file.rs')
        for text in preserved:
            with self.subTest(text=text):
                contract = dict.fromkeys(unsafe.CONTRACT_FIELDS, text)
                self.assertEqual(normalized_obligations(contract, site, records['sites'][0]),
                                 (text,) * 4)
        # Untrusted record copies never alter the comparison key.
        forged = dict(records['sites'][0], operation='4096', source_evidence='buffer4096')
        contract = dict.fromkeys(unsafe.CONTRACT_FIELDS, '4096 buffer4096')
        self.assertEqual(normalized_obligations(contract, site, forged), ('4096 buffer4096',) * 4)
        release_source = Source('crates/example/src/lib.rs',
                                'fn release() { unsafe { PyBuffer_Release(view); } }')
        release = unsafe.discover(release_source)
        release_records = unsafe_records(release)
        self.assertFalse(unsafe.validate(release, release_records))
        changed = copy.deepcopy(release_records)
        changed['sites'][0]['obligations']['ownership']['rule'] = 'borrow-only'
        self.assertEqual(changed['sites'][0]['fingerprint'], release[0].fingerprint)
        self.assertEqual(changed['sites'][0]['source_evidence'], release[0].source_evidence)
        self.assertTrue(any('invalid ownership obligation' in e
                            for e in unsafe.validate(release, changed)))
        shared = dict.fromkeys(unsafe.CONTRACT_FIELDS, 'Owner pins these allocations until both admissions drain.')
        segments = {name: dict(schema_version=1, segment=name, sites=[]) for name in SEGMENTS}
        for site, record in zip(sites, records['sites']):
            record['contract'], record['contract_ref'] = shared.copy(), 'two-segments'
            segments[segment_for(site)]['sites'].append(record)
        definition = dict(contract=shared, review_rationale='These fixture reads share the admitted owner.',
                          bindings={s.key: s.fingerprint for s in sites})
        segments['python_core']['shared_contracts'] = {'two-segments': definition}
        self.assertFalse(validate_segments(sites, segments, sources=sources))
        for mutation in ('remove', 'change', 'add'):
            changed = copy.deepcopy(segments)
            bindings = changed['python_core']['shared_contracts']['two-segments']['bindings']
            if mutation == 'remove':
                bindings.pop(sites[1].key)
            elif mutation == 'change':
                bindings[sites[1].key] = '0' * 64
            else:
                bindings['new-site'] = sites[0].fingerprint
            self.assertTrue(any('repeated owner-level' in e
                                for e in validate_segments(sites, changed, sources=sources)), mutation)

    def test_safety_continuations_cannot_import_previous_statement_proof(self):
        path = 'crates/example/src/lib.rs'
        trailing = ('// SAFETY: previous statement proof.', '/* SAFETY: previous statement proof. */')
        neighbors = []
        for marker in trailing:
            for continuation in ('// continued.', '// continued.\n// still continued.'):
                for before, after in (
                        ('fn f() { let a = g(); ', '\nunsafe { other() }; }'),
                        ('fn f() { match x { A => g(), ', '\nB => unsafe { other() }, } }'),
                        ('fn f() { call(g(), ', '\nunsafe { other() }); }')):
                    neighbors.append((before + marker + '\n' + continuation + after,
                                      marker + '\n' + continuation))
        neighbors += [
            ('fn f() { let a = g(); // SAFETY: previous statement proof.\nunsafe { other() }; }',
             '// SAFETY: previous statement proof.'),
            ('fn f() {\n// SAFETY: previous safe statement.\ng();\nunsafe { other() }; }',
             '// SAFETY: previous safe statement.'),
            ('fn f() { match x {\n// SAFETY: previous arm.\nA => g(),\nB => unsafe { other() }, } }',
             '// SAFETY: previous arm.'),
            ('fn f() { call(\n// SAFETY: previous argument.\ng(),\nunsafe { other() }); }',
             '// SAFETY: previous argument.'),
            ('fn f() {\n// SAFETY: separated marker.\n\nunsafe { other() }; }',
             '// SAFETY: separated marker.'),
        ]
        for text, quote in neighbors:
            with self.subTest(text=text):
                source = Source(path, text)
                sites = unsafe.discover(source)
                self.assertEqual(len(sites), 1)
                self.assertEqual(sites[0].source_evidence, '')
                records = unsafe_records(sites)
                self.assertFalse(unsafe.validate(sites, records, sources=[source]))
                stale = copy.deepcopy(records)
                stale['sites'][0]['source_evidence'] = quote
                self.assertTrue(any('SAFETY evidence' in e
                                    for e in unsafe.validate(sites, stale, sources=[source])))
                proof = dict(kind='safety', site=sites[0].key, fingerprint=sites[0].fingerprint, quote=quote)
                for obligation in records['sites'][0]['obligations'].values():
                    obligation['proofs'] = [proof.copy()]
                self.assertTrue(any('obligation/proof' in e
                                    for e in unsafe.validate(sites, records, sources=[source])))

    def test_owned_safety_runs_bind_complete_local_proof(self):
        path = 'crates/example/src/lib.rs'
        runs = [
            '// SAFETY: allocation remains pinned.\n// Admission remains held.\n// Owner drains after the read.',
            '/* SAFETY: allocation remains pinned.\n * Admission remains held.\n * Owner drains after the read. */',
            '/* SAFETY: allocation remains pinned. */\n/* Admission remains held. */\n/* Owner drains after the read. */',
        ]
        positives = [(f'fn f() {{\n{run}\nlet value = unsafe {{ read(x); }}; }}', run)
                     for run in runs]
        positives += [('fn f() { g(); // SAFETY: prior statement proof.\n// continued.\n'
                       '// SAFETY: fresh allocation remains pinned.\n// Fresh admission remains held.\n'
                       'let value = Ok(unsafe { read(x); }); }',
                       '// SAFETY: fresh allocation remains pinned.\n// Fresh admission remains held.')]
        for text, quote in positives:
            with self.subTest(text=text):
                source = Source(path, text)
                sites = unsafe.discover(source)
                self.assertEqual(len(sites), 1)
                self.assertEqual(sites[0].source_evidence, quote)
                records = unsafe_records(sites)
                for obligation in records['sites'][0]['obligations'].values():
                    obligation['proofs'] = [dict(kind='safety', site=sites[0].key,
                                               fingerprint=sites[0].fingerprint, quote=quote)]
                self.assertFalse(unsafe.validate(sites, records, sources=[source]))
                wrong = copy.deepcopy(records)
                wrong['sites'][0]['obligations']['lifetime']['proofs'][0]['quote'] = runs[1]
                if quote == runs[1]:
                    wrong['sites'][0]['obligations']['lifetime']['proofs'][0]['quote'] = runs[0]
                self.assertTrue(any('invalid lifetime' in e
                                    for e in unsafe.validate(sites, wrong, sources=[source])))
                for replacement in ('', quote.replace('SAFETY:', 'RATIONALE:')):
                    changed = Source(path, text.replace(quote, replacement))
                    discovered = unsafe.discover(changed)
                    self.assertEqual(discovered[0].fingerprint, sites[0].fingerprint)
                    self.assertEqual(discovered[0].source_evidence, '')
                    self.assertTrue(any('obligation/proof' in e for e in unsafe.validate(
                        discovered, records, sources=[changed])))
        doc = '/// # Safety\n/// Caller pins x.\n/// Teardown drains admissions.'
        source = Source(path, doc + '\n#[allow(unsafe_code)]\nunsafe fn f(x: X) {\nunsafe { read(x); }\n}')
        sites = unsafe.discover(source)
        declaration = next(s for s in sites if s.kind == 'unsafe-declaration')
        block = next(s for s in sites if s.kind == 'unsafe-block')
        self.assertEqual(declaration.source_evidence, doc)
        self.assertEqual(block.source_evidence, '')
        records = unsafe_records([declaration])
        for obligation in records['sites'][0]['obligations'].values():
            obligation['proofs'] = [dict(kind='safety', site=declaration.key,
                                       fingerprint=declaration.fingerprint, quote=doc)]
        self.assertFalse(unsafe.validate([declaration], records, sources=[source]))
        local = runs[0]
        owned = Source(path, source.text.replace('unsafe { read(x); }', local + '\nunsafe { read(x); }'))
        block = next(s for s in unsafe.discover(owned) if s.kind == 'unsafe-block')
        self.assertEqual(block.source_evidence, local)
        records = unsafe_records([block])
        for obligation in records['sites'][0]['obligations'].values():
            obligation['proofs'] = [dict(kind='safety', site=block.key, fingerprint=block.fingerprint, quote=local)]
        self.assertFalse(unsafe.validate([block], records, sources=[owned]))
        for replacement in ('', local.replace('SAFETY:', 'RATIONALE:')):
            changed = Source(path, owned.text.replace(local, replacement))
            changed_block = next(s for s in unsafe.discover(changed) if s.kind == 'unsafe-block')
            self.assertEqual(changed_block.source_evidence, '')
            self.assertEqual(changed_block.fingerprint, block.fingerprint)
            self.assertTrue(any('obligation/proof' in e for e in unsafe.validate(
                [changed_block], records, sources=[changed])))
        records['sites'][0]['obligations']['lifetime']['proofs'][0]['quote'] = doc
        self.assertTrue(any('invalid lifetime' in e for e in unsafe.validate([block], records, sources=[owned])))

    def test_associated_type_and_macro_body_mutations_change_fingerprint(self):
        templates = [
            'fn f<F>(x: &str) -> <F as Trait>::Output where F: Trait { BODY }',
            'fn f<F>(x: &str) -> impl Iterator<Item = u8> { BODY }',
            'macro_rules! f { ($x:ident) => {{ BODY }} }',
        ]
        for template in templates:
            with self.subTest(template=template):
                original = template.replace('BODY', 'admit(x); if x == "pop" { emit(); } else { decline(); } unsafe { read(x); }')
                source = Source('crates/example/src/lib.rs', original)
                found = methods.discover(source)
                blocks = unsafe.discover(Source(source.path, original))
                self.assertEqual(len(found), 1)
                self.assertEqual(len(blocks), 1)
                self.assertNotEqual(found[0].scope, '<module>')
                for old, new in [('emit()', 'other()'), ('admit(x);', '')]:
                    changed = Source(source.path, original.replace(old, new))
                    self.assertNotEqual(methods.discover(changed)[0].fingerprint, found[0].fingerprint)
                    self.assertNotEqual(unsafe.discover(Source(changed.path, changed.text))[0].fingerprint,
                                        blocks[0].fingerprint)
                moved = '\n// comment\n' + original.replace('admit(x)', 'admit( /* c */ x)')
                self.assertEqual(methods.discover(Source(source.path, moved))[0].fingerprint,
                                 found[0].fingerprint)

    def test_formatted_comma_less_arm_cannot_hide_later_literal(self):
        # Rustfmt-style input is the scanner contract for complex block arms.
        for expression in ('{ body() }', 'if flag { body() } else { other() }',
                           'match value { 0 => body(), _ => other() }',
                           'unsafe { body() }', 'loop { break body(); }'):
            with self.subTest(expression=expression):
                text = 'fn f(value: X) { match value {\nSome(x) => ' + expression + '\n"append" => emit(),\n_ => decline(),\n} }'
                sites = methods.discover(Source('crates/example/src/lib.rs', text))
                literal = [s for s in sites if s.kind == 'literal-match']
                self.assertEqual(len(literal), 1)
                self.assertIn('"append"', literal[0].text)
                changed = methods.discover(Source(literal[0].path, text.replace('emit()', 'different()')))
                self.assertTrue(any('changed fingerprint' in e for e in methods.validate(
                    changed, method_records(sites))))

    def test_suffix_operation_and_evidence_cannot_hide_repeated_contract(self):
        text = '''fn read_one(x: X) {
// SAFETY: first allocation remains pinned.
unsafe { read(x); }
}
fn read_two(y: X) {
// SAFETY: second allocation remains pinned.
unsafe { access(y); }
}'''
        sites = unsafe.discover(Source('crates/example/src/lib.rs', text))
        self.assertEqual(len(sites), 2)
        records = unsafe_records(sites)
        for ordinal, (site, record) in enumerate(zip(sites, records['sites'])):
            record['contract'] = {field: f'{site.scope} / unsafe-block #{ordinal}: Shared owner template. operation: {record["operation"]}; evidence: {site.source_evidence}'
                                  for field in unsafe.CONTRACT_FIELDS}
        self.assertTrue(any('repeated owner-level' in e for e in unsafe.validate(sites, records)))
        for variant in ('ordinal', 'site-kind', 'comment-body', 'compact-operation'):
            records = unsafe_records(sites)
            for ordinal, (site, record) in enumerate(zip(sites, records['sites']), 1):
                suffix = {
                    'ordinal': f'ordinal: {ordinal}',
                    'site-kind': f'unsafe-block #{ordinal}',
                    'comment-body': site.source_evidence.replace('// ', ''),
                    'compact-operation': ''.join(t.value for t in code_tokens(site.text)),
                }[variant]
                record['contract'] = {field: 'Shared owner template. ' + suffix
                                      for field in unsafe.CONTRACT_FIELDS}
            self.assertTrue(any('repeated owner-level' in e for e in unsafe.validate(sites, records)), variant)
        # A reviewed shared obligation has exact source bindings, not a generic
        # authorization inherited by a newly discovered operation.
        records = unsafe_records(sites)
        shared = {k: 'Caller pins the shared allocation and serializes these reads.'
                  for k in unsafe.CONTRACT_FIELDS}
        for record in records['sites']:
            record['contract'], record['contract_ref'] = shared.copy(), 'two-reads'
        records['shared_contracts'] = {'two-reads': dict(contract=shared,
            review_rationale='These two reads have the same admitted shared allocation.',
            bindings={s.key: s.fingerprint for s in sites})}
        self.assertFalse(unsafe.validate(sites, records))
        records['shared_contracts']['two-reads']['bindings'].pop(sites[1].key)
        self.assertTrue(any('repeated owner-level' in e for e in unsafe.validate(sites, records)))

    def test_multiline_safety_proof_is_complete_and_operation_local(self):
        from unsafe_policy_contracts import proof_reference, valid_proof
        path = 'crates/example/src/lib.rs'
        text = '''fn read_one(x: X) {
// SAFETY: the allocation is pinned through this access.
// Its alias admission remains held until the read finishes.
// Release happens only after the admission guard leaves scope.
let value = unsafe { read(x); };
// SAFETY: this comment belongs to safe_step, not the next access.
safe_step();
unsafe { read_other(x); }
}'''
        source = Source(path, text)
        sites = unsafe.discover(source)
        self.assertEqual(len(sites), 2)
        self.assertEqual(sites[0].source_evidence, '\n'.join(text.splitlines()[1:4]))
        self.assertEqual(sites[1].source_evidence, '')
        records = unsafe_records(sites)
        records['sites'][1]['source_evidence'] = sites[0].source_evidence
        self.assertTrue(any('SAFETY evidence' in e for e in unsafe.validate(sites, records)))
        doc = '/// # Safety\n/// Caller pins x.\n/// Teardown drains all admissions.\n#[allow(unsafe_code)]\nunsafe fn f(x: X) { unsafe { read(x); } }'
        declared = unsafe.discover(Source(path, doc))
        declaration = next(s for s in declared if s.kind == 'unsafe-declaration')
        self.assertIn('Teardown drains all admissions.', declaration.source_evidence)
        block = next(s for s in declared if s.kind == 'unsafe-block')
        self.assertEqual(block.source_evidence, '')
        neighbors = [
            'fn f(x: X) { match x {\n// SAFETY: read_safe owns this proof.\nA => read_safe(x),\nB => unsafe { free(x) },\n} }',
            'fn f() { call(\n// SAFETY: read_safe owns this proof.\nread_safe(),\nunsafe { other() },\n); }',
            'fn f() { let a = 1; // SAFETY: the previous statement owns this proof.\nunsafe { other() }; }',
        ]
        for neighbor in neighbors:
            self.assertEqual(unsafe.discover(Source(path, neighbor))[0].source_evidence, '', neighbor)
        proof = proof_reference(source, 'read_one', 'safe_step();')
        self.assertTrue(valid_proof(proof, sites[0], [source]))
        self.assertFalse(valid_proof(proof, sites[0], [Source(path, text.replace('safe_step();', 'bypass();'))]))
        # Structured effects reject the exact release, erasure and thread
        # template mutations while operation/source evidence remain unchanged.
        examples = [
            ('crates/example/src/lib.rs', 'fn f() { unsafe { PyBuffer_Release(view); } }', 'ownership', 'borrow-only'),
            ('crates/example/src/lib.rs', 'fn f() { drain(); unsafe { transmute(target); } }', 'lifetime', 'allocation-live'),
            ('crates/example/src/lib.rs', 'fn f() { revoke(); unsafe { transmute(future); } }', 'ownership', 'borrow-only'),
            ('crates/example/src/lib.rs', 'fn f() { drain(); unsafe { erase_target_lifetime(target_ptr); } }', 'lifetime', 'allocation-live'),
            ('crates/example/src/lib.rs', 'fn f() { revoke(); unsafe { erase_future_lifetime(prepared.invoke(sequence, cancellation.clone())); } }', 'ownership', 'borrow-only'),
            ('crates/example/src/lib.rs', 'fn f() { drain(); revoke(); unsafe { build_asyncio_callback(owner, captures); } }', 'lifetime', 'allocation-live'),
            ('crates/sifr_cache_storage/src/windows_storage_security.rs', 'fn f() { LocalFree(descriptor); unsafe { SetNamedSecurityInfoW(descriptor); } }', 'thread', 'supported-gil'),
        ]
        for path, text, field, wrong in examples:
            with self.subTest(field=field, path=path):
                source = Source(path, text)
                sites = unsafe.discover(source)
                records = unsafe_records(sites)
                if 'sifr_cache_storage' in path:
                    records['sites'][0]['owner'] = 'cache-owner'
                for obligation in records['sites'][0]['obligations'].values():
                    obligation['proofs'].append(proof_reference(source, 'f', text[text.index('{') + 1:text.rindex('}')].strip()))
                self.assertFalse(unsafe.validate(sites, records, sources=[source]))
                without_proof = copy.deepcopy(records)
                for obligation in without_proof['sites'][0]['obligations'].values():
                    obligation['proofs'] = []
                self.assertTrue(any('obligation/proof' in e
                                    for e in unsafe.validate(sites, without_proof, sources=[source])))
                records['sites'][0]['obligations'][field]['rule'] = wrong
                self.assertTrue(any(f'invalid {field} obligation' in e
                                    for e in unsafe.validate(sites, records, sources=[source])))

        from unsafe_policy_contracts import operation_family
        for operation, family in [('GetNamedSecurityInfoW(path)', 'windows-security'),
                                  ('SetNamedSecurityInfoW(path)', 'windows-security'),
                                  ('LocalFree(descriptor)', 'windows-localfree'),
                                  ('CloseHandle(token)', 'raw-access'),
                                  ('CreateDirectoryW(path, attrs)', 'raw-access'),
                                  ('File::from_raw_handle(handle)', 'raw-access'),
                                  ('MoveFileExW(old, new, flags)', 'raw-access'),
                                  ('information.assume_init()', 'raw-access')]:
            site = unsafe.discover(Source('crates/sifr_cache_storage/src/windows_storage_security.rs',
                                         'fn f() { unsafe { ' + operation + '; } }'))[0]
            self.assertEqual(operation_family(site), family, operation)

    def test_owner_segments_reject_missing_duplicate_and_stale_sites(self):
        from unsafe_policy_segments import SEGMENTS, segment_for, validate_segments
        cases = [
            ('crates/sifr_runtime/src/python.rs', '#[allow(unsafe_code)] mod arrow_ops;', 'python_resources'),
            ('crates/sifr_runtime/src/python/buffer_ops/raw.rs', 'fn f() { unsafe { read(x); } }', 'python_resources'),
            ('crates/sifr_runtime/src/python/buffer_ops/typed_access_evidence_tests.rs', 'fn f() { unsafe { read(x); } }', 'python_resources'),
            ('crates/sifr_runtime/src/python/callbacks/mod.rs', 'fn f() { unsafe { read(x); } }', 'python_core'),
            ('crates/sifr_driver/src/process.rs', 'fn f() { unsafe { read(x); } }', 'external'),
            ('crates/sifr_codegen/src/example.rs', 'fn f() { emit("unsafe { read(x); }"); }', 'external'),
        ]
        sites, segments = [], {name: dict(schema_version=1, segment=name, sites=[]) for name in SEGMENTS}
        for path, text, expected in cases:
            found = unsafe.discover(Source(path, text))
            self.assertEqual(len(found), 1)
            self.assertEqual(segment_for(found[0]), expected)
            records = unsafe_records(found)['sites']
            for field in unsafe.CONTRACT_FIELDS:
                if 'evidence_tests' in path:
                    records[0]['contract'][field] += ' This adversarial test owns the synthetic allocation through its assertion.'
                elif 'callbacks' in path:
                    records[0]['contract'][field] += ' The callback fixture retains its admitted capture until setup drains.'
                elif 'sifr_driver' in path:
                    records[0]['contract'][field] += ' The driver fixture retains its process handle through the call.'
            if 'sifr_driver' in path:
                records[0]['owner'] = 'driver-owner'
            if 'sifr_codegen' in path:
                records[0]['owner'] = 'X02'
            sites += found
            segments[expected]['sites'] += records
        self.assertFalse(validate_segments(sites, segments))
        copied = copy.deepcopy(segments)
        shared_template = {field: 'Owner allocation stays live under admitted access.'
                           for field in unsafe.CONTRACT_FIELDS}
        core = copied['python_core']['sites'][0]
        resource = copied['python_resources']['sites'][1]
        core['contract'], resource['contract'] = shared_template.copy(), shared_template.copy()
        self.assertTrue(any('repeated owner-level' in e for e in validate_segments(sites, copied)))
        for record in (core, resource):
            record['contract_ref'] = 'reviewed-cross-segment'
        copied['python_core']['shared_contracts'] = {'reviewed-cross-segment': {
            'contract': shared_template,
            'review_rationale': 'These two fixture accesses share one admitted allocation.',
            'bindings': {record['site']: record['fingerprint'] for record in (core, resource)}}}
        self.assertFalse(validate_segments(sites, copied))
        partial = {'python_core': segments['python_core']}
        self.assertFalse(validate_segments(sites, partial, selected=('python_core',)))
        self.assertTrue(any('missing required' in e for e in validate_segments(sites, partial)))
        duplicate = copy.deepcopy(segments)
        duplicate['python_resources']['sites'].append(duplicate['python_core']['sites'][0])
        self.assertTrue(any('duplicate' in e for e in validate_segments(sites, duplicate)))
        missing = copy.deepcopy(segments)
        missing['external']['sites'].pop()
        self.assertTrue(any('new/unclassified' in e for e in validate_segments(sites, missing)))
        stale = copy.deepcopy(segments)
        stale['python_core']['sites'][0]['site'] += '-obsolete'
        self.assertTrue(any('stale' in e for e in validate_segments(sites, stale)))
        mutated = copy.deepcopy(segments)
        mutated['python_resources']['sites'][1]['fingerprint'] = '0' * 64
        self.assertTrue(any('changed fingerprint' in e for e in validate_segments(sites, mutated)))

    def test_cold_self_tests_execute_independent_assertions(self):
        with tempfile.TemporaryDirectory(prefix='sifr-h02h0-independent-') as temp:
            root = Path(temp) / 'scripts'
            root.mkdir()
            for name in ('architecture_policy_test_fixtures.py', 'architecture_policy_python_core_tests.py', 'architecture_policy_python_resource_tests.py',
                         'resource_policy_effects.py', 'rust_policy_sites.py', 'method_policy_constituents.py',
                         'method_policy_nodes.py', 'method_policy_semantics.py',
                         'unsafe_policy_contracts.py', 'unsafe_policy_segments.py',
                         'check_method_dispatch_authority.py', 'check_unsafe_abi_contracts.py',
                         'test_architecture_policy_guards.py'):
                shutil.copyfile(Path(__file__).parent / name, root / name)
            self.assertEqual([p.name for p in Path(temp).iterdir()], ['scripts'])
            env = dict(os.environ, SIFR_POLICY_COLD_CHILD='1', PYTHONPATH=str(root), PYTHONDONTWRITEBYTECODE='1')
            for name, minimum, poison in (
                    ('check_method_dispatch_authority.py', 12, 'discover'),
                    ('check_unsafe_abi_contracts.py', 11, 'validate')):
                command = [sys.executable, str(root / name), '--self-test']
                result = subprocess.run(command, cwd=temp, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                import re
                count = re.search(r'Ran (\d+) tests', result.stderr)
                self.assertIsNotNone(count, result.stderr)
                self.assertGreaterEqual(int(count[1]), minimum)
                self.assertNotIn('skipped', result.stderr)
                script = root / name
                original = script.read_text()
                marker = original.index('\n', original.index('def ' + poison + '('))
                script.write_text(original[:marker] + '\n    return [] # independent mutation probe' + original[marker:])
                result = subprocess.run(command, cwd=temp, env=env, capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0, 'self-test accepted a disabled ' + poison)
                self.assertIn('FAIL', result.stderr)
                script.write_text(original)


class ColdCheckoutTests(unittest.TestCase):
    def test_self_tests_need_only_scripts(self):
        with tempfile.TemporaryDirectory(prefix='sifr-h02h-cold-') as temp:
            script_root = Path(temp) / 'scripts'
            script_root.mkdir()
            for name in ('architecture_policy_test_fixtures.py', 'architecture_policy_python_core_tests.py', 'architecture_policy_python_resource_tests.py',
                         'resource_policy_effects.py', 'rust_policy_sites.py', 'method_policy_constituents.py',
                         'method_policy_nodes.py', 'method_policy_semantics.py',
                         'unsafe_policy_contracts.py', 'unsafe_policy_segments.py',
                         'check_method_dispatch_authority.py',
                         'check_unsafe_abi_contracts.py', 'test_architecture_policy_guards.py'):
                shutil.copyfile(Path(__file__).parent / name, script_root / name)
            for name in ('check_method_dispatch_authority.py', 'check_unsafe_abi_contracts.py'):
                result = subprocess.run([sys.executable, str(script_root / name), '--self-test'],
                                        cwd=temp, capture_output=True, text=True,
                                        env=dict(os.environ, SIFR_POLICY_COLD_CHILD='1'))
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


def run_method_tests() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(MethodTests)
    if not os.environ.get('SIFR_POLICY_COLD_CHILD'):
        suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(ColdCheckoutTests))
    return 0 if unittest.TextTestRunner(verbosity=2, failfast=True).run(suite).wasSuccessful() else 1


def run_unsafe_tests() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(UnsafeTests)
    if not os.environ.get('SIFR_POLICY_COLD_CHILD'):
        suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(ColdCheckoutTests))
    return 0 if unittest.TextTestRunner(verbosity=2, failfast=True).run(suite).wasSuccessful() else 1


if __name__ == '__main__':
    unittest.main()
