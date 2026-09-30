"""Pure, source-bound unsafe policy admission; never executable safety proof.

Rule identifiers express effects rather than searching prose for keywords.
Authors must explain every obligation and bind it to exact code or an adjacent
SAFETY explanation. Scoped review verifies the meaning of those assertions.
"""
from __future__ import annotations

from collections import defaultdict
import re

from rust_policy_sites import Source, code_tokens, fingerprint

FIELDS = ('thread', 'lifetime', 'alias', 'ownership')
# Four operation-family obligations, in FIELDS order. Changes require affected
# policy records and negative fixtures to be revalidated.
FAMILY_RULES = {
    'raw-access': ('caller-thread', 'allocation-live', 'access-admitted', 'borrow-only'),
    'exporter-release': ('supported-gil', 'export-pinned-through-release',
                         'serialized-exclusive-release', 'consume-export-once'),
    'target-erasure': ('send-sync-capture-admission', 'capture-live-through-setup-drain',
                       'stable-target-under-admission', 'wrapper-owns-target-until-drained'),
    'future-erasure': ('send-future-under-admission', 'captures-live-through-revocation',
                       'revocation-excludes-poll-and-drop', 'wrapper-revokes-before-capture-end'),
    'callback-erasure': ('send-sync-capture-admission', 'captures-live-through-setup-drain-and-revocation',
                         'admission-excludes-teardown-and-poll', 'wrapper-drains-target-and-revokes-futures'),
    'windows-localfree': ('windows-api-thread', 'local-allocation-live-through-free',
                         'exclusive-exact-once-free', 'localfree-owned-allocation-once'),
    'windows-security': ('windows-api-thread', 'descriptor-live-through-acl-use',
                         'descriptor-dacl-borrow', 'localfree-owned-descriptor-once'),
    'thread-marker': ('send-sync-type-invariant', 'transferred-storage-live',
                      'shared-access-synchronized', 'transfer-owner-not-attachment'),
    'abi-declaration': ('abi-caller-thread', 'abi-arguments-live',
                        'abi-alias-preconditions', 'abi-transfer-explicit'),
    'unsafe-allowance': ('bounded-allowance-thread', 'bounded-allowance-lifetime',
                         'bounded-allowance-alias', 'bounded-allowance-owner'),
    'generated-rust': ('delegated-X02-thread', 'delegated-X02-lifetime',
                       'delegated-X02-alias', 'delegated-X02-owner'),
}


def operation_family(site) -> str:
    """Identify bounded families by Rust tokens, never by obligation prose.

    This is not an exhaustive ABI classifier. Unknown native operations use
    raw-access obligations and require source-specific authoring and review.
    """
    identifiers = {t.value for t in code_tokens(site.text) if t.kind == 'ident'}
    if site.kind == 'generated-rust':
        return 'generated-rust'
    if site.kind == 'unsafe-allowance':
        return 'unsafe-allowance'
    if 'PyBuffer_Release' in identifiers:
        return 'exporter-release'
    target_erasure = bool(identifiers & {'erase_target_lifetime'})
    future_erasure = bool(identifiers & {'erase_future_lifetime'})
    if identifiers & {'transmute', 'transmute_copy'}:
        target_erasure |= bool(identifiers & {'AsyncioTarget', 'AsyncioTargetPtr',
                                             'ForeignTarget', 'CurrentTarget', 'target'})
        future_erasure |= bool(identifiers & {'BoxCallbackFuture', 'future'})
    if 'build_asyncio_callback' in identifiers or (target_erasure and future_erasure):
        return 'callback-erasure'
    if future_erasure:
        return 'future-erasure'
    if target_erasure:
        return 'target-erasure'
    if identifiers & {'GetNamedSecurityInfoW', 'SetNamedSecurityInfoW',
                      'ConvertStringSecurityDescriptorToSecurityDescriptorW',
                      'GetSecurityDescriptorDacl', 'GetSecurityDescriptorOwner',
                      'GetSecurityDescriptorControl', 'GetAclInformation', 'GetAce'}:
        return 'windows-security'
    if 'LocalFree' in identifiers:
        return 'windows-localfree'
    if site.kind == 'unsafe-impl' and identifiers & {'Send', 'Sync'}:
        return 'thread-marker'
    if site.kind == 'unsafe-declaration':
        return 'abi-declaration'
    return 'raw-access'


