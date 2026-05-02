use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(super) fn forbidden_intrinsic(ctx: &mut LowerCtx, module: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC,
        format!("cannot import from '{module}' — _sifr.* modules are internal compiler intrinsics"),
        range,
    );
}

pub(super) fn unknown_import_target(ctx: &mut LowerCtx, module: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE,
        format!("unknown import target: '{module}'"),
        range,
    );
}
