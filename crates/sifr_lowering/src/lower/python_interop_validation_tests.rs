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
