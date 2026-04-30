use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(super) fn forbidden_intrinsic(ctx: &mut LowerCtx, module: &str) {
    ctx.error_with_code(
        DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC,
        format!("cannot import from '{module}' — _sifr.* modules are internal compiler intrinsics"),
    );
}

pub(super) fn unknown_import_target(ctx: &mut LowerCtx, module: &str) {
    ctx.error_with_code(
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE,
        format!("unknown import target: '{module}'"),
    );
}
