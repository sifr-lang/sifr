use super::LowerCtx;
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;

#[test]
fn error_with_code_at_records_primary_range() {
    let mut ctx = LowerCtx::new();
    let range = TextRange::new(TextSize::new(4), TextSize::new(7));

    ctx.error_with_code_at(
        DiagnosticCode::TYPE_MISMATCH,
        "expected int, got str".to_string(),
        range,
    );

    assert_eq!(ctx.errors.len(), 1);
    assert_eq!(ctx.errors[0].code, Some(DiagnosticCode::TYPE_MISMATCH));
    assert_eq!(ctx.errors[0].message, "expected int, got str");
    assert_eq!(ctx.errors[0].primary_range, Some(range));
}
