use crate::{lower_module, HirDiagnostic};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should produce lowering errors"),
        Err(errors) => errors,
    }
}

fn assert_lowers(source: &str) {
    let parsed = parse_module(source).expect("source should parse");
    if let Err(errors) = lower_module(parsed.suite()) {
        panic!("active async Python declaration should lower: {errors:?}");
    }
}

fn has_code(errors: &[HirDiagnostic], code: DiagnosticCode) -> bool {
    errors.iter().any(|error| error.code == Some(code))
}

const PYTHON_ERROR: &str = r"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str
";

const ASYNC_CLOSE_PREFIX: &str = r"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python.opaque(type=pkg.Client, cleanup=async_close)
class Client:
    @python.coroutine(Self.aclose)
    async def aclose(own self) -> Result[None, PythonError]: ...

@python.coroutine(pkg.make_client)
async def make_client() -> Result[Client, PythonError]: ...
";

#[test]
fn valid_coroutine_contract_is_active() {
    assert_lowers(&format!(
        "{PYTHON_ERROR}\n@python.coroutine(pkg.compute)\nasync def compute(value: int) -> Result[int, PythonError]: ...\n"
    ));
}

#[test]
fn active_coroutine_contract_validates_async_shape_and_conversion() {
    let sync_errors = lower_errors(&format!(
        "{PYTHON_ERROR}\n@python.coroutine(pkg.compute)\ndef compute(value: int) -> Result[int, PythonError]: ...\n"
    ));
    assert!(has_code(&sync_errors, DiagnosticCode::PYCALL_INVALID_SHAPE));

    let conversion_errors = lower_errors(&format!(
        "{PYTHON_ERROR}\n@python.coroutine(pkg.compute)\nasync def compute(values: set[int]) -> Result[int, PythonError]: ...\n"
    ));
    assert!(has_code(
        &conversion_errors,
        DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE
    ));
}

#[test]
fn synchronous_python_decorator_is_rejected_on_async_def() {
    let errors = lower_errors(&format!(
        "{PYTHON_ERROR}\n@python(pkg.compute)\nasync def compute(value: int) -> Result[int, PythonError]: ...\n"
    ));
    assert!(has_code(&errors, DiagnosticCode::PYCALL_INVALID_SHAPE));
}

#[test]
fn active_sync_method_decorator_reports_shape_not_reserved_on_async_def() {
    let errors = lower_errors(&format!(
        "{PYTHON_ERROR}\n@python.opaque(type=pkg.Client, cleanup=drop)\nclass Client:\n    @python.attr(Self.name)\n    async def name(self) -> Result[str, PythonError]: ...\n"
    ));
    assert!(has_code(&errors, DiagnosticCode::PYCALL_INVALID_SHAPE));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYRES_UNSUPPORTED_RESOURCE_DECLARATION
    ));
}

#[test]
fn valid_async_close_contract_is_active() {
    assert_lowers(&format!(
        "{PYTHON_ERROR}\n@python.opaque(type=pkg.Client, cleanup=async_close)\nclass Client:\n    @python.coroutine(Self.aclose)\n    async def aclose(own self) -> Result[None, PythonError]: ...\n"
    ));
}

#[test]
fn async_close_requires_one_consuming_aclose_coroutine() {
    for source in [
        format!(
            "{PYTHON_ERROR}\n@python.opaque(type=pkg.Client, cleanup=async_close)\nclass Client:\n    @python.coroutine(Self.aclose)\n    async def aclose(self) -> Result[None, PythonError]: ...\n"
        ),
        format!(
            "{PYTHON_ERROR}\n@python.opaque(type=pkg.Client, cleanup=async_close)\nclass Client:\n    @python.coroutine(Self.shutdown)\n    async def shutdown(own self) -> Result[None, PythonError]: ...\n"
        ),
    ] {
        let errors = lower_errors(&source);
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)
                && error.message.contains("cleanup=async_close")
        }));
    }
}

#[test]
fn active_async_close_consumption_discharges_obligation() {
    assert_lowers(&format!(
        "{ASYNC_CLOSE_PREFIX}\nasync def use_client() -> Result[None, PythonError]:\n    try:\n        client: Client = await make_client()\n        _closed: None = await client.aclose()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
}

#[test]
fn async_close_obligation_rejects_abandonment_and_partial_consumption() {
    let abandoned = lower_errors(&format!(
        "{ASYNC_CLOSE_PREFIX}\nasync def abandon_client() -> Result[None, PythonError]:\n    try:\n        client: Client = await make_client()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        abandoned.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                && error.message.contains("must-use binding 'client'")
        }),
        "{abandoned:?}"
    );

    let partial = lower_errors(&format!(
        "{ASYNC_CLOSE_PREFIX}\nasync def partial_close(flag: bool) -> Result[None, PythonError]:\n    try:\n        client: Client = await make_client()\n        if flag:\n            _closed: None = await client.aclose()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        partial.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                && error
                    .message
                    .contains("only some continuing control-flow branches")
        }),
        "{partial:?}"
    );
}

#[test]
fn async_close_obligation_rejects_duplicate_close_and_reuse() {
    let errors = lower_errors(&format!(
        "{ASYNC_CLOSE_PREFIX}\nasync def double_close() -> Result[None, PythonError]:\n    try:\n        client: Client = await make_client()\n        _first: None = await client.aclose()\n        _second: None = await client.aclose()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                && error.message.contains("client")
        }),
        "{errors:?}"
    );
}
