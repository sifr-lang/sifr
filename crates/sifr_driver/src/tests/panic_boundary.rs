use crate::{run_codegen_with_boundary, CompilePhase};

#[test]
fn test_run_codegen_with_boundary_reports_string_panic_as_codegen_error() {
    let err = run_codegen_with_boundary("panic boundary test", || {
        panic!("boom");
    })
    .expect_err("panic should be converted into a codegen error");
    assert!(matches!(err.phase, CompilePhase::Codegen));
    assert!(err.message.contains("panic boundary test: boom"));
}

#[test]
fn test_run_codegen_with_boundary_reports_non_string_payload() {
    let err = run_codegen_with_boundary("panic boundary test", || {
        std::panic::panic_any(42_u8);
    })
    .expect_err("panic should be converted into a codegen error");
    assert!(matches!(err.phase, CompilePhase::Codegen));
    assert!(err
        .message
        .contains("panic boundary test: non-string panic payload"));
}
