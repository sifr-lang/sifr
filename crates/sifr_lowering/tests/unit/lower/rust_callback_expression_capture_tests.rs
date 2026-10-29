use crate::lower_module;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn callback_contract_errors(source: &str) -> Vec<crate::HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("unsafe retained callback expressions should fail lowering"),
        Err(errors) => errors,
    }
}

#[test]
fn rust_threadsafe_callback_finds_captures_in_nested_expression_forms() {
    let source = r#"
class SubscriptionError(Error):
    message: str

class LocalState(NonSend):
    value: int
    values: list[int]

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def apply_int(producer: Callable[[], int]) -> int:
    return producer()

def attach_fstring(state: LocalState) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def fstring_handler(event: str) -> Result[None, SubscriptionError]:
        _message: str = f"value={state.value}"
        return None
    return subscribe(fstring_handler)

def attach_lambda(state: LocalState) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def lambda_handler(event: str) -> Result[None, SubscriptionError]:
        _value: int = apply_int(lambda: state.value)
        return None
    return subscribe(lambda_handler)

def attach_slice(state: LocalState) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def slice_handler(event: str) -> Result[None, SubscriptionError]:
        _values: list[int] = [4, 5, 6][state.value:2]
        return None
    return subscribe(slice_handler)

def attach_starred(state: LocalState) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def starred_handler(event: str) -> Result[None, SubscriptionError]:
        _values = [*state.values]
        return None
    return subscribe(starred_handler)

def attach_callable(
    own hook: Callable[[], int],
) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def callable_handler(event: str) -> Result[None, SubscriptionError]:
        _value: int = apply_int(lambda: hook())
        return None
    return subscribe(callable_handler)
"#;
    let errors = callback_contract_errors(source);

    for capture in [
        "handler `fstring_handler` capture `state`",
        "handler `lambda_handler` capture `state`",
        "handler `slice_handler` capture `state`",
        "handler `starred_handler` capture `state`",
    ] {
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
                    && error.message.contains(capture)
                    && error.message.contains("not sendable")
            }),
            "missing nested-expression NonSend rejection for {capture}: {errors:#?}"
        );
    }
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
                && error
                    .message
                    .contains("handler `callable_handler` capture `hook`")
                && error
                    .message
                    .contains("captures cannot be proven thread-safe")
        }),
        "missing lambda-hidden callable rejection: {errors:#?}"
    );
}

#[test]
fn rust_threadsafe_callback_finds_mutation_in_comprehensions_fstrings_and_slices() {
    let source = r#"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def attach_comprehension() -> Result[Subscription, SubscriptionError | RustPanicError]:
    seen: list[int] = []
    def comprehension_handler(event: str) -> Result[None, SubscriptionError]:
        _values = [seen.append(value) for value in [1]]
        return None
    return subscribe(comprehension_handler)

def attach_fstring() -> Result[Subscription, SubscriptionError | RustPanicError]:
    seen: list[int] = [1]
    def fstring_handler(event: str) -> Result[None, SubscriptionError]:
        _message: str = f"value={seen.pop()}"
        return None
    return subscribe(fstring_handler)

def attach_slice() -> Result[Subscription, SubscriptionError | RustPanicError]:
    seen: list[int] = [1]
    def slice_handler(event: str) -> Result[None, SubscriptionError]:
        _values: list[int] = [1, 2][seen.pop():]
        return None
    return subscribe(slice_handler)
"#;
    let errors = callback_contract_errors(source);

    for capture in [
        "handler `comprehension_handler` capture `seen`",
        "handler `fstring_handler` capture `seen`",
        "handler `slice_handler` capture `seen`",
    ] {
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
                    && error.message.contains(capture)
                    && error.message.contains("requires `FnMut`")
            }),
            "missing nested-expression mutation rejection for {capture}: {errors:#?}"
        );
    }
}

#[test]
fn rust_threadsafe_callback_clones_fstring_capture() {
    let source = r#"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def attach() -> Result[Subscription, SubscriptionError | RustPanicError]:
    label: str = "snapshot"
    def handler(event: str) -> Result[None, SubscriptionError]:
        _message: str = f"{label}:{event}"
        return None
    result: Result[Subscription, SubscriptionError | RustPanicError] = subscribe(handler)
    assert label == "snapshot"
    return result
"#;
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite()).expect("f-string captures should use the verified clone plan");
}
