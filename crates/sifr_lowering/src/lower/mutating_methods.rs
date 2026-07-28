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

    if let Some(name) = mutation_receiver_root_name(object) {
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

fn mutation_receiver_root_name(expr: &HirExpr) -> Option<&str> {
    match expr {
        HirExpr::Name { name, .. } => Some(name),
        HirExpr::FieldAccess { object, .. } | HirExpr::Index { object, .. } => {
            mutation_receiver_root_name(object)
        }
        _ => None,
    }
}

pub(in crate::lower) fn reject_immutable_method_mut_borrow_arguments(
    ctx: &mut LowerCtx,
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
) -> bool {
    let methods = match object_ty.resolve_alias() {
        Type::Class { methods, .. } | Type::Protocol { methods, .. } => methods,
        _ => return false,
    };
    let Some((_, signature)) = methods.iter().find(|(name, _)| name == method) else {
        return false;
    };

    let mut rejected = false;
    for (index, (arg, (_, _, convention))) in args.iter().zip(&signature.params).enumerate() {
        if !convention.is_mut_borrow() {
            continue;
        }
        let Some(root_name) = mutation_receiver_root_name(arg) else {
            continue;
        };
        let range = arg_ranges.get(index).copied().unwrap_or_default();
        if ctx
            .scope
            .lookup(root_name)
            .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding())
        {
            super::ownership_diagnostics::immutable_parameter_mutation(ctx, root_name, range);
            rejected = true;
            continue;
        }
        ctx.record_flow_effect(sifr_ir::FlowEffect::Borrow {
            binding: root_name.to_string(),
            mutable: true,
        });
    }
    rejected
}

pub(in crate::lower) fn is_collection_mutating_method(object_ty: &Type, method: &str) -> bool {
    if let Type::Alias { body, .. } = object_ty {
        return is_collection_mutating_method(body, method);
    }

    match object_ty {
        Type::PythonBuffer(_) => method == "write",
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

pub(in crate::lower) fn is_potential_collection_mutating_method(method: &str) -> bool {
    matches!(
        method,
        "write"
            | "append"
            | "extend"
            | "insert"
            | "clear"
            | "reverse"
            | "sort"
            | "pop"
            | "popleft"
            | "appendleft"
            | "remove"
            | "update"
            | "setdefault"
            | "add"
            | "discard"
            | "intersection_update"
            | "difference_update"
            | "symmetric_difference_update"
    )
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
        ctx.record_flow_effect(sifr_ir::FlowEffect::Mutation {
            target: target.clone(),
            operation: format!("method {method}"),
        });
        ctx.record_flow_effect(sifr_ir::FlowEffect::ClearNarrowing {
            binding: target.clone(),
        });
        ctx.clear_sequence_guards_for_target(&target);
    }
}
