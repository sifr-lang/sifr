use super::{
    codegen_body_always_exits, detect_and_not_none_vars, detect_is_none_var,
    detect_is_not_none_var, detect_option_truthiness_alias, detect_or_is_none_vars, is_none_type,
    is_option_like_type, lower_if_not_none_chain, option_binding_pattern, resolve_alias_type,
    try_lower_leaf_expr, try_lower_leaf_or_name_expr, try_lower_simple_stmt_block, HashSet,
    HirExpr, HirStmt, RustExpr, RustLiteral, RustParam, RustStmt, RustType, SimpleStmtBindings,
    SimpleStmtLoweringCtx, Type,
};
pub(super) fn try_lower_simple_if_stmt(
    condition: &HirExpr,
    then_body: &[HirStmt],
    elif_clauses: &[(HirExpr, Vec<HirStmt>)],
    maybe_else_body: Option<&[HirStmt]>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if elif_clauses.is_empty() && maybe_else_body.is_none() && codegen_body_always_exits(then_body)
    {
        if let Some(option_vars) = detect_or_is_none_vars(condition) {
            let lowered_then_body =
                try_lower_simple_stmt_block(then_body, in_loop_with_else, bindings, ctx)?;
            let pattern = format!(
                "({})",
                option_vars
                    .iter()
                    .map(|option_var| option_binding_pattern(option_var, bindings))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return Some(vec![RustStmt::LetElse {
                pattern,
                value: RustExpr::Tuple(
                    option_vars
                        .iter()
                        .map(|option_var| RustExpr::Ident(option_var.clone()))
                        .collect(),
                ),
                else_body: lowered_then_body,
            }]);
        }
        if let Some(option_var) = detect_is_none_var(condition) {
            let lowered_then_body =
                try_lower_simple_stmt_block(then_body, in_loop_with_else, bindings, ctx)?;
            return Some(vec![RustStmt::LetElse {
                pattern: option_binding_pattern(&option_var, bindings),
                value: RustExpr::Ident(option_var),
                else_body: lowered_then_body,
            }]);
        }
    }

    let mut nested_else = if let Some(else_body) = maybe_else_body {
        Some(try_lower_simple_stmt_block(
            else_body,
            in_loop_with_else,
            bindings,
            ctx,
        )?)
    } else {
        None
    };

    for (elif_cond, elif_body) in elif_clauses.iter().rev() {
        nested_else = Some(vec![try_lower_simple_if_clause(
            elif_cond,
            elif_body,
            nested_else,
            in_loop_with_else,
            bindings,
            ctx,
        )?]);
    }

    Some(vec![try_lower_simple_if_clause(
        condition,
        then_body,
        nested_else,
        in_loop_with_else,
        bindings,
        ctx,
    )?])
}

pub(super) fn try_lower_simple_if_clause(
    condition: &HirExpr,
    then_body: &[HirStmt],
    nested_else: Option<Vec<RustStmt>>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<RustStmt> {
    let lowered_then_body =
        try_lower_simple_stmt_block(then_body, in_loop_with_else, bindings, ctx)?;

    if let Some(option_var) = detect_is_not_none_var(condition) {
        return Some(RustStmt::IfLet {
            pattern: option_binding_pattern(&option_var, bindings),
            expr: RustExpr::Ident(option_var),
            then_body: lowered_then_body,
            else_body: nested_else,
        });
    }

    if let Some(option_vars) = detect_and_not_none_vars(condition) {
        return lower_if_not_none_chain(&option_vars, lowered_then_body, nested_else, bindings);
    }

    if let Some(option_var) = detect_option_truthiness_alias(condition) {
        return Some(RustStmt::IfLet {
            pattern: option_binding_pattern(&option_var, bindings),
            expr: RustExpr::Ident(option_var),
            then_body: lowered_then_body,
            else_body: nested_else,
        });
    }

    if let Some(option_var) = detect_is_none_var(condition) {
        let lowered_cond =
            try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?;
        let lowered_else = nested_else.map(|else_body| {
            vec![RustStmt::IfLet {
                pattern: option_binding_pattern(&option_var, bindings),
                expr: RustExpr::Ident(option_var.clone()),
                then_body: else_body,
                else_body: None,
            }]
        });
        return Some(RustStmt::If {
            cond: lowered_cond,
            then_body: lowered_then_body,
            else_body: lowered_else,
        });
    }

    Some(RustStmt::If {
        cond: try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?,
        then_body: lowered_then_body,
        else_body: nested_else,
    })
}

pub(super) fn try_lower_simple_condition_test_expr(
    expr: &HirExpr,
    borrowed_params: &HashSet<String>,
) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_borrowed_typevar_compare_condition(expr, borrowed_params) {
        return Some(lowered);
    }
    // Borrowed-name comparisons require context-sensitive ownership rewrites.
    // Defer them to the structured stmt emitter path.
    if expr_uses_borrowed_name(expr, borrowed_params) {
        return None;
    }
    if let Some(lowered) = try_lower_structured_compare_condition_expr(expr) {
        return Some(lowered);
    }
    if let Some(lowered) = try_lower_numeric_truthiness_condition_expr(expr) {
        return Some(lowered);
    }
    if let Some(lowered) = try_lower_leaf_expr(expr) {
        return Some(lowered);
    }
    let option_var = detect_option_truthiness_alias(expr)?;
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(option_var)),
        method: "is_some".to_string(),
        args: vec![],
    })
}

