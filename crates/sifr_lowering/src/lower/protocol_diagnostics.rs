use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(in crate::lower) fn bound_not_satisfied(
    ctx: &mut LowerCtx,
    actual: &str,
    protocol: &str,
    type_param: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_BOUND_NOT_SATISFIED,
        format!(
            "type '{actual}' does not implement protocol '{protocol}' required by type parameter '{type_param}'"
        ),
        range,
    );
}

pub(in crate::lower) fn context_manager_missing(
    ctx: &mut LowerCtx,
    type_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING,
        format!(
            "type '{type_name}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"
        ),
        range,
    );
}

pub(in crate::lower) fn context_manager_incomplete(
    ctx: &mut LowerCtx,
    type_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING,
        format!(
            "type '{type_name}' used in 'with' statement must implement both __enter__ and __exit__ methods"
        ),
        range,
    );
}

pub(in crate::lower) fn iterator_invalid_return_signature(
    ctx: &mut LowerCtx,
    type_name: &str,
    expected: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
        format!("class '{type_name}' must return {expected}"),
        range,
    );
}

pub(in crate::lower) fn iterator_invalid_parameter_signature(
    ctx: &mut LowerCtx,
    type_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
        format!("class '{type_name}' must not declare parameters besides self"),
        range,
    );
}

pub(in crate::lower) fn iterator_element_mismatch(
    ctx: &mut LowerCtx,
    class_name: &str,
    left_method: &str,
    left_type: &str,
    right_method: &str,
    right_type: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
        format!(
            "class '{class_name}' iteration protocol mismatch: '{left_method}' yields '{left_type}' but '{right_method}' yields '{right_type}'"
        ),
        range,
    );
}

#[cfg(test)]
#[path = "../../tests/unit/lower/protocol_diagnostics/tests.rs"]
mod tests;
