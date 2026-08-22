use crate::{lower_module, HirDiagnostic, HirStmt};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("parse failed");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    }
}

fn range_for(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle should exist") as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

fn range_for_after(source: &str, after: &str, needle: &str) -> TextRange {
    let after_start = source.find(after).expect("anchor should exist") + after.len();
    let relative_start = source[after_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = (after_start + relative_start) as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

#[test]
fn yield_without_value_has_statement_form_code() {
    let source = "def count():\n    yield\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::FLOW_UNSUPPORTED_STATEMENT_FORM)
            && error.message == "unsupported statement form: yield without a value"
            && error.primary_range == Some(range_for(source, "yield"))
    }));
}

#[test]
fn annotated_assignment_target_has_assignment_target_code() {
    let source = "def main():\n    values: list[int] = [0]\n    values[0]: int = 1\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.message
                == "invalid assignment target: annotated assignment target must be a simple name"
            && error.primary_range == Some(range_for(source, "values[0]"))
    }));
}

#[test]
fn annotated_variable_without_initializer_has_name_code() {
    let source = "def main():\n    value: int\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::NAME_UNINITIALIZED_VARIABLE)
            && error.message == "variable 'value' must be initialized"
            && error.primary_range == Some(range_for(source, "value"))
    }));
}

#[test]
fn match_tuple_pattern_subject_mismatch_has_match_code() {
    let source = "def main():\n    value: int = 1\n    match value:\n        case (a, b):\n            pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::MATCH_INVALID_PATTERN_FORM)
            && error
                .message
                .contains("tuple pattern requires subject of tuple type")
            && error.primary_range == Some(range_for(source, "(a, b)"))
    }));
}

#[test]
fn for_loop_invalid_iterable_has_iteration_code() {
    let source = "def main():\n    value: int = 1\n    for item in value:\n        pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.message == "invalid for-loop iteration: cannot iterate over type 'int'"
            && error.primary_range == Some(range_for_after(source, "for item in ", "value"))
    }));
}

#[test]
fn unknown_except_type_has_result_code() {
    let source = "\
def fallible() -> Result[int, ValueError]:
    raise ValueError(\"bad\")

def main():
    try:
        value: int = fallible()
    except MissingError as e:
        pass
";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RESULT_UNKNOWN_EXCEPT_TYPE)
            && error.message == "unknown except error type 'MissingError'"
            && error.primary_range == Some(range_for(source, "MissingError"))
    }));
}

#[test]
fn invalid_except_type_form_has_result_code() {
    let source = "\
def main():
    try:
        pass
    except ValueError() as e:
        pass
";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RESULT_INVALID_EXCEPT_TYPE)
            && error.message
                == "invalid except error type: except type must be a simple error class name"
            && error.primary_range == Some(range_for(source, "ValueError()"))
    }));
}

#[test]
fn uncovered_try_errors_have_result_code() {
    let source = "\
def fallible() -> Result[int, ValueError]:
    raise ValueError(\"bad\")

def main():
    try:
        value: int = fallible()
    except IOError as e:
        pass
";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RESULT_UNCOVERED_TRY_ERRORS)
            && error.message == "except arms do not cover all error types from try body: ValueError"
            && error.primary_range.map(|range| range.start())
                == Some(range_for(source, "try:").start())
    }));
}

#[test]
fn ioerror_subclass_handlers_do_not_cover_the_open_base_error() {
    let source = "\
def fallible() -> Result[int, IOError]:
    raise IOError(\"other\")

def main():
    try:
        value: int = fallible()
    except FileNotFoundError:
        pass
    except PermissionError:
        pass
    except FileExistsError:
        pass
    except IsADirectoryError:
        pass
    except NotADirectoryError:
        pass
    except DirectoryNotEmptyError:
        pass
";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RESULT_UNCOVERED_TRY_ERRORS)
            && error.message == "except arms do not cover all error types from try body: IOError"
            && error.primary_range.map(|range| range.start())
                == Some(range_for(source, "try:").start())
    }));
}

