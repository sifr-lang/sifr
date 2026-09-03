use crate::RustExpr;
use sifr_type_system::Type;

pub(crate) struct LoweredMethod {
    pub(crate) expr: RustExpr,
}

pub(crate) fn is_in_place_collection_method(object_ty: &Type, method: &str) -> bool {
    match object_ty.resolve_alias() {
        Type::List(_) => matches!(
            method,
            "append" | "extend" | "insert" | "clear" | "reverse" | "sort" | "pop" | "remove"
        ),
        Type::Set(_) => matches!(
            method,
            "add"
                | "update"
                | "intersection_update"
                | "difference_update"
                | "symmetric_difference_update"
                | "remove"
                | "discard"
                | "clear"
                | "pop"
        ),
        _ => false,
    }
}

pub(crate) fn lower_method(
    object_ty: &Type,
    method: &str,
    object: &RustExpr,
    args: &[RustExpr],
) -> Option<LoweredMethod> {
    lower_method_with_context(object_ty, method, object, args, false)
}

pub(crate) fn lower_method_with_context(
    object_ty: &Type,
    method: &str,
    object: &RustExpr,
    args: &[RustExpr],
    is_deque_data_field: bool,
) -> Option<LoweredMethod> {
    lower_method_with_discard_context(object_ty, method, object, args, is_deque_data_field, false)
}

pub(crate) fn lower_method_with_discard_context(
    object_ty: &Type,
    method: &str,
    object: &RustExpr,
    args: &[RustExpr],
    is_deque_data_field: bool,
    discard_result: bool,
) -> Option<LoweredMethod> {
    super::lower_method_impl(
        object_ty,
        method,
        object,
        args,
        is_deque_data_field,
        discard_result,
    )
}
