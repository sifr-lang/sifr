use super::LowerCtx;
use sifr_diagnostics::DiagnosticCode;

pub(super) fn invalid_raise_string(ctx: &mut LowerCtx) {
    invalid_raise(
        ctx,
        "raise requires an Error class instance — `raise \"message\"` is not allowed, use e.g. `raise ValueError(\"message\")`".to_string(),
    );
}

pub(super) fn invalid_raise_non_error(ctx: &mut LowerCtx, type_name: &str) {
    invalid_raise(
        ctx,
        format!("raise requires an Error class instance — `{type_name}` is not an Error class"),
    );
}

pub(super) fn invalid_bare_raise(ctx: &mut LowerCtx) {
    invalid_raise(
        ctx,
        "bare 'raise' without an expression is not supported".to_string(),
    );
}

fn invalid_raise(ctx: &mut LowerCtx, message: String) {
    ctx.error_with_code(DiagnosticCode::RESULT_INVALID_RAISE, message);
}
