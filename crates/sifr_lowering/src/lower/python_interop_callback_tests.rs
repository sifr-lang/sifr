use crate::{lower_module, HirDiagnostic};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

#[test]
fn callback_call_policy_is_available_before_body_lowering() {
    let parsed = parse_module(
        "@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=serial)\n@python(pkg.compute)\ndef compute(handler: Callable[[int], int]) -> int: ...\n",
    )
    .expect("source should parse");
    let sifr_python_ast::Stmt::FunctionDef(function) = &parsed.suite()[0] else {
        panic!("expected function");
    };
    let policies = crate::lower::python_interop::callback_call_policies(
        &function.decorator_list,
        &function.parameters,
        false,
    );
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0].parameter_index, 0);
    assert_eq!(
        policies[0].dispatch,
        sifr_ir::PythonCallbackDispatch::Foreign
    );
}

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

fn lower_success(source: &str) {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite()).expect("source should lower");
}

fn assert_callback_error(source: &str, message: &str) {
    let errors = lower_errors(source);
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCB_INVALID_DECLARATION)
                && error.message.contains(message)
        }),
        "expected callback diagnostic containing {message:?}, got {errors:?}"
    );
    assert!(!errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION)));
}

#[test]
fn valid_callback_policy_is_active() {
    lower_success(
        r"
class PythonError(Error):
    message: str

class HandlerError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=current)
@python(pkg.compute)
def compute(
    value: int,
    handler: Callable[[int], Result[int, HandlerError]],
) -> Result[int, PythonError | HandlerError]: ...
",
    );
}

#[test]
fn callback_policy_validates_before_reservation() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

class HandlerError(Error):
    message: str

@python(pkg.compute)
@python.callback(handler, lifetime=call, dispatch=foreign)
def compute(
    handler: Callable[[int], Result[int, HandlerError]],
) -> Result[int, PythonError | HandlerError]: ...
",
    );
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCB_INVALID_DECLARATION)
            && error
                .message
                .contains("require `concurrency=serial | parallel`")
    }));
    assert!(!errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION)));
}

#[test]
fn asyncio_callback_requires_async_callable_and_coroutine_target() {
    lower_success(
        r"
class PythonError(Error):
    message: str

class HandlerError(Error):
    message: str

@python.coroutine(pkg.compute)
@python.callback(handler, lifetime=call, dispatch=asyncio, concurrency=parallel)
async def compute(
    handler: AsyncCallable[[int], Result[int, HandlerError]],
) -> Result[int, PythonError | HandlerError]: ...
",
    );
    let invalid = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(pkg.compute)
@python.callback(handler, lifetime=call, dispatch=asyncio, concurrency=serial)
def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...
",
    );
    assert!(invalid.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCB_INVALID_DECLARATION)
            && error.message.contains("requires an `AsyncCallable`")
    }));
    assert!(!invalid
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION)));
}

#[test]
fn foreign_callback_rejects_python_identity_boundary() {
    let errors = lower_errors(
        r#"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Client, cleanup=close)
class Client:
    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python(pkg.compute)
@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=parallel)
def compute(handler: Callable[[Client], int]) -> Result[int, PythonError]: ...
"#,
    );
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCB_INVALID_DECLARATION)
            && error.message.contains("contains Python identity")
    }));
    assert!(!errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION)));
}

#[test]
fn callback_adjunct_rejects_non_implementation_python_declaration() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=current)
@python.attribute(name=value)
def value(handler: Callable[[int], int]) -> Result[int, PythonError]: ...
",
    );
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCB_INVALID_DECLARATION)
            && error
                .message
                .contains("requires one ordinary `@python(...)`")
    }));
    assert!(!errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION)));
}

#[test]
fn callback_policy_surface_is_closed() {
    let cases = [
        (
            "@python.callback(handler, dispatch=current)",
            "requires `lifetime=`",
        ),
        (
            "@python.callback(handler, lifetime=call)",
            "requires `dispatch=`",
        ),
        (
            "@python.callback(handler, lifetime=result, dispatch=current)",
            "requires `lifetime=call`",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=current, concurrency=serial)",
            "does not accept `concurrency=`",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=foreign)",
            "require `concurrency=serial | parallel`",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=unknown, concurrency=serial)",
            "unknown callback dispatch policy",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=serial, extra=value)",
            "unknown `@python.callback` argument `extra`",
        ),
    ];
    for (decorator, message) in cases {
        let source = format!(
            "class PythonError(Error):\n    message: str\n\n{decorator}\n@python(pkg.compute)\ndef compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...\n"
        );
        assert_callback_error(&source, message);
    }
}

