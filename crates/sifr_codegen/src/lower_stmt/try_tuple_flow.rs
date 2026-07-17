use super::{
    try_lower_leaf_or_name_expr, try_lower_simple_stmt_with_ctx, HashSet, HirExceptHandler,
    HirExpr, HirStmt, RustExpr, RustLiteral, RustStmt, RustType, SimpleStmtBindings,
    SimpleStmtLoweringCtx, Type,
};
pub(super) fn try_lower_simple_try_except_stmt(
    body: &[HirStmt],
    handlers: &[HirExceptHandler],
    body_error_types: &[Type],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if crate::try_error_carrier::exact_try_error_carrier(body_error_types)
        .is_some_and(|carrier| carrier.rust_type() != "Error")
    {
        return None;
    }
    if handlers.len() != 1 {
        return None;
    }
    let handler = handlers.first()?;
    if handler
        .error_type
        .as_deref()
        .is_some_and(|error_type| error_type != "Error")
    {
        return None;
    }
    if !body.iter().all(is_simple_try_except_body_stmt)
        || !handler.body.iter().all(is_simple_try_except_body_stmt)
    {
        return None;
    }
    if !body.iter().any(stmt_has_result_flow) {
        return None;
    }

    let lowered_try_body = try_lower_simple_stmt_block(
        body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?;
    let lowered_handler_body = try_lower_simple_stmt_block(
        &handler.body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?;

    let mut closure_body = lowered_try_body;
    closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![RustExpr::Literal(RustLiteral::Unit)],
    })));
    let handler_name = handler.name.clone().unwrap_or_else(|| "_e".to_string());

    Some(vec![
        RustStmt::Let {
            mutable: false,
            name: "__sifr_try_res".to_string(),
            ty: Some(RustType::Result(
                Box::new(RustType::Unit),
                Box::new(RustType::Named("Error".to_string())),
            )),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::ClosureBlock {
                    params: vec![],
                    body: closure_body,
                    is_move: false,
                    is_async: false,
                }),
                args: vec![],
            },
        },
        RustStmt::IfLet {
            pattern: format!("Err({handler_name})"),
            expr: RustExpr::Ident("__sifr_try_res".to_string()),
            then_body: lowered_handler_body,
            else_body: None,
        },
    ])
}

pub(super) fn is_simple_try_except_body_stmt(stmt: &HirStmt) -> bool {
    matches!(
        stmt,
        HirStmt::Expr { .. }
            | HirStmt::Let { .. }
            | HirStmt::Assign { .. }
            | HirStmt::AugAssign { .. }
            | HirStmt::AttributeAugAssign { .. }
            | HirStmt::FieldAssign { .. }
            | HirStmt::NestedFieldAssign { .. }
            | HirStmt::Assert { .. }
            | HirStmt::Raise { .. }
            | HirStmt::TupleUnpack { .. }
            | HirStmt::StarUnpack { .. }
            | HirStmt::SubscriptAssign { .. }
            | HirStmt::NestedSubscriptAssign { .. }
            | HirStmt::AttributeNestedSubscriptAssign { .. }
            | HirStmt::SubscriptAugAssign { .. }
            | HirStmt::AttributeSubscriptAssign { .. }
            | HirStmt::Delete { .. }
            | HirStmt::Pass
    )
}

