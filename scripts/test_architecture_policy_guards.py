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


def method_records(sites, classification='package-protocol'):
    return {'sites': [dict(s.record(), classification=classification,
                          owner=methods.CLASSES[classification],
                          contract='The selected declaration owns this closed protocol dispatch.')
                      for s in sites]}


def unsafe_records(sites):
    from unsafe_policy_contracts import FIELDS, FAMILY_RULES, operation_family
    result = {'sites': []}
    for site in sites:
        family = operation_family(site)
        proof = dict(kind='operation', site=site.key, fingerprint=site.fingerprint, quote=site.text)
        contract = {field: f'Fixture {site.kind} establishes {rule} for its allocation.'
                    for field, rule in zip(FIELDS, FAMILY_RULES[family])}
        result['sites'].append(dict(site.record(), owner='runtime-owner', scope='function',
                                   operation=' '.join(t.value for t in code_tokens(site.text)),
                                   operation_family=family, source_evidence=site.source_evidence,
                                   contract=contract, obligations={
                                       field: dict(rule=rule, proofs=[proof.copy()])
                                       for field, rule in zip(FIELDS, FAMILY_RULES[family])}))
    return result


class MethodTests(unittest.TestCase):
    def setUp(self):
        self.path = 'crates/example/src/lib.rs'
        self.text = 'fn dispatch(renamed: &str) { match renamed { "append" => emit(), _ => decline() } }'
        self.sites = methods.discover(Source(self.path, self.text))
        self.inventory = method_records(self.sites)

    def test_constituent_admission_and_new_reference_are_bound(self):
        import method_policy_constituents as constituents
        from unittest.mock import patch
        path = 'crates/sifr_codegen/src/example.rs'
        scope = 'emit_builtin'
        original = Source(path, 'fn emit_builtin(x: X) { admit(x); emit(); } fn caller(x: X) { emit_builtin(x); }')
        with patch.dict(constituents.CONSTITUENTS, {(path, scope): 'admit before emission'}, clear=True):
            records = {'constituents': constituents.relationships([original])}
            self.assertFalse(constituents.validate_relationships([original], records))
            for changed in (original.text.replace('admit(x);', ''),
                            original.text + ' fn bypass(x: X) { emit_builtin(x); }'):
                self.assertTrue(constituents.validate_relationships([Source(path, changed)], records))

    def test_normalization_is_site_based(self):
        self.assertEqual(len(self.sites), 1)
        moved = '\n/* preceding comment */\n' + self.text.replace('match renamed', 'match /* c */ renamed')
        self.assertFalse(methods.validate(methods.discover(Source(self.path, moved)), self.inventory))
        body_changed = self.text.replace('emit()', 'other()')
        self.assertTrue(any('changed fingerprint' in e for e in methods.validate(
            methods.discover(Source(self.path, body_changed)), self.inventory)))

    def test_comparison_branch_body_and_default_are_fingerprinted(self):
        original = 'fn f(x: &str) { if x == "append" { emit(); } else { decline(); } }'
        sites = methods.discover(Source(self.path, original))
        records = method_records(sites)
        for old, new in [('emit()', 'other()'), ('decline()', 'fallback()')]:
            changed = methods.discover(Source(self.path, original.replace(old, new)))
            self.assertTrue(any('changed fingerprint' in e for e in methods.validate(changed, records)))

    def test_generic_signatures_and_macro_context_bind_branch_bodies(self):
        bodies = [
            'fn f(x: &str) -> impl Iterator<Item = u8> { BODY }',
            'fn f<F>(x: &str) where F: Future<Output = ()> { BODY }',
            'macro_rules! f { ($x:ident) => {{ BODY }} }',
        ]
        for body in bodies:
            original = body.replace('BODY', 'if x == "append" { a(); } else { b(); }')
            sites = methods.discover(Source(self.path, original))
            self.assertEqual(len(sites), 1)
            self.assertNotEqual(sites[0].scope, '<module>')
            changed = methods.discover(Source(self.path, original.replace('a()', 'c()')))
            self.assertTrue(any('changed fingerprint' in e
                                for e in methods.validate(changed, method_records(sites))))

    def test_comma_less_block_arms_do_not_hide_later_patterns(self):
        for first in ('_ if g() => { a() }', 'Some(x) => { a() }'):
            text = 'fn f(n: X) { match n { ' + first + ' "append" => { b() } _ => {} } }'
            self.assertEqual(len(methods.discover(Source(self.path, text))), 1)

    def test_renamed_dispatch_and_new_site_are_detected(self):
        renamed = self.text.replace('renamed', 'operation')
        found = methods.discover(Source(self.path, renamed))
        self.assertEqual(len(found), 1)
        self.assertTrue(any('changed fingerprint' in e for e in methods.validate(found, self.inventory)))
        new = self.text + ' fn other(x: &str) { if "cloned" == x { emit(); } }'
        self.assertTrue(any('new/unclassified' in e for e in methods.validate(
            methods.discover(Source(self.path, new)), self.inventory)))

    def test_stale_site_and_fingerprint(self):
        self.assertTrue(any('stale site' in e for e in methods.validate([], self.inventory)))
        stale = copy.deepcopy(self.inventory)
        stale['sites'][0]['fingerprint'] = '0' * 64
        self.assertTrue(any('changed fingerprint' in e for e in methods.validate(self.sites, stale)))

    def test_second_language_semantics_owner(self):
        second = method_records(self.sites, 'language-emission')
        self.assertTrue(any('second language-semantics owner' in e
                            for e in methods.validate(self.sites, second)))
        canonical = methods.discover(Source(methods.LANGUAGE_AUTHORITY[0], self.text.replace(
            'fn dispatch', 'fn lower_method_impl')))
        self.assertFalse(methods.validate(canonical, method_records(canonical, 'language-emission')))
        self.assertTrue(any('classification changed' in e for e in methods.validate(
            canonical, method_records(canonical))))

    def test_constituents_cannot_be_extended_unbound_or_relabelled(self):
        from unittest.mock import patch
        path, scope = 'crates/sifr_codegen/src/fixture.rs', 'emit_builtin'
        text = self.text.replace('fn dispatch', 'fn ' + scope)
        sites = methods.discover(Source(path, text))
        with patch.dict(methods.CONSTITUENTS, {(path, scope): 'fixture admission'}, clear=True):
            records = method_records(sites, 'language-emission-constituent')
            for record in records['sites']:
                record['canonical_authority'] = list(methods.LANGUAGE_AUTHORITY)
                record['admission_relationship'] = 'fixture admission'
            self.assertFalse(methods.validate(sites, records))
            self.assertTrue(any('classification changed' in e
                                for e in methods.validate(sites, method_records(sites))))
            records['sites'][0]['admission_relationship'] = 'invented gate'
            self.assertTrue(any('unbound' in e for e in methods.validate(sites, records)))
        second = method_records(self.sites, 'language-emission-constituent')
        self.assertTrue(any('unregistered' in e for e in methods.validate(self.sites, second)))

    def test_comparisons_and_matches_ignore_binding_spelling(self):
        text = 'fn f(x: &str) { if x == "append" {} if "cloned" != x {} matches!(x, "upper" | "lower"); }'
        self.assertEqual(len(methods.discover(Source(self.path, text))), 3)
        self.assertEqual(methods.discover(Source(self.path,
            'fn f(x: i32) { match x { 1 => log("append"), _ => log("lower") } }')), [])

    def test_typed_carrier_and_authority_are_classified(self):
        text = 'fn f(x: X) { match x { MethodDispatchAuthority::Builtin => emit(), _ => decline() } HirExpr::MethodCall { method: renamed, authority: a }; }'
        found = methods.discover(Source(self.path, text))
        self.assertEqual([s.kind for s in found], ['typed-authority-match', 'method-call-carrier'])
        self.assertTrue(found[1].text.startswith('HirExpr::MethodCall'))
        self.assertFalse(methods.validate(found, method_records(found)))

    def test_literals_and_comments_are_not_dispatch(self):
        text = '''fn f() { let c = '{'; let x = r###"match renamed { "append" => unsafe {} }"###;
          let b = b'}'; let s = "if method == \\\"append\\\"";
          /* outer /* match x { "append" => 1 } */ still comment */
          // if x == "upper" {}
        }'''
        self.assertFalse(methods.discover(Source(self.path, text)))


