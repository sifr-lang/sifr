use super::builtin_calls::{DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS};
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;

pub(in crate::lower) fn refine_defaultdict_binding_expr(
    expr: HirExpr,
    method_name: &str,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let inferred_value_ty = match method_name {
        "append" if args.len() == 1 => Type::List(Box::new(args[0].ty().clone())),
        "add" if args.len() == 1 => Type::Set(Box::new(args[0].ty().clone())),
        _ => return None,
    };
    let HirExpr::Index { object, index, .. } = expr else {
        return None;
    };
    let HirExpr::Name { name, ty } = object.as_ref() else {
        return None;
    };
    let Type::Alias {
        name: alias_name,
        body,
        ..
    } = ty
    else {
        return None;
    };
    if !matches!(
        alias_name.as_str(),
        DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
    ) {
        return None;
    }
    let Type::Dict(key_ty, value_ty) = body.as_ref() else {
        return None;
    };
    let expected_unrefined = match alias_name.as_str() {
        DEFAULTDICT_LIST_ALIAS => Type::List(Box::new(Type::Any)),
        DEFAULTDICT_SET_ALIAS => Type::Set(Box::new(Type::Any)),
        DEFAULTDICT_INT_ALIAS => Type::Int,
        _ => return None,
    };
    if *value_ty.as_ref() != expected_unrefined {
        return None;
    }
    let refined_key_ty = if matches!(key_ty.as_ref(), Type::Any | Type::Unknown) {
        index.ty().clone()
    } else {
        *key_ty.clone()
    };
    let refined_ty = Type::Alias {
        name: alias_name.clone(),
        type_args: Vec::new(),
        body: Box::new(Type::Dict(
            Box::new(refined_key_ty),
            Box::new(inferred_value_ty.clone()),
        )),
    };
    ctx.scope.narrow_var(name, refined_ty.clone());
    Some(HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: name.clone(),
            ty: refined_ty,
        }),
        index,
        ty: inferred_value_ty,
    })
}
