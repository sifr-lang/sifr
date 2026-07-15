use super::{type_bounds::type_satisfies_bound, LowerCtx};
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

pub(in crate::lower) fn reject_unhashable_container_type(
    ctx: &mut LowerCtx,
    container_part: &str,
    ty: &Type,
    range: TextRange,
) -> bool {
    let resolved = ty.resolve_alias();
    if super::classes::is_hashable_type(ty)
        || matches!(resolved, Type::TypeVar(_)) && type_satisfies_bound(ty, "Hashable", ctx)
    {
        return false;
    }
    let (code, reason) = if matches!(resolved, Type::Any | Type::Unknown) {
        (
            DiagnosticCode::TYPE_MISMATCH,
            "does not have a statically known hash/equality capability",
        )
    } else if matches!(resolved, Type::TypeVar(_)) {
        (
            DiagnosticCode::TYPE_MISMATCH,
            "requires a Hashable bound before it can be used here",
        )
    } else if ty.contains_affine_resource() {
        (
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            "contains an affine Python buffer",
        )
    } else {
        (DiagnosticCode::TYPE_MISMATCH, "is not hashable")
    };
    ctx.error_with_code_at(
        code,
        format!("{container_part} type '{}' {reason}", ty.display_name()),
        range,
    );
    true
}
