use super::LowerCtx;
use sifr_diagnostics::DiagnosticCode;

pub(super) fn use_after_move(ctx: &mut LowerCtx, name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_USE_AFTER_MOVE,
        format!("use of moved value: '{name}'"),
    );
}

pub(super) fn double_mutable_borrow(ctx: &mut LowerCtx, name: &str, func_name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
        format!(
            "cannot borrow '{name}' as mutable more than once in the same call to '{func_name}'"
        ),
    );
}

pub(super) fn mutable_borrow_after_immutable(ctx: &mut LowerCtx, name: &str, func_name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
        format!(
            "cannot borrow '{name}' as mutable because it is already borrowed as immutable in the same call to '{func_name}'"
        ),
    );
}

pub(super) fn immutable_borrow_after_mutable(ctx: &mut LowerCtx, name: &str, func_name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
        format!(
            "cannot borrow '{name}' as immutable because it is already borrowed as mutable in the same call to '{func_name}'"
        ),
    );
}

pub(super) fn borrowed_parameter_store_escape(ctx: &mut LowerCtx, name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
        format!(
            "cannot store borrowed parameter `{name}`: borrowed parameters cannot escape -- add `own` at the signature boundary or store `{name}.clone()`"
        ),
    );
}

pub(super) fn borrowed_parameter_return_escape(ctx: &mut LowerCtx, name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
        format!(
            "cannot return borrowed parameter `{name}`: borrowed parameters cannot escape -- add `own` at the signature boundary or return `{name}.clone()`"
        ),
    );
}

pub(super) fn moved_across_loop(ctx: &mut LowerCtx, name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_MOVED_ACROSS_LOOP,
        format!(
            "value '{name}' is moved inside loop body; it would be unavailable on subsequent iterations"
        ),
    );
}

pub(super) fn immutable_parameter_mutation(ctx: &mut LowerCtx, name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION,
        format!("cannot mutate through immutable parameter `{name}`: add `mut` to the parameter declaration"),
    );
}

pub(super) fn immutable_parameter_reassignment(ctx: &mut LowerCtx, name: &str) {
    ctx.error_with_code(
        DiagnosticCode::OWN_IMMUTABLE_PARAMETER_REASSIGNMENT,
        format!(
            "cannot reassign immutable parameter `{name}`: add `mut` to the parameter declaration"
        ),
    );
}
