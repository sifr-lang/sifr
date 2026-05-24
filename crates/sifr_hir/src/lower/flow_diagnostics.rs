use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(in crate::lower) fn break_outside_loop(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP,
        "'break' outside of loop".to_string(),
        range,
    );
}

pub(in crate::lower) fn continue_outside_loop(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_CONTINUE_OUTSIDE_LOOP,
        "'continue' outside of loop".to_string(),
        range,
    );
}

pub(in crate::lower) fn invalid_nonlocal_at(ctx: &mut LowerCtx, message: String, range: TextRange) {
    ctx.error_with_code_at(DiagnosticCode::FLOW_INVALID_NONLOCAL, message, range);
}

pub(in crate::lower) fn missing_return_value(
    ctx: &mut LowerCtx,
    function_name: &str,
    return_type: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_MISSING_RETURN_VALUE,
        format!(
            "function '{function_name}' must return a value of type '{return_type}' on all control-flow paths"
        ),
        range,
    );
}

pub(in crate::lower) fn invalid_condition_type(
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

pub(in crate::lower) fn nonlocal_requires_enclosing_binding(ctx: &mut LowerCtx, range: TextRange) {
    invalid_nonlocal_at(
        ctx,
        "nonlocal declaration requires an enclosing function binding".to_string(),
        range,
    );
}

pub(in crate::lower) fn nonlocal_conflicts_with_current_binding(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    invalid_nonlocal_at(
        ctx,
        format!("nonlocal name '{name}' conflicts with a binding in the current function scope"),
        range,
    );
}

pub(in crate::lower) fn nonlocal_missing_enclosing_binding(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    invalid_nonlocal_at(
        ctx,
        format!("nonlocal name '{name}' does not resolve to an enclosing function binding"),
        range,
    );
}

pub(in crate::lower) fn captured_augassign_requires_nonlocal(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    invalid_nonlocal_at(
        ctx,
        format!(
            "captured variable `{name}` must be declared with `nonlocal` before augmented assignment"
        ),
        range,
    );
}

pub(in crate::lower) fn tuple_unpack_nonlocal_rebind(ctx: &mut LowerCtx, range: TextRange) {
    invalid_nonlocal_at(
        ctx,
        "tuple unpacking cannot rebind captured state with `nonlocal` yet".to_string(),
        range,
    );
}

pub(in crate::lower) fn recursive_nonlocal_nested_function(
    ctx: &mut LowerCtx,
    function_name: &str,
    range: TextRange,
) {
    invalid_nonlocal_at(
        ctx,
        format!(
            "recursive nested function '{function_name}' cannot mutate captured state with `nonlocal` yet"
        ),
        range,
    );
}
