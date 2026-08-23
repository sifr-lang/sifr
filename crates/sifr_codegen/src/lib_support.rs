use crate::{CodegenError, RustExpr, RustParam, RustType, try_lower_leaf_expr_result};
use sifr_ir::HirExpr;
use sifr_type_system::Type;
use std::collections::HashSet;

pub(crate) fn is_self_field_access_expr(expr: &HirExpr) -> bool {
    if let HirExpr::FieldAccess { object, .. } = expr {
        return matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self");
    }
    false
}

pub(crate) fn is_copyish_structured_stmt_expr_type(ty: &Type) -> bool {
    match ty {
        Type::Alias { body, .. } => is_copyish_structured_stmt_expr_type(body),
        Type::Int | Type::Float | Type::Bool | Type::Enum { .. } => true,
        _ => false,
    }
}

pub(crate) fn resolve_alias_type_for_plain_call(ty: &Type) -> &Type {
    match ty {
        Type::Alias { body, .. } => resolve_alias_type_for_plain_call(body),
        _ => ty,
    }
}

pub(crate) fn homogeneous_large_tuple_backing_array(ty: &Type) -> Option<(&Type, usize)> {
    let Type::Tuple(items) = resolve_alias_type_for_plain_call(ty) else {
        return None;
    };
    let first = items.first()?;
    if items.len() <= 12 || !items.iter().all(|item| item == first) {
        return None;
    }
    Some((first, items.len()))
}

pub(crate) fn type_has_typevar(ty: &Type) -> bool {
    match ty {
        Type::Alias {
            type_args, body, ..
        } => type_args.iter().any(type_has_typevar) || type_has_typevar(body),
        Type::TypeVar(_) => true,
        Type::List(inner) | Type::Set(inner) => type_has_typevar(inner),
        Type::Dict(key, val) => type_has_typevar(key) || type_has_typevar(val),
        Type::Tuple(elems) | Type::Union(elems) => elems.iter().any(type_has_typevar),
        Type::Result(ok, err) => type_has_typevar(ok) || type_has_typevar(err),
        Type::Class {
            fields, methods, ..
        } => {
            fields.iter().any(|(_, t)| type_has_typevar(t))
                || methods.iter().any(|(_, ft)| {
                    ft.params.iter().any(|(_, t, _)| type_has_typevar(t))
                        || type_has_typevar(&ft.return_type)
                })
        }
        _ => false,
    }
}

pub(crate) fn try_lower_leaf_or_name_expr_result(
    expr: &HirExpr,
) -> Result<Option<RustExpr>, CodegenError> {
    if let HirExpr::Lambda { params, body, .. } = expr {
        let lowered_body = try_lower_leaf_or_name_expr_result(body)?
            .ok_or_else(|| CodegenError::new("lambda body could not be lowered"))?;
        let lowered_params = params
            .iter()
            .map(|param| RustParam::Named {
                name: param.name.clone(),
                ty: RustType::Named("_".to_string()),
            })
            .collect::<Vec<_>>();
        return Ok(Some(RustExpr::Closure {
            params: lowered_params,
            body: Box::new(lowered_body),
            is_move: false,
        }));
    }
    if let Some(lowered) = try_lower_leaf_expr_result(expr)? {
        return Ok(Some(lowered));
    }
    if let HirExpr::Name { name, .. } = expr {
        return Ok(Some(RustExpr::Ident(name.clone())));
    }
    Ok(None)
}

pub(crate) fn expr_uses_borrowed_param(
    expr: &HirExpr,
    borrowed_params: &HashSet<String>,
    mut_borrowed_params: &HashSet<String>,
) -> bool {
    match expr {
        HirExpr::Name { name, .. } => {
            borrowed_params.contains(name) || mut_borrowed_params.contains(name)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            expr_uses_borrowed_param(left, borrowed_params, mut_borrowed_params)
                || comparators
                    .iter()
                    .any(|c| expr_uses_borrowed_param(c, borrowed_params, mut_borrowed_params))
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|v| expr_uses_borrowed_param(v, borrowed_params, mut_borrowed_params)),
        HirExpr::UnaryOp { operand, .. } => {
            expr_uses_borrowed_param(operand, borrowed_params, mut_borrowed_params)
        }
        HirExpr::BinOp { left, right, .. } => {
            expr_uses_borrowed_param(left, borrowed_params, mut_borrowed_params)
                || expr_uses_borrowed_param(right, borrowed_params, mut_borrowed_params)
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_uses_borrowed_param(condition, borrowed_params, mut_borrowed_params)
                || expr_uses_borrowed_param(then_expr, borrowed_params, mut_borrowed_params)
                || expr_uses_borrowed_param(else_expr, borrowed_params, mut_borrowed_params)
        }
        _ => false,
    }
}
