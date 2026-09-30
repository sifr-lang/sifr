"""Independent method/unsafe fixtures shared by cold and foundation suites."""
from __future__ import annotations

import copy
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
    @staticmethod
    def semantic_records(sources):
        from method_policy_semantics import classification, detail
        from method_policy_constituents import relationships
        from method_policy_nodes import nodes
        sites, records = [], []
        for source in sources:
            for site in methods.discover(source):
                kind = classification(source, site, methods)
                record = dict(site.record(), classification=kind, owner=methods.CLASSES[kind],
                              **detail(source, site, kind))
                if kind == 'language-emission-constituent':
                    record.update(canonical_authority=list(methods.LANGUAGE_AUTHORITY),
                                  admission_relationship=methods.CONSTITUENTS[(site.path, site.scope)])
                sites.append(site)
                records.append(record)
        return sites, dict(sites=records, constituents=relationships(sources, nodes()))

    @staticmethod
    def bounded_nodes(adaptations=None):
        from contextlib import ExitStack
        from unittest.mock import patch
        import method_policy_nodes as nodes
        stack = ExitStack()
        for mapping, values in ((nodes.CONSTITUENTS, {}), (nodes.ADAPTATIONS, adaptations or {}),
                                (nodes.ANALYSES, {}), (nodes.ROUTERS, {})):
            stack.enter_context(patch.dict(mapping, values, clear=True))
        return stack

    def test_generated_rust_and_ir_rewrites_cannot_be_protocol_dispatch(self):
        with self.bounded_nodes():
            for path in ('generated_rust_canonicalizer/rewrites.rs', 'ir_optimize/rewrites.rs'):
                source = Source('crates/sifr_codegen/src/' + path,
                    'fn rewrite(expr: &RustExpr) -> bool { matches!(expr, '
                    'RustExpr::MethodCall { method, .. } if method == "clone") }')
                sites, records = self.semantic_records([source])
                self.assertGreater(len(sites), 0)
                self.assertFalse(methods.validate(sites, records, [source]))
                for record in records['sites']:
                    record.update(classification='user-protocol-dispatch',
                                  owner=methods.CLASSES['user-protocol-dispatch'])
                self.assertTrue(any('actual semantic role' in e
                                    for e in methods.validate(sites, records, [source])))

    def test_builtin_specialization_cannot_hide_as_protocol_or_ir_consumer(self):
        with self.bounded_nodes():
            source = Source('crates/sifr_codegen/src/new_specialization.rs',
                'fn hidden(expr: &HirExpr) -> RustExpr { if let HirExpr::MethodCall '
                '{ method: renamed, .. } = expr { if renamed == "append" { '
                'return RustExpr::MethodCall { method: "push", args: vec![] }; } } decline() }')
            sites, records = self.semantic_records([source])
            self.assertGreater(len(sites), 0)
            for kind in ('user-protocol-dispatch', 'rust-ir-consumption', 'compiler-policy'):
                mutated = copy.deepcopy(records)
                for record in mutated['sites']:
                    record.update(classification=kind, owner=methods.CLASSES[kind])
                self.assertTrue(any('unregistered source-method specialization' in e
                                    for e in methods.validate(sites, mutated, [source])))

    def adaptation_fixture(self):
        path = 'crates/sifr_codegen/src/intrinsic_method_emitters/narrowing_helpers.rs'
        source = Source(path, 'fn adapt_owned_mapping_default(method: &str, args: &[HirExpr], '
                        'lowered: &mut [RustExpr]) { if matches!(method, "get" | "pop" | "remove") '
                        '{ coerce_owned(args, lowered); } } '
                        'fn caller(expr: &HirExpr) { let path = source_method_path(expr)?; '
                        'if path.is_builtin() { adapt_owned_mapping_default(method, args, lowered); } }')
        key = (path, 'adapt_owned_mapping_default')
        return source, {key: 'caller source_method_path -> is_builtin before owned default adaptation'}

    def test_narrowing_and_mapping_helpers_require_actual_admission_relationships(self):
        path = 'crates/sifr_codegen/src/stmt_support_emitter/expr_call_metadata.rs'
        narrowing = Source(path, 'fn is_narrowable_pop_call_for_ir(method: &str, '
                           'args: &[HirExpr]) -> bool { matches!((method, args), '
                           '("pop", []) | ("popleft", [])) } '
                           'fn comparison(expr: &HirExpr) -> bool { lower_stmt_expr_for_ir(expr)?; '
                           'is_narrowable_pop_call_for_ir(method, args) }')
        narrowing_graph = {(path, 'is_narrowable_pop_call_for_ir'):
                           'argument shape predicate; comparison caller first lowers/admit operands'}
        for source, graph in (self.adaptation_fixture(), (narrowing, narrowing_graph)):
            with self.subTest(helper=next(iter(graph))[1]), self.bounded_nodes(graph):
                sites, records = self.semantic_records([source])
                self.assertFalse(methods.validate(sites, records, [source]))
                for field in ('contract', 'caller_admission_relationship', 'inputs', 'outputs', 'role'):
                    mutated = copy.deepcopy(records)
                    mutated['sites'][0][field] = 'The selected declaration owns this closed protocol dispatch.'
                    self.assertTrue(any('actual ' + field in e
                                        for e in methods.validate(sites, mutated, [source])))
                mutated = copy.deepcopy(records)
                mutated['constituents'][0]['references'] = []
                self.assertTrue(any('relationships changed' in e
                                    for e in methods.validate(sites, mutated, [source])))

    def test_new_or_changed_specialization_caller_invalidates_admission(self):
        source, graph = self.adaptation_fixture()
        with self.bounded_nodes(graph):
            sites, records = self.semantic_records([source])
            self.assertFalse(methods.validate(sites, records, [source]))
            for text in (source.text.replace('if path.is_builtin()', 'if true'),
                         source.text.replace('source_method_path(expr)?', 'invented_path(expr)'),
                         source.text[:source.text.index('fn caller')],
                         source.text + ' fn bypass() { adapt_owned_mapping_default(m, a, r); }',
                         source.text + ' use self::adapt_owned_mapping_default as bypass;',
                         source.text + ' macro_rules! bypass { () => { adapt_owned_mapping_default(m,a,r) } }'):
                changed = Source(source.path, text)
                # Re-author isolated site fingerprints so rejection must come
                # from the independently retained caller/admission graph.
                changed_sites, fresh = self.semantic_records([changed])
                fresh['constituents'] = records['constituents']
                self.assertTrue(any('relationships changed' in e
                                    for e in methods.validate(changed_sites, fresh, [changed])))

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
