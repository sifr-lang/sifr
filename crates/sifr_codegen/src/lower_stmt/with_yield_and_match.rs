use super::{
    resolve_alias_type, try_lower_leaf_or_name_expr, try_lower_simple_stmt_block, HashSet, HirExpr,
    HirPattern, HirStmt, RustExpr, RustLiteral, RustMatchArm, RustStmt, SimpleStmtBindings,
    SimpleStmtLoweringCtx, Type,
};
pub(super) fn try_lower_loop_else_stmts(
    loop_stmt: RustStmt,
    else_body: &[HirStmt],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    Some(vec![
        RustStmt::Let {
            mutable: true,
            name: "_broke".to_string(),
            ty: None,
            value: RustExpr::Literal(RustLiteral::Bool(false)),
        },
        loop_stmt,
        RustStmt::If {
            cond: RustExpr::UnaryOp {
                op: "!".to_string(),
                operand: Box::new(RustExpr::Ident("_broke".to_string())),
            },
            // Else body executes outside this loop scope. Preserve enclosing
            // loop-else context for any break/continue lowering there.
            then_body: try_lower_simple_stmt_block(
                else_body,
                in_loop_with_else,
                bindings.mutated_vars,
                bindings.borrowed_params,
                ctx,
            )?,
            else_body: None,
        },
    ])
}

pub(super) fn try_lower_simple_with_stmt(
    items: &[(String, HirExpr, bool)],
    body: &[HirStmt],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if items.iter().any(|(_, _, has_cm)| *has_cm) {
        return None;
    }

    let mut block = Vec::new();
    for (name, value, _) in items {
        block.push(RustStmt::Let {
            mutable: false,
            name: name.clone(),
            ty: None,
            value: try_lower_leaf_or_name_expr(value)?,
        });
    }

    block.extend(try_lower_simple_stmt_block(
        body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?);

    Some(vec![RustStmt::Block(block)])
}

pub(super) fn try_lower_simple_async_with_stmt(
    kind: &sifr_hir::HirAsyncWithKind,
    target: Option<&str>,
    body: &[HirStmt],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if matches!(kind, sifr_hir::HirAsyncWithKind::UserDefined { .. }) {
        return None;
    }

    if let sifr_hir::HirAsyncWithKind::TaskTimeout { duration } = kind {
        let _ = try_lower_leaf_or_name_expr(duration)?;
    }

    let mut block = Vec::new();
    if let Some(target) = target {
        let constructor = if matches!(kind, sifr_hir::HirAsyncWithKind::TaskGroup) {
            "new_task_group"
        } else {
            "new"
        };
        block.push(RustStmt::Let {
            mutable: true,
            name: target.to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "__SifrTaskScope".to_string(),
                    constructor.to_string(),
                ])),
                args: vec![],
            },
        });
    }

    block.extend(try_lower_simple_stmt_block(
        body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?);
    if let (true, Some(target)) = (
        matches!(
            kind,
            sifr_hir::HirAsyncWithKind::TaskScope | sifr_hir::HirAsyncWithKind::TaskGroup
        ),
        target,
    ) {
        let propagates_scope_failure = ctx.return_type.is_some_and(|ty| {
            matches!(ty.resolve_alias(), Type::Result(_, err) if matches!(err.resolve_alias(), Type::Class { name, .. } if name == "ScopeFailure" || name == "Error"))
        });
        let join_expr = format!("{target}.__sifr_join_all().await");
        let stmt = if propagates_scope_failure {
            format!(
                "if let Err(__sifr_scope_failure) = {join_expr} {{ return Err(__sifr_scope_failure.into()); }}"
            )
        } else {
            format!("let _ = {join_expr};")
        };
        block.push(RustStmt::Expr(RustExpr::Ident(stmt)));
    }

    Some(vec![RustStmt::Block(block)])
}

