use super::{
    HirExpr, RustExpr, RustLiteral, RustParam, RustStmt, RustType, SimpleStmtBindings, Type,
    try_lower_leaf_expr,
};
pub(super) fn resolve_alias_type(ty: &Type) -> &Type {
    match ty {
        Type::Alias { body, .. } => resolve_alias_type(body),
        _ => ty,
    }
}

pub(super) fn is_option_like_type(ty: &Type) -> bool {
    ty.optional_member_type().is_some()
}

pub(super) fn detect_option_truthiness_alias(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty, .. } = expr {
        if is_option_like_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

pub(super) fn option_binding_pattern(option_var: &str, bindings: SimpleStmtBindings<'_>) -> String {
    let requires_mut = crate::option_binding_mutability::option_binding_requires_mut(
        option_var,
        bindings.mutated_vars,
        bindings.borrowed_params,
        bindings.mut_borrowed_params,
        bindings.local_binding_types,
        bindings.recursive_fields,
    );
    if requires_mut {
        format!("Some(mut {option_var})")
    } else {
        format!("Some({option_var})")
    }
}

pub(super) fn option_binding_value_expr(
    option_var: &str,
    bindings: SimpleStmtBindings<'_>,
) -> RustExpr {
    let base = RustExpr::Ident(option_var.to_string());
    if bindings.borrowed_params.contains(option_var)
        || bindings.mut_borrowed_params.contains(option_var)
    {
        RustExpr::MethodCall {
            receiver: Box::new(base),
            method: "as_ref".to_string(),
            args: vec![],
        }
    } else if bindings
        .local_binding_types
        .get(option_var)
        .is_some_and(crate::helpers::is_logically_copy_rust_move_type)
    {
        RustExpr::Clone(Box::new(base))
    } else {
        base
    }
}

pub(super) fn lower_if_not_none_chain(
    option_vars: &[String],
    lowered_then_body: Vec<RustStmt>,
    nested_else: Option<Vec<RustStmt>>,
    bindings: SimpleStmtBindings<'_>,
) -> Option<RustStmt> {
    let mut chain_then = lowered_then_body;
    for option_var in option_vars.iter().rev() {
        chain_then = vec![RustStmt::IfLet {
            pattern: option_binding_pattern(option_var, bindings),
            expr: option_binding_value_expr(option_var, bindings),
            then_body: chain_then,
            else_body: None,
        }];
    }

    let mut chain_root = chain_then.into_iter().next()?;
    if let RustStmt::IfLet { else_body, .. } = &mut chain_root {
        *else_body = nested_else;
    }
    Some(chain_root)
}

pub(super) fn is_alias_equivalent_type(left: &Type, right: &Type) -> bool {
    left == right || resolve_alias_type(left) == resolve_alias_type(right)
}

pub(super) fn is_none_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::None)
}

pub(super) fn is_okwrap_none_expr(expr: &HirExpr) -> bool {
    matches!(
        expr,
        HirExpr::OkWrap { value, .. }
            if matches!(value.as_ref(), HirExpr::NoneLiteral) || is_none_type(value.ty())
    )
}

pub(super) fn try_lower_name_ident_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, .. } = expr {
        return Some(RustExpr::Ident(name.clone()));
    }
    None
}

pub(super) fn try_lower_leaf_or_name_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_leaf_expr(expr) {
        return Some(lowered);
    }
    if let HirExpr::Lambda { params, body, .. } = expr {
        let lowered_params = params
            .iter()
            .map(|param| RustParam::Named {
                name: param.name.clone(),
                ty: RustType::Named("_".to_string()),
            })
            .collect::<Vec<_>>();
        return Some(RustExpr::Closure {
            params: lowered_params,
            body: Box::new(try_lower_leaf_or_name_expr(body)?),
            is_move: false,
        });
    }
    if let Some(lowered) = try_lower_stmt_index_expr(expr) {
        return Some(lowered);
    }
    if let Some(lowered) = try_lower_stmt_string_concat_expr(expr) {
        return Some(lowered);
    }
    try_lower_name_ident_expr(expr)
}

