use super::builtin_calls::callable_builtin_element_type;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;

pub(in crate::lower) fn refine_empty_set_binding_expr(
    expr: HirExpr,
    inferred_elem_ty: Type,
    ctx: &mut LowerCtx,
) -> HirExpr {
    let HirExpr::Name { name, ty } = &expr else {
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
        ty: refined_ty,
    }
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
    let HirExpr::Name { name, ty } = &expr else {
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
        ty: refined_ty,
    }
}
