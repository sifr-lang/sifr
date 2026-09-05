use super::generate_rust_from_source;

#[test]
fn mixed_character_string_comparisons_preserve_all_three_presence_states() {
    let generated = generate_rust_from_source(
        r#"
def variable_right(text: str, index: int, expected: str) -> bool:
    return text[index] == expected

def variable_left(text: str, index: int, expected: str) -> bool:
    return expected != text[index]

def literal_right(text: str, index: int) -> bool:
    return text[index] == "🦀"

def literal_left(text: str, index: int) -> bool:
    return "" != text[index]

def optional_right(text: str, index: int, values: list[str], value_index: int) -> bool:
    return text[index] == values[value_index]

def optional_left(text: str, index: int, values: list[str], value_index: int) -> bool:
    return values[value_index] != text[index]
"#,
    );

    assert_eq!(generated.matches(".map(Some)").count(), 6, "{generated}");
    assert_eq!(
        generated.matches("let mut __sifr_cmp_chars").count(),
        4,
        "{generated}"
    );
    assert_eq!(
        generated
            .matches("__sifr_cmp_chars.next().is_some()")
            .count(),
        4,
        "{generated}"
    );
    assert!(
        generated.contains("Some(Some('\\u{1f980}'))"),
        "{generated}"
    );
    assert!(generated.contains("Some(None)"), "{generated}");
    assert!(
        !generated.contains(".and_then(|__sifr_cmp_s"),
        "{generated}"
    );
    assert!(
        !generated.contains("__sifr_cmp_s.to_string()"),
        "{generated}"
    );
    assert!(
        !generated.contains("__sifr_cmp_s.to_owned()"),
        "{generated}"
    );
    assert!(
        !generated.contains("__sifr_cmp_s.chars().collect"),
        "{generated}"
    );
}