def proof_reference(source: Source, scope: str, quote: str) -> dict:
    """Create a code reference; validators independently resolve the binding."""
    contexts = [c for c in source.contexts if c[3] == scope]
    if len(contexts) != 1:
        raise ValueError(f'proof scope missing/ambiguous: {source.path}::{scope}')
    a, _b, c, _name = contexts[0]
    return dict(kind='code', path=source.path, scope=scope, quote=quote,
                context_fingerprint=fingerprint(source.ts[a:c + 1]))


def valid_proof(proof, site, sources) -> bool:
    if not isinstance(proof, dict):
        return False
    quote = proof.get('quote')
    if not isinstance(quote, str) or not quote.strip():
        return False
    if proof.get('kind') == 'operation':
        return (proof.get('site') == site.key
                and proof.get('fingerprint') == site.fingerprint
                and code_tokens(quote) == code_tokens(site.text))
    if proof.get('kind') == 'safety':
        return (bool(site.source_evidence) and proof.get('site') == site.key
                and proof.get('fingerprint') == site.fingerprint
                and quote == site.source_evidence)
    if proof.get('kind') != 'code':
        return False
    candidates = [s for s in sources if s.path == proof.get('path')]
    if len(candidates) != 1:
        return False
    source = candidates[0]
    contexts = [c for c in source.contexts if c[3] == proof.get('scope')]
    if len(contexts) != 1:
        return False
    a, _b, c, _name = contexts[0]
    context = source.ts[a:c + 1]
    quoted = [(t.kind, t.value) for t in code_tokens(quote)]
    actual = [(t.kind, t.value) for t in context]
    return (bool(quoted) and proof.get('context_fingerprint') == fingerprint(context)
            and sum(actual[i:i + len(quoted)] == quoted
                    for i in range(len(actual) - len(quoted) + 1)) == 1)


def normalized_obligations(contract, site, record) -> tuple:
    """Compare prose after removing discovered bindings, never authored meaning.

    Gather complete spans against the original text before removing any of
    them. In particular, a scope inside a key cannot destroy the key match.
    Inventory values are checked separately and cannot influence this key.
    """
    boundary = r'[\w/.:#-]'
    patterns = [
        rf'(?<!{boundary}){re.escape(site.key)}(?!{boundary})',
        rf'(?<!{boundary}){re.escape(site.path)}(?::{site.line})?(?!{boundary})',
        rf'(?<!{boundary}){re.escape(site.scope)}(?!{boundary})',
    ]
    kind = r'[ _-]'.join(re.escape(part) for part in site.kind.split('-'))
    patterns += [
        rf'(?<!\w){kind}(?!\w)(?:\s*#?\s*\d+\b)?\s*:?',
        r'\b(?:site|ordinal|line)\s*[:=]?\s*#?\s*\d+\b',
        r'(?<!\w)#\s*\d+\b',
        r'\(\s*\d+\s*\)',
        rf'(?<![\w/.:#-])(?:{site.ordinal}|{site.line})(?![\w/.:#-])',
    ]
    operation = r'\s*'.join(re.escape(t.value) for t in code_tokens(site.text))
    if operation:
        patterns.append(r'(?<!\w)' + operation + r'(?!\w)')
    if site.source_evidence:
        patterns.append(re.escape(site.source_evidence))
        bodies = [re.sub(r'^\s*(?://[/!]?|/\*+|\*|\*/)?\s*', '', line)
                  .removesuffix('*/').strip() for line in site.source_evidence.splitlines()]
        patterns += [r'(?<!\w)' + r'\s*'.join(re.escape(word) for word in body.split())
                     + r'(?!\w)' for body in bodies if body]
    values = []
    for field in FIELDS:
        text = contract[field]
        spans = [match.span() for pattern in patterns for match in re.finditer(pattern, text)]
        # A maximal hex token must itself be a prefix; matching part of a
        # different hash or digits inside an identifier is not metadata.
        spans += [match.span() for match in re.finditer(r'(?<![\w/])[0-9a-fA-F]+(?![\w/])', text)
                  if len(match[0]) <= 64 and site.fingerprint.lower().startswith(match[0].lower())]
        expanded = []
        for start, end in spans:
            label = re.search(r'\b(?:site|path|operation|source_evidence|evidence|scope|'
                              r'kind|ordinal|line|fingerprint)\s*[:=]\s*$', text[:start])
            expanded.append((label.start() if label else start, end))
        selected = []
        for start, end in sorted(expanded, key=lambda span: (-(span[1] - span[0]), span[0])):
            if not any(start < b and end > a for a, b in selected):
                selected.append((start, end))
        for start, end in sorted(selected, reverse=True):
            text = text[:start] + '\0' + text[end:]
        # Separators only between removed spans cannot distinguish bindings.
        text = re.sub(r'(?<=\0)[\s/:;]*(?=\0)', '', text).replace('\0', '')
        values.append(' '.join(text.split()).strip(' ;:/'))
    return tuple(values)


