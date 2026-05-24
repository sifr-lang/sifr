use super::str;
use ruff_text_size::TextRange;

use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(in crate::lower) fn guard_not_bool(ctx: &mut LowerCtx, actual: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::MATCH_GUARD_NOT_BOOL,
        format!("match guard must be a bool expression, got '{actual}'"),
        range,
    );
}

pub(in crate::lower) fn non_exhaustive_union(
    ctx: &mut LowerCtx,
    subject_type: &str,
    uncovered: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::MATCH_NON_EXHAUSTIVE,
        format!(
            "non-exhaustive match: type '{subject_type}' has uncovered variants: {uncovered} — add matching case(s) or `case _:`"
        ),
        range,
    );
}

pub(in crate::lower) fn non_exhaustive_enum(
    ctx: &mut LowerCtx,
    enum_name: &str,
    uncovered: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::MATCH_NON_EXHAUSTIVE,
        format!(
            "non-exhaustive match: enum '{enum_name}' has uncovered variants: {uncovered} — add matching case(s) or `case _:`"
        ),
        range,
    );
}

pub(in crate::lower) fn non_exhaustive_literal(
    ctx: &mut LowerCtx,
    subject_type: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::MATCH_NON_EXHAUSTIVE,
        format!(
            "non-exhaustive match: type '{subject_type}' cannot be fully covered by literal patterns — add `case _:` to handle remaining values"
        ),
        range,
    );
}

pub(in crate::lower) fn invalid_class_pattern_field(
    ctx: &mut LowerCtx,
    class_name: &str,
    field_name: &str,
    available_fields: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::MATCH_INVALID_CLASS_PATTERN_FIELD,
        format!(
            "class '{class_name}' has no field '{field_name}' — available fields: {available_fields}"
        ),
        range,
    );
}

pub(in crate::lower) fn invalid_pattern_form(ctx: &mut LowerCtx, reason: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::MATCH_INVALID_PATTERN_FORM,
        format!("invalid match pattern: {reason}"),
        range,
    );
}

#[cfg(test)]
mod tests;