pub(super) fn stmt_has_result_flow(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Raise { .. } => true,
        HirStmt::Expr { expr } => expr_has_result_flow(expr),
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::NestedFieldAssign { value, .. } => expr_has_result_flow(value),
        HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::SubscriptAssign { value, .. }
        | HirStmt::NestedSubscriptAssign { value, .. }
        | HirStmt::AttributeNestedSubscriptAssign { value, .. }
        | HirStmt::SubscriptAugAssign { value, .. }
        | HirStmt::AttributeSubscriptAssign { value, .. } => expr_has_result_flow(value),
        HirStmt::Assert { test, msg } => {
            expr_has_result_flow(test) || msg.as_ref().is_some_and(expr_has_result_flow)
        }
        HirStmt::TupleUnpack { value, .. } | HirStmt::StarUnpack { value, .. } => {
            expr_has_result_flow(value)
        }
        HirStmt::Delete { object, index } => {
            expr_has_result_flow(object) || expr_has_result_flow(index)
        }
        HirStmt::Pass => false,
        HirStmt::Return { .. }
        | HirStmt::If { .. }
        | HirStmt::While { .. }
        | HirStmt::For { .. }
        | HirStmt::AsyncFor { .. }
        | HirStmt::Break
        | HirStmt::Continue
        | HirStmt::TryExcept { .. }
        | HirStmt::TryFinally { .. }
        | HirStmt::With { .. }
        | HirStmt::AsyncWith { .. }
        | HirStmt::Match { .. }
        | HirStmt::Yield { .. }
        | HirStmt::NestedFunction { .. } => false,
    }
}

pub(super) fn expr_has_result_flow(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::QuestionMark { .. } | HirExpr::OkWrap { .. } | HirExpr::ErrWrap { .. } => true,
        HirExpr::Await { value, .. } => expr_has_result_flow(value),
        HirExpr::UnaryOp { operand, .. } => expr_has_result_flow(operand),
        HirExpr::BinOp { left, right, .. } => {
            expr_has_result_flow(left) || expr_has_result_flow(right)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => expr_has_result_flow(left) || comparators.iter().any(expr_has_result_flow),
        HirExpr::BoolOp { values, .. } => values.iter().any(expr_has_result_flow),
        HirExpr::Call { args, .. }
        | HirExpr::PythonCall { args, .. }
        | HirExpr::IntrinsicCall { args, .. }
        | HirExpr::IteratorCall { args, .. }
        | HirExpr::MethodCall { args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::SuperCall { args, .. } => args.iter().any(expr_has_result_flow),
        HirExpr::Index { object, index, .. } => {
            expr_has_result_flow(object) || expr_has_result_flow(index)
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            expr_has_result_flow(object)
                || start.as_ref().is_some_and(|e| expr_has_result_flow(e))
                || stop.as_ref().is_some_and(|e| expr_has_result_flow(e))
                || step.as_ref().is_some_and(|e| expr_has_result_flow(e))
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_has_result_flow(condition)
                || expr_has_result_flow(then_expr)
                || expr_has_result_flow(else_expr)
        }
        HirExpr::TupleLiteral { elements, .. }
        | HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. } => elements.iter().any(expr_has_result_flow),
        HirExpr::DictLiteral { keys, values, .. } => keys
            .iter()
            .zip(values.iter())
            .any(|(k, v)| expr_has_result_flow(k) || expr_has_result_flow(v)),
        HirExpr::FString { parts, .. } => parts.iter().any(|part| match part {
            sifr_ir::HirFStringPart::Literal(_) => false,
            sifr_ir::HirFStringPart::Expr(e) => expr_has_result_flow(e),
        }),
        HirExpr::Lambda { body, .. } => expr_has_result_flow(body),
        HirExpr::WalrusExpr { value, .. } => expr_has_result_flow(value),
        HirExpr::FieldAccess { object, .. } => expr_has_result_flow(object),
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => expr_has_result_flow(element) || expr_has_result_flow(collection),
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            expr_has_result_flow(start)
                || expr_has_result_flow(end)
                || step.as_ref().is_some_and(|e| expr_has_result_flow(e))
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            expr_has_result_flow(expr)
                || generators.iter().any(|(_, iter, cond)| {
                    expr_has_result_flow(iter) || cond.as_ref().is_some_and(expr_has_result_flow)
                })
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            expr_has_result_flow(key_expr)
                || expr_has_result_flow(val_expr)
                || generators.iter().any(|(_, iter, cond)| {
                    expr_has_result_flow(iter) || cond.as_ref().is_some_and(expr_has_result_flow)
                })
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            expr_has_result_flow(expr)
                || expr_has_result_flow(iter)
                || filter.as_ref().is_some_and(|c| expr_has_result_flow(c))
        }
        HirExpr::Name { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => false,
    }
}

