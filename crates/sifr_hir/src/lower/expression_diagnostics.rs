use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(super) fn unsupported_form(ctx: &mut LowerCtx, form: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        format!("unsupported expression form: {form}"),
        range,
    );
}

pub(super) fn unsupported_operator(
    ctx: &mut LowerCtx,
    operator: &str,
    operand_types: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
        format!("unsupported operator {operator} for {operand_types}"),
        range,
    );
}

pub(super) fn matrix_multiplication(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
        "matrix multiplication operator (@) is not supported".to_string(),
        range,
    );
}
