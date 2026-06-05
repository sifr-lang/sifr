use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

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
