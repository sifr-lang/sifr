"""Review-bound builtin emitter constituents and their actual reference graph.

References are conservative lexical uses, including function-value/alias uses.
Context hashes bind admission, early returns and the complete emitter bodies.
This validates reviewed relationships; it does not prove Rust control flow.
"""
from __future__ import annotations

from rust_policy_sites import Source, fingerprint

CODEGEN = 'crates/sifr_codegen/src/'
# H02h1 supplies the reviewed graph explicitly. Foundation has no live nodes.
CONSTITUENTS = {}



def relationships(sources: list[Source], constituents: dict | None = None) -> list[dict]:
    """Capture node context plus every lexical caller/reference and its context."""
    sources = [s for s in sources if s.path.startswith(CODEGEN)]
    by_path = {s.path: s for s in sources}
    result = []
    for (path, scope), relationship in (CONSTITUENTS if constituents is None else constituents).items():
        source = by_path.get(path)
        contexts = [c for c in source.contexts if c[3] == scope] if source else []
        if len(contexts) != 1:
            raise ValueError(f'missing/ambiguous language constituent: {path}::{scope}')
        a, _b, c, _name = contexts[0]
        name = scope.removeprefix('macro_rules!::')
        refs = {}
        for caller in sources:
            for i, token in enumerate(caller.ts):
                if token.kind != 'ident' or token.value != name:
                    continue
                if caller.path == path and a <= i <= c and i == a + (2 if scope.startswith('macro_rules!') else 1):
                    continue
                caller_scope = caller.scope(i)
                # An own declaration qualifier can precede fn; exclude the
                # definition by token shape, never by the caller's name alone.
                if i and caller.ts[i - 1].value == 'fn':
                    continue
                ctx = [x for x in caller.contexts if x[0] <= i <= x[2]]
                if ctx:
                    left, _body, right, _scope = max(ctx, key=lambda x: x[0])
                    digest = fingerprint(caller.ts[left:right + 1])
                else:
                    digest = fingerprint(caller.ts)  # aliases/module references
                refs[(caller.path, caller_scope)] = digest
        result.append({'path': path, 'scope': scope, 'relationship': relationship,
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
