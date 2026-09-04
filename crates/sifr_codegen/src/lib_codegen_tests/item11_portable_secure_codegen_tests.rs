use super::generate_rust_from_source;

#[test]
fn item11_user_integer_widths_never_reach_panic_on_proof_usize_conversions() {
    let generated = generate_rust_from_source(
        r#"
def exercise(mut values: list[int], index: int, text: str, count: int) -> tuple[list[int], str, list[str], str]:
    values.insert(index, 7)
    repeated = text * count
    parts = text.split(",", count)
    replaced = text.replace("x", "y", count)
    return (values, repeated, parts, replaced)

def reuse_owned_string() -> str:
    source = "owned"
    repeated = source * 2
    return source + repeated
"#,
    );

    assert!(
        generated.contains("index.clamp_slice_bound(values.len())"),
        "{generated}"
    );
    assert!(
        generated.contains("count.clamp_slice_bound("),
        "{generated}"
    );
    assert!(
        generated.contains("while &__sifr_repeat_i < &__n"),
        "{generated}"
    );
    assert!(
        generated.contains("let __sifr_repeat_src: &str = &"),
        "{generated}"
    );
    assert!(generated.contains("+ SifrInt::from_i64(1)"), "{generated}");
    assert!(!generated.contains("to_usize_proven"), "{generated}");
    assert!(!generated.contains(".repeat("), "{generated}");
}
