use super::LowerCtx;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

pub(in crate::lower) fn invalid_raise_string(ctx: &mut LowerCtx, range: TextRange) {
    invalid_raise(
        ctx,
        "raise requires an Error class instance — `raise \"message\"` is not allowed, use e.g. `raise ValueError(\"message\")`".to_string(),
        range,
    );
}

pub(in crate::lower) fn invalid_raise_non_error(
    ctx: &mut LowerCtx,
    type_name: &str,
    range: TextRange,
) {
    invalid_raise(
        ctx,
        format!("raise requires an Error class instance — `{type_name}` is not an Error class"),
        range,
    );
}

pub(in crate::lower) fn invalid_bare_raise(ctx: &mut LowerCtx, range: TextRange) {
    invalid_raise(
        ctx,
        "bare 'raise' without an expression is not supported".to_string(),
        range,
    );
}

pub(in crate::lower) fn invalid_raise_for_return_type(
    ctx: &mut LowerCtx,
    raised_type: &str,
    return_type: &str,
    range: TextRange,
) {
    invalid_raise(
        ctx,
        format!(
            "unhandled raise of '{raised_type}' requires a compatible Result error channel; function returns '{return_type}'"
        ),
        range,
    );
}

fn invalid_raise(ctx: &mut LowerCtx, message: String, range: TextRange) {
    ctx.error_with_code_at(DiagnosticCode::RESULT_INVALID_RAISE, message, range);
}

pub(in crate::lower) fn unknown_except_type(
    ctx: &mut LowerCtx,
    error_type: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::RESULT_UNKNOWN_EXCEPT_TYPE,
        format!("unknown except error type '{error_type}'"),
        range,
    );
}

pub(in crate::lower) fn uncovered_try_errors(
    ctx: &mut LowerCtx,
    uncovered: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::RESULT_UNCOVERED_TRY_ERRORS,
        format!("except arms do not cover all error types from try body: {uncovered}"),
        range,
    );
}

pub(in crate::lower) fn invalid_except_type(ctx: &mut LowerCtx, reason: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::RESULT_INVALID_EXCEPT_TYPE,
        format!("invalid except error type: {reason}"),
        range,
    );
}

pub(in crate::lower) fn unhandled_checked_place_error(
    ctx: &mut LowerCtx,
    operation: &str,
    error_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::RESULT_UNUSED_VALUE,
        format!("{operation} may fail with '{error_name}'; handle it inside try/except"),
        range,
    );
}
