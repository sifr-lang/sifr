use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::{make_union, Type};

pub(super) fn refine_nonempty_method_return_type(
    object_ty: &Type,
    object: &HirExpr,
    method_name: &str,
    args: &[HirExpr],
    return_ty: &Type,
    ctx: &LowerCtx,
) -> Type {
    refine_nonempty_pop_return_type(
        object_ty,
        object,
        method_name,
        args.len(),
        return_ty,
        ctx,
    )
    .unwrap_or_else(|| return_ty.clone())
}

pub(super) fn refine_nonempty_pop_return_type(
    object_ty: &Type,
    object: &HirExpr,
    method_name: &str,
    arg_count: usize,
    return_ty: &Type,
    ctx: &LowerCtx,
) -> Option<Type> {
    if !matches!(object_ty.resolve_alias(), Type::List(_)) {
        return None;
    }
    if !matches!(method_name, "pop" | "popleft") {
        return None;
    }
    if method_name == "pop" && arg_count != 0 {
        return None;
    }
    if method_name == "popleft" && arg_count != 0 {
        return None;
    }
    let HirExpr::Name { name, .. } = object else {
        return None;
    };
    if ctx.min_length_guard(name.as_str()) == 0 {
        return None;
    }
    non_optional_union_variant(return_ty)
}

fn non_optional_union_variant(ty: &Type) -> Option<Type> {
    let Type::Union(variants) = ty.resolve_alias() else {
        return None;
    };
    let non_none: Vec<Type> = variants
        .iter()
        .filter(|variant| **variant != Type::None)
        .cloned()
        .collect();
    if non_none.is_empty() || non_none.len() == variants.len() {
        return None;
    }
    Some(make_union(non_none))
}
