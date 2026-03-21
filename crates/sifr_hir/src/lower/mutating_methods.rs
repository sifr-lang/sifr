use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;

pub(super) fn reject_immutable_parameter_method_mutation(
    ctx: &mut LowerCtx,
    object: &HirExpr,
    object_ty: &Type,
    method: &str,
) -> bool {
    if !is_collection_mutating_method(object_ty, method) {
        return false;
    }

    let HirExpr::Name { name, .. } = object else {
        return false;
    };

    if ctx
        .scope
        .lookup(name)
        .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding)
    {
        ctx.error(format!(
            "cannot mutate through immutable parameter `{name}`: add `mut` to the parameter declaration"
        ));
        return true;
    }

    false
}

pub(super) fn is_collection_mutating_method(object_ty: &Type, method: &str) -> bool {
    if let Type::Alias { body, .. } = object_ty {
        return is_collection_mutating_method(body, method);
    }

    match object_ty {
        Type::List(_) => matches!(
            method,
            "append"
                | "extend"
                | "insert"
                | "clear"
                | "reverse"
                | "sort"
                | "pop"
                | "popleft"
                | "appendleft"
                | "remove"
        ),
        Type::Dict(_, _) => matches!(method, "update" | "clear" | "pop" | "setdefault"),
        Type::Set(_) => matches!(
            method,
            "add"
                | "remove"
                | "discard"
                | "clear"
                | "update"
                | "intersection_update"
                | "difference_update"
                | "symmetric_difference_update"
        ),
        _ => false,
    }
}
