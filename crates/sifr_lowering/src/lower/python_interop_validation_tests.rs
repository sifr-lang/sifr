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
fn unsupported_python_conversion_reports_pyconv_0001() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python(pkg.compute)
def compute(values: set[int]) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| { error.code == Some(DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE) }));
}

#[test]
fn python_declaration_rejects_shadow_python_error_contract() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str
    code: int

@python(pkg.compute)
def compute(value: int) -> Result[int, PythonError]: ...
",
    );
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE)
            && error
                .message
                .contains("canonical `PythonError` field contract")
    }));
}

#[test]
fn positional_variadics_after_omission_are_rejected() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python(pkg.compute)
def compute(value: int = python.omit, *rest: int) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)));
}

#[test]
fn python_declarations_require_exactly_one_ellipsis_statement() {
    for source in [
        r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python(pkg.compute)
def compute(value: int) -> Result[int, PythonError]:
    return Ok(value)
"#,
        r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python.coroutine(pkg.compute)
async def compute(value: int) -> Result[int, PythonError]:
    return Ok(value)
"#,
        r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python.opaque(type=pkg.Client, cleanup=drop)
class Client:
    @python(Self.refresh)
    def refresh(self) -> Result[None, PythonError]:
        return None
"#,
    ] {
        let errors = lower_errors(source);
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)
                    && error.message.contains("exactly one ellipsis")
            }),
            "{errors:?}"
        );
    }
}

#[test]
fn nested_python_declarations_are_rejected_without_discarding_decorators() {
    for body in ["return 123", "..."] {
        let errors = lower_errors(&format!(
            "def outer() -> int:\n    @python(math.sqrt)\n    def compute(value: float) -> Result[float, PythonError]:\n        {body}\n    return 0\n"
        ));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)
                    && error
                        .message
                        .contains("nested Python declarations are not supported")
            }),
            "{errors:?}"
        );
    }
}

#[test]
fn python_class_decorators_require_the_opaque_declaration_form() {
    for decorator in ["@python(pkg.Client)", "@python"] {
        let errors = lower_errors(&format!("{decorator}\nclass Client:\n    value: int\n"));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)
                    && error.message.contains("Python class declarations must use")
            }),
            "{errors:?}"
        );
    }
}

#[test]
fn wrapped_python_rooted_decorators_are_rejected_on_every_declaration_surface() {
    let cases = [
        "@python(math.sqrt)()\ndef compute(value: float) -> float:\n    return value\n",
        "@python(math.sqrt).extra\ndef compute(value: float) -> float: ...\n",
        "@python.opaque(type=pkg.Client, cleanup=drop)()\nclass Client:\n    value: int\n",
        "@python.opaque(type=pkg.Client, cleanup=drop)\nclass Client:\n    @python(Self.read)()\n    def read(self) -> int: ...\n",
        "def outer() -> int:\n    @python(math.sqrt)()\n    def compute(value: float) -> float: ...\n    return 0\n",
    ];
    for source in cases {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)),
            "source passed without a Python declaration shape diagnostic:\n{source}\n{errors:?}"
        );
    }
}

#[test]
fn cleanup_opaque_reusable_callable_captures_are_rejected() {
    const PREFIX: &str = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python.opaque(type=pkg.Client, cleanup=close)
class Client:
    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python(pkg.Client)
def make_client() -> Result[Client, PythonError]: ...
"#;
    for body in [
        "def keep() -> Callable[[], Client]:\n    client: Client = make_client()\n    return lambda: client\n",
        "def keep() -> Callable[[], Client]:\n    client: Client = make_client()\n    def inner() -> Client:\n        return client\n    return inner\n",
    ] {
        let errors = lower_errors(&format!("{PREFIX}\n{body}"));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                    && error.message.contains("must-use Python resource")
            }),
            "{errors:?}"
        );
    }

    let errors = lower_errors(&format!(
        "{PREFIX}\ndef close_through_capture() -> Result[None, PythonError]:\n    try:\n        client = make_client()\n        def close_client() -> Result[None, PythonError]:\n            return client.close()\n        _first: None = close_client()\n        _second: None = close_client()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("type 'Client'")
                && !error.message.contains("Result[")
        }),
        "{errors:?}"
    );
}

#[test]
fn bridge_opaque_target_is_terminally_unsupported() {
    let errors = lower_errors(
        r#"
@python.opaque(type=bridge.local.Client, cleanup=drop)
class Client:
    pass
"#,
    );
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYRES_UNSUPPORTED_RESOURCE_DECLARATION)
            && error.message.contains("package-local `bridge.*`")
            && !error.message.contains("later phase")
    }));
}
