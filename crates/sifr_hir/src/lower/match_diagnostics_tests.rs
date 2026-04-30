use crate::{lower_module, LoweringError};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_errors(source: &str) -> Vec<LoweringError> {
    let parsed = parse_module(source).expect("parse failed");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    }
}

#[test]
fn match_guard_type_error_has_match_code() {
    let errors = lower_errors(
        "def classify(x: int) -> str:\n    match x:\n        case n if n + 1:\n            return \"truthy\"\n        case _:\n            return \"other\"\n",
    );

    assert!(errors.iter().any(|error| {
        error.message == "match guard must be a bool expression, got 'int'"
            && error.code == Some(DiagnosticCode::MATCH_GUARD_NOT_BOOL)
    }));
}

#[test]
fn enum_non_exhaustive_match_has_match_code() {
    let errors = lower_errors(
        "from enum import Enum\n\nclass Color(Enum):\n    RED = 1\n    GREEN = 2\n    BLUE = 3\n\ndef describe(c: Color) -> str:\n    match c:\n        case Color.RED:\n            return \"red\"\n        case Color.GREEN:\n            return \"green\"\n",
    );

    assert!(errors.iter().any(|error| {
        error.message
            == "non-exhaustive match: enum 'Color' has uncovered variants: BLUE — add matching case(s) or `case _:`"
            && error.code == Some(DiagnosticCode::MATCH_NON_EXHAUSTIVE)
    }));
}

#[test]
fn union_non_exhaustive_match_has_match_code() {
    let errors = lower_errors(
        "def describe(x: int | None) -> str:\n    match x:\n        case int():\n            return \"integer\"\n",
    );

    assert!(errors.iter().any(|error| {
        error.message
            == "non-exhaustive match: type 'None | int' has uncovered variants: None — add matching case(s) or `case _:`"
            && error.code == Some(DiagnosticCode::MATCH_NON_EXHAUSTIVE)
    }));
}

#[test]
fn literal_non_exhaustive_match_has_match_code() {
    let errors = lower_errors(
        "def describe(x: int) -> str:\n    match x:\n        case 1:\n            return \"one\"\n        case 2:\n            return \"two\"\n",
    );

    assert!(errors.iter().any(|error| {
        error.message
            == "non-exhaustive match: type 'int' cannot be fully covered by literal patterns — add `case _:` to handle remaining values"
            && error.code == Some(DiagnosticCode::MATCH_NON_EXHAUSTIVE)
    }));
}

#[test]
fn invalid_class_pattern_field_has_match_code() {
    let errors = lower_errors(
        "class Point:\n    x: int\n    y: int\n\ndef classify(p: Point) -> str:\n    match p:\n        case Point(x=px, z=pz):\n            return \"has z\"\n        case _:\n            return \"other\"\n",
    );

    assert!(errors.iter().any(|error| {
        error.message == "class 'Point' has no field 'z' — available fields: x, y"
            && error.code == Some(DiagnosticCode::MATCH_INVALID_CLASS_PATTERN_FIELD)
    }));
}
