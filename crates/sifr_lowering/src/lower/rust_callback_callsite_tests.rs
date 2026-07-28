use crate::lower_module;
use crate::HirStmt;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

#[test]
fn rust_threadsafe_callback_rejects_non_send_nested_capture() {
    let source = r"
class SubscriptionError(Error):
    message: str

class LocalState(NonSend):
    value: int

@rust.opaque(type=bridge.events.Subscription, send=True, sync=False, clone=none, close=async_close)
class Subscription:
    @rust(Self.aclose)
    async def aclose(own self) -> Result[None, SubscriptionError | RustPanicError]: ...

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def run(state: LocalState) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def handler(event: str) -> Result[None, SubscriptionError]:
        _ = state.value
        return None
    return subscribe(handler)
";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
            && error.message.contains("handler `handler` capture `state`")
            && error.message.contains("not sendable")
    }));
}

#[test]
fn rust_threadsafe_callback_rejects_inline_handler() {
    let source = r"
class SubscriptionError(Error):
    message: str

class Subscription:
    pass

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def run() -> Result[Subscription, SubscriptionError | RustPanicError]:
    return subscribe(lambda event: None)
";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
            && error.message.contains("handlers must be named functions")
    }));
}

#[test]
fn rust_threadsafe_callback_rejects_unprovable_callable_binding() {
    let source = r"
class SubscriptionError(Error):
    message: str

class Subscription:
    pass

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def run(
    handler: Callable[[str], Result[None, SubscriptionError]],
) -> Result[Subscription, SubscriptionError | RustPanicError]:
    return subscribe(handler)
";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
            && error
                .message
                .contains("captures cannot be proven thread-safe")
    }));
}

#[test]
fn rust_threadsafe_callback_checks_method_hosted_attachments() {
    let source = r"
class SubscriptionError(Error):
    message: str

class LocalState(NonSend):
    value: int

class Subscription:
    pass

class Registrar:
    @rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
    @rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
    def subscribe(
        self,
        own handler: Callable[[str], Result[None, SubscriptionError]],
    ) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def run(
    registrar: Registrar,
    state: LocalState,
) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def handler(event: str) -> Result[None, SubscriptionError]:
        _ = state.value
        return None
    return registrar.subscribe(handler)
";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
            && error.message.contains("handler `handler` capture `state`")
            && error.message.contains("not sendable")
    }));
}

#[test]
fn rust_threadsafe_callback_marks_valid_nested_handler_as_move() {
    let source = r"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def run() -> Result[Subscription, SubscriptionError | RustPanicError]:
    prefix: str = 'event'
    def handler(event: str) -> Result[None, SubscriptionError]:
        _ = prefix
        return None
    return subscribe(handler)
";
    let parsed = parse_module(source).expect("source should parse");
    let result = lower_module(parsed.suite()).expect("valid retained capture should lower");
    let run = result
        .module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");
    let move_plan = run.body.iter().find_map(|stmt| match stmt {
        HirStmt::NestedFunction {
            func,
            move_captures,
            capture_clones,
        } if func.name == "handler" => Some((*move_captures, capture_clones.clone())),
        _ => None,
    });

    assert_eq!(move_plan, Some((true, vec!["prefix".to_string()])));
}

#[test]
fn rust_threadsafe_callback_rejects_indirect_non_send_capture() {
    let source = r"
class SubscriptionError(Error):
    message: str

class LocalState(NonSend):
    value: int

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def run(state: LocalState) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def inner(event: str) -> Result[None, SubscriptionError]:
        _ = state.value
        return None
    def handler(event: str) -> Result[None, SubscriptionError]:
        return inner(event)
    return subscribe(handler)
";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("indirect non-send capture should fail lowering"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
            && error
                .message
                .contains("handler `handler` capture `inner.state`")
            && error.message.contains("not sendable")
    }));
}

#[test]
fn rust_threadsafe_callback_rejects_captured_callable_parameter() {
    let source = r"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def attach(
    own hook: Callable[[str], Result[None, SubscriptionError]],
) -> Result[Subscription, SubscriptionError | RustPanicError]:
    def handler(event: str) -> Result[None, SubscriptionError]:
        return hook(event)
    return subscribe(handler)
";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("captured callable parameter should fail lowering"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
            && error.message.contains("handler `handler` capture `hook`")
            && error
                .message
                .contains("captures cannot be proven thread-safe")
    }));
}

