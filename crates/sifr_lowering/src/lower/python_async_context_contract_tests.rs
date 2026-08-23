use crate::{HirDiagnostic, lower_module};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

const ASYNC_CONTEXT_PREFIX: &str = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

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
        Ok(_) => panic!("source should fail async-context validation"),
        Err(errors) => errors,
    }
}

fn lower_success(source: &str) {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite()).expect("source should lower without diagnostics");
}

fn has_code(errors: &[HirDiagnostic], code: DiagnosticCode) -> bool {
    errors.iter().any(|error| error.code == Some(code))
}

#[test]
fn valid_async_context_contract_is_active() {
    lower_success(ASYNC_CONTEXT_PREFIX);
}

#[test]
fn async_context_requires_async_protocol_methods() {
    let source = ASYNC_CONTEXT_PREFIX.replacen("async def __aenter__", "def __aenter__", 1);
    let errors = lower_errors(&source);
    assert!(has_code(&errors, DiagnosticCode::PYCALL_INVALID_SHAPE));
    assert!(has_code(&errors, DiagnosticCode::PYCTX_INVALID_DECLARATION));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYRES_UNSUPPORTED_RESOURCE_DECLARATION
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
        DiagnosticCode::PYRES_UNSUPPORTED_RESOURCE_DECLARATION
    ));
}

#[test]
fn async_context_validates_aexit_signature() {
    let source = ASYNC_CONTEXT_PREFIX.replace("cause: ExitCause", "cause: int");
    let errors = lower_errors(&source);
    assert!(has_code(&errors, DiagnosticCode::PYCTX_INVALID_DECLARATION));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYRES_UNSUPPORTED_RESOURCE_DECLARATION
    ));
}

#[test]
fn async_context_rejects_distinct_entered_resource_without_drop_cleanup() {
    let source = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

class ExitCause:
    pass

class ExitDecision:
    pass

@python.opaque(type=pkg.Session, cleanup=async_close)
class Session:
    @python.coroutine(Self.aclose)
    async def aclose(own self) -> Result[None, PythonError]: ...

@python.opaque(type=pkg.Transaction, cleanup=async_context)
class Transaction:
    @python.context.aenter(Self.__aenter__)
    async def __aenter__(self) -> Result[Session, PythonError]: ...

    @python.context.aexit(Self.__aexit__)
    async def __aexit__(own self, cause: ExitCause) -> Result[ExitDecision, PythonError]: ...
"#;
    let errors = lower_errors(source);
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("distinct opaque `Session`")
            && error
                .message
                .contains("only the manager identity or `cleanup=drop`")
    }));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYRES_UNSUPPORTED_RESOURCE_DECLARATION
    ));
}

#[test]
fn async_context_obligation_is_reported_on_the_active_surface() {
    let source = format!(
        "{ASYNC_CONTEXT_PREFIX}\n@python.coroutine(pkg.make_transaction)\nasync def make_transaction() -> Result[Transaction, PythonError]: ...\n\nasync def abandon() -> Result[None, PythonError]:\n    transaction: Transaction = await make_transaction()\n    return None\n"
    );
    let errors = lower_errors(&source);
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && error.message.contains("must be consumed by `async with`")
    }));
    assert!(!has_code(
        &errors,
        DiagnosticCode::PYRES_UNSUPPORTED_RESOURCE_DECLARATION
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
fn active_async_with_selects_python_protocol_without_native_cause_diagnostics() {
    let source = format!(
        "{ASYNC_CONTEXT_PREFIX}\n@python.coroutine(pkg.make_transaction)\nasync def make_transaction() -> Result[Transaction, PythonError]: ...\n\nasync def use_transaction() -> Result[None, PythonError]:\n    try:\n        manager: Transaction = await make_transaction()\n        async with manager as transaction:\n            pass\n        return None\n    except PythonError as error:\n        raise error\n"
    );
    lower_success(&source);
}

#[test]
fn active_async_with_accepts_python_errors_under_the_builtin_error_supertype() {
    let source = format!(
        "{ASYNC_CONTEXT_PREFIX}\n@python.coroutine(pkg.make_transaction)\nasync def make_transaction() -> Result[Transaction, PythonError]: ...\n\nasync def use_transaction() -> Result[None, Error]:\n    try:\n        manager: Transaction = await make_transaction()\n        async with manager as transaction:\n            raise ValueError(\"body failure\")\n        return None\n    except Error as error:\n        raise error\n"
    );
    lower_success(&source);
}
