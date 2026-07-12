use crate::{lower_module, HirDiagnostic, HirExpr, HirModule, HirStmt};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirClassKind, HirWithItemKind, PythonCleanupPolicy, PythonInteropEffect, PythonParameterKind,
};
use sifr_python_parser::parse_module;

fn lower_ok(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source should lower")
}

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

#[test]
fn sync_python_declaration_retains_target_effect_and_call_shape() {
    let module = lower_ok(
        r#"
class PythonError(Error):
    message: str

@python(pkg.api.compute)
def compute(value: int, *rest: int, flag: bool = False, missing: int = python.omit, **extra: int) -> Result[int, PythonError]: ...
"#,
    );
    let function = &module.functions[0];
    let declaration = &function.python_interop[0];
    assert!(function.body.is_empty());
    assert_eq!(declaration.effect, PythonInteropEffect::BlockingIo);
    assert_eq!(declaration.required_import_root.as_deref(), Some("pkg"));
    assert_eq!(
        declaration.target.as_ref().expect("target").dotted(),
        "pkg.api.compute"
    );
    assert_eq!(
        declaration
            .parameters
            .iter()
            .map(|parameter| parameter.kind)
            .collect::<Vec<_>>(),
        vec![
            PythonParameterKind::Positional,
            PythonParameterKind::PositionalVariadic,
            PythonParameterKind::KeywordOnly,
            PythonParameterKind::KeywordOnly,
            PythonParameterKind::KeywordVariadic,
        ]
    );
    assert!(declaration.parameters[3].omit_when_absent);
    assert_eq!(function.params.len(), 5);
}

#[test]
fn invalid_python_target_reports_pyimp_0001() {
    let errors = lower_errors(
        r#"
class PythonError(Error):
    message: str

@python("pkg.compute")
def compute(value: int) -> Result[int, PythonError]: ...
"#,
    );
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYIMP_INVALID_TARGET)));
}

#[test]
fn invalid_python_declaration_shape_reports_pycall_0001() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(pkg.compute, pkg.other)
def compute(value: int) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)));
}

#[test]
fn unsupported_python_conversion_reports_pyconv_0001() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(pkg.compute)
def compute(values: set[int]) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| { error.code == Some(DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE) }));
}

#[test]
fn later_python_decorator_is_a_hard_error() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python.coroutine(pkg.compute)
async def compute(value: int) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| { error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION) }));
}

#[test]
fn positional_variadics_after_omission_are_rejected() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(pkg.compute)
def compute(value: int = python.omit, *rest: int) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)));
}

#[test]
fn opaque_class_retains_type_and_cleanup_policy() {
    let module = lower_ok(
        r"
@python.opaque(type=pkg.Token, cleanup=drop)
class Token:
    pass
",
    );
    let HirClassKind::PythonOpaque(declaration) = &module.classes[0].kind else {
        panic!("class should be Python opaque");
    };
    assert_eq!(declaration.cleanup, Some(PythonCleanupPolicy::Drop));
    assert_eq!(
        declaration.target.as_ref().expect("target").dotted(),
        "pkg.Token"
    );
    assert_eq!(declaration.required_import_root.as_deref(), Some("pkg"));
}

#[test]
fn opaque_methods_retain_self_attribute_and_item_declarations() {
    let module = lower_ok(
        r#"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Token, cleanup=drop)
class Token:
    @python.attr(Self.name)
    def name(self) -> Result[str, PythonError]: ...

    @python.item
    def get(self, key: str) -> Result[int, PythonError]: ...

    @python(Self.refresh)
    def refresh(self, force: bool) -> Result[None, PythonError]: ...
"#,
    );
    let methods = &module.classes[1].methods;
    assert_eq!(methods.len(), 3);
    assert_eq!(
        methods[0].python_interop[0].kind,
        sifr_ir::PythonInteropDecoratorKind::Attribute
    );
    assert_eq!(
        methods[1].python_interop[0].kind,
        sifr_ir::PythonInteropDecoratorKind::Item
    );
    assert_eq!(
        methods[2].python_interop[0]
            .target
            .as_ref()
            .expect("target")
            .dotted(),
        "Self.refresh"
    );
}

const CLOSE_OPAQUE_PREFIX: &str = r#"
class PythonError(Error):
    message: str

