"""Pure, source-bound unsafe policy admission; never executable safety proof.

Rule identifiers express effects rather than searching prose for keywords.
Authors must explain every obligation and bind it to exact code or an adjacent
SAFETY explanation. Scoped review verifies the meaning of those assertions.
"""
from __future__ import annotations

from itertools import combinations
import re

from rust_policy_sites import Source, code_tokens, fingerprint

from resource_policy_effects import RESOURCE_RULES, resource_operation_family
from external_policy_effects import EXTERNAL_RULES, external_operation_family

FIELDS = ('thread', 'lifetime', 'alias', 'ownership')
# Four operation-family obligations, in FIELDS order. Changes require affected
# policy records and negative fixtures to be revalidated.
FAMILY_RULES = {
    'current-target-erasure': ('creator-thread-admission', 'capture-live-through-registry-drain',
                               'thread-local-shared-target', 'wrapper-removes-target-before-capture-end'),
    'cpython-config-clear': ('serialized-initialization', 'initialized-config-through-clear',
                             'exclusive-config-clear', 'consume-config-allocations-once'),
    'cpython-config-init': ('serialized-initialization', 'writable-config-storage',
                            'exclusive-config-initialization', 'initialize-config-owner'),
    'cpython-config-move': ('serialized-initialization', 'initialized-config-storage',
                            'exclusive-initialized-config-move', 'transfer-initialized-config-owner'),
    'cpython-config-copy': ('serialized-initialization', 'config-and-input-live-through-copy',
                            'exclusive-config-field-write', 'config-owns-copied-input'),
    'cpython-initialize': ('serialized-initialization', 'configured-storage-live-through-init',
                           'process-init-exclusive', 'borrow-config-process-owns-interpreter'),
    'cpython-detach': ('initializing-thread-holds-gil', 'process-thread-state-live',
                       'release-current-attachment', 'interpreter-keeps-thread-state'),
    'cpython-observation': ('cpython-api-caller', 'observed-state-live',
                            'observe-without-data-alias', 'borrow-only'),
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
FAMILY_RULES.update(RESOURCE_RULES)
FAMILY_RULES.update(EXTERNAL_RULES)


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
        if (identifiers & {'CurrentTarget', 'CurrentTargetPtr'}
                or site.path.endswith('/callbacks/current.rs')):
            return 'current-target-erasure'
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
    resource_family = resource_operation_family(site)
    if resource_family is not None:
        return resource_family
    if site.kind == 'unsafe-declaration':
        return 'abi-declaration'
    external_family = external_operation_family(site)
    if external_family is not None:
        return external_family
    # Preserve the stronger established release/erasure/ABI effects above.
    for name, family in {
        'PyConfig_Clear': 'cpython-config-clear',
        'PyConfig_InitPythonConfig': 'cpython-config-init',
        'PyConfig_SetBytesString': 'cpython-config-copy',
        'PyConfig_SetBytesArgv': 'cpython-config-copy',
        'PyWideStringList_Append': 'cpython-config-copy',
        'Py_InitializeFromConfig': 'cpython-initialize',
        'PyEval_SaveThread': 'cpython-detach',
        'PyGILState_Check': 'cpython-observation',
        'Py_IsInitialized': 'cpython-observation',
        'PyStatus_Exception': 'cpython-observation',
    }.items():
        if name in identifiers:
            return family
    if {'assume_init', 'raw_config'} <= identifiers:
        return 'cpython-config-move'
    if 'asyncio_callback_scoped_with_owner' in identifiers:
        return 'callback-erasure'
    if 'foreign_callback_scoped_with_owner' in identifiers:
        return 'target-erasure'
    if 'current_callback_scoped_with_owner' in identifiers:
        return 'current-target-erasure'
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


def _binding_patterns(site) -> list[str]:
    boundary = r'[\w/.:#-]'
    # Sentence punctuation is a boundary; a path extension/component or an
    # identifier suffix is not. Keep the punctuation in the remaining prose.
    ending = rf'(?:(?!{boundary})|(?=[.:,](?:\s|$)))'
    patterns = [
        rf'(?<!{boundary}){re.escape(site.key)}{ending}',
        rf'(?<!{boundary}){re.escape(site.path)}(?::{site.line})?{ending}',
        rf'(?<!{boundary}){re.escape(site.scope)}{ending}',
    ]
    kind = r'[ _-]'.join(re.escape(part) for part in site.kind.split('-'))
    patterns += [
        rf'(?<!\w){kind}(?!\w)(?:\s*#?\s*\d+\b)?\s*:?',
        r'\b(?:site|ordinal|line)\s*[:=]?\s*#?\s*\d+\b',
        r'(?<!\w)#\s*\d+\b',
        r'\(\s*\d+\s*\)',
        rf'(?<!{boundary})(?:{site.ordinal}|{site.line}){ending}',
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
    return patterns


def normalized_obligations(contract, site, record, *, comparison_sites=None) -> tuple:
    """Compare prose after removing discovered bindings, never authored meaning.

    Gather complete spans against the original text before removing any of
    them. In particular, a scope inside a key cannot destroy the key match.
    Inventory values are checked separately and cannot influence this key.
    Repetition supplies exactly the two compared sites to both records. A
    third discovered binding cannot redefine either record's substantive prose.
    The default single-site form remains useful for exact preservation checks.
    """
    binding_sites = tuple(comparison_sites) if comparison_sites is not None else (site,)
    patterns = set(pattern for binding in binding_sites for pattern in _binding_patterns(binding))
    fingerprints = {binding.fingerprint.lower() for binding in binding_sites}
    values = []
    for field in FIELDS:
        text = contract[field]
        spans = [match.span() for pattern in patterns for match in re.finditer(pattern, text)]
        # A maximal hex token must itself be a prefix; matching part of a
        # different hash or digits inside an identifier is not metadata.
        spans += [match.span() for match in re.finditer(r'(?<![\w/])[0-9a-fA-F]+(?![\w/])', text)
                  if len(match[0]) <= 64 and any(value.startswith(match[0].lower())
                                               for value in fingerprints)]
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
            elif family in ('current-target-erasure', 'target-erasure', 'future-erasure', 'callback-erasure',
                            'windows-security', 'windows-localfree', *RESOURCE_RULES, *EXTERNAL_RULES) and not any(
                    p.get('kind') == 'code' for p in proofs):
                errors.append(f'{field} requires source admission/teardown proof: {site.key}')
    return errors + (validate_repeated_contracts(sites, inventory) if compare_repeated else [])


def repeated_contract_groups(sites, inventory) -> list[list[dict]]:
    """Connected components of raw-equal or symmetrically pair-equal records.

    Pair equivalence need not be transitive. Component membership never expands
    the bindings used to compare an edge; stable site ordering makes diagnostics
    and the exact reviewed-sharing map independent of discovery/segment order.
    """
    by_key = {s.key: s for s in sites}
    records = sorted((r for r in inventory['sites']
                      if r.get('site') in by_key and isinstance(r.get('contract'), dict)
                      and set(r['contract']) == set(FIELDS)
                      and all(isinstance(r['contract'][k], str) and r['contract'][k].strip()
                              for k in FIELDS)), key=lambda r: r['site'])
    parents = list(range(len(records)))

    def root(index):
        while parents[index] != index:
            parents[index] = parents[parents[index]]
            index = parents[index]
        return index

    for a, b in combinations(range(len(records)), 2):
        left, right = records[a], records[b]
        pair = (by_key[left['site']], by_key[right['site']])
        raw_equal = tuple(left['contract'][k] for k in FIELDS) == tuple(
            right['contract'][k] for k in FIELDS)
        if raw_equal or normalized_obligations(
                left['contract'], pair[0], left, comparison_sites=pair) == normalized_obligations(
                right['contract'], pair[1], right, comparison_sites=pair):
            parents[root(b)] = root(a)
    groups = {}
    for index, record in enumerate(records):
        groups.setdefault(root(index), []).append(record)
    return [group for group in groups.values() if len(group) > 1]


def validate_repeated_contracts(sites, inventory) -> list[str]:
    """Compare every pair in the selected union; admit only exact reviewed groups."""
    errors = []
    shared = inventory.get('shared_contracts', {})
    for group in repeated_contract_groups(sites, inventory):
        refs = {r.get('contract_ref') for r in group}
        definition = shared.get(next(iter(refs))) if len(refs) == 1 else None
        rationale = definition.get('review_rationale') if isinstance(definition, dict) else None
        if (not isinstance(rationale, str) or not rationale.strip()
                or any(r['contract'] != definition.get('contract') for r in group)
                or definition.get('bindings') != {
                    r['site']: r['fingerprint'] for r in group}):
            errors.append('repeated owner-level contract lacks local obligations: '
                          + ', '.join(r['site'] for r in group))
    return errors
