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