pub(super) fn try_lower_simple_yield_stmt(
    value: &HirExpr,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    let lowered_value = try_lower_leaf_or_name_expr(value)?;
    if ctx.in_generator_closure {
        return Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_value],
        }))]);
    }

    Some(vec![RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("_yields".to_string())),
        method: "push".to_string(),
        args: vec![lowered_value],
    })])
}

pub(super) fn try_lower_simple_match_stmt(
    subject: &HirExpr,
    subject_ty: &Type,
    arms: &[sifr_hir::HirMatchArm],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    let lowered_subject = try_lower_leaf_or_name_expr(subject)?;
    let subject_is_borrowed_name =
        matches!(subject, HirExpr::Name { name, .. } if bindings.borrowed_params.contains(name));
    let lowered_arms = arms
        .iter()
        .map(|arm| {
            let (pattern, arm_bindings, auto_guard) =
                if matches!(resolve_alias_type(subject_ty), Type::Str) {
                    try_lower_match_pattern_for_string_subject(&arm.pattern)?
                } else if let Some((pattern, bindings)) =
                    try_lower_union_class_match_pattern(&arm.pattern, subject_ty)
                {
                    (pattern, bindings, None)
                } else {
                    let (pattern, arm_bindings) = try_lower_match_pattern(&arm.pattern)?;
                    (pattern, arm_bindings, None)
                };
            let mut lowered_guard = arm.guard.as_ref().and_then(try_lower_leaf_or_name_expr);
            if subject_is_borrowed_name {
                let copy_captures = collect_copy_capture_names(&arm.pattern);
                if !copy_captures.is_empty() {
                    lowered_guard =
                        lowered_guard.map(|guard| deref_guard_copy_captures(guard, &copy_captures));
                }
            }
            let guard = match (auto_guard, lowered_guard) {
                (Some(left), Some(right)) => Some(RustExpr::BinOp {
                    left: Box::new(left),
                    op: "&&".to_string(),
                    right: Box::new(right),
                }),
                (Some(left), None) => Some(left),
                (None, Some(right)) => Some(right),
                (None, None) => None,
            };
            let body = try_lower_simple_stmt_block(
                &arm.body,
                in_loop_with_else,
                bindings.mutated_vars,
                bindings.borrowed_params,
                ctx,
            )?;
            Some(RustMatchArm {
                pattern,
                bindings: arm_bindings,
                guard,
                body,
            })
        })
        .collect::<Option<Vec<_>>>()?;

    Some(vec![RustStmt::Match {
        expr: lowered_subject,
        arms: lowered_arms,
    }])
}

pub(super) fn try_lower_match_pattern_for_string_subject(
    pattern: &HirPattern,
) -> Option<(String, Vec<String>, Option<RustExpr>)> {
    match pattern {
        HirPattern::Literal {
            value: HirExpr::StringLiteral(_),
        } => Some((
            "__s".to_string(),
            vec![],
            Some(try_lower_string_literal_match_guard(pattern)?),
        )),
        HirPattern::Or { patterns } => {
            if patterns.iter().any(|p| matches!(p, HirPattern::Wildcard)) {
                return Some(("_".to_string(), vec![], None));
            }
            Some((
                "__s".to_string(),
                vec![],
                Some(try_lower_string_literal_match_guard(pattern)?),
            ))
        }
        _ => {
            let (pattern, bindings) = try_lower_match_pattern(pattern)?;
            Some((pattern, bindings, None))
        }
    }
}

pub(super) fn try_lower_string_literal_match_guard(pattern: &HirPattern) -> Option<RustExpr> {
    match pattern {
        HirPattern::Literal {
            value: HirExpr::StringLiteral(expected),
        } => Some(RustExpr::BinOp {
            left: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__s".to_string())),
                method: "as_str".to_string(),
                args: vec![],
            }),
            op: "==".to_string(),
            right: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Literal(RustLiteral::Str(expected.clone()))),
                method: "as_str".to_string(),
                args: vec![],
            }),
        }),
        HirPattern::Or { patterns } => {
            let mut guards = Vec::with_capacity(patterns.len());
            for pattern in patterns {
                guards.push(try_lower_string_literal_match_guard(pattern)?);
            }
            guards.into_iter().reduce(|left, right| RustExpr::BinOp {
                left: Box::new(left),
                op: "||".to_string(),
                right: Box::new(right),
            })
        }
        _ => None,
    }
}

