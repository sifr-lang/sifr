use super::generate_rust_from_source;

#[test]
fn item12_nontrivial_string_receiver_is_evaluated_once_for_bounded_operations() {
    let generated = generate_rust_from_source(
        r#"
class Source:
    value: str

    def text(self) -> str:
        return self.value

def transform(source: Source, count: int) -> tuple[list[str], str]:
    return (source.text().split(",", count), source.text().replace("x", "y", count))
"#,
    );

    assert_eq!(generated.matches("source.text()").count(), 2, "{generated}");
    assert_eq!(
        generated
            .matches("let __sifr_string_receiver = &source.text()")
            .count(),
        2,
        "{generated}"
    );
}

#[test]
fn item12_iterator_lowering_uses_result_type_not_lookalike_method_name() {
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
fn item12_nested_boolean_ordering_keeps_checked_read_guard() {
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
fn item12_anext_marks_the_advanced_iterator_binding_mutable() {
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
