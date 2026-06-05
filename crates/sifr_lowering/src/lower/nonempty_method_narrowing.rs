use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::{make_union, Type};

pub(in crate::lower) fn refine_nonempty_method_return_type(
    object_ty: &Type,
    object: &HirExpr,
    method_name: &str,
    args: &[HirExpr],
    return_ty: &Type,
    ctx: &LowerCtx,
) -> Type {
    refine_nonempty_pop_return_type(object_ty, object, method_name, args, return_ty, ctx)
        .unwrap_or_else(|| return_ty.clone())
}

pub(in crate::lower) fn refine_nonempty_pop_return_type(
    object_ty: &Type,
    object: &HirExpr,
    method_name: &str,
    args: &[HirExpr],
    return_ty: &Type,
    ctx: &LowerCtx,
) -> Option<Type> {
    if !supports_nonempty_pop_narrowing_on_type(object_ty) {
        return None;
    }
    if !matches!(method_name, "pop" | "popleft") {
        return None;
    }
    if !is_narrowable_pop_call(method_name, args) {
        return None;
    }
    let sequence_name = sequence_guard_name_from_hir_expr(object)?;
    if ctx.min_length_guard(sequence_name.as_str()) == 0 {
        return None;
    }
    nonempty_pop_element_type(object_ty).or_else(|| non_optional_union_variant(return_ty))
}

fn sequence_guard_name_from_hir_expr(expr: &HirExpr) -> Option<String> {
    match expr {
        HirExpr::Name { name, .. } => Some(name.clone()),
        HirExpr::FieldAccess { object, field, .. } => {
            let base = sequence_guard_name_from_hir_expr(object)?;
            Some(format!("{base}.{field}"))
        }
        _ => None,
    }
}

fn supports_nonempty_pop_narrowing_on_type(object_ty: &Type) -> bool {
    match object_ty.resolve_alias() {
        Type::List(_) => true,
        Type::Class { name, .. } => is_deque_class_name(name),
        _ => false,
    }
}

fn nonempty_pop_element_type(object_ty: &Type) -> Option<Type> {
    match object_ty.resolve_alias() {
        Type::List(elem) => Some(*elem.clone()),
        Type::Class { name, fields, .. } if is_deque_class_name(name) => {
            fields.iter().find_map(|(field_name, field_ty)| {
                if field_name != "_data" {
                    return None;
                }
                let Type::List(elem) = field_ty.resolve_alias() else {
                    return None;
                };
                Some(*elem.clone())
            })
        }
        _ => None,
    }
}

fn is_deque_class_name(name: &str) -> bool {
    name == "deque"
        || name
            .rsplit_once('.')
            .is_some_and(|(_, tail)| tail == "deque")
}

fn is_narrowable_pop_call(method_name: &str, args: &[HirExpr]) -> bool {
    match method_name {
        "pop" => matches!(args, [] | [HirExpr::IntLiteral(0)]),
        "popleft" => args.is_empty(),
        _ => false,
    }
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
