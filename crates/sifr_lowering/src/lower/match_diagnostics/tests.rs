use super::*;
use crate::{HirDiagnostic, lower_module};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

pub(super) fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("parse failed");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    }
}

pub(super) fn range_for_after(source: &str, after: &str, needle: &str) -> TextRange {
    let search_start = source.find(after).expect("anchor should exist") + after.len();
    let relative_start = source[search_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = u32::try_from(search_start + relative_start).expect("fixture offset must fit u32");
    let needle_len = u32::try_from(needle.len()).expect("fixture length must fit u32");
    TextRange::new(TextSize::new(start), TextSize::new(start + needle_len))
}

#[test]
pub(super) fn match_guard_type_error_has_match_code() {
    let source = "def classify(x: int) -> str:\n    match x:\n        case n if n + 1:\n            return \"truthy\"\n        case _:\n            return \"other\"\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "match guard must be a bool expression, got 'int'"
            && error.code == Some(DiagnosticCode::MATCH_GUARD_NOT_BOOL)
            && error.primary_range == Some(range_for_after(source, "case n if ", "n + 1"))
    }));
}

#[test]
pub(super) fn enum_non_exhaustive_match_has_match_code() {
    let source = "from enum import Enum\n\nclass Color(Enum):\n    RED = 1\n    GREEN = 2\n    BLUE = 3\n\ndef describe(c: Color) -> str:\n    match c:\n        case Color.RED:\n            return \"red\"\n        case Color.GREEN:\n            return \"green\"\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "non-exhaustive match: enum 'Color' has uncovered variants: BLUE — add matching case(s) or `case _:`"
            && error.code == Some(DiagnosticCode::MATCH_NON_EXHAUSTIVE)
            && error.primary_range == Some(range_for_after(source, "match ", "c"))
    }));
}

#[test]
pub(super) fn union_non_exhaustive_match_has_match_code() {
    let source = "def describe(x: int | None) -> str:\n    match x:\n        case int():\n            return \"integer\"\n";
    let errors = lower_errors(source);

    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
    assert!(errors.iter().any(|error| {
        error.message
            == "non-exhaustive match: type 'None | int' has uncovered variants: None — add matching case(s) or `case _:`"
            && error.code == Some(DiagnosticCode::MATCH_NON_EXHAUSTIVE)
            && error.primary_range == Some(range_for_after(source, "match ", "x"))
    }));
}

#[test]
pub(super) fn optional_non_exhaustive_match_has_only_match_code() {
    let source = "def describe(x: str | None) -> str:\n    match x:\n        case str():\n            return \"has value\"\n";
    let errors = lower_errors(source);

    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
    assert_eq!(errors[0].code, Some(DiagnosticCode::MATCH_NON_EXHAUSTIVE));
    assert_eq!(
        errors[0].primary_range,
        Some(range_for_after(source, "match ", "x"))
    );
}

#[test]
pub(super) fn literal_non_exhaustive_match_has_match_code() {
    let source = "def describe(x: int) -> str:\n    match x:\n        case 1:\n            return \"one\"\n        case 2:\n            return \"two\"\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "non-exhaustive match: type 'int' cannot be fully covered by literal patterns — add `case _:` to handle remaining values"
            && error.code == Some(DiagnosticCode::MATCH_NON_EXHAUSTIVE)
            && error.primary_range == Some(range_for_after(source, "match ", "x"))
    }));
}

#[test]
pub(super) fn invalid_class_pattern_field_has_match_code() {
    let source = "class Point:\n    x: int\n    y: int\n\ndef classify(p: Point) -> str:\n    match p:\n        case Point(x=px, z=pz):\n            return \"has z\"\n        case _:\n            return \"other\"\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "class 'Point' has no field 'z' — available fields: x, y"
            && error.code == Some(DiagnosticCode::MATCH_INVALID_CLASS_PATTERN_FIELD)
            && error.primary_range == Some(range_for_after(source, "x=px, ", "z"))
    }));
}
