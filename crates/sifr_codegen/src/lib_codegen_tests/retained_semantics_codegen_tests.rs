use super::generate_rust_from_source;

#[test]
fn bounded_string_receiver_and_arguments_are_evaluated_once() {
    let generated = generate_rust_from_source(
        r#"
class Source:
    value: str

    def text(self) -> str:
        return self.value

def transform(source: Source, __sifr_string_receiver: int) -> tuple[list[str], str]:
    return (source.text().split(",", __sifr_string_receiver), source.text().replace("x", "y", __sifr_string_receiver))
"#,
    );
    assert_eq!(generated.matches("source.text()").count(), 2, "{generated}");
    assert!(generated.contains("__sifr_string_argument_"), "{generated}");
}

#[test]
fn iterator_lowering_uses_result_type_not_lookalike_method_name() {
    let generated = generate_rust_from_source(
        r#"
class Values:
    values: list[int]

    def map(self) -> list[int]:
        return self.values

def total(source: Values) -> int:
    result: int = 0
    for value in source.map():
        result = result + value
    return result
"#,
    );

    assert!(
        generated.contains("source.map().into_iter()"),
        "{generated}"
    );
}

#[test]
fn nested_boolean_ordering_keeps_checked_read_guard() {
    let generated = generate_rust_from_source(
        r#"
def positive(enabled: bool, values: list[int], index: int) -> bool:
    return enabled and (values[index] > 0 or not enabled)
"#,
    );

    assert!(generated.contains(".is_some_and("), "{generated}");
    assert!(!generated.contains("values["), "{generated}");
}

#[test]
fn anext_marks_the_advanced_iterator_binding_mutable() {
    let generated = generate_rust_from_source(
        r#"
async def values() -> AsyncGenerator[int, GeneratorCloseError]:
    yield 1

async def first() -> Result[Option[int], GeneratorCloseError]:
    stream = values()
    return await anext(stream)
"#,
    );

    assert!(generated.contains("let mut stream:"), "{generated}");
}

#[test]
fn string_list_membership_compares_string_views() {
    let generated = generate_rust_from_source(
        r#"
def contains_word(needle: str, words: list[str]) -> bool:
    return needle in words
"#,
    );
    assert!(generated.contains("AsRef::<str>::as_ref"), "{generated}");
    assert!(generated.contains(".iter().any("), "{generated}");
    assert!(!generated.contains(".contains(&needle"), "{generated}");
}
