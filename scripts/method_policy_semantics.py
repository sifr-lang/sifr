"""Semantic roles and operation bindings for the reviewed literal-site inventory.

Literal discovery is deliberately broader than source methods. The explicit
node graph owns source-method specializations, adaptations and shape analyses.
Other records describe the actual lexical operation and input/output signature;
that evidence is review material, not inferred admission or safety proof.
"""
from __future__ import annotations

from rust_policy_sites import Source, Site, code_tokens
from method_policy_nodes import ADAPTATIONS, ANALYSES, ROUTERS, nodes

IR_PATHS = ('/generated_rust_canonicalizer/', '/ir_optimize/')


def context(source: Source, site: Site):
    candidates = [c for c in source.contexts if c[3] == site.scope]
    return candidates[0] if candidates else None


def binding(source: Source, site: Site) -> dict:
    enclosing = context(source, site)
    if enclosing:
        a, body, _end, _scope = enclosing
        header = source.ts[a:body]
    else:
        header = []
    arrow = next((i for i, token in enumerate(header) if token.value == '->'), len(header))
    return dict(operation=' '.join(t.value for t in code_tokens(site.text)),
                inputs=' '.join(t.value for t in header[:arrow]) or 'module scope',
                outputs=' '.join(t.value for t in header[arrow + 1:]) or 'implicit unit / macro expansion')


def is_test(source: Source, site: Site) -> bool:
    # Functions inside a cfg(test) module are covered by that module boundary;
    # test-named production functions do not acquire an exemption from their name.
    ts = source.ts
    for i, token in enumerate(ts):
        if token.value != 'mod' or i + 2 >= len(ts) or ts[i + 2].value != '{':
            continue
        right = source.pairs[i + 2]
        if ts[i].start <= site.start <= ts[right].end and source.test_only(i):
            return True
    return ('/tests/' in site.path or site.path.endswith(('_tests.rs', '/tests.rs'))
            or '/method_authority_tests/' in site.path or site.test_only)


def is_rust_consumption(source: Source, site: Site) -> bool:
    if any(part in site.path for part in IR_PATHS) or '/generated_rust_canonicalizer.rs' in site.path:
        return True
    # A RustExpr carrier is already generated representation, including carriers
    # embedded in an emitter that also consumes HIR elsewhere.
    operation = binding(source, site)['operation']
    if site.kind == 'method-call-carrier':
        return 'RustExpr :: MethodCall' in operation
    enclosing = context(source, site)
    if enclosing:
        a, _b, c, _scope = enclosing
        body = ' '.join(t.value for t in source.ts[a:c + 1])
        return 'RustExpr' in body and 'HirExpr' not in body
    return False


def requires_source_node(source: Source, site: Site) -> bool:
    """Conservative literal/typed-carrier trigger; no selector spelling assumption.

    Literal decisions in a codegen function carrying a HIR method, or a string
    dispatch function taking HIR operands, must have a reviewed node.
    No classification label can hide such a newly introduced specialization.
    """
    if not site.path.startswith('crates/sifr_codegen/src/') or is_test(source, site):
        return False
    if is_rust_consumption(source, site):
        return False
    enclosing = context(source, site)
    if enclosing is None:
        return False
    a, b, c, _scope = enclosing
    body = source.ts[a:c + 1]
    header = source.ts[a:b]
    values = [t.value for t in body]
    has_source_method = any(values[i:i + 3] == ['HirExpr', '::', 'MethodCall']
                            for i in range(len(values) - 2))
    has_dispatch = any(t.kind == 'string' for t in code_tokens(site.text))
    header_values = [t.value for t in header]
    string_method_input = any(header_values[i:i + 4] == ['method', ':', '&', 'str']
                              for i in range(len(header_values) - 3))
    return (has_source_method and has_dispatch
            or string_method_input and 'HirExpr' in header_values and has_dispatch)


def classification(source: Source, site: Site, methods) -> str:
    pair = (site.path, site.scope)
    if is_test(source, site):
        return 'test-assertion'
    if pair == (methods.CODEGEN_ADMISSION_PATH, 'source_method_path'):
        return 'source-method-admission'
    if pair == methods.LANGUAGE_AUTHORITY:
        return 'language-emission'
    if pair == methods.CONST_AUTHORITY:
        return 'compile-time-semantics'
    if pair in ADAPTATIONS:
        return 'contextual-rust-adaptation'
    if pair in ANALYSES:
        return 'source-shape-analysis'
    if pair in ROUTERS:
        return 'source-method-routing'
    if pair in methods.CONSTITUENTS:
        return 'language-emission-constituent'
    if is_rust_consumption(source, site):
        return 'rust-ir-consumption'
    if site.path.startswith('crates/sifr_lowering/'):
        return 'typed-lowering'
    # Literal-based configuration, type relations, operators, crate protocols,
    # renderer choices etc. are compiler decisions, not source method dispatch.
    return 'compiler-policy'


def detail(source: Source, site: Site, kind: str) -> dict:
    bound = binding(source, site)
    relationship = nodes().get((site.path, site.scope))
    if relationship is None:
        relationship = (f'{site.path}::{site.scope} enclosing context owns this {site.kind}; '
                        'source method admission is not inferred from literal matching')
    roles = {
        'rust-ir-consumption': 'inspect or rewrite generated Rust syntax/IR',
        'source-shape-analysis': 'inspect source shape and produce analysis/binding facts',
        'contextual-rust-adaptation': 'argument/result representation predicate or adaptation',
        'language-emission-constituent': 'implement the canonical builtin method registry',
        'language-emission': 'canonical builtin method semantics',
        'source-method-admission': 'validate typed method declaration authority',
        'compile-time-semantics': 'evaluate constant method semantics',
        'typed-lowering': 'lower/check typed source or declaration facts',
        'test-assertion': 'test fixture construction, execution or assertion',
        'compiler-policy': 'select compiler policy by the recorded lexical operation',
        'source-method-routing': 'forward source methods or consume lowered expression representation',
    }
    role = roles.get(kind, kind)
    return dict(bound, role=role, caller_admission_relationship=relationship,
                contract=f'{role}: {relationship}')


def validate_semantics(sources: list[Source], sites: list[Site], inventory: dict, methods) -> list[str]:
    by_path = {s.path: s for s in sources}
    by_key = {s.key: s for s in sites}
    errors = []
    for record in inventory['sites']:
        site = by_key.get(record.get('site'))
        source = by_path.get(site.path) if site else None
        if source is None:
            continue
        pair = (site.path, site.scope)
        if requires_source_node(source, site) and pair not in nodes() and pair != (
                methods.CODEGEN_ADMISSION_PATH, 'source_method_path'):
            errors.append(f'unregistered source-method specialization/consumer: {site.key}')
        expected = classification(source, site, methods)
        if record.get('classification') != expected:
            errors.append(f'actual semantic role changed ({expected}): {site.key}')
        expected_detail = detail(source, site, expected)
        for field, value in expected_detail.items():
            if record.get(field) != value:
                errors.append(f'actual {field} binding changed: {site.key}')
    return errors
