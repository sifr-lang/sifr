"""Bounded positive/negative fixtures for H02h; runnable without repository state."""
from __future__ import annotations

import copy
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
    return {'sites': [dict(s.record(), owner='runtime-owner', scope='function',
                          contract={k: 'This fixture pins one allocation and transfers it once.'
                                    for k in unsafe.CONTRACT_FIELDS}) for s in sites]}


class MethodTests(unittest.TestCase):
    def setUp(self):
        self.path = 'crates/example/src/lib.rs'
        self.text = 'fn dispatch(renamed: &str) { match renamed { "append" => emit(), _ => decline() } }'
        self.sites = methods.discover(Source(self.path, self.text))
        self.inventory = method_records(self.sites)

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

    def test_missing_local_contract_is_rejected(self):
        for field in unsafe.CONTRACT_FIELDS:
            mutated = copy.deepcopy(self.inventory)
            del mutated['sites'][0]['contract'][field]
            self.assertTrue(any('missing local' in e for e in unsafe.validate(self.sites, mutated)))
        self.assertTrue(any('new/unclassified' in e for e in unsafe.validate(self.sites, {'sites': []})))

    def test_broad_allowance_rejected(self):
        for item in ('impl T { fn f() {} }', 'struct T;', 'enum T {}', 'mod runtime;'):
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


class ColdCheckoutTests(unittest.TestCase):
    def test_self_tests_need_only_scripts(self):
        if '--cold-child' in sys.argv:
            return
        with tempfile.TemporaryDirectory(prefix='sifr-h02h-cold-') as temp:
            script_root = Path(temp) / 'scripts'
            script_root.mkdir()
            for name in ('rust_policy_sites.py', 'check_method_dispatch_authority.py',
                         'check_unsafe_abi_contracts.py', 'test_architecture_policy_guards.py'):
                shutil.copyfile(Path(__file__).parent / name, script_root / name)
            for name in ('check_method_dispatch_authority.py', 'check_unsafe_abi_contracts.py'):
                result = subprocess.run([sys.executable, str(script_root / name), '--self-test'],
                                        cwd=temp, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


def run_method_tests() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(MethodTests)
    return 0 if unittest.TextTestRunner(verbosity=2, failfast=True).run(suite).wasSuccessful() else 1


def run_unsafe_tests() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(UnsafeTests)
    return 0 if unittest.TextTestRunner(verbosity=2, failfast=True).run(suite).wasSuccessful() else 1


if __name__ == '__main__':
    unittest.main()
