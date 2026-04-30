use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(super) fn undefined_variable(ctx: &mut LowerCtx, name: &str) {
    ctx.error_with_code(
        DiagnosticCode::NAME_UNDEFINED_VARIABLE,
        format!("undefined variable: '{name}'"),
    );
}

pub(super) fn undefined_function(ctx: &mut LowerCtx, name: &str) {
    ctx.error_with_code(
        DiagnosticCode::NAME_UNDEFINED_CALLABLE,
        format!("undefined function: '{name}'"),
    );
}

pub(super) fn missing_member(ctx: &mut LowerCtx, container: &str, member: &str) {
    ctx.error_with_code(
        DiagnosticCode::NAME_MISSING_MODULE_MEMBER,
        format!("module '{container}' has no member '{member}'"),
    );
}