class UnsafeTests(unittest.TestCase):
    def setUp(self):
        self.path = 'crates/example/src/lib.rs'
        self.text = "fn load<'a>(x: &'a u8) { unsafe { deref(x); } }"
        self.sites = unsafe.discover(Source(self.path, self.text))
        self.inventory = unsafe_records(self.sites)

    def test_lifetimes_do_not_mask_unsafe(self):
        self.assertEqual(len(self.sites), 1)
        text = "unsafe fn read<'a>(x: &'a u8) { unsafe { deref(x); } }"
        found = unsafe.discover(Source(self.path, text))
        self.assertEqual([s.kind for s in found], ['unsafe-declaration', 'unsafe-block'])
        self.assertFalse(unsafe.validate(found, unsafe_records(found)))

    def test_character_literal_masking(self):
        text = r'''fn f<'a>(x: &'a u8) { let a = '\''; let b = '\\'; let c = '"';
           let d = '\u{7b}'; let e = b'}'; let s = c"unsafe { hidden() }";
           let r = br##"unsafe { hidden() }"##; unsafe { deref(x); } }'''
        found = unsafe.discover(Source(self.path, text))
        self.assertEqual(len(found), 1)
        self.assertEqual(found[0].text, 'unsafe { deref(x); }')
        self.assertTrue(any(t.value == 'a' for t in code_tokens(text)))

    def test_admission_context_is_part_of_unsafe_fingerprint(self):
        original = "fn load<'a>(x: &'a u8) { validate(x); unsafe { deref(x); } }"
        sites = unsafe.discover(Source(self.path, original))
        changed = unsafe.discover(Source(self.path, original.replace('validate(x);', '')))
        self.assertTrue(any('changed fingerprint' in e for e in unsafe.validate(changed, unsafe_records(sites))))
        generic = original.replace("fn load<'a>(x: &'a u8)",
                                   "fn load<'a, F>(x: &'a u8) where F: Future<Output = ()>")
        sites = unsafe.discover(Source(self.path, generic))
        self.assertEqual(sites[0].scope, 'load')
        changed = unsafe.discover(Source(self.path, generic.replace('validate(x);', '')))
        self.assertTrue(any('changed fingerprint' in e for e in unsafe.validate(changed, unsafe_records(sites))))

    def test_operation_and_local_safety_evidence_cannot_be_pasted(self):
        text = 'fn load(x: X) {\n // SAFETY: caller pins x through this read.\n unsafe { read(x); } }'
        sites = unsafe.discover(Source(self.path, text))
        records = unsafe_records(sites)
        self.assertFalse(unsafe.validate(sites, records))
        records['sites'][0]['operation'] = 'unsafe { unrelated(); }'
        self.assertTrue(any('operation binding' in e for e in unsafe.validate(sites, records)))
        records = unsafe_records(sites)
        changed = unsafe.discover(Source(self.path, text.replace('pins x', 'pins another')))
        self.assertTrue(any('SAFETY evidence' in e for e in unsafe.validate(changed, records)))

    def test_missing_local_contract_is_rejected(self):
        for field in unsafe.CONTRACT_FIELDS:
            mutated = copy.deepcopy(self.inventory)
            del mutated['sites'][0]['contract'][field]
            self.assertTrue(any('missing local' in e for e in unsafe.validate(self.sites, mutated)))
        self.assertTrue(any('new/unclassified' in e for e in unsafe.validate(self.sites, {'sites': []})))

    def test_repeated_owner_contract_cannot_be_pasted_for_new_operation(self):
        sites = unsafe.discover(Source(self.path, self.text + ' fn free() { unsafe { release(); } }'))
        records = unsafe_records(sites)
        for i, record in enumerate(records['sites']):
            record['contract'] = {k: f'owner / unsafe-block #{i}: Shared owner template.'
                                  for k in unsafe.CONTRACT_FIELDS}
        self.assertTrue(any('repeated owner-level' in e for e in unsafe.validate(sites, records)))

    def test_broad_allowance_rejected(self):
        for item in ('impl T { fn f() {} }', 'struct T;', 'enum T {}', 'mod runtime;',
                     'unsafe extern "C" { fn f(); }'):
            text = '#[allow(unsafe_code)] ' + item
            sites = unsafe.discover(Source(self.path, text))
            records = unsafe_records(sites)
            self.assertTrue(any('broad item-level' in e for e in unsafe.validate(sites, records)), item)
        sites = unsafe.discover(Source(self.path, '#![allow(unsafe_code)] fn f() {}'))
        self.assertTrue(any('broad item-level' in e for e in unsafe.validate(sites, unsafe_records(sites))))

    def test_fake_test_module_cannot_bypass_broad_allowance(self):
        sites = unsafe.discover(Source(self.path, '#[allow(unsafe_code)] mod tests;'))
        records = unsafe_records(sites)
        records['sites'][0]['scope'] = 'test-module'
        self.assertTrue(any('lacks adjacent cfg(test)' in e for e in unsafe.validate(sites, records)))
        sites = unsafe.discover(Source(self.path, '#[cfg(test)] #[allow(unsafe_code)] mod tests;'))
        records = unsafe_records(sites)
        records['sites'][0]['scope'] = 'test-module'
        self.assertFalse(unsafe.validate(sites, records))

    def test_narrow_function_marker_and_cohesive_abi(self):
        for text in ('#[allow(unsafe_code)] unsafe fn f() {}',
                     '#[allow(unsafe_code)] unsafe impl Send for T {}'):
            sites = unsafe.discover(Source(self.path, text))
            self.assertFalse(unsafe.validate(sites, unsafe_records(sites)))
        path = 'crates/sifr_runtime/src/python/buffer_ops.rs'
        sites = unsafe.discover(Source(path, '#[allow(unsafe_code)] mod raw;'))
        self.assertFalse(unsafe.validate(sites, unsafe_records(sites)))

    def test_mutation_stale_and_external_owners(self):
        mutated = self.text.replace('deref(x)', 'free(x)')
        self.assertTrue(any('changed fingerprint' in e for e in unsafe.validate(
            unsafe.discover(Source(self.path, mutated)), self.inventory)))
        self.assertTrue(any('stale site' in e for e in unsafe.validate([], self.inventory)))
        for path, expected in [('crates/sifr_sql_postgresql/src/ffi.rs', 'SQL'),
                               ('crates/sifr_driver/src/process.rs', 'driver'),
                               ('crates/sifr_cache_storage/src/unix.rs', 'cache')]:
            sites = unsafe.discover(Source(path, self.text))
            self.assertTrue(any(expected + ' ownership changed' in e
                                for e in unsafe.validate(sites, unsafe_records(sites))))

    def test_generated_rust_is_delegated(self):
        path = 'crates/sifr_codegen/src/emit.rs'
        sites = unsafe.discover(Source(path, 'fn f() { emit("unsafe { callback() }"); }'))
        self.assertEqual(len(sites), 1)
        records = unsafe_records(sites)
        self.assertTrue(any('X02' in e for e in unsafe.validate(sites, records)))
        records['sites'][0]['owner'] = 'X02'
        self.assertFalse(unsafe.validate(sites, records))


class PolicyFoundationTests(unittest.TestCase):
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
                     site.path.upper(), site.path + '/different.rs')
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
            for name in ('rust_policy_sites.py', 'method_policy_constituents.py',
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
            for name in ('rust_policy_sites.py', 'method_policy_constituents.py',
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
