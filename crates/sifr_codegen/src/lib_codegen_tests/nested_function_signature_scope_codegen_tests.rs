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

    assert!(generated.contains("helper(40_i64, &"));
    assert!(generated.contains("helper(1_i64, &vec![2_i64, 3_i64])"));
    assert!(!generated.contains("helper(&(40_i64)"));
    syn::parse_file(&generated).expect("scoped nested-function calls should parse as Rust");
}