pub(super) fn try_lower_stmt_index_expr(expr: &HirExpr) -> Option<RustExpr> {
    let HirExpr::Index { object, index, .. } = expr else {
        return None;
    };
    if !is_option_like_type(expr.ty())
        && matches!(resolve_alias_type(object.ty()), Type::List(_) | Type::Str)
    {
        return None;
    }
    match resolve_alias_type(object.ty()) {
        Type::Dict(_, value_ty) => {
            let projection_method =
                crate::helpers::option_projection_method_for_owned_type(value_ty.as_ref());
            let lowered_key = if let HirExpr::StringLiteral(value) = index.as_ref() {
                RustExpr::Verbatim(format!("{value:?}"))
            } else {
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(try_lower_leaf_or_name_expr(index)?),
                }
            };
            let lowered_get = RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                    method: "get".to_string(),
                    args: vec![lowered_key],
                }),
                method: projection_method.to_string(),
                args: vec![],
            };
            if is_option_like_type(value_ty.as_ref()) {
                Some(RustExpr::MethodCall {
                    receiver: Box::new(lowered_get),
                    method: "and_then".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "__v".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Ident("__v".to_string())),
                        is_move: false,
                    }],
                })
            } else {
                Some(lowered_get)
            }
        }
        Type::List(element_ty) => {
            let projection_method =
                crate::helpers::option_projection_method_for_owned_type(element_ty.as_ref());
            Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                    method: "get".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(try_lower_leaf_or_name_expr(index)?),
                        ty: RustType::Named("usize".to_string()),
                    }],
                }),
                method: projection_method.to_string(),
                args: vec![],
            })
        }
        _ => None,
    }
}

pub(super) fn try_lower_stmt_string_concat_expr(expr: &HirExpr) -> Option<RustExpr> {
    let HirExpr::BinOp {
        left,
        op,
        right,
        ty,
    } = expr
    else {
        return None;
    };
    if op != "+" || !matches!(resolve_alias_type(ty), Type::Str) {
        return None;
    }

    let mut parts = Vec::new();
    collect_stmt_string_concat_parts(left, &mut parts);
    collect_stmt_string_concat_parts(right, &mut parts);

    if parts
        .iter()
        .all(|part| matches!(part, HirExpr::StringLiteral(_)))
    {
        let mut combined = String::new();
        for part in parts {
            if let HirExpr::StringLiteral(value) = part {
                combined.push_str(value);
            }
        }
        return Some(RustExpr::Literal(RustLiteral::Str(combined)));
    }

    let capacity = string_concat_capacity_expr(&parts);
    let mut stmts = vec![RustStmt::Let {
        mutable: true,
        name: "__sifr_concat".to_string(),
        ty: Some(RustType::String_),
        value: RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "String".to_string(),
                "with_capacity".to_string(),
            ])),
            args: vec![capacity],
        },
    }];
    for part in parts {
        let (method, arg) = if let HirExpr::StringLiteral(value) = part {
            let mut chars = value.chars();
            if let (Some(ch), None) = (chars.next(), chars.next()) {
                ("push", RustExpr::Literal(crate::RustLiteral::Char(ch)))
            } else {
                ("push_str", RustExpr::Verbatim(format!("{value:?}")))
            }
        } else {
            (
                "push_str",
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(try_lower_leaf_or_name_expr(
                        part,
                    )?))),
                    method: "as_ref".to_string(),
                    args: vec![],
                },
            )
        };
        stmts.push(RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__sifr_concat".to_string())),
            method: method.to_string(),
            args: vec![arg],
        }));
    }
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident("__sifr_concat".to_string()))),
    })
}

fn string_concat_capacity_expr(parts: &[&HirExpr]) -> RustExpr {
    let mut capacity_parts = Vec::with_capacity(parts.len());
    for part in parts {
        let len_expr = if let HirExpr::StringLiteral(value) = part {
            RustExpr::Verbatim(format!("{}usize", value.len()))
        } else if let HirExpr::Name { name, .. } = part {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(name.clone())),
                method: "len".to_string(),
                args: vec![],
            }
        } else {
            RustExpr::Verbatim("0usize".to_string())
        };
        capacity_parts.push(len_expr);
    }
    let mut iter = capacity_parts.into_iter();
    let mut capacity = iter
        .next()
        .unwrap_or_else(|| RustExpr::Verbatim("0usize".to_string()));
    for part in iter {
        capacity = RustExpr::MethodCall {
            receiver: Box::new(capacity),
            method: "saturating_add".to_string(),
            args: vec![part],
        };
    }
    capacity
}

pub(super) fn collect_stmt_string_concat_parts<'a>(
    expr: &'a HirExpr,
    parts: &mut Vec<&'a HirExpr>,
) {
    if let HirExpr::BinOp {
        left,
        op,
        right,
        ty,
    } = expr
    {
        if op == "+" && matches!(resolve_alias_type(ty), Type::Str) {
            collect_stmt_string_concat_parts(left, parts);
            collect_stmt_string_concat_parts(right, parts);
            return;
        }
    }
    parts.push(expr);
}

pub(super) fn try_lower_attribute_dict_insert_key_expr(
    index: &HirExpr,
    field_ty: &Type,
) -> Option<RustExpr> {
    let Type::Dict(key_ty, _) = resolve_alias_type(field_ty) else {
        return None;
    };

    if matches!(resolve_alias_type(key_ty), Type::Str | Type::TypeVar(_))
        && matches!(index, HirExpr::Name { .. })
    {
        // Preserve borrowed-name key cloning semantics.
        return None;
    }

    try_lower_leaf_or_name_expr(index)
}
