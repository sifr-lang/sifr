use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(in crate::lower) fn undefined_variable(ctx: &mut LowerCtx, name: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_UNDEFINED_VARIABLE,
        format!("undefined variable: '{name}'"),
        range,
    );
}

pub(in crate::lower) fn undefined_function(ctx: &mut LowerCtx, name: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_UNDEFINED_CALLABLE,
        format!("undefined function: '{name}'"),
        range,
    );
}

pub(in crate::lower) fn missing_member(
    ctx: &mut LowerCtx,
    container: &str,
    member: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_MISSING_MODULE_MEMBER,
        format!("module '{container}' has no member '{member}'"),
        range,
    );
}

pub(in crate::lower) fn deferred_compat_member(
    ctx: &mut LowerCtx,
    container: &str,
    member: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_MISSING_MODULE_MEMBER,
        format!("'{container}.{member}' is intentionally deferred: {reason}"),
        range,
    );
}

pub(in crate::lower) fn uninitialized_variable(ctx: &mut LowerCtx, name: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_UNINITIALIZED_VARIABLE,
        format!("variable '{name}' must be initialized"),
        range,
    );
}
