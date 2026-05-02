use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(super) fn break_outside_loop(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP,
        "'break' outside of loop".to_string(),
        range,
    );
}

pub(super) fn continue_outside_loop(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_CONTINUE_OUTSIDE_LOOP,
        "'continue' outside of loop".to_string(),
        range,
    );
}

pub(super) fn invalid_nonlocal(ctx: &mut LowerCtx, message: String) {
    ctx.error_with_code(DiagnosticCode::FLOW_INVALID_NONLOCAL, message);
}

pub(super) fn missing_return_value(ctx: &mut LowerCtx, function_name: &str, return_type: &str) {
    ctx.error_with_code(
        DiagnosticCode::FLOW_MISSING_RETURN_VALUE,
        format!(
            "function '{function_name}' must return a value of type '{return_type}' on all control-flow paths"
        ),
    );
}

pub(super) fn invalid_condition_type(ctx: &mut LowerCtx, keyword: &str, actual: &str) {
    ctx.error_with_code(
        DiagnosticCode::FLOW_INVALID_CONDITION_TYPE,
        format!("{keyword} condition must be bool or collection/string truthiness, got '{actual}'"),
    );
}

pub(super) fn invalid_condition_type_at(
    ctx: &mut LowerCtx,
    keyword: &str,
    actual: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_CONDITION_TYPE,
        format!("{keyword} condition must be bool or collection/string truthiness, got '{actual}'"),
        range,
    );
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