pub(super) fn try_lower_simple_stmt_block(
    stmts: &[HirStmt],
    in_loop_with_else: bool,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    let mut lowered = Vec::new();
    for stmt in stmts {
        lowered.extend(try_lower_simple_stmt_with_ctx(
            stmt,
            in_loop_with_else,
            mutated_vars,
            borrowed_params,
            ctx,
        )?);
    }
    Some(lowered)
}

pub(crate) fn tuple_unpack_pattern(
    targets: &[sifr_ir::HirTupleTarget],
    mutated_vars: &HashSet<String>,
) -> Option<String> {
    let mut names = Vec::new();
    for target in targets {
        let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding else {
            return None;
        };
        if mutated_vars.contains(name) {
            names.push(format!("mut {name}"));
        } else {
            names.push(name.clone());
        }
    }
    Some(format!("({})", names.join(", ")))
}

pub(super) fn tuple_unpack_field_target_expr(object: &str, field: &str) -> RustExpr {
    RustExpr::Field {
        expr: Box::new(RustExpr::Ident(object.to_string())),
        field: field.to_string(),
    }
}

pub(super) fn lower_tuple_target_assignments(
    targets: &[sifr_ir::HirTupleTarget],
    temp_names: &[String],
    mutated_vars: &HashSet<String>,
) -> Vec<RustStmt> {
    let mut lowered = Vec::new();
    for (target, temp_name) in targets.iter().zip(temp_names.iter()) {
        match &target.binding {
            sifr_ir::HirTupleTargetBinding::Name(name) => {
                if target.rebind_existing {
                    lowered.push(RustStmt::Assign {
                        target: RustExpr::Ident(name.clone()),
                        value: RustExpr::Ident(temp_name.clone()),
                    });
                } else {
                    lowered.push(RustStmt::Let {
                        mutable: mutated_vars.contains(name),
                        name: name.clone(),
                        ty: None,
                        value: RustExpr::Ident(temp_name.clone()),
                    });
                }
            }
            sifr_ir::HirTupleTargetBinding::Field { object, field } => {
                lowered.push(RustStmt::Assign {
                    target: tuple_unpack_field_target_expr(object, field),
                    value: RustExpr::Ident(temp_name.clone()),
                });
            }
        }
    }
    lowered
}

pub(super) fn try_lower_simple_tuple_unpack_stmt(
    targets: &[sifr_ir::HirTupleTarget],
    value: &HirExpr,
    mutated_vars: &HashSet<String>,
    source_is_borrowed: bool,
) -> Option<Vec<RustStmt>> {
    if targets.is_empty() {
        return None;
    }
    if targets.iter().any(|target| target.rebind_existing) {
        return None;
    }
    let pattern = tuple_unpack_pattern(targets, mutated_vars)?;
    let lowered_value = tuple_unpack_source_expr(
        value,
        try_lower_leaf_or_name_expr(value)?,
        source_is_borrowed,
    );
    Some(vec![RustStmt::LetPattern {
        pattern,
        value: lowered_value,
    }])
}

fn tuple_unpack_source_expr(
    value: &HirExpr,
    lowered_value: RustExpr,
    source_is_borrowed: bool,
) -> RustExpr {
    if source_is_borrowed
        && matches!(value, HirExpr::Name { ty, .. } if ty.ownership() == sifr_type_system::OwnershipKind::Move)
    {
        RustExpr::Clone(Box::new(RustExpr::Paren(Box::new(RustExpr::Deref(
            Box::new(lowered_value),
        )))))
    } else {
        lowered_value
    }
}

pub(crate) fn tuple_unpack_source_is_borrowed(
    value: &HirExpr,
    borrowed_params: &HashSet<String>,
    mut_borrowed_params: &HashSet<String>,
) -> bool {
    matches!(
        value,
        HirExpr::Name { name, .. }
            if borrowed_params.contains(name) || mut_borrowed_params.contains(name)
    )
}