#[test]
fn rust_threadsafe_callback_consumes_nested_handler_binding() {
    let source = r#"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def attach_twice() -> Result[Subscription, SubscriptionError | RustPanicError]:
    seed: str = "twice"
    def twice_handler(event: str) -> Result[None, SubscriptionError]:
        assert seed != ""
        return None
    first: Result[Subscription, SubscriptionError | RustPanicError] = subscribe(twice_handler)
    return subscribe(twice_handler)

def call_after_attachment() -> Result[Subscription, SubscriptionError | RustPanicError]:
    seed: str = "call"
    def call_handler(event: str) -> Result[None, SubscriptionError]:
        assert seed != ""
        return None
    attached: Result[Subscription, SubscriptionError | RustPanicError] = subscribe(call_handler)
    _called: Result[None, SubscriptionError] = call_handler("after")
    return attached

def attach_outer_handler_in_loop() -> Result[None, SubscriptionError]:
    seed: str = "loop"
    def loop_handler(event: str) -> Result[None, SubscriptionError]:
        assert seed != ""
        return None
    labels: list[str] = ["first", "second"]
    for label in labels:
        _attached: Result[Subscription, SubscriptionError | RustPanicError] = subscribe(loop_handler)
    return None

def attach_conditionally_then_reuse(
    should_attach: bool,
) -> Result[Subscription, SubscriptionError | RustPanicError]:
    seed: str = "conditional"
    def conditional_handler(event: str) -> Result[None, SubscriptionError]:
        assert seed != ""
        return None
    if should_attach:
        first: Result[Subscription, SubscriptionError | RustPanicError] = subscribe(conditional_handler)
    return subscribe(conditional_handler)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("retained handler reuse should fail lowering"),
        Err(errors) => errors,
    };

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                && error.message.contains("twice_handler")
        }),
        "{errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                && error.message.contains("call_handler")
        }),
        "{errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)
                && error.message.contains("loop_handler")
        }),
        "{errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                && error.message.contains("conditional_handler")
        }),
        "{errors:#?}"
    );
}

#[test]
fn rust_threadsafe_callback_resolves_attribute_and_method_capture_types() {
    let source = r#"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

class Registry:
    label: str

    def upper_label(self) -> str:
        return self.label.upper()

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def attach(registry: Registry) -> Result[Subscription, SubscriptionError | RustPanicError]:
    attribute_label: str = registry.label
    method_label: str = registry.upper_label()
    def handler(event: str) -> Result[None, SubscriptionError]:
        assert attribute_label != ""
        assert method_label != ""
        return None
    return subscribe(handler)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let result = lower_module(parsed.suite())
        .expect("declared attribute and method result captures should lower");
    let attach = result
        .module
        .functions
        .iter()
        .find(|function| function.name == "attach")
        .expect("attach function");
    let capture_clones = attach.body.iter().find_map(|stmt| match stmt {
        HirStmt::NestedFunction {
            func,
            capture_clones,
            ..
        } if func.name == "handler" => Some(capture_clones.clone()),
        _ => None,
    });

    assert_eq!(
        capture_clones,
        Some(vec![
            "attribute_label".to_string(),
            "method_label".to_string()
        ])
    );
}

#[test]
fn rust_threadsafe_callback_rejects_direct_and_transitive_mutating_captures() {
    let source = r#"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def attach_direct() -> Result[Subscription, SubscriptionError | RustPanicError]:
    direct_counter: int = 0
    def direct_handler(event: str) -> Result[None, SubscriptionError]:
        nonlocal direct_counter
        direct_counter = direct_counter + 1
        return None
    return subscribe(direct_handler)

def attach_transitive() -> Result[Subscription, SubscriptionError | RustPanicError]:
    transitive_counter: int = 0
    def bump(event: str) -> Result[None, SubscriptionError]:
        nonlocal transitive_counter
        transitive_counter = transitive_counter + 1
        return None
    def transitive_handler(event: str) -> Result[None, SubscriptionError]:
        return bump(event)
    return subscribe(transitive_handler)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("mutating retained captures should fail lowering"),
        Err(errors) => errors,
    };

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
                && error
                    .message
                    .contains("handler `direct_handler` capture `direct_counter`")
                && error.message.contains("requires `FnMut`")
        }),
        "{errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RUST_CALLBACK_CONTRACT)
                && error
                    .message
                    .contains("handler `transitive_handler` capture `bump.transitive_counter`")
                && error.message.contains("requires `FnMut`")
        }),
        "{errors:#?}"
    );
}
