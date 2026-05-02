use super::LowerCtx;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::TypeCheckDiagnostic;

#[test]
fn error_with_code_records_structured_identity() {
    let mut ctx = LowerCtx::new();

    ctx.error_with_code(
        DiagnosticCode::TYPE_MISMATCH,
        "expected int, got str".to_string(),
    );

    assert_eq!(ctx.errors.len(), 1);
    assert_eq!(ctx.errors[0].code, Some(DiagnosticCode::TYPE_MISMATCH));
    assert_eq!(ctx.errors[0].message, "expected int, got str");
}

#[test]
fn legacy_error_records_no_structured_identity() {
    let mut ctx = LowerCtx::new();

    ctx.error("expected int, got str".to_string());

    assert_eq!(ctx.errors.len(), 1);
    assert_eq!(ctx.errors[0].code, None);
    assert_eq!(ctx.errors[0].message, "expected int, got str");
}

#[test]
fn type_check_diagnostic_records_structured_identity() {
    let mut ctx = LowerCtx::new();

    ctx.type_check_diagnostic(TypeCheckDiagnostic {
        code: DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
        message: "unsupported operand type(s) for +: Option[int] and int".to_string(),
    });

    assert_eq!(ctx.errors.len(), 1);
    assert_eq!(
        ctx.errors[0].code,
        Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
    );
    assert_eq!(
        ctx.errors[0].message,
        "unsupported operand type(s) for +: Option[int] and int"
    );
}