pub(super) fn try_lower_numeric_truthiness_condition_expr(expr: &HirExpr) -> Option<RustExpr> {
    fn zero_literal_for_type(ty: &Type) -> Option<RustExpr> {
        match resolve_alias_type(ty) {
            Type::Int | Type::LiteralInt(_) => Some(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                ty: RustType::I64,
            }),
            Type::BigInt => Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "BigInt".to_string(),
                    "from".to_string(),
                ])),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    ty: RustType::I64,
                }],
            }),
            Type::Float => Some(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                ty: RustType::F64,
            }),
            _ => None,
        }
    }

    match expr {
        HirExpr::Name { name, ty, .. } => Some(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident(name.clone())),
            op: "!=".to_string(),
            right: Box::new(zero_literal_for_type(ty)?),
        }),
        HirExpr::MethodCall {
            object,
            method,
            args,
            ty,
            ..
        } if method == "len" && args.is_empty() => {
            let receiver = try_lower_leaf_expr(object.as_ref())?;
            let lhs = RustExpr::Cast {
                expr: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method: "len".to_string(),
                    args: vec![],
                }),
                ty: RustType::I64,
            };
            Some(RustExpr::BinOp {
                left: Box::new(lhs),
                op: "!=".to_string(),
                right: Box::new(zero_literal_for_type(ty)?),
            })
        }
        HirExpr::UnaryOp { op, operand, .. } if op == "not" => match operand.as_ref() {
            HirExpr::Name { name, ty, .. } => Some(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident(name.clone())),
                op: "==".to_string(),
                right: Box::new(zero_literal_for_type(ty)?),
            }),
            HirExpr::MethodCall {
                object,
                method,
                args,
                ty,
                ..
            } if method == "len" && args.is_empty() => {
                let receiver = try_lower_leaf_expr(object.as_ref())?;
                let lhs = RustExpr::Cast {
                    expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(receiver),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                    ty: RustType::I64,
                };
                Some(RustExpr::BinOp {
                    left: Box::new(lhs),
                    op: "==".to_string(),
                    right: Box::new(zero_literal_for_type(ty)?),
                })
            }
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn try_lower_structured_compare_condition_expr(expr: &HirExpr) -> Option<RustExpr> {
    if try_lower_leaf_expr(expr).is_some() {
        return None;
    }
    let HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = expr
    else {
        return None;
    };
    if ops.len() != 1 || comparators.len() != 1 {
        return None;
    }
    let rhs_expr = comparators.first()?;
    let lowered_op = match ops[0].as_str() {
        "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].as_str(),
        "is" => "==",
        "is not" => "!=",
        _ => return None,
    };
    if matches!(left.as_ref(), HirExpr::NoneLiteral) || matches!(rhs_expr, HirExpr::NoneLiteral) {
        let other = if matches!(rhs_expr, HirExpr::NoneLiteral) {
            left.as_ref()
        } else {
            rhs_expr
        };
        let is_equal_op = lowered_op == "==";
        if is_option_like_type(other.ty()) {
            let lowered_other = try_lower_condition_operand_expr(other)?;
            return Some(RustExpr::MethodCall {
                receiver: Box::new(lowered_other),
                method: if is_equal_op { "is_none" } else { "is_some" }.to_string(),
                args: vec![],
            });
        }
        if is_none_type(other.ty()) {
            return Some(RustExpr::Literal(RustLiteral::Bool(is_equal_op)));
        }
        if !matches!(
            resolve_alias_type(other.ty()),
            Type::Any | Type::Unknown | Type::TypeVar(_)
        ) {
            return Some(RustExpr::Literal(RustLiteral::Bool(!is_equal_op)));
        }
    }
    let mut lowered_left = try_lower_condition_operand_expr(left)?;
    let mut lowered_right = try_lower_condition_operand_expr(rhs_expr)?;
    if is_option_like_type(left.ty())
        && !is_option_like_type(rhs_expr.ty())
        && !matches!(rhs_expr, HirExpr::NoneLiteral)
    {
        lowered_right = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_right],
        };
    } else if !is_option_like_type(left.ty())
        && is_option_like_type(rhs_expr.ty())
        && !matches!(left.as_ref(), HirExpr::NoneLiteral)
    {
        lowered_left = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_left],
        };
    } else if matches!(
        resolve_alias_type(left.ty()),
        Type::Str | Type::LiteralStr(_)
    ) && matches!(
        resolve_alias_type(rhs_expr.ty()),
        Type::Str | Type::LiteralStr(_)
    ) {
        lowered_left = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(lowered_left))),
            method: "as_str".to_string(),
            args: vec![],
        };
        lowered_right = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(lowered_right))),
            method: "as_str".to_string(),
            args: vec![],
        };
    }
    Some(RustExpr::BinOp {
        left: Box::new(lowered_left),
        op: lowered_op.to_string(),
        right: Box::new(lowered_right),
    })
}

