use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(super) fn undefined_variable(ctx: &mut LowerCtx, name: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_UNDEFINED_VARIABLE,
        format!("undefined variable: '{name}'"),
        range,
    );
}

pub(super) fn undefined_function(ctx: &mut LowerCtx, name: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_UNDEFINED_CALLABLE,
        format!("undefined function: '{name}'"),
        range,
    );
}

pub(super) fn missing_member(ctx: &mut LowerCtx, container: &str, member: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_MISSING_MODULE_MEMBER,
        format!("module '{container}' has no member '{member}'"),
        range,
    );
}