#[test]
fn callback_parameter_and_dispatch_shapes_are_validated() {
    let cases = [
        (
            "@python.callback(missing, lifetime=call, dispatch=current)\n@python(pkg.compute)\ndef compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...",
            "unknown callback parameter `missing`",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=current)\n@python(pkg.compute)\ndef compute(handler: int) -> Result[int, PythonError]: ...",
            "must use `Callable[[...], R]` or `AsyncCallable[[...], R]`",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=current)\n@python.callback(handler, lifetime=call, dispatch=current)\n@python(pkg.compute)\ndef compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...",
            "more than one `@python.callback` declaration",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=current)\n@python(pkg.compute)\ndef compute(*handler: Callable[[int], int]) -> Result[int, PythonError]: ...",
            "requires one ordinary positional or keyword-only callable parameter",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=current)\n@python(pkg.compute)\ndef compute(**handler: Callable[[int], int]) -> Result[int, PythonError]: ...",
            "requires one ordinary positional or keyword-only callable parameter",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=serial)\n@python(pkg.compute)\ndef compute(handler: AsyncCallable[[int], int]) -> Result[int, PythonError]: ...",
            "`dispatch=foreign` requires a synchronous `Callable`",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=asyncio, concurrency=serial)\n@python.coroutine(pkg.compute)\nasync def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...",
            "`dispatch=asyncio` requires an `AsyncCallable`",
        ),
        (
            "@python.callback(handler, lifetime=call, dispatch=current)\n@python.coroutine(pkg.compute)\nasync def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...",
            "requires a synchronous `@python(...)` target",
        ),
    ];
    for (declaration, message) in cases {
        let source = format!("class PythonError(Error):\n    message: str\n\n{declaration}\n");
        assert_callback_error(&source, message);
    }
}

#[test]
fn callback_owner_error_conversion_and_sendability_are_validated() {
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

@python.callback(handler, lifetime=result, dispatch=foreign, concurrency=serial)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...
",
        "retained callback lifetime requires a declared opaque result",
    );
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Client, cleanup=drop)
class Client:
    pass

@python.callback(handler, lifetime=result, dispatch=foreign, concurrency=serial)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[Client, PythonError]: ...
",
        "requires deterministic close",
    );
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

class HandlerError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=current)
@python(pkg.compute)
def compute(handler: Callable[[int], Result[int, HandlerError]]) -> Result[int, PythonError]: ...
",
        "error channel must contain callback handler error `HandlerError`",
    );
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

class HandlerError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=current)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[int, HandlerError]: ...
",
        "error channel must contain `PythonError`",
    );
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=current)
@python(pkg.compute)
def compute(handler: Callable[[Callable[[int], int]], int]) -> Result[int, PythonError]: ...
",
        "has no direct Python conversion",
    );
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

class LocalState(NonSend):
    value: int

@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=parallel)
@python(pkg.compute)
def compute(handler: Callable[[LocalState], int]) -> Result[int, PythonError]: ...
",
        "is not sendable",
    );
}

#[test]
fn retained_result_and_receiver_owners_are_active() {
    lower_success(
        r"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Client, cleanup=close)
class Client:
    @python.callback(handler, lifetime=Self, dispatch=foreign, concurrency=serial)
    @python(Self.register)
    def register(self, own handler: Callable[[int], int]) -> Result[None, PythonError]: ...

    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python.callback(handler, lifetime=result, dispatch=foreign, concurrency=parallel)
@python(pkg.create)
def create(own handler: Callable[[int], int]) -> Result[Client, PythonError]: ...
",
    );
}