pub(super) fn try_lower_match_pattern(pattern: &HirPattern) -> Option<(String, Vec<String>)> {
    match pattern {
        HirPattern::Wildcard => Some(("_".to_string(), vec![])),
        HirPattern::Capture { name, .. } => Some((name.clone(), vec![name.clone()])),
        HirPattern::Literal { value } => Some((try_lower_match_literal_pattern(value)?, vec![])),
        HirPattern::None => Some(("None".to_string(), vec![])),
        HirPattern::Value { path } => Some((path.join("::"), vec![])),
        HirPattern::Or { patterns } => {
            let mut rendered = Vec::new();
            for p in patterns {
                let (pat, binds) = try_lower_match_pattern(p)?;
                // Keep conservative support only for OR-patterns without bindings.
                if !binds.is_empty() {
                    return None;
                }
                rendered.push(pat);
            }
            Some((rendered.join(" | "), vec![]))
        }
        HirPattern::Tuple { elements } => {
            let mut rendered = Vec::new();
            let mut bindings = Vec::new();
            for element in elements {
                let (pat, binds) = try_lower_match_pattern(element)?;
                rendered.push(pat);
                bindings.extend(binds);
            }
            Some((format!("({})", rendered.join(", ")), bindings))
        }
        HirPattern::Class { class_name, fields } => {
            let mut rendered_fields = Vec::new();
            let mut bindings = Vec::new();
            for (field_name, field_pattern) in fields {
                let (field_pat, field_binds) = try_lower_match_pattern(field_pattern)?;
                rendered_fields.push(format!("{field_name}: {field_pat}"));
                bindings.extend(field_binds);
            }
            if rendered_fields.is_empty() {
                Some((format!("{class_name} {{ .. }}"), bindings))
            } else {
                Some((
                    format!("{class_name} {{ {}, .. }}", rendered_fields.join(", ")),
                    bindings,
                ))
            }
        }
    }
}

pub(super) fn try_lower_union_class_match_pattern(
    pattern: &HirPattern,
    subject_ty: &Type,
) -> Option<(String, Vec<String>)> {
    let Type::Union(members) = resolve_alias_type(subject_ty) else {
        return None;
    };
    let HirPattern::Class { class_name, fields } = pattern else {
        return None;
    };

    let target_ty = match class_name.as_str() {
        "int" => Some(Type::Int),
        "str" => Some(Type::Str),
        "float" => Some(Type::Float),
        "bool" => Some(Type::Bool),
        other => members
            .iter()
            .find(|m| matches!(m, Type::Class { name, .. } if name == other))
            .cloned(),
    }?;
    if !members.contains(&target_ty) {
        return None;
    }

    let enum_name = resolve_alias_type(subject_ty).union_enum_name();
    let variant_name = target_ty.union_variant_name();
    if fields.is_empty() {
        return Some((format!("{enum_name}::{variant_name}(..)"), vec![]));
    }
    if !matches!(target_ty, Type::Class { .. }) {
        return None;
    }
    let mut rendered_fields = Vec::new();
    let mut bindings = Vec::new();
    for (field_name, field_pattern) in fields {
        let (field_pat, field_binds) = try_lower_match_pattern(field_pattern)?;
        rendered_fields.push(format!("{field_name}: {field_pat}"));
        bindings.extend(field_binds);
    }
    Some((
        format!(
            "{enum_name}::{variant_name}({class_name} {{ {}, .. }})",
            rendered_fields.join(", ")
        ),
        bindings,
    ))
}

