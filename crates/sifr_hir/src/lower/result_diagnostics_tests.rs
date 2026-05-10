use crate::{lower_module, HirDiagnostic};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_source(source: &str) -> Result<(), Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|_| ())
}

fn range_for(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle missing from source");
    TextRange::new(
        TextSize::try_from(start).expect("start offset fits in TextSize"),
        TextSize::try_from(start + needle.len()).expect("end offset fits in TextSize"),
    )
}

fn range_for_after(source: &str, anchor: &str, needle: &str) -> TextRange {
    let anchor_end = source.find(anchor).expect("anchor missing from source") + anchor.len();
    let relative_start = source[anchor_end..]
        .find(needle)
        .expect("needle missing after anchor");
    let start = anchor_end + relative_start;
    TextRange::new(
        TextSize::try_from(start).expect("start offset fits in TextSize"),
        TextSize::try_from(start + needle.len()).expect("end offset fits in TextSize"),
    )
}

#[test]
fn bare_raise_has_result_invalid_raise_primary_range() {
    let source = "def main():\n    raise\n";
    let result = lower_source(source);
    let errors = result.expect_err("bare raise should fail lowering");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::RESULT_INVALID_RAISE)
                && error.primary_range == Some(range_for(source, "raise"))
        ),
        "expected SIFR-RESULT-0003 on bare raise range, got {errors:?}"
    );
}

#[test]
fn string_raise_has_result_invalid_raise_primary_range() {
    let source = "def main():\n    raise \"message\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("string raise should fail lowering");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::RESULT_INVALID_RAISE)
                && error.primary_range == Some(range_for(source, "\"message\""))
        ),
        "expected SIFR-RESULT-0003 on string expression range, got {errors:?}"
    );
}

#[test]
fn non_error_raise_has_result_invalid_raise_primary_range() {
    let source = "def main():\n    raise 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("non-error raise should fail lowering");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::RESULT_INVALID_RAISE)
                && error.primary_range == Some(range_for_after(source, "raise ", "1"))
        ),
        "expected SIFR-RESULT-0003 on non-error expression range, got {errors:?}"
    );
}

#[test]
fn unused_result_has_result_unused_value_primary_range() {
    let source = "\
def fallible() -> Result[int, ValueError]:
    return 42

def main():
    fallible()
";
    let result = lower_source(source);
    let errors = result.expect_err("unused Result expression should fail lowering");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::RESULT_UNUSED_VALUE)
                && error.primary_range
                    == Some(range_for_after(source, "def main():\n    ", "fallible()"))
        ),
        "expected SIFR-RESULT-0001 on unused call range, got {errors:?}"
    );
}

#[test]
fn invalid_result_error_type_has_primary_range() {
    let source = "\
def broken() -> Result[int, str]:
    return 1
";
    let result = lower_source(source);
    let errors = result.expect_err("invalid Result error type should fail lowering");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::RESULT_INVALID_ERROR_TYPE)
                && error.primary_range == Some(range_for_after(source, "Result[int, ", "str"))
        ),
        "expected SIFR-RESULT-0002 on invalid error type range, got {errors:?}"
    );
}

#[test]
fn failure_type_is_not_valid_result_error_channel() {
    let source = "\
def broken() -> Result[None, Failure[ValueError]]:
    return None
";
    let result = lower_source(source);
    let errors = result.expect_err("Failure should not be a Result error type");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::RESULT_INVALID_ERROR_TYPE)
                && error.primary_range
                    == Some(range_for_after(
                        source,
                        "Result[None, ",
                        "Failure[ValueError]"
                    ))
        ),
        "expected SIFR-RESULT-0002 on Failure error channel range, got {errors:?}"
    );
}

#[test]
fn timeout_result_type_is_valid_result_error_channel_when_inner_error_is_valid() {
    let source = "\
def ok() -> Result[None, TimeoutResult[ValueError]]:
    return None
";
    let result = lower_source(source);

    assert!(
        result.is_ok(),
        "TimeoutResult[E] should be valid when E is a valid Error: {result:?}"
    );
}
