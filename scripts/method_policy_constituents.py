"""Review-bound builtin emitter constituents and their actual reference graph.

References are conservative lexical uses, including function-value/alias uses.
Context hashes bind admission, early returns and the complete emitter bodies.
This validates reviewed relationships; it does not prove Rust control flow.
"""
from __future__ import annotations

from rust_policy_sites import Source, fingerprint

CODEGEN = 'crates/sifr_codegen/src/'
CONSTITUENTS = {
    ('stmt_support_emitter/stmt_expr_method_and_question_mark.rs', 'macro_rules!::stmt_expr_method_call'): 'source_method_path -> is_builtin; builtin specializations end in registry or builtin_method_decline; nonbuiltins branch separately',
    ('intrinsic_method_emitters/recursive_exprs.rs', 'try_lower_registry_expr_recursive'): 'source_method_path -> nonbuiltin early return; builtin specializations end in methods registry or decline',
    ('lower_expr/iterators_and_callables.rs', 'try_lower_simple_method_call_expr'): 'source_method_path -> is_builtin before task/iterator specialization',
    ('intrinsic_method_emitters/collection_methods.rs', 'try_lower_registry_method_call_expr'): 'admitted stmt_expr_method_call -> operands and checked/defaultdict or unchecked registry',
    ('intrinsic_method_emitters/collection_methods.rs', 'try_lower_registry_discarded_method_call_expr'): 'admitted stmt_expr_method_call -> setdefault discard policy or ordinary registry',
    ('intrinsic_method_emitters/collection_methods.rs', 'lower_registry_method_operands'): 'admitted ordinary/discarded registry -> checked receiver and arguments',
    ('intrinsic_method_emitters/collection_methods.rs', 'try_lower_registry_method_call_expr_unchecked'): 'admitted ordinary/discarded registry -> storage specializations or methods::lower_method_with_context',
    ('intrinsic_method_emitters/collection_methods.rs', 'try_lower_registry_set_method_call_expr'): 'admitted unchecked registry -> set iterable specialization',
    ('intrinsic_method_emitters/collection_defaultdict_methods.rs', 'try_lower_defaultdict_index_method_call_expr'): 'admitted ordinary registry -> checked defaultdict bucket -> methods registry or set helper',
    ('intrinsic_method_emitters/defaultdict_iterable_mutations.rs', 'try_lower_defaultdict_set_update_expr'): 'admitted defaultdict bucket -> set iterable mutation',
    ('intrinsic_method_emitters/recursive_method_calls.rs', 'try_lower_recursive_indexed_list_append'): 'admitted recursive builtin branch -> checked indexed append',
    ('string_char_cache.rs', 'try_lower_dict_indexed_list_append_expr'): 'admitted stmt/recursive builtin branch -> checked indexed append',
    ('string_char_cache.rs', 'try_lower_dict_indexed_list_pop_expr'): 'admitted stmt/recursive builtin branch -> checked indexed pop',
    ('string_char_cache.rs', 'try_lower_dict_indexed_list_len_expr'): 'admitted stmt/recursive builtin branch -> checked indexed len',
    ('python_raw_api_codegen.rs', 'try_lower_python_raw_object_method'): 'admitted stmt builtin branch -> canonical Python Object runtime substrate',
    ('python_buffer_codegen.rs', 'lower_python_buffer_method'): 'admitted recursive builtin branch -> canonical Buffer ABI runtime substrate',
    ('python_arrow_codegen.rs', 'lower_python_arrow_method'): 'admitted recursive builtin branch -> canonical Arrow ABI runtime substrate',
    ('python_dlpack_codegen.rs', 'lower_python_dlpack_method'): 'admitted recursive builtin branch -> canonical DLPack ABI runtime substrate',
}
CONSTITUENTS = {(CODEGEN + path, scope): relationship
                for (path, scope), relationship in CONSTITUENTS.items()}


def relationships(sources: list[Source]) -> list[dict]:
    """Capture node context plus every lexical caller/reference and its context."""
    sources = [s for s in sources if s.path.startswith(CODEGEN)]
    by_path = {s.path: s for s in sources}
    result = []
    for (path, scope), relationship in CONSTITUENTS.items():
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


def validate_relationships(sources: list[Source], inventory: dict) -> list[str]:
    actual = relationships(sources)
    expected = inventory.get('constituents')
    if expected != actual:
        return ['builtin constituent admission/call relationships changed; review the exact nodes and references']
    return []
