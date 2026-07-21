use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

use super::LowerCtx;

pub(in crate::lower) fn unsupported_form(ctx: &mut LowerCtx, form: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_UNSUPPORTED_STATEMENT_FORM,
        format!("unsupported statement form: {form}"),
        range,
    );
}

pub(in crate::lower) fn invalid_assignment_target(
    ctx: &mut LowerCtx,
    target: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET,
        format!("invalid assignment target: {target}"),
        range,
    );
}

pub(in crate::lower) fn invalid_iteration(ctx: &mut LowerCtx, reason: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_ITERATION,
        format!("invalid for-loop iteration: {reason}"),
        range,
    );
}

pub(in crate::lower) fn reject_affine_iteration(
    ctx: &mut LowerCtx,
    element_type: &Type,
    range: TextRange,
) -> bool {
    if !element_type.contains_affine_resource() {
        return false;
    }
    invalid_iteration(
        ctx,
        "cannot iterate over affine Python resource elements because iteration projects values from an aggregate",
        range,
    );
    true
}

pub(in crate::lower) fn reject_affine_iterator_builtin(
    ctx: &mut LowerCtx,
    builtin: &str,
    element_type: &Type,
    range: TextRange,
) -> bool {
    if !element_type.supports_derived_clone() && !element_type.contains_affine_resource() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!("{builtin}() requires a statically known element clone/comparison capability"),
            range,
        );
        return true;
    }
    if !element_type.contains_affine_resource() {
        return false;
    }
    ctx.error_with_code_at(
        DiagnosticCode::PYZC_INVALID_DECLARATION,
        format!(
            "{builtin}() cannot project elements containing an affine Python resource from an iterable"
        ),
        range,
    );
    true
}

pub(in crate::lower) fn reject_affine_comprehension_value(
    ctx: &mut LowerCtx,
    value_type: &Type,
    range: TextRange,
) -> bool {
    if !value_type.contains_affine_resource() {
        return false;
    }
    ctx.error_with_code_at(
        DiagnosticCode::PYZC_INVALID_DECLARATION,
        "cannot produce a value containing an affine Python resource from a comprehension or generator because its body may execute repeatedly"
            .to_string(),
        range,
    );
    true
}

pub(in crate::lower) fn mutation_during_iteration(
    ctx: &mut LowerCtx,
    source_name: &str,
    range: TextRange,
) {
    invalid_iteration(
        ctx,
        &format!("cannot mutate '{source_name}' while iterating over it in a for loop"),
        range,
    );
}
