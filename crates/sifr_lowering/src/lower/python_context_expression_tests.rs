use crate::{lower_module, HirDiagnostic};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

const CONTEXT_OPAQUE_PREFIX: &str = r#"
class PythonError(Error):
    message: str

class ExitCause:
    pass

class ExitDecision:
    pass

@python.opaque(type=pkg.Transaction, cleanup=context)
class Transaction:
    @python.context.enter(Self.__enter__)
    def __enter__(self) -> Result[Transaction, PythonError]: ...

    @python.context.exit(Self.__exit__)
    def __exit__(own self, cause: ExitCause) -> Result[ExitDecision, PythonError]: ...

@python(pkg.Transaction)
def make_transaction() -> Result[Transaction, PythonError]: ...
"#;

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

fn has_context_borrow_error(errors: &[HirDiagnostic]) -> bool {
    errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("context-scoped borrow")
    })
}

#[test]
fn python_context_entered_borrow_cannot_escape_through_walrus() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_alias() -> Result[Transaction, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            (alias := transaction)\n            return alias\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(has_context_borrow_error(&errors), "{errors:?}");
}

#[test]
fn python_context_entered_borrow_cannot_be_implicitly_discarded() {
    for expression in ["transaction", "[transaction]", "{transaction}"] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_discard() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            {expression}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                    && error.message.contains("cannot be discarded")
            }),
            "{errors:?}"
        );
    }
}

#[test]
fn python_context_entered_borrow_cannot_move_through_temporary_call_argument() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef sink(values: list[Transaction]) -> None:\n    return None\n\ndef invalid_move() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            sink([transaction])\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(has_context_borrow_error(&errors), "{errors:?}");
}

#[test]
fn python_context_entered_borrow_cannot_move_through_temporary_method_argument() {
    for invocation in [
        "sink.collect([transaction])",
        "Registry.store([transaction])",
    ] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\nclass Sink:\n    def collect(self, values: list[Transaction]) -> None:\n        return None\n\nclass Registry:\n    @staticmethod\n    def store(values: list[Transaction]) -> None:\n        return None\n\ndef invalid_move(sink: Sink) -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            {invocation}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(has_context_borrow_error(&errors), "{errors:?}");
    }
}

#[test]
fn python_context_entered_borrow_composite_is_rejected_in_return_and_condition() {
    for body in [
        "return sink([transaction])",
        "if predicate([transaction]):\n                return None",
        "assert predicate([transaction])",
    ] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\ndef sink(values: list[Transaction]) -> None:\n    return None\n\ndef predicate(values: list[Transaction]) -> bool:\n    return True\n\ndef invalid_use() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            {body}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(has_context_borrow_error(&errors), "{errors:?}");
    }
}

#[test]
fn python_context_entered_borrow_cannot_move_into_collection_method() {
    for (parameter, statement) in [
        (
            "mut stored: list[Transaction]",
            "stored.append(transaction)",
        ),
        ("mut stored: set[Transaction]", "stored.add(transaction)"),
    ] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_store({parameter}) -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            {statement}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(has_context_borrow_error(&errors), "{errors:?}");
    }
}