@python.opaque(type=pkg.Client, cleanup=close)
class Client:
    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python(pkg.Client)
def make_client() -> Result[Client, PythonError]: ...
"#;

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

#[test]
fn context_opaque_retains_protocol_declarations_and_context_only_obligation() {
    let module = lower_ok(CONTEXT_OPAQUE_PREFIX);
    let transaction = module
        .classes
        .iter()
        .find(|class| class.name == "Transaction")
        .expect("transaction class");
    let HirClassKind::PythonOpaque(declaration) = &transaction.kind else {
        panic!("transaction should be Python opaque");
    };
    assert_eq!(declaration.cleanup, Some(PythonCleanupPolicy::Context));
    assert_eq!(
        transaction.methods[0].python_interop[0].kind,
        sifr_ir::PythonInteropDecoratorKind::ContextEnter
    );
    assert_eq!(
        transaction.methods[1].python_interop[0].kind,
        sifr_ir::PythonInteropDecoratorKind::ContextExit
    );
}

#[test]
fn invalid_python_context_declaration_reports_pyctx_0001() {
    let errors = lower_errors(&CONTEXT_OPAQUE_PREFIX.replace("cause: ExitCause", "cause: int"));
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)));
}

#[test]
fn context_only_obligation_cannot_transfer_through_return() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef forward() -> Result[Transaction, PythonError]:\n    try:\n        transaction: Transaction = make_transaction()\n        return transaction\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("rather than returned or aggregated")
    }));
}

#[test]
fn context_exit_method_cannot_be_called_directly() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_exit(own transaction: Transaction, cause: ExitCause) -> Result[None, PythonError]:\n    try:\n        _decision: ExitDecision = transaction.__exit__(cause)\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("cannot be called directly")
    }));
}

#[test]
fn context_enter_rejects_distinct_close_required_opaque_result() {
    let source = r#"
class PythonError(Error):
    message: str

class ExitCause:
    pass

class ExitDecision:
    pass

@python.opaque(type=pkg.Session, cleanup=close)
class Session:
    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python.opaque(type=pkg.Transaction, cleanup=context)
class Transaction:
    @python.context.enter(Self.__enter__)
    def __enter__(self) -> Result[Session, PythonError]: ...

    @python.context.exit(Self.__exit__)
    def __exit__(own self, cause: ExitCause) -> Result[ExitDecision, PythonError]: ...
"#;
    let errors = lower_errors(source);
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("distinct opaque `Session`")
    }));
}

#[test]
fn context_enter_rejects_aggregate_hiding_close_required_opaque_result() {
    let source = r#"
class PythonError(Error):
    message: str

class ExitCause:
    pass

class ExitDecision:
    pass

@python.opaque(type=pkg.Session, cleanup=close)
class Session:
    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...

@python.opaque(type=pkg.Transaction, cleanup=context)
class Transaction:
    @python.context.enter(Self.__enter__)
    def __enter__(self) -> Result[list[Session], PythonError]: ...

    @python.context.exit(Self.__exit__)
    def __exit__(own self, cause: ExitCause) -> Result[ExitDecision, PythonError]: ...
"#;
    let errors = lower_errors(source);
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("entered aggregates cannot hide")
    }));
}

#[test]
fn python_with_retains_dedicated_hir_and_scoped_opaque_borrow() {
    let module = lower_ok(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef use_transaction() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            pass\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "use_transaction")
        .expect("function");
    let HirStmt::TryExcept { body, .. } = &function.body[0] else {
        panic!("function should retain try body");
    };
    let HirStmt::With { items, .. } = &body[0] else {
        panic!("body should retain with statement");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].target, "transaction");
    let HirWithItemKind::Python {
        entered_type,
        enter_error_type,
        exit_error_type,
        entered_is_opaque_borrow,
    } = &items[0].kind
    else {
        panic!("item should use dedicated Python context HIR");
    };
    assert_eq!(entered_type.display_name(), "Transaction");
    assert_eq!(enter_error_type.display_name(), "PythonError");
    assert_eq!(exit_error_type.display_name(), "PythonError");
    assert!(*entered_is_opaque_borrow);
}

