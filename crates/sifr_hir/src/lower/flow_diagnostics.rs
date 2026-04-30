use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(super) fn break_outside_loop(ctx: &mut LowerCtx) {
    ctx.error_with_code(
        DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP,
        "'break' outside of loop".to_string(),
    );
}

pub(super) fn continue_outside_loop(ctx: &mut LowerCtx) {
    ctx.error_with_code(
        DiagnosticCode::FLOW_CONTINUE_OUTSIDE_LOOP,
        "'continue' outside of loop".to_string(),
    );
}

pub(super) fn invalid_nonlocal(ctx: &mut LowerCtx, message: String) {
    ctx.error_with_code(DiagnosticCode::FLOW_INVALID_NONLOCAL, message);
}

pub(super) fn nonlocal_requires_enclosing_binding(ctx: &mut LowerCtx) {
    invalid_nonlocal(
        ctx,
        "nonlocal declaration requires an enclosing function binding".to_string(),
    );
}

pub(super) fn nonlocal_conflicts_with_current_binding(ctx: &mut LowerCtx, name: &str) {
    invalid_nonlocal(
        ctx,
        format!("nonlocal name '{name}' conflicts with a binding in the current function scope"),
    );
}

pub(super) fn nonlocal_missing_enclosing_binding(ctx: &mut LowerCtx, name: &str) {
    invalid_nonlocal(
        ctx,
        format!("nonlocal name '{name}' does not resolve to an enclosing function binding"),
    );
}

pub(super) fn captured_augassign_requires_nonlocal(ctx: &mut LowerCtx, name: &str) {
    invalid_nonlocal(
        ctx,
        format!(
            "captured variable `{name}` must be declared with `nonlocal` before augmented assignment"
        ),
    );
}

pub(super) fn tuple_unpack_nonlocal_rebind(ctx: &mut LowerCtx) {
    invalid_nonlocal(
        ctx,
        "tuple unpacking cannot rebind captured state with `nonlocal` yet".to_string(),
    );
}

pub(super) fn recursive_nonlocal_nested_function(ctx: &mut LowerCtx, function_name: &str) {
    invalid_nonlocal(
        ctx,
        format!(
            "recursive nested function '{function_name}' cannot mutate captured state with `nonlocal` yet"
        ),
    );
}