pub(super) fn try_lower_condition_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_leaf_or_name_expr(expr) {
        return Some(lowered);
    }
    match expr {
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } if method == "len" && args.is_empty() => {
            if matches!(
                resolve_alias_type(object.ty()),
                Type::Str | Type::LiteralStr(_)
            ) {
                return None;
            }
            Some(RustExpr::Cast {
                expr: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                    method: "len".to_string(),
                    args: vec![],
                }),
                ty: RustType::I64,
            })
        }
        HirExpr::Index {
            object, index, ty, ..
        } => try_lower_condition_index_operand_expr(object, index, ty),
        _ => None,
    }
}

pub(super) fn try_lower_condition_index_operand_expr(
    object: &HirExpr,
    index: &HirExpr,
    result_ty: &Type,
) -> Option<RustExpr> {
    match resolve_alias_type(object.ty()) {
        Type::Dict(_, value_ty) => {
            let projection_method =
                crate::helpers::option_projection_method_for_owned_type(value_ty.as_ref());
            let lowered_key = if let HirExpr::StringLiteral(value) = index {
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
        Type::List(element_ty) if !is_option_like_type(result_ty) => {
            let indexed_expr = RustExpr::Index {
                expr: Box::new(try_lower_leaf_or_name_expr(object)?),
                index: Box::new(RustExpr::Cast {
                    expr: Box::new(try_lower_leaf_or_name_expr(index)?),
                    ty: RustType::Named("usize".to_string()),
                }),
            };
            Some(
                if crate::helpers::is_copy_type_for_codegen(element_ty.as_ref()) {
                    indexed_expr
                } else {
                    RustExpr::Clone(Box::new(indexed_expr))
                },
            )
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
        Type::Str if !is_option_like_type(result_ty) => None,
        _ => None,
    }
}

pub(super) fn try_lower_borrowed_typevar_compare_condition(
    expr: &HirExpr,
    borrowed_params: &HashSet<String>,
) -> Option<RustExpr> {
    let HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = expr
    else {
        return None;
    };
    if ops.len() != 1 || comparators.len() != 1 {
        return None;
    }

    let rhs_expr = comparators.first()?;
    if !matches!(resolve_alias_type(left.ty()), Type::TypeVar(_))
        || !matches!(resolve_alias_type(rhs_expr.ty()), Type::TypeVar(_))
    {
        return None;
    }

    let lowered_op = match ops[0].as_str() {
        "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].as_str(),
        "is" => "==",
        "is not" => "!=",
        _ => return None,
    };

    let lower_operand = |operand: &HirExpr| -> Option<RustExpr> {
        let HirExpr::Name { name, .. } = operand else {
            return None;
        };
        let ident = RustExpr::Ident(name.clone());
        if borrowed_params.contains(name) {
            return Some(RustExpr::Deref(Box::new(ident)));
        }
        Some(ident)
    };

    Some(RustExpr::BinOp {
        left: Box::new(lower_operand(left)?),
        op: lowered_op.to_string(),
        right: Box::new(lower_operand(rhs_expr)?),
    })
}

pub(super) fn expr_uses_borrowed_name(expr: &HirExpr, borrowed_params: &HashSet<String>) -> bool {
    match expr {
        HirExpr::Name { name, .. } => borrowed_params.contains(name),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            expr_uses_borrowed_name(left, borrowed_params)
                || comparators
                    .iter()
                    .any(|c| expr_uses_borrowed_name(c, borrowed_params))
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|v| expr_uses_borrowed_name(v, borrowed_params)),
        HirExpr::UnaryOp { operand, .. } => expr_uses_borrowed_name(operand, borrowed_params),
        HirExpr::BinOp { left, right, .. } => {
            expr_uses_borrowed_name(left, borrowed_params)
                || expr_uses_borrowed_name(right, borrowed_params)
        }
        _ => false,
    }
}
