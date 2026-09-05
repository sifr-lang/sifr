use super::generate_rust_from_source;

#[test]
fn item12_top_level_unused_projection_retains_only_effectful_receiver() {
    let generated = generate_rust_from_source(
        r#"
class Payload:
    value: str

    def __init__(self, value: str):
        self.value = value

class Source:
    calls: int

    def __init__(self):
        self.calls = 0

    def read(mut self) -> Payload:
        self.calls = self.calls + 1
        return Payload("value")

def use(mut source: Source) -> None:
    _unused_call_projection: str = source.read().value
    local = Payload("local")
    _unused_name_projection: str = local.value
"#,
    );
    assert_eq!(generated.matches("source.read()").count(), 1, "{generated}");
    assert!(
        !generated.contains("let _unused_call_projection"),
        "{generated}"
    );
    assert!(
        !generated.contains("let _unused_name_projection"),
        "{generated}"
    );
}

#[test]
fn item12_handler_bindings_follow_cached_body_and_capture_uses() {
    let generated = generate_rust_from_source(
        r#"
def fail() -> Result[int, ValueError]:
    raise ValueError("missing")

def unused() -> None:
    try:
        value: int = fail()
        print(value)
    except ValueError as unused_error:
        print("handled")

def unused_projection() -> None:
    try:
        value: int = fail()
        print(value)
    except ValueError as projection_error:
        _message: str = projection_error.message
        print("handled")

def retained_projection() -> None:
    try:
        value: int = fail()
        print(value)
    except ValueError as retained_error:
        _retained: str = retained_error.message
        def retained() -> str:
            return _retained
        print(retained())

def captured() -> None:
    try:
        value: int = fail()
        print(value)
    except ValueError as captured_error:
        def message() -> str:
            return captured_error.message
        print(message())
"#,
    );
    assert!(!generated.contains("let unused_error ="), "{generated}");
    assert!(!generated.contains("let projection_error ="), "{generated}");
    assert!(generated.contains("let _retained"), "{generated}");
    assert!(generated.contains("let captured_error ="), "{generated}");
}

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
