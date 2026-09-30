"""Require reviewed semantic classifications of literal-based Rust dispatch sites.

Lexical discovery deliberately ignores selector spelling: renamed bindings and
new files still require review. Classification expresses ownership, not a claim
that this scan can prove language semantics or generated runtime safety.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import sys

from rust_policy_sites import Source, Site, read_inventory, reconcile, rust_sources

INVENTORY = 'method_dispatch_sites.json'
CLASSES = {
    'source-method-admission': 'sifr_codegen',
    'builtin-call-emission': 'sifr_codegen',
    'typed-lowering': 'sifr_lowering',
    'compile-time-semantics': 'sifr_frontend',
    'language-emission': 'sifr_codegen::methods',
    'language-emission-constituent': 'sifr_codegen::methods',
    'user-protocol-dispatch': 'source-declaration-owner',
    'contextual-rust-adaptation': 'sifr_codegen',
    'rust-ir-consumption': 'sifr_codegen',
    'declaration-resolution': 'source-declaration-owner',
    'test-assertion': 'test-owner',
    'package-protocol': 'package-owner',
    'sql-protocol': 'SQL-owner',
    'cache-protocol': 'cache-owner',
    'driver-protocol': 'driver-owner',
    'generated-code': 'X02',
}
# The only source-language runtime emission registry. Constituent Rust helpers
# implement that authority; they cannot open a second string dispatch registry.
LANGUAGE_AUTHORITY = ('crates/sifr_codegen/src/methods/mod.rs', 'lower_method_impl')
CODEGEN_ADMISSION_PATH = 'crates/sifr_codegen/src/method_call_emitter.rs'
CONST_AUTHORITY = ('crates/sifr_frontend/src/const_evaluator.rs', 'eval_method_call')
from method_policy_constituents import CONSTITUENTS, validate_relationships


def discover(source: Source) -> list[Site]:
    ts, ps = source.ts, source.pairs
    spans = []
    for i, t in enumerate(ts):
        if t.kind == 'ident' and t.value == 'MethodCall' and i + 1 < len(ts) and ts[i + 1].value == '{':
            start = i - 2 if i >= 2 and ts[i - 1].value == '::' else i
            spans.append(('method-call-carrier', start, ps[i + 1]))
        if t.kind != 'ident' or t.value not in ('match', 'matches'):
            continue
        if t.value == 'matches' and i + 2 < len(ts) and ts[i + 1].value == '!':
            opening = i + 2
            if ts[opening].value != '(':
                continue
            end = ps[opening]
        elif t.value == 'match':
            opening = i + 1
            while opening < len(ts) and ts[opening].value != '{':
                if ts[opening].value in ('(', '['):
                    opening = ps[opening] + 1
                else:
                    opening += 1
            if opening == len(ts):
                continue
            end = ps[opening]
        else:
            continue
        # A literal in a branch body is not by itself a string pattern. Look
        # for a literal on the pattern side of =>, or matches!' pattern side.
        has_pattern = False
        if t.value == 'matches':
            comma = next((j for j in range(opening + 1, end) if ts[j].value == ','), end)
            has_pattern = any(x.kind == 'string' for x in ts[comma + 1:end])
        else:
            j = opening + 1
            arm_start = j
            while j < end:
                if ts[j].value == '=>':
                    if any(x.kind == 'string' for x in ts[arm_start:j]):
                        has_pattern = True
                        break
                    j += 1
                    if j < end and ts[j].value == '{':
                        j = ps[j] + 1
                        if j < end and ts[j].value == ',':
                            j += 1
                        arm_start = j
                        continue
                    # Skip the arm expression's nested groups until its comma.
                    while j < end and ts[j].value != ',':
                        if ts[j].value in ('{', '(', '['):
                            j = ps[j] + 1
                        else:
                            j += 1
                    arm_start = j + 1
                j += 1
        if has_pattern:
            spans.append(('literal-match', i, end))
        elif any(x.kind == 'ident' and x.value in ('MethodDispatchAuthority', 'MethodAuthority')
                 for x in ts[opening + 1:end]):
            spans.append(('typed-authority-match', i, end))
    # A string comparison on either side, including equality in guard closures.
    # No assumptions about method/name/attr spelling or variable provenance.
    for i, t in enumerate(ts):
        if t.value not in ('==', '!='):
            continue
        a, b = i - 1, i + 1
        while a > 0 and ts[a - 1].value not in (';', '{', '}', '=>', '&&', '||', ','):
            a -= 1
        while b + 1 < len(ts) and ts[b + 1].value not in (';', '{', '}', '=>', '&&', '||', ','):
            b += 1
        if any(x.kind == 'string' for x in ts[a:b + 1]):
            spans.append(('literal-comparison', a, b))
    return [source.site(kind, a, b) for kind, a, b in sorted(set(spans), key=lambda x: x[1])]


def validate(sites: list[Site], inventory: dict) -> list[str]:
    records = inventory['sites']
    errors = reconcile(sites, records)
    by_key = {s.key: s for s in sites}
    for record in records:
        kind = record.get('classification')
        key = record.get('site')
        if kind not in CLASSES or record.get('owner') != CLASSES.get(kind):
            errors.append(f'invalid semantic owner/classification: {key}')
        if not isinstance(record.get('contract'), str) or not record['contract'].strip():
            errors.append(f'missing semantic contract: {key}')
        site = by_key.get(key)
        if not site:
            continue
        admission = (CODEGEN_ADMISSION_PATH, 'source_method_path')
        if kind == 'source-method-admission' and (site.path, site.scope) != admission:
            errors.append(f'second source-method admission: {key}')
        if (site.path, site.scope) == admission and kind != 'source-method-admission':
            errors.append(f'typed admission classification changed: {key}')
        if kind == 'language-emission' and (site.path, site.scope) != LANGUAGE_AUTHORITY:
            errors.append(f'second language-semantics owner: {key}')
        constituent = CONSTITUENTS.get((site.path, site.scope))
        if kind == 'language-emission-constituent' and (not constituent
                or record.get('canonical_authority') != list(LANGUAGE_AUTHORITY)
                or record.get('admission_relationship') != constituent):
            errors.append(f'unregistered or unbound language constituent: {key}')
        if constituent and kind != 'language-emission-constituent':
            errors.append(f'language constituent classification changed: {key}')
        if kind == 'compile-time-semantics' and (site.path, site.scope) != CONST_AUTHORITY:
            errors.append(f'second compile-time-semantics owner: {key}')
        if (site.path, site.scope) == LANGUAGE_AUTHORITY and kind != 'language-emission':
            errors.append(f'canonical registry classification changed: {key}')
        if (site.path, site.scope) == CONST_AUTHORITY and kind != 'compile-time-semantics':
            errors.append(f'constant evaluator classification changed: {key}')
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        from test_architecture_policy_guards import run_method_tests
        return run_method_tests()
    root = Path(__file__).resolve().parent.parent
    try:
        sources = list(rust_sources(root))
        sites = [site for source in sources for site in discover(source)]
        inventory = read_inventory(root, INVENTORY)
        errors = validate(sites, inventory) + validate_relationships(sources, inventory)
    except (ValueError, OSError, KeyError) as error:
        print(f'method dispatch inventory error: {error}', file=sys.stderr)
        return 1
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print(f'method dispatch authority: {len(sites)} classified sites; one runtime language registry')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
