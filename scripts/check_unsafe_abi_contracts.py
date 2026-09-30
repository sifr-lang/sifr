"""Bind each unsafe operation/ABI declaration/allowance to a local policy record.

The record belongs to one normalized lexical site and names thread, lifetime,
alias and ownership obligations. It is a review inventory, not runtime safety
proof. SQL, driver, cache and emitted Rust retain their existing owners.
"""
from __future__ import annotations

import argparse
import re
import sys

from unsafe_policy_contracts import validate_contracts
from rust_policy_sites import Source, Site, code_tokens, item_header_end, read_inventory, reconcile, rust_sources

INVENTORY = 'unsafe_abi_sites.json'
CONTRACT_FIELDS = ('thread', 'lifetime', 'alias', 'ownership')
OWNERS = {'H02d0', 'H02d1', 'H02e', 'H02f', 'H02g', 'Python-bridge-owner',
          'SQL-owner', 'cache-owner', 'driver-owner', 'X02', 'runtime-owner', 'test-owner'}
# These are the reviewed ABI module boundaries established by predecessors.
# Adding an arbitrary cohesive=true row cannot authorize a broad allowance.
COHESIVE_MODULES = {
    ('crates/sifr_runtime/src/python.rs', 'arrow_ops'),
    ('crates/sifr_runtime/src/python/buffer_ops.rs', 'raw'),
    ('crates/sifr_runtime/src/python/buffer_ops.rs', 'layout'),
    ('crates/sifr_runtime/src/python/dlpack_ops.rs', 'abi'),
    ('crates/sifr_runtime/src/python/callbacks/mod.rs', 'abi'),
    ('crates/sifr_runtime/src/python/initialization.rs', '<file>'),
    ('crates/sifr_sql_postgresql/src/ffi.rs', '<file>'),
    ('crates/sifr_sql_postgresql/src/guest.rs', '<file>'),
    ('crates/sifr_sql_mysql/src/guest.rs', '<file>'),
    ('crates/sifr_sql_sqlite/src/guest.rs', '<file>'),
    ('crates/sifr_cache_storage/src/windows_storage_security.rs', '<file>'),
}


def discover(source: Source) -> list[Site]:
    ts, ps = source.ts, source.pairs
    spans = []
    for i, t in enumerate(ts):
        if t.kind == 'string':
            if 'sifr_codegen' in source.path and re.search(r'\bunsafe\b', t.value):
                spans.append(('generated-rust', i, i))
            continue
        if t.kind != 'ident' or t.value != 'unsafe':
            continue
        j = i + 1
        if j < len(ts) and ts[j].value == '{':
            spans.append(('unsafe-block', i, ps[j]))
        elif j < len(ts) and ts[j].value == 'impl':
            while j < len(ts) and ts[j].value != '{':
                j += 1
            if j < len(ts):
                spans.append(('unsafe-impl', i, ps[j]))
        elif any(a == i for a, _b, _c, _name in source.functions):
            end = next(c for a, _b, c, _name in source.functions if a == i)
            spans.append(('unsafe-declaration', i, end))
        else:
            # Unsafe fn declarations, extern blocks and callback fn-pointer ABI
            # types all get records; fn-pointer declarations end at ,/>/;/}.
            while j < len(ts) and ts[j].value not in ('{', ';', ',', '>', '}'):
                if ts[j].value in ('(', '['):
                    j = ps[j] + 1
                else:
                    j += 1
            if j < len(ts) and ts[j].value == '{':
                end = ps[j]
            else:
                end = max(i, j - 1)
            spans.append(('unsafe-declaration', i, end))
    for i, t in enumerate(ts):
        if t.value != '#' or i + 1 >= len(ts):
            continue
        j = i + 1
        if ts[j].value == '!':
            j += 1
        if ts[j].value != '[':
            continue
        end = ps[j]
        attr = [x.value for x in ts[j + 1:end]]
        if not ('unsafe_code' in attr and any(x in attr for x in ('allow', 'expect'))):
            continue
        # Bind allowance and its item header, not merely its attribute token.
        k = item_header_end(ts, ps, end + 1)
        # An inner attribute is attached to its file/module, not its next use.
        if ts[i + 1].value == '!':
            k = end
        if k < len(ts) and ts[k].value == '{':
            k = ps[k]
        spans.append(('unsafe-allowance', i, min(k, len(ts) - 1)))
    return [source.site(kind, a, b) for kind, a, b in sorted(spans, key=lambda x: x[1])]


