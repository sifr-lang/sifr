use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_type_system::Type;

pub(in crate::lower) fn reject_immutable_parameter_method_mutation(
    ctx: &mut LowerCtx,
    object: &HirExpr,
    object_ty: &Type,
    method: &str,
    object_range: TextRange,
) -> bool {
    if !is_collection_mutating_method(object_ty, method) {
        return false;
    }

    if let HirExpr::Name { name, .. } = object {
        if ctx
            .scope
            .lookup(name)
            .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding())
        {
            super::ownership_diagnostics::immutable_parameter_mutation(ctx, name, object_range);
            return true;
        }
    }

    false
}

pub(in crate::lower) fn is_collection_mutating_method(object_ty: &Type, method: &str) -> bool {
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

pub(in crate::lower) fn method_invalidates_collection_flow_facts(
    object_ty: &Type,
    method: &str,
) -> bool {
    if let Type::Alias { body, .. } = object_ty {
        return method_invalidates_collection_flow_facts(body, method);
    }

    match object_ty {
        Type::List(_) => matches!(method, "clear" | "pop" | "popleft" | "remove"),
        Type::Dict(_, _) => matches!(method, "clear" | "pop"),
        Type::Set(_) => matches!(
            method,
            "remove"
                | "discard"
                | "clear"
                | "intersection_update"
                | "difference_update"
                | "symmetric_difference_update"
        ),
        _ => false,
    }
}

pub(in crate::lower) fn invalidate_collection_flow_facts_for_method(
    ctx: &mut LowerCtx,
    object: &HirExpr,
    object_ty: &Type,
    method: &str,
) {
    if !method_invalidates_collection_flow_facts(object_ty, method) {
        return;
    }
    if let Some(target) = super::sequence_guards::hir_sequence_guard_target_name(object) {
        ctx.clear_sequence_guards_for_target(&target);
    }
}
