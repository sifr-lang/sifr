use crate::{render_codegen_error, run_codegen_with_boundary};
use sifr_diagnostics::DiagnosticCode;

#[test]
fn test_run_codegen_with_boundary_reports_string_panic_as_internal_compiler_panic() {
    let err = run_codegen_with_boundary("panic boundary test", || {
        panic!("boom");
    })
    .expect_err("panic should be converted into an internal compiler panic");
    assert_eq!(err.code, DiagnosticCode::INTERNAL_COMPILER_PANIC.code());
    assert!(err.message.contains("panic boundary test: boom"));
}

#[test]
fn test_run_codegen_with_boundary_reports_non_string_payload_as_internal_compiler_panic() {
    let err = run_codegen_with_boundary("panic boundary test", || {
        std::panic::panic_any(42_u8);
    })
    .expect_err("panic should be converted into an internal compiler panic");
    assert_eq!(err.code, DiagnosticCode::INTERNAL_COMPILER_PANIC.code());
    assert!(
        err.message
            .contains("panic boundary test: non-string panic payload")
    );
}

#[test]
fn structured_codegen_failure_uses_internal_0003() {
    let diagnostic = render_codegen_error(&sifr_codegen::CodegenError::new(
        "assembled Rust source was invalid",
    ));

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::INTERNAL_CODEGEN_FAILURE.code()
    );
    assert!(
        diagnostic
            .message
            .contains("assembled Rust source was invalid")
    );
    assert_ne!(
        diagnostic.code,
        DiagnosticCode::INTERNAL_COMPILER_PANIC.code()
    );
}