#[test]
fn python_with_consumes_prebound_context_manager_obligation() {
    lower_ok(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef use_transaction() -> Result[None, PythonError]:\n    try:\n        transaction: Transaction = make_transaction()\n        with transaction as entered:\n            pass\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
}

#[test]
fn python_with_supports_multiple_context_items_with_distinct_borrows() {
    let module = lower_ok(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef use_transactions() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as first, make_transaction() as second:\n            pass\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "use_transactions")
        .expect("function");
    let HirStmt::TryExcept { body, .. } = &function.body[0] else {
        panic!("function should retain try body");
    };
    let HirStmt::With { items, .. } = &body[0] else {
        panic!("body should retain with statement");
    };
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|item| matches!(
        item.kind,
        HirWithItemKind::Python {
            entered_is_opaque_borrow: true,
            ..
        }
    )));
}

#[test]
fn python_with_nonopaque_entered_value_is_ordinary_owned_data() {
    let source = r#"
class PythonError(Error):
    message: str

class ExitCause:
    pass

class ExitDecision:
    pass

@python.opaque(type=pkg.Counter, cleanup=context)
class Counter:
    @python.context.enter(Self.__enter__)
    def __enter__(self) -> Result[int, PythonError]: ...

    @python.context.exit(Self.__exit__)
    def __exit__(own self, cause: ExitCause) -> Result[ExitDecision, PythonError]: ...

@python(pkg.Counter)
def make_counter() -> Result[Counter, PythonError]: ...

def use_counter() -> Result[int, PythonError]:
    try:
        with make_counter() as value:
            copied: int = value
            return copied
    except PythonError as error:
        raise error
"#;
    let module = lower_ok(source);
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "use_counter")
        .expect("function");
    let HirStmt::TryExcept { body, .. } = &function.body[0] else {
        panic!("function should retain try body");
    };
    let HirStmt::With { items, .. } = &body[0] else {
        panic!("body should retain with statement");
    };
    assert!(matches!(
        items[0].kind,
        HirWithItemKind::Python {
            entered_is_opaque_borrow: false,
            ..
        }
    ));
}

#[test]
fn python_context_entered_borrow_cannot_escape_by_return_or_aggregate() {
    for return_expr in ["transaction", "[transaction]"] {
        let return_type = if return_expr.starts_with('[') {
            "list[Transaction]"
        } else {
            "Transaction"
        };
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\ndef escape() -> Result[{return_type}, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            return {return_expr}\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                && error.message.contains("context-scoped borrow")
                && error.message.contains("cannot escape")
        }));
    }
}

#[test]
fn python_context_entered_borrow_cannot_be_aliased_or_discarded() {
    for body in [
        "alias: Transaction = transaction",
        "alias = transaction",
        "alias = shortcut = transaction",
        "_ = transaction",
    ] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_store() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            {body}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                && error.message.contains("context-scoped borrow")
        }));
    }
}

#[test]
fn python_context_entered_borrow_cannot_escape_through_match_capture() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_capture() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            match transaction:\n                case alias if False:\n                    pass\n                case _:\n                    pass\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                && error.message.contains("context-scoped borrow")
        }),
        "{errors:?}"
    );
}

#[test]
fn python_context_entered_borrow_cannot_escape_through_augmented_assignment() {
    for (extra, setup, statement) in [
        (
            "",
            "stored: list[Transaction] = []",
            "stored += [transaction]",
        ),
        (
            "class Box:\n    items: list[Transaction]\n",
            "box: Box = Box([])",
            "box.items += [transaction]",
        ),
        (
            "",
            "outer: list[list[Transaction]] = [[]]",
            "outer[0] += [transaction]",
        ),
    ] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\n{extra}\ndef invalid_store() -> Result[None, PythonError]:\n    try:\n        {setup}\n        with make_transaction() as transaction:\n            {statement}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                    && error.message.contains("context-scoped borrow")
            }),
            "{errors:?}"
        );
    }
}

#[test]
fn python_context_entered_borrow_cannot_move_into_owned_parameter() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef consume(own transaction: Transaction) -> Result[None, PythonError]:\n    try:\n        with transaction as entered:\n            pass\n        return None\n    except PythonError as error:\n        raise error\n\ndef invalid_consume() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            consumed: None = consume(transaction)\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                && error
                    .message
                    .contains("cannot be moved or closed independently")
        }),
        "{errors:?}"
    );
}

