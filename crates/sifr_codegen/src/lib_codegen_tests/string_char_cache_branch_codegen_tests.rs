use super::generate_rust_from_source;

#[test]
fn branch_local_string_cache_does_not_suppress_later_declaration() {
    let generated = generate_rust_from_source(
        r#"
def count_text(enabled: bool, flag: bool) -> int:
    selected: int = 0
    if enabled:
        if flag:
            text: str = "left"
            selected = len(text)
        text: str = "right"
        selected += len(text)
    return selected
"#,
    );

    assert_eq!(
        generated
            .matches("let __sifr_chars_text: Vec<char> = text.chars().collect")
            .count(),
        2,
        "branch and outer declarations need separate caches:\n{generated}"
    );
}

#[test]
fn walrus_if_branch_string_cache_does_not_suppress_later_declaration() {
    let generated = generate_rust_from_source(
        r#"
def count_text(enabled: bool) -> int:
    selected: int = 0
    if enabled:
        if (value := 1) > 0:
            text: str = "left"
            selected = len(text)
        text: str = "right"
        selected += len(text)
    return selected
"#,
    );

    assert_eq!(
        generated
            .matches("let __sifr_chars_text: Vec<char> = text.chars().collect")
            .count(),
        2,
        "walrus branch and outer declarations need separate caches:\n{generated}"
    );
}