def validate_contracts(sites, inventory, sources=(), *, compare_repeated=True) -> list[str]:
    errors = []
    records = {r.get('site'): r for r in inventory['sites']}
    shared = inventory.get('shared_contracts', {})
    for site in sites:
        record = records.get(site.key)
        if record is None:
            continue
        contract = record.get('contract')
        if not isinstance(contract, dict) or set(contract) != set(FIELDS) or any(
                not isinstance(contract.get(k), str) or not contract[k].strip() for k in FIELDS):
            errors.append(f'missing local thread/lifetime/alias/ownership contract: {site.key}')
            continue
        family = operation_family(site)
        if record.get('operation_family') != family:
            errors.append(f'operation family mismatch: {site.key}; expected {family}')
        obligations = record.get('obligations')
        if not isinstance(obligations, dict) or set(obligations) != set(FIELDS):
            errors.append(f'missing structured local obligations: {site.key}')
            continue
        for field, rule in zip(FIELDS, FAMILY_RULES[family]):
            obligation = obligations[field]
            proofs = obligation.get('proofs') if isinstance(obligation, dict) else None
            if (not isinstance(obligation, dict) or obligation.get('rule') != rule
                    or not isinstance(proofs, list) or not proofs
                    or not all(valid_proof(p, site, sources) for p in proofs)):
                errors.append(f'invalid {field} obligation/proof ({rule}): {site.key}')
            # The operation alone cannot establish external capture lifetime,
            # revocation or thread admission. Require a source code reference.
            elif family in ('target-erasure', 'future-erasure', 'callback-erasure',
                            'windows-security', 'windows-localfree') and not any(
                    p.get('kind') == 'code' for p in proofs):
                errors.append(f'{field} requires source admission/teardown proof: {site.key}')
    return errors + (validate_repeated_contracts(sites, inventory) if compare_repeated else [])


def validate_repeated_contracts(sites, inventory) -> list[str]:
    """Compare the selected union, resolving explicitly reviewed shared proofs."""
    errors, repeated = [], defaultdict(list)
    by_key = {s.key: s for s in sites}
    shared = inventory.get('shared_contracts', {})
    for record in inventory['sites']:
        site = by_key.get(record.get('site'))
        contract = record.get('contract')
        if site and isinstance(contract, dict) and all(
                isinstance(contract.get(k), str) for k in FIELDS):
            repeated[normalized_obligations(contract, site, record)].append(record)
    for group in repeated.values():
        if len(group) < 2:
            continue
        refs = {r.get('contract_ref') for r in group}
        definition = shared.get(next(iter(refs))) if len(refs) == 1 else None
        if (not isinstance(definition, dict) or not definition.get('review_rationale')
                or any(r['contract'] != definition.get('contract') for r in group)
                or definition.get('bindings') != {
                    r['site']: r['fingerprint'] for r in group}):
            errors.append('repeated owner-level contract lacks local obligations: '
                          + ', '.join(r['site'] for r in group))
    return errors
