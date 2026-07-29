use super::builtin_calls::callable_builtin_element_type;
use super::type_bounds::supports_hash_key_in_context;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;

pub(in crate::lower) fn refine_empty_set_binding_expr(
    expr: HirExpr,
    inferred_elem_ty: Type,
    ctx: &mut LowerCtx,
) -> HirExpr {
    let HirExpr::Name {
        name,
        binding_id,
        ty,
    } = &expr
    else {
        return expr;
    };
    let Type::Set(inner) = ty.resolve_alias() else {
        return expr;
    };
    if !matches!(inner.as_ref(), Type::Any | Type::Unknown) {
        return expr;
    }
    let refined_ty = Type::Set(Box::new(inferred_elem_ty));
    let _ = ctx.scope.set_type(name, refined_ty.clone());
    ctx.narrow_var_with_flow(
        name,
        refined_ty.clone(),
        "empty-collection-refinement".to_string(),
        true,
    );
    HirExpr::Name {
        name: name.clone(),
        binding_id: *binding_id,
        ty: refined_ty,
    }
}

pub(in crate::lower) fn refine_empty_dict_membership_expr(
    expr: HirExpr,
    inferred_key_ty: Type,
    ctx: &mut LowerCtx,
) -> HirExpr {
    let supports_hash = supports_hash_key_in_context(&inferred_key_ty, ctx);
    if matches!(inferred_key_ty.resolve_alias(), Type::Any | Type::Unknown) || !supports_hash {
        return expr;
    }
    let HirExpr::Name {
        name,
        binding_id,
        ty,
    } = &expr
    else {
        return expr;
    };
    let Type::Dict(key, value) = ty.resolve_alias() else {
        return expr;
    };
    if !matches!(key.as_ref(), Type::Any | Type::Unknown) {
        return expr;
    }
    let refined_ty = Type::Dict(Box::new(inferred_key_ty), value.clone());
    record_empty_dict_refinement(name, &refined_ty, ctx);
    HirExpr::Name {
        name: name.clone(),
        binding_id: *binding_id,
        ty: refined_ty,
    }
}

pub(in crate::lower) fn refine_empty_dict_index_comparison_expr(
    expr: HirExpr,
    inferred_value_ty: &Type,
    ctx: &mut LowerCtx,
) -> HirExpr {
    if matches!(inferred_value_ty.resolve_alias(), Type::Any | Type::Unknown)
        || !inferred_value_ty.supports_structural_equality()
    {
        return expr;
    }
    let HirExpr::Index { object, index, ty } = expr else {
        return expr;
    };
    if !matches!(ty.resolve_alias(), Type::Any | Type::Unknown) {
        return HirExpr::Index { object, index, ty };
    }
    let HirExpr::Name {
        name,
        binding_id,
        ty: object_ty,
    } = object.as_ref()
    else {
        return HirExpr::Index { object, index, ty };
    };
    let Type::Dict(key, value) = object_ty.resolve_alias() else {
        return HirExpr::Index { object, index, ty };
    };
    if !matches!(value.as_ref(), Type::Any | Type::Unknown) {
        return HirExpr::Index { object, index, ty };
    }
    let refined_object_ty = Type::Dict(key.clone(), Box::new(inferred_value_ty.clone()));
    record_empty_dict_refinement(name, &refined_object_ty, ctx);
    HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: name.clone(),
            binding_id: *binding_id,
            ty: refined_object_ty,
        }),
        index,
        ty: inferred_value_ty.clone(),
    }
}

fn record_empty_dict_refinement(name: &str, ty: &Type, ctx: &mut LowerCtx) {
    let _ = ctx.scope.set_type(name, ty.clone());
    ctx.narrow_var_with_flow(
        name,
        ty.clone(),
        "empty-dict-comparison-refinement".to_string(),
        true,
    );
    ctx.pending_container_specialization_patches
        .insert(name.to_string(), ty.clone());
    ctx.empty_dict_specializations
        .insert(name.to_string(), ty.clone());
}

pub(in crate::lower) fn refine_empty_list_binding_expr(
    expr: HirExpr,
    method_name: &str,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> HirExpr {
    let inferred_elem_ty = match method_name {
        "append" if args.len() == 1 => Some(args[0].ty().clone()),
        "insert" if args.len() == 2 => Some(args[1].ty().clone()),
        "extend" if args.len() == 1 => callable_builtin_element_type(args[0].ty()),
        _ => None,
    };
    let Some(inferred_elem_ty) = inferred_elem_ty else {
        return expr;
    };
    if matches!(inferred_elem_ty.resolve_alias(), Type::Any | Type::Unknown) {
        return expr;
    }
    let HirExpr::Name {
        name,
        binding_id,
        ty,
    } = &expr
    else {
        return expr;
    };
    let Type::List(inner) = ty.resolve_alias() else {
        return expr;
    };
    if !matches!(inner.as_ref(), Type::Any | Type::Unknown) {
        return expr;
    }
    let refined_ty = Type::List(Box::new(inferred_elem_ty));
    let _ = ctx.scope.set_type(name, refined_ty.clone());
    ctx.narrow_var_with_flow(
        name,
        refined_ty.clone(),
        "empty-collection-refinement".to_string(),
        true,
    );
    ctx.pending_container_specialization_patches
        .insert(name.clone(), refined_ty.clone());
    HirExpr::Name {
        name: name.clone(),
        binding_id: *binding_id,
        ty: refined_ty,
    }
}
