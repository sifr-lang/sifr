use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

use super::builtin_calls::{callable_builtin_dict_output_type, callable_builtin_element_type};
use super::LowerCtx;

pub(in crate::lower) fn validate_list_extend_arg(
    list_elem_ty: &Type,
    iterable_ty: &Type,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(iterable_elem_ty) = callable_builtin_element_type(iterable_ty) else {
        ctx.error_with_code_at(
            DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
            format!(
                "list.extend() argument must be an iterable with a statically-known element type, got '{}'",
                iterable_ty.display_name()
            ),
            range,
        );
        return false;
    };
    if !iterable_elem_ty.is_assignable_to(list_elem_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "list.extend() iterable element type '{}' is not compatible with list element type '{}'",
                iterable_elem_ty.display_name(),
                list_elem_ty.display_name()
            ),
            range,
        );
        return false;
    }
    true
}

pub(in crate::lower) fn validate_dict_update_arg(
    key_ty: &Type,
    value_ty: &Type,
    update_ty: &Type,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(Type::Dict(update_key_ty, update_value_ty)) =
        callable_builtin_dict_output_type(update_ty)
    else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "dict.update() argument must be a dict or iterable of key/value tuples, got '{}'",
                update_ty.display_name()
            ),
            range,
        );
        return false;
    };
    let mut valid = true;
    if !update_key_ty.is_assignable_to(key_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "dict.update() key type '{}' is not compatible with dict key type '{}'",
                update_key_ty.display_name(),
                key_ty.display_name()
            ),
            range,
        );
        valid = false;
    }
    if !update_value_ty.is_assignable_to(value_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "dict.update() value type '{}' is not compatible with dict value type '{}'",
                update_value_ty.display_name(),
                value_ty.display_name()
            ),
            range,
        );
        valid = false;
    }
    valid
}

pub(in crate::lower) fn validate_set_iterable_arg(
    set_elem_ty: &Type,
    iterable_ty: &Type,
    method: &str,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(iterable_elem_ty) = callable_builtin_element_type(iterable_ty) else {
        ctx.error_with_code_at(
            DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
            format!(
                "set.{method}() arguments must be iterables with a statically-known element type, got '{}'",
                iterable_ty.display_name()
            ),
            range,
        );
        return false;
    };
    if !iterable_elem_ty.is_assignable_to(set_elem_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "set.{method}() iterable element type '{}' is not compatible with set element type '{}'",
                iterable_elem_ty.display_name(),
                set_elem_ty.display_name()
            ),
            range,
        );
        return false;
    }
    true
}
