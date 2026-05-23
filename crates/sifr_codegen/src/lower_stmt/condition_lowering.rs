fn try_lower_simple_if_stmt(
    condition: &HirExpr,
    then_body: &[HirStmt],
    elif_clauses: &[(HirExpr, Vec<HirStmt>)],
    maybe_else_body: Option<&[HirStmt]>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    let option_binding_pattern = |name: &str| {
        if bindings.borrowed_params.contains(name) {
            format!("Some({name})")
        } else {
            format!("Some(mut {name})")
        }
    };
    if elif_clauses.is_empty() && maybe_else_body.is_none() && codegen_body_always_exits(then_body)
    {
        if let Some(option_vars) = detect_or_is_none_vars(condition) {
            let lowered_then_body = try_lower_simple_stmt_block(
                then_body,
                in_loop_with_else,
                bindings.mutated_vars,
                bindings.borrowed_params,
                ctx,
            )?;
            let pattern = format!(
                "({})",
                option_vars
                    .iter()
                    .map(|option_var| option_binding_pattern(option_var))
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
            let lowered_then_body = try_lower_simple_stmt_block(
                then_body,
                in_loop_with_else,
                bindings.mutated_vars,
                bindings.borrowed_params,
                ctx,
            )?;
            return Some(vec![RustStmt::LetElse {
                pattern: option_binding_pattern(&option_var),
                value: RustExpr::Ident(option_var),
                else_body: lowered_then_body,
            }]);
        }
    }

    let mut nested_else = if let Some(else_body) = maybe_else_body {
        Some(try_lower_simple_stmt_block(
            else_body,
            in_loop_with_else,
            bindings.mutated_vars,
            bindings.borrowed_params,
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

fn try_lower_simple_if_clause(
    condition: &HirExpr,
    then_body: &[HirStmt],
    nested_else: Option<Vec<RustStmt>>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<RustStmt> {
    let option_binding_pattern = |name: &str| {
        if bindings.borrowed_params.contains(name) {
            format!("Some({name})")
        } else {
            format!("Some(mut {name})")
        }
    };
    let lowered_then_body = try_lower_simple_stmt_block(
        then_body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?;

    if let Some(option_var) = detect_is_not_none_var(condition) {
        return Some(RustStmt::IfLet {
            pattern: option_binding_pattern(&option_var),
            expr: RustExpr::Ident(option_var),
            then_body: lowered_then_body,
            else_body: nested_else,
        });
    }

    if let Some(option_vars) = detect_and_not_none_vars(condition) {
        return lower_if_not_none_chain(
            &option_vars,
            lowered_then_body,
            nested_else,
            bindings.mutated_vars,
        );
    }

    if let Some(option_var) = detect_option_truthiness_alias(condition) {
        return Some(RustStmt::IfLet {
            pattern: option_binding_pattern(&option_var),
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
                pattern: option_binding_pattern(&option_var),
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

fn try_lower_simple_condition_test_expr(
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

fn try_lower_numeric_truthiness_condition_expr(expr: &HirExpr) -> Option<RustExpr> {
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
        HirExpr::Name { name, ty } => Some(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident(name.clone())),
            op: "!=".to_string(),
            right: Box::new(zero_literal_for_type(ty)?),
        }),
        HirExpr::MethodCall {
            object,
            method,
            args,
            ty,
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
            HirExpr::Name { name, ty } => Some(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident(name.clone())),
                op: "==".to_string(),
                right: Box::new(zero_literal_for_type(ty)?),
            }),
            HirExpr::MethodCall {
                object,
                method,
                args,
                ty,
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

fn try_lower_structured_compare_condition_expr(expr: &HirExpr) -> Option<RustExpr> {
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

fn try_lower_condition_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_leaf_or_name_expr(expr) {
        return Some(lowered);
    }
    match expr {
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } if method == "len" && args.is_empty() => Some(RustExpr::Cast {
            expr: Box::new(RustExpr::MethodCall {
                receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                method: "len".to_string(),
                args: vec![],
            }),
            ty: RustType::I64,
        }),
        HirExpr::Index {
            object, index, ty, ..
        } => try_lower_condition_index_operand_expr(object, index, ty),
        _ => None,
    }
}

fn try_lower_condition_index_operand_expr(
    object: &HirExpr,
    index: &HirExpr,
    result_ty: &Type,
) -> Option<RustExpr> {
    match resolve_alias_type(object.ty()) {
        Type::Dict(_, value_ty) => {
            let projection_method =
                crate::helpers::option_projection_method_for_owned_type(value_ty.as_ref());
            let lowered_key = if let HirExpr::StringLiteral(value) = index {
                RustExpr::Ident(format!("{value:?}"))
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
        Type::Str if !is_option_like_type(result_ty) => Some(RustExpr::Block {
            stmts: vec![RustStmt::LetElse {
                pattern: "Some(__indexed_char)".to_string(),
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                        method: "chars".to_string(),
                        args: vec![],
                    }),
                    method: "nth".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(try_lower_leaf_or_name_expr(index)?),
                        ty: RustType::Named("usize".to_string()),
                    }],
                },
                else_body: vec![RustStmt::Expr(RustExpr::MacroCall {
                    name: "unreachable".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Str(
                        "compiler-verified string index should be in range".to_string(),
                    ))],
                })],
            }],
            expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__indexed_char".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            })),
        }),
        _ => None,
    }
}

fn try_lower_borrowed_typevar_compare_condition(
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

fn expr_uses_borrowed_name(expr: &HirExpr, borrowed_params: &HashSet<String>) -> bool {
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

fn resolve_alias_type(ty: &Type) -> &Type {
    match ty {
        Type::Alias { body, .. } => resolve_alias_type(body),
        _ => ty,
    }
}

fn is_option_like_type(ty: &Type) -> bool {
    if let Type::Union(members) = resolve_alias_type(ty) {
        let non_none = members.iter().filter(|m| !matches!(m, Type::None)).count();
        let has_none = members.iter().any(|m| matches!(m, Type::None));
        has_none && non_none == 1
    } else {
        false
    }
}

fn detect_option_truthiness_alias(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty } = expr {
        if is_option_like_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

fn lower_if_not_none_chain(
    option_vars: &[String],
    lowered_then_body: Vec<RustStmt>,
    nested_else: Option<Vec<RustStmt>>,
    mutated_vars: &HashSet<String>,
) -> Option<RustStmt> {
    let mut chain_then = lowered_then_body;
    for option_var in option_vars.iter().rev() {
        let pattern = if mutated_vars.contains(option_var) {
            format!("Some(mut {option_var})")
        } else {
            format!("Some({option_var})")
        };
        chain_then = vec![RustStmt::IfLet {
            pattern,
            expr: RustExpr::Ident(option_var.clone()),
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

fn is_alias_equivalent_type(left: &Type, right: &Type) -> bool {
    left == right || resolve_alias_type(left) == resolve_alias_type(right)
}

fn is_none_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::None)
}

fn is_okwrap_none_expr(expr: &HirExpr) -> bool {
    matches!(
        expr,
        HirExpr::OkWrap { value, .. }
            if matches!(value.as_ref(), HirExpr::NoneLiteral) || is_none_type(value.ty())
    )
}

fn try_lower_name_ident_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, .. } = expr {
        return Some(RustExpr::Ident(name.clone()));
    }
    None
}

fn try_lower_leaf_or_name_expr(expr: &HirExpr) -> Option<RustExpr> {
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

fn try_lower_stmt_index_expr(expr: &HirExpr) -> Option<RustExpr> {
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
                RustExpr::Ident(format!("{value:?}"))
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

fn try_lower_stmt_string_concat_expr(expr: &HirExpr) -> Option<RustExpr> {
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

    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{}".repeat(parts.len()),
        args: parts
            .iter()
            .map(|part| try_lower_leaf_or_name_expr(part))
            .collect::<Option<Vec<_>>>()?,
    })
}

fn collect_stmt_string_concat_parts<'a>(expr: &'a HirExpr, parts: &mut Vec<&'a HirExpr>) {
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

fn try_lower_attribute_dict_insert_key_expr(index: &HirExpr, field_ty: &Type) -> Option<RustExpr> {
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

