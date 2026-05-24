use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(in crate::lower) fn forbidden_intrinsic(ctx: &mut LowerCtx, module: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC,
        format!("cannot import from '{module}' — _sifr.* modules are internal compiler intrinsics"),
        range,
    );
}

pub(in crate::lower) fn unknown_import_target(ctx: &mut LowerCtx, module: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE,
        format!("unknown import target: '{module}'"),
        range,
    );
}

pub(in crate::lower) fn deferred_compat_module(
    ctx: &mut LowerCtx,
    module: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE,
        format!("module '{module}' is intentionally deferred: {reason}"),
        range,
    );
}

pub(in crate::lower) fn unsupported_form(ctx: &mut LowerCtx, form: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_UNSUPPORTED_FORM,
        format!("unsupported import form: {form}"),
        range,
    );
}

pub(in crate::lower) fn private_member(
    ctx: &mut LowerCtx,
    module: &str,
    name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_PRIVATE_MEMBER,
        format!("cannot import private name '{name}' from module '{module}'"),
        range,
    );
}
