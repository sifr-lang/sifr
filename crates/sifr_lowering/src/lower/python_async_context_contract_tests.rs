use crate::{lower_module, HirDiagnostic};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

const ASYNC_CONTEXT_PREFIX: &str = r#"
class PythonError(Error):
    message: str

class ExitCause:
    pass

class ExitDecision:
    pass

@python.opaque(type=pkg.Transaction, cleanup=async_context)
class Transaction:
    @python.context.aenter(Self.__aenter__)
    async def __aenter__(self) -> Result[Transaction, PythonError]: ...

    @python.context.aexit(Self.__aexit__)
    async def __aexit__(own self, cause: ExitCause) -> Result[ExitDecision, PythonError]: ...
"#;

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should retain the async-context reservation"),
        Err(errors) => errors,
    }
}

fn has_code(errors: &[HirDiagnostic], code: DiagnosticCode) -> bool {
    errors.iter().any(|error| error.code == Some(code))
}

#[test]
fn valid_async_context_contract_is_retained_behind_reservation() {
    let errors = lower_errors(ASYNC_CONTEXT_PREFIX);
    assert!(has_code(
        &errors,
        DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION
    ));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYCTX_INVALID_DECLARATION
    ));
}

#[test]
fn async_context_requires_async_protocol_methods() {
    let source = ASYNC_CONTEXT_PREFIX.replacen("async def __aenter__", "def __aenter__", 1);
    let errors = lower_errors(&source);
    assert!(has_code(&errors, DiagnosticCode::PYCALL_INVALID_SHAPE));
    assert!(has_code(&errors, DiagnosticCode::PYCTX_INVALID_DECLARATION));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION
    ));
}

#[test]
fn async_context_rejects_sync_protocol_substitution() {
    let source = ASYNC_CONTEXT_PREFIX
        .replace("python.context.aenter", "python.context.enter")
        .replace("Self.__aenter__", "Self.__enter__")
        .replace("async def __aenter__", "def __enter__")
        .replace("python.context.aexit", "python.context.exit")
        .replace("Self.__aexit__", "Self.__exit__")
        .replace("async def __aexit__", "def __exit__");
    let errors = lower_errors(&source);
    assert!(has_code(&errors, DiagnosticCode::PYCTX_INVALID_DECLARATION));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION
    ));
}

#[test]
fn async_context_validates_aexit_signature_before_reservation() {
    let source = ASYNC_CONTEXT_PREFIX.replace("cause: ExitCause", "cause: int");
    let errors = lower_errors(&source);
    assert!(has_code(&errors, DiagnosticCode::PYCTX_INVALID_DECLARATION));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION
    ));
}

#[test]
fn async_context_obligation_is_reported_even_while_surface_is_gated() {
    let source = format!(
        "{ASYNC_CONTEXT_PREFIX}\n@python(pkg.Transaction)\ndef make_transaction() -> Result[Transaction, PythonError]: ...\n\nasync def abandon() -> Result[None, PythonError]:\n    transaction: Transaction = make_transaction()\n    return None\n"
    );
    let errors = lower_errors(&source);
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && error.message.contains("must be consumed by `async with`")
    }));
    assert!(has_code(
        &errors,
        DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION
    ));
}

#[test]
fn async_context_exit_cannot_be_called_directly() {
    let source = format!(
        "{ASYNC_CONTEXT_PREFIX}\nasync def invalid_exit(own transaction: Transaction, cause: ExitCause) -> Result[None, PythonError]:\n    decision: ExitDecision = await transaction.__aexit__(cause)\n    return None\n"
    );
    let errors = lower_errors(&source);
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("cannot be called directly")
    }));
}

#[test]
fn gated_async_with_selects_python_protocol_without_native_cause_diagnostics() {
    let source = format!(
        "{ASYNC_CONTEXT_PREFIX}\n@python(pkg.Transaction)\ndef make_transaction() -> Result[Transaction, PythonError]: ...\n\nasync def use_transaction() -> Result[None, PythonError]:\n    try:\n        async with make_transaction() as transaction:\n            pass\n        return None\n    except PythonError as error:\n        raise error\n"
    );
    let errors = lower_errors(&source);
    assert!(has_code(
        &errors,
        DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION
    ));
    assert!(!has_code(&errors, DiagnosticCode::TYPE_MISMATCH));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYCTX_INVALID_DECLARATION
    ));
    assert!(!has_code(&errors, DiagnosticCode::OWN_USE_AFTER_MOVE));
}
