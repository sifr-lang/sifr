"""Review-bound builtin emitter constituents and their actual reference graph.

References are conservative lexical uses, including function-value/alias uses.
Context hashes bind admission, early returns and the complete emitter bodies.
This validates reviewed relationships; it does not prove Rust control flow.
"""
from __future__ import annotations

from rust_policy_sites import Source, fingerprint

CODEGEN = 'crates/sifr_codegen/src/'
from method_policy_nodes import CONSTITUENTS, ADAPTATIONS, ANALYSES, ROUTERS



def relationships(sources: list[Source], constituents: dict | None = None) -> list[dict]:
    """Capture node context plus every lexical caller/reference and its context."""
    sources = [s for s in sources if s.path.startswith(CODEGEN)]
    by_path = {s.path: s for s in sources}
    # Index names once; no Rust control-flow or alias resolution is inferred.
    references = {}
    for caller in sources:
        context_by_token = {}
        for left, _body, right, scope in sorted(caller.contexts):
            digest = fingerprint(caller.ts[left:right + 1])
            for i in range(left, right + 1):
                context_by_token[i] = (scope, digest)
        module_digest = None
        for i, token in enumerate(caller.ts):
            if token.kind != 'ident' or (i and caller.ts[i - 1].value == 'fn'):
                continue
            if i >= 2 and caller.ts[i - 2].value == 'macro_rules':
                continue
            context = context_by_token.get(i)
            if context is None:
                if module_digest is None:
                    module_digest = fingerprint(caller.ts)
                context = ('<module>', module_digest)
            scope, digest = context
            references.setdefault(token.value, {})[(caller.path, scope)] = digest
    result = []
    for (path, scope), relationship in (CONSTITUENTS if constituents is None else constituents).items():
        source = by_path.get(path)
        contexts = [c for c in source.contexts if c[3] == scope] if source else []
        if len(contexts) != 1:
            raise ValueError(f'missing/ambiguous language constituent: {path}::{scope}')
        a, b, c, _name = contexts[0]
        name = scope.removeprefix('macro_rules!::').split('#')[0]
        refs = references.get(name, {})
        header = source.ts[a:b]
        arrow = next((i for i, token in enumerate(header) if token.value == '->'), len(header))
        pair = (path, scope)
        role = ('contextual-rust-adaptation' if pair in ADAPTATIONS else
                'source-shape-analysis' if pair in ANALYSES else
                'source-method-routing' if pair in ROUTERS else 'language-emission-constituent')
        result.append({'path': path, 'scope': scope, 'role': role, 'relationship': relationship,
                       'inputs': ' '.join(t.value for t in header[:arrow]),
                       'outputs': ' '.join(t.value for t in header[arrow + 1:]) or 'implicit unit / macro expansion',
                       'context_fingerprint': fingerprint(source.ts[a:c + 1]),
                       'references': [{'path': p, 'scope': s, 'context_fingerprint': h}
                                      for (p, s), h in sorted(refs.items())]})
    return result


def validate_relationships(sources: list[Source], inventory: dict, constituents: dict | None = None) -> list[str]:
    actual = relationships(sources, constituents)
    expected = inventory.get('constituents')
    if expected != actual:
        return ['builtin constituent admission/call relationships changed; review the exact nodes and references']
    return []