#[test]
fn result_error_channel_accepts_an_unmatched_conditional_handler_error() {
    let source = "\
def classify() -> Result[str, IOError]:
    try:
        raise IOError(\"other\")
    except FileNotFoundError:
        return \"missing\"
";
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).expect("Result channel should accept the unmatched error");
}

#[test]
fn outer_try_accepts_an_unmatched_conditional_handler_error() {
    let source = "\
def classify() -> str:
    try:
        try:
            raise IOError(\"other\")
        except FileNotFoundError:
            return \"missing\"
    except IOError as error:
        return error.message
";
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).expect("outer try should accept the unmatched error");
}

#[test]
fn outer_try_does_not_accept_an_error_from_a_nested_function() {
    let source = "\
def outer() -> str:
    try:
        def inner() -> str:
            try:
                raise IOError(\"leak\")
            except FileNotFoundError:
                return \"missing\"
        return inner()
    except IOError as error:
        return error.message
";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RESULT_UNCOVERED_TRY_ERRORS)
            && error.message == "except arms do not cover all error types from try body: IOError"
    }));
}

#[test]
fn user_error_parent_handler_covers_its_child() {
    let source = r#"
class BaseError(Error):
    message: str

class ChildError(BaseError):
    def __init__(self, message: str) -> None:
        super().__init__(message)

def classify() -> str:
    try:
        raise ChildError("child")
    except BaseError as error:
        return error.message
"#;
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).expect("the parent handler should cover the child error");
}

#[test]
fn try_finally_without_except_preserves_cleanup_boundary() {
    let source = "\
def main():
    value: int = 1
    try:
        value = 2
    finally:
        value = 3
";
    let parsed = parse_module(source).expect("parse failed");
    let module = lower_module(parsed.suite()).expect("lowering should succeed");
    let body = &module.module.functions[0].body;

    assert_eq!(body.len(), 2);
    assert!(matches!(body[0], HirStmt::Let { .. }));
    let HirStmt::TryFinally {
        body: try_body,
        finalbody,
    } = &body[1]
    else {
        panic!("expected try/finally HIR node");
    };
    assert!(matches!(
        try_body.as_slice(),
        [HirStmt::Assign { name, .. }] if name == "value"
    ));
    assert!(matches!(
        finalbody.as_slice(),
        [HirStmt::Assign { name, .. }] if name == "value"
    ));
}

#[test]
fn successful_try_binding_is_visible_when_every_handler_exits() {
    let source = "\
def fallible() -> Result[str, ValueError]:
    return \"value\"

def main() -> Result[str, ValueError]:
    try:
        value: str = fallible()
    except ValueError as error:
        raise error
    return value
";
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).expect("successful try binding should remain visible");
}

#[test]
fn try_binding_is_undefined_when_a_handler_continues_without_it() {
    let source = "\
def fallible() -> Result[str, ValueError]:
    return \"value\"

def main() -> str:
    try:
        value: str = fallible()
    except ValueError as error:
        _ = error.message
    return value
";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::NAME_UNDEFINED_VARIABLE)
            && error.message == "undefined variable: 'value'"
    }));
}

#[test]
fn successful_try_binding_preserves_moved_state() {
    let source = "\
def fallible() -> Result[str, ValueError]:
    return \"value\"

def consume(own value: str) -> None:
    pass

def main() -> Result[str, ValueError]:
    try:
        value: str = fallible()
        consume(value)
    except ValueError as error:
        raise error
    return value
";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && error.message == "use of moved value: 'value'"
    }));
}

#[test]
fn moved_try_binding_rejects_value_dependent_subscript_assignment() {
    let source = "\
def fallible() -> Result[list[int], ValueError]:
    return [1]

def consume(own value: list[int]) -> None:
    pass

def main() -> Result[int, ValueError]:
    try:
        values: list[int] = fallible()
        consume(values)
    except ValueError as error:
        raise error
    values[0] = 2
    return 0
";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && error.message == "use of moved value: 'values'"
    }));
}