def allowance_target(site: Site) -> tuple[str, str]:
    source = Source(site.path, site.text)
    ts = source.ts
    if len(ts) > 1 and ts[1].value == '!':
        return 'module', '<file>'
    end = source.pairs[1]
    # Ignore further attributes and pub(...) visibility before target keywords.
    header_end = item_header_end(ts, source.pairs, end + 1)
    remaining = ts[end + 1:header_end]
    for i, t in enumerate(remaining):
        if t.value == 'fn':
            return 'function', remaining[i + 1].value if i + 1 < len(remaining) else ''
        if t.value == 'mod':
            return 'module', remaining[i + 1].value
        if t.value == 'impl':
            words = [x.value for x in remaining[:i + 3]]
            if 'unsafe' in words and any(x in words for x in ('Send', 'Sync')):
                return 'thread-marker', ''
            return 'broad-item', 'impl'
        if t.value == 'extern' and not any(x.value == 'fn' for x in remaining):
            return 'broad-item', 'extern'
        if t.value in ('struct', 'enum', 'trait', 'type', 'const', 'static', 'use'):
            return 'broad-item', t.value
    return 'expression', ''


def validate(sites: list[Site], inventory: dict, *, sources=(), compare_repeated=True) -> list[str]:
    errors = reconcile(sites, inventory['sites'])
    by_key = {s.key: s for s in sites}
    errors += validate_contracts(sites, inventory, sources, compare_repeated=compare_repeated)
    for record in inventory['sites']:
        key = record.get('site')
        owner = record.get('owner')
        if owner not in OWNERS:
            errors.append(f'invalid unsafe owner: {key}')
        site = by_key.get(key)
        if not site:
            continue
        operation = ' '.join(t.value for t in code_tokens(site.text))
        if record.get('operation') != operation:
            errors.append(f'missing or stale local operation binding: {key}')
        if record.get('source_evidence') != site.source_evidence:
            errors.append(f'stale local SAFETY evidence: {key}')
        if site.kind == 'generated-rust' and owner != 'X02':
            errors.append(f'generated Rust must remain with X02: {key}')
        if 'sifr_sql_' in site.path and owner != 'SQL-owner':
            errors.append(f'SQL ownership changed: {key}')
        if 'sifr_cache_storage/' in site.path and owner != 'cache-owner':
            errors.append(f'cache ownership changed: {key}')
        if 'sifr_driver/' in site.path and owner != 'driver-owner':
            errors.append(f'driver ownership changed: {key}')
        if site.kind == 'unsafe-allowance':
            target, name = allowance_target(site)
            if target == 'broad-item' or (target == 'module' and
                                        (site.path, name) not in COHESIVE_MODULES and
                                        record.get('scope') != 'test-module'):
                errors.append(f'broad item-level unsafe allowance: {key}')
            if record.get('scope') == 'test-module' and not site.test_only:
                errors.append(f'test allowance lacks adjacent cfg(test): {key}')
            if record.get('scope') == 'test-module' and name not in ('tests', 'h02_contract_tests',
                                                                    'h02_core_tests', 'declaration_tests',
                                                                    'current_tests', 'asyncio_tests',
                                                                    'foreign_tests', 'ownership_tests',
                                                                    'release_evidence_tests',
                                                                    'typed_access_evidence_tests'):
                errors.append(f'non-test module allowance labelled test: {key}')
            if target == 'expression' and record.get('scope') != 'expression':
                errors.append(f'unsafe allowance has no recognized local target: {key}')
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        from test_architecture_policy_guards import run_unsafe_tests
        return run_unsafe_tests()
    parser.error('live unsafe segments are not delivered by H02h0; use validate_segments')



if __name__ == '__main__':
    raise SystemExit(main())