#[test]
fn python_context_entered_borrow_cannot_be_stored_in_field_or_subscript() {
    for statement in ["holder[0] = transaction", "box.value = transaction"] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\nclass Box:\n    value: Transaction | None\n\ndef invalid_store() -> Result[None, PythonError]:\n    try:\n        holder: list[Transaction] = []\n        box: Box = Box(None)\n        with make_transaction() as transaction:\n            {statement}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                && error.message.contains("context-scoped borrow")
        }));
    }
}

#[test]
fn python_context_entered_borrow_does_not_escape_lexical_scope() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_use() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            pass\n        alias: Transaction = transaction\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::NAME_UNDEFINED_VARIABLE)
            && error.message.contains("transaction")
    }));
}

#[test]
fn python_context_entered_borrow_cannot_escape_by_yield() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef leak() -> list[Transaction]:\n    try:\n        with make_transaction() as transaction:\n            yield transaction\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("cannot escape by yield")
    }));
}

#[test]
fn python_context_entered_borrow_cannot_escape_through_unpacking() {
    for assignment in [
        "first, number = transaction, 1",
        "first, *rest = [transaction, transaction]",
    ] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_unpack() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            {assignment}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                && error.message.contains("cannot escape through unpacking")
        }));
    }
}

#[test]
fn result_typed_prebound_python_manager_is_consumed_by_first_with() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_reuse() -> Result[None, PythonError]:\n    try:\n        pending = make_transaction()\n        with pending as first:\n            pass\n        with pending as second:\n            pass\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE) && error.message.contains("pending")
    }));
}

#[test]
fn python_context_entered_borrow_cannot_escape_through_comprehension_or_closure() {
    for expression in [
        "[transaction for _ in range(1)]",
        "[item for item in [transaction]]",
        "{item for item in [transaction]}",
        "{0: item for item in [transaction]}",
        "[1 for _ in range(1) if transaction == transaction]",
        "(transaction for _ in range(1))",
        "lambda: transaction",
        "lambda: f'{transaction}'",
    ] {
        let errors = lower_errors(&format!(
            "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_capture() -> Result[None, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            stored = {expression}\n        return None\n    except PythonError as error:\n        raise error\n"
        ));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                    && error.message.contains("context-scoped borrow")
            }),
            "{errors:?}"
        );
    }
}

#[test]
fn python_context_entered_borrow_cannot_escape_through_for_iterable() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_iteration() -> Result[Transaction, PythonError]:\n    try:\n        with make_transaction() as transaction:\n            for item in [transaction]:\n                return item\n        return make_transaction()\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
                && error.message.contains("context-scoped borrow")
        }),
        "{errors:?}"
    );
}

#[test]
fn fallible_python_context_outside_try_has_dedicated_diagnostic() {
    let errors = lower_errors(&format!(
        "{CONTEXT_OPAQUE_PREFIX}\ndef invalid_context() -> None:\n    with make_transaction() as transaction:\n        pass\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYCTX_INVALID_DECLARATION)
            && error.message.contains("requires an enclosing try block")
    }));
}

#[test]
fn close_opaque_obligation_is_discharged_by_consuming_close() {
    lower_ok(&format!(
        "{CLOSE_OPAQUE_PREFIX}\ndef use_client() -> Result[None, PythonError]:\n    try:\n        client: Client = make_client()\n        _closed: None = client.close()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
}

#[test]
fn close_opaque_obligation_rejects_abandonment() {
    let errors = lower_errors(&format!(
        "{CLOSE_OPAQUE_PREFIX}\ndef abandon_client() -> Result[None, PythonError]:\n    try:\n        client: Client = make_client()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && error.message.contains("must-use binding 'client'")
    }));
}

#[test]
fn close_opaque_obligation_transfers_through_return() {
    lower_ok(&format!(
        "{CLOSE_OPAQUE_PREFIX}\ndef forward_client() -> Result[Client, PythonError]:\n    try:\n        client: Client = make_client()\n        return client\n    except PythonError as error:\n        raise error\n"
    ));
}

#[test]
fn close_opaque_obligation_rejects_partial_branch_consumption() {
    let errors = lower_errors(&format!(
        "{CLOSE_OPAQUE_PREFIX}\ndef partial(flag: bool) -> Result[None, PythonError]:\n    try:\n        client: Client = make_client()\n        if flag:\n            _closed: None = client.close()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && error
                .message
                .contains("only some continuing control-flow branches")
    }));
}

