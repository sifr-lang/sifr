use crate::lower_module;
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