pub(super) fn is_copy_capture_type(ty: &Type) -> bool {
    matches!(
        resolve_alias_type(ty),
        Type::Int
            | Type::LiteralInt(_)
            | Type::Float
            | Type::Bool
            | Type::LiteralBool(_)
            | Type::Decimal
    )
}

pub(super) fn collect_copy_capture_names(pattern: &HirPattern) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_copy_capture_names_inner(pattern, &mut names);
    names
}

pub(super) fn collect_copy_capture_names_inner(pattern: &HirPattern, out: &mut HashSet<String>) {
    match pattern {
        HirPattern::Capture { name, ty } if is_copy_capture_type(ty) => {
            out.insert(name.clone());
        }
        HirPattern::Class { fields, .. } => {
            for (_, field_pattern) in fields {
                collect_copy_capture_names_inner(field_pattern, out);
            }
        }
        HirPattern::Tuple { elements } => {
            for element in elements {
                collect_copy_capture_names_inner(element, out);
            }
        }
        HirPattern::Or { patterns } => {
            for pattern in patterns {
                collect_copy_capture_names_inner(pattern, out);
            }
        }
        _ => {}
    }
}

pub(super) fn deref_guard_copy_captures(expr: RustExpr, captures: &HashSet<String>) -> RustExpr {
    match expr {
        RustExpr::Ident(name) if captures.contains(&name) => {
            RustExpr::Deref(Box::new(RustExpr::Ident(name)))
        }
        RustExpr::BinOp { left, op, right } => RustExpr::BinOp {
            left: Box::new(deref_guard_copy_captures(*left, captures)),
            op,
            right: Box::new(deref_guard_copy_captures(*right, captures)),
        },
        RustExpr::UnaryOp { op, operand } => RustExpr::UnaryOp {
            op,
            operand: Box::new(deref_guard_copy_captures(*operand, captures)),
        },
        RustExpr::FnCall { func, args } => RustExpr::FnCall {
            func: Box::new(deref_guard_copy_captures(*func, captures)),
            args: args
                .into_iter()
                .map(|arg| deref_guard_copy_captures(arg, captures))
                .collect(),
        },
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        } => RustExpr::MethodCall {
            receiver: Box::new(deref_guard_copy_captures(*receiver, captures)),
            method,
            args: args
                .into_iter()
                .map(|arg| deref_guard_copy_captures(arg, captures))
                .collect(),
        },
        RustExpr::Ref { mutable, expr } => RustExpr::Ref {
            mutable,
            expr: Box::new(deref_guard_copy_captures(*expr, captures)),
        },
        RustExpr::Deref(expr) => {
            RustExpr::Deref(Box::new(deref_guard_copy_captures(*expr, captures)))
        }
        RustExpr::Cast { expr, ty } => RustExpr::Cast {
            expr: Box::new(deref_guard_copy_captures(*expr, captures)),
            ty,
        },
        RustExpr::Field { expr, field } => RustExpr::Field {
            expr: Box::new(deref_guard_copy_captures(*expr, captures)),
            field,
        },
        RustExpr::Index { expr, index } => RustExpr::Index {
            expr: Box::new(deref_guard_copy_captures(*expr, captures)),
            index: Box::new(deref_guard_copy_captures(*index, captures)),
        },
        other => other,
    }
}

pub(super) fn try_lower_match_literal_pattern(expr: &HirExpr) -> Option<String> {
    match expr {
        HirExpr::IntLiteral(v) => Some(v.to_string()),
        HirExpr::FloatLiteral(v) => {
            let mut s = v.to_string();
            if !s.contains('.') {
                s.push_str(".0");
            }
            Some(s)
        }
        HirExpr::StringLiteral(s) => Some(format!("{s:?}")),
        HirExpr::BoolLiteral(v) => Some(v.to_string()),
        HirExpr::NoneLiteral => Some("None".to_string()),
        HirExpr::EnumVariant {
            enum_name, variant, ..
        } => Some(format!("{enum_name}::{variant}")),
        _ => None,
    }
}
