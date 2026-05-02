use crate::{lower_module, LoweringError};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_source(source: &str) -> Result<(), Vec<LoweringError>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|_| ())
}

#[test]
fn bare_raise_has_result_invalid_raise_code() {
    let result = lower_source("def main():\n    raise\n");
    let errors = result.expect_err("bare raise should fail lowering");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::RESULT_INVALID_RAISE)),
        "expected SIFR-RESULT-0003 for bare raise, got {errors:?}"
    );
}
