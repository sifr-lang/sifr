use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(in crate::lower) fn unsupported_form(ctx: &mut LowerCtx, form: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        format!("unsupported expression form: {form}"),
        range,
    );
}

pub(in crate::lower) fn unsupported_operator(
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

pub(in crate::lower) fn matrix_multiplication(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
        "matrix multiplication operator (@) is not supported".to_string(),
        range,
    );
}

pub(in crate::lower) fn call_not_callable_or_arity(
    ctx: &mut LowerCtx,
    message: String,
    range: TextRange,
) {
    ctx.error_with_code_at(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY, message, range);
}

pub(in crate::lower) fn call_unexpected_keyword(
    ctx: &mut LowerCtx,
    message: String,
    range: TextRange,
) {
    ctx.error_with_code_at(DiagnosticCode::CALL_UNEXPECTED_KEYWORD, message, range);
}

pub(in crate::lower) fn call_wrong_positional_count(
    ctx: &mut LowerCtx,
    message: String,
    range: TextRange,
) {
    ctx.error_with_code_at(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT, message, range);
}

pub(in crate::lower) fn call_duplicate_argument(
    ctx: &mut LowerCtx,
    message: String,
    range: TextRange,
) {
    ctx.error_with_code_at(DiagnosticCode::CALL_DUPLICATE_ARGUMENT, message, range);
}

pub(in crate::lower) fn call_missing_required_argument(
    ctx: &mut LowerCtx,
    message: String,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT,
        message,
        range,
    );
}

pub(in crate::lower) fn type_mismatch(ctx: &mut LowerCtx, message: String, range: TextRange) {
    ctx.error_with_code_at(DiagnosticCode::TYPE_MISMATCH, message, range);
}
