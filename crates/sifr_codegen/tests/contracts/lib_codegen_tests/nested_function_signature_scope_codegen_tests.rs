use super::generate_rust_from_source;

#[test]
fn shadowed_nested_helper_restores_outer_calling_conventions() {
    let generated = generate_rust_from_source(
        r#"def evaluate(flag: bool) -> int:
    def helper(value: int = 40, *extra: int) -> int:
        return value + len(extra)

    if flag:
        def helper(prefix: str, suffix: str = "!") -> str:
            return prefix + suffix
        assert helper("ok") == "ok!"

    return helper() + helper(value=1) + helper(1, 2, 3)
"#,
    );

    assert!(generated.contains("helper(SifrInt::from_i64(40), &"));
    assert!(generated.contains(
        "helper(SifrInt::from_i64(1), &vec![SifrInt::from_i64(2), SifrInt::from_i64(3)])"
    ));
    assert!(!generated.contains("helper(&(SifrInt::from_i64(40))"));
    syn::parse_file(&generated).expect("scoped nested-function calls should parse as Rust");
}

#[test]
fn loop_scopes_restore_outer_calling_conventions() {
    let generated = generate_rust_from_source(
        r#"def outer(n: int) -> int:
    def helper(value: int = 3, *rest: int) -> int:
        return value + len(rest)

    for i in range(n):
        def helper(value: str) -> str:
            return value
        assert helper("x") == "x"

    return helper() + helper(1, 2, 3)
"#,
    );

    assert!(generated.contains("helper(SifrInt::from_i64(3), &"));
    assert!(generated.contains(
        "helper(SifrInt::from_i64(1), &vec![SifrInt::from_i64(2), SifrInt::from_i64(3)])"
    ));
    syn::parse_file(&generated).expect("loop-scoped nested-function calls should parse as Rust");
}

#[test]
fn exiting_branch_scopes_restore_outer_calling_conventions() {
    let generated = generate_rust_from_source(
        r#"def outer(flag: bool) -> int:
    def helper(value: int = 3, *rest: int) -> int:
        return value + len(rest)

    if flag:
        def helper(value: str) -> str:
            return value
        return len(helper("x"))

    return helper() + helper(1, 2, 3)
"#,
    );

    assert!(generated.contains("helper(SifrInt::from_i64(3), &"));
    assert!(generated.contains(
        "helper(SifrInt::from_i64(1), &vec![SifrInt::from_i64(2), SifrInt::from_i64(3)])"
    ));
    syn::parse_file(&generated).expect("exiting-branch nested-function calls should parse as Rust");
}