#[test]
fn receiver_lifetime_rejects_static_and_class_methods() {
    for (method_decorator, parameters) in [
        ("@staticmethod", "handler: Callable[[int], int]"),
        ("@classmethod", "cls, handler: Callable[[int], int]"),
    ] {
        let source = format!(
            r"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Client, cleanup=close)
class Client:
    @python.callback(handler, lifetime=Self, dispatch=foreign, concurrency=serial)
    @python(Self.register)
    {method_decorator}
    def register({parameters}) -> Result[None, PythonError]: ...

    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...
"
        );
        assert_callback_error(
            &source,
            "`lifetime=Self` is valid only on an opaque receiver method",
        );
    }
}

#[test]
fn retained_handler_error_must_be_declared_by_owner_cleanup() {
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

class HandlerError(Error):
    message: str

@python.opaque(type=pkg.Subscription, cleanup=close)
class Subscription:
    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python.callback(handler, lifetime=result, dispatch=foreign, concurrency=serial)
@python(pkg.subscribe)
def subscribe(
    own handler: Callable[[int], Result[int, HandlerError]],
) -> Result[Subscription, PythonError | HandlerError]: ...
",
        "owner cleanup `Subscription.close` error channel must contain handler error `HandlerError`",
    );
}

#[test]
fn foreign_callback_rejects_non_send_handler_capture_at_attachment() {
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

class LocalState(NonSend):
    value: int

@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=serial)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...

def run(state: LocalState) -> Result[int, PythonError]:
    def handler(value: int) -> int:
        return value + state.value
    return compute(handler)
",
        "handler `handler` capture `state`",
    );
}

#[test]
fn foreign_callback_rejects_python_identity_handler_capture_at_attachment() {
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Client, cleanup=close)
class Client:
    @python(Self.value)
    def value(self) -> Result[int, PythonError]: ...

    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=serial)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...

def run(client: Client) -> Result[int, PythonError]:
    def captured(value: int) -> int:
        _ = client.value()
        return value
    return compute(captured)
",
        "capture `client` of type `Client`",
    );
}

#[test]
fn parallel_callback_rejects_mutable_handler_capture_at_attachment() {
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=parallel)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...

def run(values: list[int]) -> Result[int, PythonError]:
    def handler(value: int) -> int:
        return value + len(values)
    return compute(handler)
",
        "is not share-safe",
    );
}

#[test]
fn receiver_callback_rejects_capturing_its_owner() {
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Client, cleanup=close)
class Client:
    @python.callback(handler, lifetime=Self, dispatch=asyncio, concurrency=serial)
    @python.coroutine(Self.register)
    async def register(self, own handler: AsyncCallable[[int], int]) -> Result[None, PythonError]: ...

    @python(Self.value)
    def value(self) -> Result[int, PythonError]: ...

    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

async def run(client: Client) -> Result[None, PythonError]:
    async def handler(value: int) -> int:
        _ = client.value()
        return value
    return await client.register(handler)
",
        "captures its retained owner",
    );
}

#[test]
fn foreign_callback_rejects_forwarded_callable_with_unknown_captures() {
    assert_callback_error(
        r"
class PythonError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=serial)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...

def run(handler: Callable[[int], int]) -> Result[int, PythonError]:
    return compute(handler)
",
        "callable value whose captures cannot be proven safe",
    );
}

#[test]
fn foreign_callback_accepts_capture_free_nested_handler() {
    lower_success(
        r"
class PythonError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=serial)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...

def run() -> Result[int, PythonError]:
    def handler(value: int) -> int:
        return value + 1
    return compute(handler)
",
    );
}

#[test]
fn foreign_and_asyncio_callbacks_accept_top_level_handlers() {
    lower_success(
        r"
class PythonError(Error):
    message: str

@python.callback(handler, lifetime=call, dispatch=foreign, concurrency=serial)
@python(pkg.compute)
def compute(handler: Callable[[int], int]) -> Result[int, PythonError]: ...

@python.callback(handler, lifetime=call, dispatch=asyncio, concurrency=serial)
@python.coroutine(pkg.compute_async)
async def compute_async(handler: AsyncCallable[[int], int]) -> Result[int, PythonError]: ...

def sync_handler(value: int) -> int:
    return value + 1

async def async_handler(value: int) -> int:
    await task.sleep(0.0)
    return value + 1

def run_sync() -> Result[int, PythonError]:
    return compute(sync_handler)

async def run_async() -> Result[int, PythonError]:
    return await compute_async(async_handler)
",
    );
}