pub(crate) fn lower_tuple_unpack_targets(
    targets: &[sifr_ir::HirTupleTarget],
    value: &HirExpr,
    lowered_value: RustExpr,
    mutated_vars: &HashSet<String>,
    source_is_borrowed: bool,
) -> Vec<RustStmt> {
    let lowered_value = tuple_unpack_source_expr(value, lowered_value, source_is_borrowed);
    if targets.iter().all(|target| !target.rebind_existing) {
        if let Some(pattern) = tuple_unpack_pattern(targets, mutated_vars) {
            return vec![RustStmt::LetPattern {
                pattern,
                value: lowered_value,
            }];
        }
    }

    let temp_names = targets
        .iter()
        .enumerate()
        .map(|(index, _)| format!("__sifr_tuple_unpack_{index}"))
        .collect::<Vec<_>>();
    let temp_pattern = format!("({})", temp_names.join(", "));

    let mut lowered = vec![RustStmt::LetPattern {
        pattern: temp_pattern,
        value: lowered_value,
    }];

    lowered.extend(lower_tuple_target_assignments(
        targets,
        &temp_names,
        mutated_vars,
    ));

    lowered
}

pub(super) fn try_lower_simple_star_unpack_stmt(
    before: &[(String, Type)],
    star: &(String, Type),
    after: &[(String, Type)],
    value: &HirExpr,
) -> Option<Vec<RustStmt>> {
    let lowered_value = try_lower_leaf_or_name_expr(value)?;
    let source_plan = crate::helpers::plan_iterator_ownership(value);
    let mut lowered = vec![RustStmt::Let {
        mutable: false,
        name: "_star_tmp".to_string(),
        ty: None,
        value: match source_plan.source_access_mode {
            crate::helpers::SourceAccessMode::Preserve => RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_value),
            },
            crate::helpers::SourceAccessMode::Consume => lowered_value,
        },
    }];

    let tmp_ident = || RustExpr::Ident("_star_tmp".to_string());
    let tmp_len = || RustExpr::MethodCall {
        receiver: Box::new(tmp_ident()),
        method: "len".to_string(),
        args: vec![],
    };

    for (idx, (name, element_ty)) in before.iter().enumerate() {
        let indexed_expr = RustExpr::Index {
            expr: Box::new(tmp_ident()),
            index: Box::new(RustExpr::Literal(RustLiteral::Int(
                i64::try_from(idx).ok()?,
            ))),
        };
        let extracted_expr = if crate::helpers::is_copy_type_for_codegen(element_ty) {
            indexed_expr
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(indexed_expr),
                method: "clone".to_string(),
                args: vec![],
            }
        };
        lowered.push(RustStmt::Let {
            mutable: false,
            name: name.clone(),
            ty: None,
            value: extracted_expr,
        });
    }

    let (star_name, _) = star;
    let slice_end = if after.is_empty() {
        tmp_len()
    } else {
        RustExpr::BinOp {
            left: Box::new(tmp_len()),
            op: "-".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(
                i64::try_from(after.len()).ok()?,
            ))),
        }
    };
    lowered.push(RustStmt::Let {
        mutable: false,
        name: star_name.clone(),
        ty: None,
        value: RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Index {
                expr: Box::new(tmp_ident()),
                index: Box::new(RustExpr::Range {
                    start: Box::new(RustExpr::Literal(RustLiteral::Int(
                        i64::try_from(before.len()).ok()?,
                    ))),
                    end: Box::new(slice_end),
                }),
            }),
            method: "to_vec".to_string(),
            args: vec![],
        },
    });

    for (idx, (name, element_ty)) in after.iter().enumerate() {
        let indexed_expr = RustExpr::Index {
            expr: Box::new(tmp_ident()),
            index: Box::new(RustExpr::BinOp {
                left: Box::new(tmp_len()),
                op: "-".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Int(
                    i64::try_from(after.len() - idx).ok()?,
                ))),
            }),
        };
        let extracted_expr = if crate::helpers::is_copy_type_for_codegen(element_ty) {
            indexed_expr
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(indexed_expr),
                method: "clone".to_string(),
                args: vec![],
            }
        };
        lowered.push(RustStmt::Let {
            mutable: false,
            name: name.clone(),
            ty: None,
            value: extracted_expr,
        });
    }

    Some(lowered)
}
