"""Inventory runtime namespaces in emitted text and structured Rust path literals.

This is source admission, not a Rust evaluator or a reachability proof. Structured
paths retain their adjacent namespace/item literals; comments cannot join paths.
"""
import re

from rust_policy_sites import code_tokens

DIRECT = re.compile(r"\bsifr_runtime::([A-Za-z_][A-Za-z0-9_]*)")
IDENTIFIER_LITERAL = re.compile(r'"([A-Za-z_][A-Za-z0-9_]*)"\Z')
RAW_IDENTIFIER_LITERAL = re.compile(r'r(#{0,16})"([A-Za-z_][A-Za-z0-9_]*)"\1\Z')


def identifier_literal(token):
    if token.kind != "string":
        return None
    if match := IDENTIFIER_LITERAL.fullmatch(token.value):
        return match[1]
    if match := RAW_IDENTIFIER_LITERAL.fullmatch(token.value):
        return match[2]
    return None


def runtime_roots(source: str) -> set[str]:
    roots = {f"sifr_runtime::{name}" for name in DIRECT.findall(source)}
    # Most files have no structured runtime path. Avoid lexing those files.
    if '"sifr_runtime"' not in source:
        return roots
    tokens = code_tokens(source)
    for index, token in enumerate(tokens):
        if identifier_literal(token) != "sifr_runtime":
            continue
        cursor = index + 1
        # RustExpr::Path owns Strings; slice-based import builders borrow strs.
        if (cursor + 3 < len(tokens) and tokens[cursor].value == "."
                and tokens[cursor + 1].value in {"to_string", "to_owned", "into"}
                and [t.value for t in tokens[cursor + 2:cursor + 4]] == ["(", ")"]):
            cursor += 4
        if cursor + 1 < len(tokens) and tokens[cursor].value == ",":
            item = identifier_literal(tokens[cursor + 1])
            if item is not None:
                roots.add(f"sifr_runtime::{item}")
    return roots


def run_self_test():
    forms = (
        '::sifr_runtime::alpha::run()',
        'RustExpr::Path(vec!["sifr_runtime".to_string(), "alpha".to_string()])',
        'RustExpr::Path(vec!["sifr_runtime".to_owned(), /* nested /* comment */ */ "alpha".into()])',
        '&["", "sifr_runtime", "alpha"]',
        '&[r##"sifr_runtime"##, r#"alpha"#]',
    )
    for source in forms:
        assert runtime_roots(source) == {"sifr_runtime::alpha"}, source
    for source in (
        '// &["sifr_runtime", "fake"]',
        '/* &["sifr_runtime", "fake"] */',
        'r#"a textual example: ["sifr_runtime", "fake"]"#',
        'let namespace = "sifr_runtime"; let other = "fake";',
    ):
        assert not runtime_roots(source), source
    assert runtime_roots('&["sifr_runtime", "new_unowned_root"]') == {
        "sifr_runtime::new_unowned_root"}