#[test]
fn close_opaque_obligation_transfers_through_owned_aggregate_return() {
    lower_ok(&format!(
        "{CLOSE_OPAQUE_PREFIX}\ndef forward_many() -> Result[list[Client], PythonError]:\n    try:\n        client: Client = make_client()\n        return [client]\n    except PythonError as error:\n        raise error\n"
    ));
}

#[test]
fn consuming_close_rejects_double_close_and_use_after_close() {
    let errors = lower_errors(&format!(
        "{CLOSE_OPAQUE_PREFIX}\ndef double_close() -> Result[None, PythonError]:\n    try:\n        client: Client = make_client()\n        _first: None = client.close()\n        _second: None = client.close()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE) && error.message.contains("client")
    }));
}

#[test]
fn close_opaque_obligation_rejects_live_reassignment() {
    let errors = lower_errors(&format!(
        "{CLOSE_OPAQUE_PREFIX}\ndef replace_client() -> Result[None, PythonError]:\n    try:\n        client: Client = make_client()\n        client = make_client()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && error
                .message
                .contains("cannot reassign must-use binding 'client'")
    }));
}

#[test]
fn close_opaque_obligation_transfers_through_comprehension_return() {
    lower_ok(&format!(
        "{CLOSE_OPAQUE_PREFIX}\ndef forward_comprehension(own clients: list[Client]) -> list[Client]:\n    return [client for client in clients]\n"
    ));
}

#[test]
fn method_exit_ignores_consumed_obligations_from_popped_nested_scopes() {
    lower_ok(&format!(
        "{CLOSE_OPAQUE_PREFIX}\nclass Owner:\n    def use_nested(self) -> Result[None, PythonError]:\n        try:\n            if True:\n                client: Client = make_client()\n                _closed: None = client.close()\n            return None\n        except PythonError as error:\n            raise error\n"
    ));
}

#[test]
fn python_declaration_rejects_non_stub_body() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(pkg.compute)
def compute(value: int) -> Result[int, PythonError]:
    ...
    return Ok(value)
",
    );
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)));
}

#[test]
fn sync_python_declaration_is_blocking_in_async_context() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(math.sqrt)
def sqrt(value: float) -> Result[float, PythonError]: ...

async def use_sqrt():
    sqrt(4.0)
    await sleep(0.0)
",
    );
    assert!(errors
        .iter()
        .any(|error| { error.code == Some(DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL) }));
}

#[test]
fn python_calls_lower_omit_none_variadics_and_typed_kwargs() {
    let module = lower_ok(
        r#"
class PythonError(Error):
    message: str

@python(pkg.collect)
def collect(value: int, *rest: int, label: str | None = python.omit, **extra: int) -> Result[int, PythonError]: ...

def main():
    first = collect(1)
    second = collect(1, 2, 3, label=None, count=4)
    options: dict[str, int] = {"count": 5}
    third = collect(1, **options)
    _ = first
    _ = second
    _ = third
"#,
    );
    let declaration = &module.functions[0];
    assert_eq!(declaration.params.len(), 4);
    assert_eq!(declaration.python_interop[0].parameters.len(), 4);
    assert!(declaration.python_interop[0].parameters[2].omit_when_absent);
}

#[test]
fn closed_record_expansion_retains_fields_and_span_in_hir() {
    let module = lower_ok(
        r#"
class PythonError(Error):
    message: str

class Options:
    label: str
    count: int

    def __init__(self, label: str, count: int):
        self.label = label
        self.count = count

@python(pkg.collect)
def collect(*, label: str, count: int) -> Result[int, PythonError]: ...

def main():
    options = Options("x", 2)
    result = collect(**options)
    _ = result
"#,
    );
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main");
    let call = main.body.iter().find_map(|statement| match statement {
        HirStmt::Let {
            value: call @ HirExpr::PythonCall { .. },
            ..
        } => Some(call),
        _ => None,
    });
    let Some(HirExpr::PythonCall {
        record_expansions, ..
    }) = call
    else {
        panic!("record call should retain PythonCall metadata");
    };
    assert_eq!(record_expansions.len(), 1);
    assert_eq!(record_expansions[0].fields, ["label", "count"]);
    assert!(!record_expansions[0].span.is_empty());
}
