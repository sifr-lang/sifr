use crate::{lower_module, HirDiagnostic};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
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
fn valid_callback_policy_is_retained_before_reservation() {
    let errors = lower_errors(
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
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION)));
    assert!(!errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCB_INVALID_DECLARATION)
            || error.code == Some(DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE)
    }));
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
    let valid = lower_errors(
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
    assert!(valid
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION)));
    assert!(!valid
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCB_INVALID_DECLARATION)));

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
fn retained_result_and_receiver_owners_pass_validation_before_reservation() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Client, cleanup=close)
class Client:
    @python.callback(handler, lifetime=Self, dispatch=foreign, concurrency=serial)
    @python(Self.register)
    def register(self, handler: Callable[[int], int]) -> Result[None, PythonError]: ...

    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python.callback(handler, lifetime=result, dispatch=foreign, concurrency=parallel)
@python(pkg.create)
def create(handler: Callable[[int], int]) -> Result[Client, PythonError]: ...
",
    );
    assert!(!errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCB_INVALID_DECLARATION)));
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION))
            .count(),
        2,
        "unexpected diagnostics: {errors:?}"
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
