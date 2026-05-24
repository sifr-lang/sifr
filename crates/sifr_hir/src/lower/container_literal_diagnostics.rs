use super::LowerCtx;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

pub(in crate::lower) fn container_literal_type_conflict(
    ctx: &mut LowerCtx,
    element_kind: &str,
    expected: &Type,
    actual: &Type,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT,
        format!(
            "container literal has conflicting {element_kind} types: {} and {}",
            expected.display_name(),
            actual.display_name()
        ),
        range,
    );
}
