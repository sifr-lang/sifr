use super::{
    body_calls_function, collect_locally_defined_vars, collect_mutated_vars,
    collect_referenced_vars_with_types, is_option_like_type, resolve_alias_type,
    try_lower_expr_stmt_with_bindings, try_lower_leaf_or_name_expr, try_lower_name_ident_expr,
    try_lower_simple_assign_value, try_lower_simple_async_with_stmt,
    try_lower_simple_attribute_nested_subscript_assign_stmt,
    try_lower_simple_attribute_subscript_assign_stmt, try_lower_simple_augassign_stmt,
    try_lower_simple_condition_test_expr, try_lower_simple_delete_stmt,
    try_lower_simple_field_assign_stmt, try_lower_simple_for_stmt, try_lower_simple_if_stmt,
    try_lower_simple_let_value, try_lower_simple_match_stmt,
    try_lower_simple_nested_subscript_assign_stmt, try_lower_simple_return_stmt,
    try_lower_simple_star_unpack_stmt, try_lower_simple_subscript_assign_stmt,
    try_lower_simple_subscript_augassign_stmt, try_lower_simple_try_except_stmt,
    try_lower_simple_tuple_unpack_stmt, try_lower_simple_while_stmt, try_lower_simple_with_stmt,
    try_lower_simple_yield_stmt, HashMap, HashSet, HirExpr, HirFunction, HirStmt, MethodKind,
    RustExpr, RustLiteral, RustParam, RustStmt, RustType, SimpleForStmtParts, SimpleStmtBindings,
    SimpleStmtLoweringCtx, Type,
};
pub(crate) fn try_lower_simple_stmt_with_ctx(
    stmt: &HirStmt,
    in_loop_with_else: bool,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    try_lower_simple_stmt_with_ctx_and_bindings(
        stmt,
        in_loop_with_else,
        SimpleStmtBindings {
            mutated_vars,
            borrowed_params,
            mut_borrowed_params: &HashSet::new(),
            local_binding_types: &HashMap::new(),
            recursive_fields: &HashSet::new(),
        },
        ctx,
    )
}

pub(super) fn try_lower_simple_stmt_with_ctx_and_bindings(
    stmt: &HirStmt,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    match stmt {
        HirStmt::Expr { expr } => {
            try_lower_expr_stmt_with_bindings(expr, bindings.local_binding_types)
        }
        HirStmt::Let {
            name, ty, value, ..
        } => {
            let effective_ty = bindings.local_binding_types.get(name).unwrap_or(ty);
            if matches!(resolve_alias_type(effective_ty), Type::Iterable(_)) {
                return None;
            }
            let lowered_value = try_lower_simple_let_value(effective_ty, value)?;
            Some(vec![RustStmt::Let {
                mutable: bindings.mutated_vars.contains(name)
                    || crate::stmt_support_emitter::should_force_mutable_binding(effective_ty),
                name: name.clone(),
                ty: if name == "_" || should_omit_local_type_annotation(effective_ty, value) {
                    None
                } else if let Some(ty) = exact_int_floor_result_local_rust_type(effective_ty, value)
                {
                    Some(ty)
                } else {
                    Some(crate::sifr_type_to_rust_type(effective_ty))
                },
                value: lowered_value,
            }])
        }
        HirStmt::Assign { name, value }
            if try_lower_simple_assign_value(value, bindings.borrowed_params).is_some() =>
        {
            let lowered_value = try_lower_simple_assign_value(value, bindings.borrowed_params)?;
            let lowered_value = coerce_simple_assign_value_for_target_type(
                bindings.local_binding_types.get(name),
                value,
                lowered_value,
            );
            Some(vec![RustStmt::Assign {
                target: crate::RustExpr::Ident(name.clone()),
                value: lowered_value,
            }])
        }
        HirStmt::AugAssign { name, op, value } => {
            try_lower_simple_augassign_stmt(crate::RustExpr::Ident(name.clone()), op, value)
        }
        HirStmt::AttributeAugAssign {
            object,
            field,
            op,
            value,
        } => try_lower_simple_augassign_stmt(
            RustExpr::Field {
                expr: Box::new(RustExpr::Ident(object.clone())),
                field: field.clone(),
            },
            op,
            value,
        ),
        HirStmt::FieldAssign {
            object,
            field,
            field_ty,
            value,
        } => try_lower_simple_field_assign_stmt(object, field, field_ty, value),
        HirStmt::NestedFieldAssign { .. } => None,
        HirStmt::Return { value: None } => {
            if ctx.in_display_impl {
                return None;
            }
            if ctx.return_type.is_some_and(is_option_like_type) {
                Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                    RustLiteral::None,
                )))])
            } else {
                Some(vec![RustStmt::Return(None)])
            }
        }
        HirStmt::Return { value: Some(value) } => {
            let mut lowered = try_lower_simple_return_stmt(value, ctx)?;
            if matches!(value, HirExpr::Name { name, ty, .. }
                if bindings.borrowed_params.contains(name)
                    && ty.ownership() != sifr_type_system::OwnershipKind::Copy)
            {
                if let Some(RustStmt::Return(Some(returned))) = lowered.first_mut() {
                    *returned = RustExpr::Clone(Box::new(returned.clone()));
                }
            }
            Some(lowered)
        }
        HirStmt::Assert { test, msg } => {
            let lowered_msg = if let Some(msg_expr) = msg.as_ref() {
                Some(if is_option_like_type(msg_expr.ty()) {
                    RustExpr::MethodCall {
                        receiver: Box::new(try_lower_name_ident_expr(msg_expr)?),
                        method: "map_or".to_string(),
                        args: vec![
                            RustExpr::Literal(RustLiteral::Str("None".to_string())),
                            RustExpr::Closure {
                                params: vec![RustParam::Named {
                                    name: "_v".to_string(),
                                    ty: RustType::Named("_".to_string()),
                                }],
                                body: Box::new(RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str: "{}".to_string(),
                                    args: vec![RustExpr::Ident("_v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    }
                } else {
                    try_lower_leaf_or_name_expr(msg_expr)?
                })
            } else {
                None
            };
            Some(vec![RustStmt::Assert {
                cond: try_lower_simple_condition_test_expr(test, bindings.borrowed_params)?,
                msg: lowered_msg,
            }])
        }
        HirStmt::Raise { value } => {
            let lowered = try_lower_leaf_or_name_expr(value)?;
            let lowered = coerce_simple_raised_error(value, lowered, ctx.return_type);
            Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![lowered],
            }))])
        }
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body: maybe_else_body,
        } => try_lower_simple_if_stmt(
            condition,
            then_body,
            elif_clauses,
            maybe_else_body.as_deref(),
            in_loop_with_else,
            bindings,
            ctx,
        ),
        HirStmt::While {
            condition,
            body,
            else_body,
        } => try_lower_simple_while_stmt(
            condition,
            body,
            else_body.as_deref(),
            in_loop_with_else,
            bindings,
            ctx,
        ),
        HirStmt::For {
            target,
            target_ty,
            iter,
            body,
            else_body,
            ..
        } => try_lower_simple_for_stmt(
            SimpleForStmtParts {
                target,
                target_ty,
                iter,
                body,
                else_body: else_body.as_deref(),
                in_loop_with_else,
            },
            bindings,
            ctx,
        ),
        HirStmt::TupleUnpack { targets, value } => {
            let source_is_borrowed = matches!(
                value,
                HirExpr::Name { name, .. } if bindings.borrowed_params.contains(name)
            );
            try_lower_simple_tuple_unpack_stmt(
                targets,
                value,
                bindings.mutated_vars,
                source_is_borrowed,
            )
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
        } => try_lower_simple_star_unpack_stmt(before, star, after, value),
        HirStmt::AttributeSubscriptAssign {
            object,
            field,
            index,
            value,
            field_ty,
        } => {
            try_lower_simple_attribute_subscript_assign_stmt(object, field, index, value, field_ty)
        }
        HirStmt::SubscriptAssign {
            object,
            index,
            value,
            object_ty,
        } => try_lower_simple_subscript_assign_stmt(object, index, value, object_ty),
        HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            object_ty,
        } => try_lower_simple_nested_subscript_assign_stmt(
            object,
            outer_index,
            inner_index,
            value,
            object_ty,
        ),
        HirStmt::AttributeNestedSubscriptAssign {
            object,
            field,
            outer_index,
            inner_index,
            value,
            field_ty,
        } => try_lower_simple_attribute_nested_subscript_assign_stmt(
            object,
            field,
            outer_index,
            inner_index,
            value,
            field_ty,
        ),
        HirStmt::SubscriptAugAssign {
            object,
            index,
            op,
            value,
            object_ty,
        } => try_lower_simple_subscript_augassign_stmt(object, index, op, value, object_ty),
        HirStmt::Delete { object, index } => try_lower_simple_delete_stmt(object, index),
        HirStmt::Yield { value } => try_lower_simple_yield_stmt(value, ctx),
        HirStmt::With { items, body } => {
            try_lower_simple_with_stmt(items, body, in_loop_with_else, bindings, ctx)
        }
        HirStmt::AsyncWith { kind, target, body } => try_lower_simple_async_with_stmt(
            kind,
            target.as_deref(),
            body,
            in_loop_with_else,
            bindings,
            ctx,
        ),
        HirStmt::Match {
            subject,
            subject_ty,
            arms,
        } => {
            try_lower_simple_match_stmt(subject, subject_ty, arms, in_loop_with_else, bindings, ctx)
        }
        HirStmt::NestedFunction {
            func,
            move_captures,
            capture_clones,
        } => try_lower_simple_nested_function_stmt(
            func,
            *move_captures,
            capture_clones,
            in_loop_with_else,
            bindings,
        ),
        HirStmt::TryExcept {
            body,
            handlers,
            body_error_types,
        } => try_lower_simple_try_except_stmt(
            body,
            handlers,
            body_error_types,
            in_loop_with_else,
            bindings,
            ctx,
        ),
        HirStmt::TryFinally { .. } => None,
        HirStmt::Pass => Some(vec![]),
        HirStmt::Continue => Some(vec![RustStmt::Continue]),
        HirStmt::Break => {
            if in_loop_with_else {
                Some(vec![
                    RustStmt::Assign {
                        target: crate::RustExpr::Ident("_broke".to_string()),
                        value: crate::RustExpr::Literal(crate::RustLiteral::Bool(true)),
                    },
                    RustStmt::Break,
                ])
            } else {
                Some(vec![RustStmt::Break])
            }
        }
        _ => None,
    }
}

fn coerce_simple_raised_error(
    value: &HirExpr,
    lowered: RustExpr,
    return_type: Option<&Type>,
) -> RustExpr {
    let Some(Type::Result(_, target)) = return_type.map(Type::resolve_alias) else {
        return lowered;
    };
    let source_name = crate::render_type(&crate::sifr_type_to_rust_type(value.ty()));
    let target_name = crate::render_type(&crate::sifr_type_to_rust_type(target));
    if source_name == target_name {
        lowered
    } else {
        RustExpr::MethodCall {
            receiver: Box::new(lowered),
            method: "into".to_string(),
            args: Vec::new(),
        }
    }
}

pub(super) fn coerce_simple_assign_value_for_target_type(
    target_ty: Option<&Type>,
    value: &HirExpr,
    lowered_value: RustExpr,
) -> RustExpr {
    let Some(target_ty) = target_ty else {
        return lowered_value;
    };
    if !crate::helpers::is_option_type(target_ty) {
        return lowered_value;
    }
    if matches!(value, HirExpr::NoneLiteral) || matches!(value.ty(), Type::None) {
        return RustExpr::Literal(RustLiteral::None);
    }
    if crate::helpers::is_option_type(value.ty()) {
        return lowered_value;
    }
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
        args: vec![lowered_value],
    }
}

pub(super) fn should_omit_local_type_annotation(ty: &Type, value: &HirExpr) -> bool {
    match (ty, value) {
        (resolved_ty, HirExpr::Call { func, args, .. })
            if matches!(
                crate::resolve_alias_type_for_plain_call(resolved_ty),
                Type::Set(_)
            ) && func == "set"
                && args.is_empty() =>
        {
            true
        }
        (
            Type::Alias {
                name: alias_name,
                body,
                ..
            },
            HirExpr::Call { func, args, .. },
        ) if func == alias_name
            && args.is_empty()
            && alias_name.starts_with("__sifr_defaultdict_") =>
        {
            let Type::Dict(key_ty, value_ty) = body.resolve_alias() else {
                return false;
            };
            matches!(key_ty.as_ref(), Type::Any | Type::Unknown)
                || matches!(value_ty.as_ref(), Type::List(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
                || matches!(value_ty.as_ref(), Type::Set(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
        }
        (_, HirExpr::MethodCall { method, args, .. })
            if method == "get"
                && args.len() == 2
                && matches!(&args[1], HirExpr::ListLiteral { elements, .. } if elements.is_empty()) =>
        {
            true
        }
        _ => false,
    }
}

pub(super) fn exact_int_floor_result_local_rust_type(
    ty: &Type,
    value: &HirExpr,
) -> Option<RustType> {
    if matches!(
        crate::resolve_alias_type_for_plain_call(ty),
        Type::Int | Type::LiteralInt(_)
    ) && matches!(
        value,
        HirExpr::QuestionMark { expr, .. } if is_exact_int_floor_result_expr(expr)
    ) {
        return Some(RustType::Named("SifrInt".to_string()));
    }

    if is_result_int_division_error_type(ty) && is_exact_int_floor_result_expr(value) {
        let Type::Result(_, err_ty) = ty else {
            return None;
        };
        return Some(RustType::Result(
            Box::new(RustType::Named("SifrInt".to_string())),
            Box::new(crate::sifr_type_to_rust_type(err_ty)),
        ));
    }

    None
}

pub(super) fn is_exact_int_floor_result_expr(value: &HirExpr) -> bool {
    matches!(
        value,
        HirExpr::BinOp { op, ty, .. }
            if matches!(op.as_str(), "//" | "%") && is_result_int_division_error_type(ty)
    )
}

pub(super) fn is_result_int_division_error_type(ty: &Type) -> bool {
    let Type::Result(ok_ty, err_ty) = crate::resolve_alias_type_for_plain_call(ty) else {
        return false;
    };
    matches!(
        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
        Type::Int | Type::LiteralInt(_)
    ) && matches!(
        crate::resolve_alias_type_for_plain_call(err_ty.as_ref()),
        Type::Class { name, .. } if name == "DivisionError"
    )
}

pub(super) fn try_lower_simple_nested_function_stmt(
    func: &HirFunction,
    move_captures: bool,
    capture_clones: &[String],
    in_loop_with_else: bool,
    outer_bindings: SimpleStmtBindings<'_>,
) -> Option<Vec<RustStmt>> {
    if func.method_kind != MethodKind::Regular
        || !func.decorators.is_empty()
        || !func.type_params.is_empty()
    {
        return None;
    }
    if func
        .params
        .iter()
        .any(|param| param.default.is_some() || param.keyword_only)
    {
        return None;
    }

    let nested_mutated_vars = collect_mutated_vars(&func.body, None);
    let nested_borrowed_params: HashSet<String> = func
        .params
        .iter()
        .filter(|param| {
            param.convention.is_shared_borrow()
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
        })
        .map(|param| param.name.clone())
        .collect();
    let nested_mut_borrowed_params: HashSet<String> = func
        .params
        .iter()
        .filter(|param| {
            param.convention.is_mut_borrow()
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
        })
        .map(|param| param.name.clone())
        .collect();
    let is_recursive = body_calls_function(&func.body, &func.name);
    let param_names: HashSet<String> = func.params.iter().map(|param| param.name.clone()).collect();
    let referenced_with_types = collect_referenced_vars_with_types(&func.body);
    let locally_defined = collect_locally_defined_vars(&func.body);
    let captures: Vec<(String, Type)> = referenced_with_types
        .into_iter()
        .filter(|(name, _)| !param_names.contains(name) && !locally_defined.contains(name))
        .collect();
    let allowed_calls = if is_recursive {
        vec![func.name.clone()]
    } else {
        vec![]
    };
    let nested_local_binding_types: HashMap<String, Type> = func
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .chain(captures.iter().cloned())
        .collect();
    let nested_bindings = SimpleStmtBindings {
        mutated_vars: &nested_mutated_vars,
        borrowed_params: &nested_borrowed_params,
        mut_borrowed_params: &nested_mut_borrowed_params,
        local_binding_types: &nested_local_binding_types,
        recursive_fields: outer_bindings.recursive_fields,
    };
    let mut lowered_body = crate::with_allowed_plain_calls(&allowed_calls, || {
        let mut lowered = Vec::new();
        for stmt in &func.body {
            lowered.extend(try_lower_simple_stmt_with_ctx_and_bindings(
                stmt,
                in_loop_with_else,
                nested_bindings,
                SimpleStmtLoweringCtx {
                    return_type: Some(&func.return_type),
                    in_display_impl: false,
                    in_class_scope: false,
                    in_generator_closure: false,
                },
            )?);
        }
        Some(lowered)
    })?;
    if is_recursive {
        let capture_names = captures
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        if !capture_names.is_empty() {
            append_recursive_capture_args_to_stmts(&mut lowered_body, &func.name, &capture_names);
        }
        let fn_params = func
            .params
            .iter()
            .map(|param| RustParam::Named {
                name: param.name.clone(),
                ty: crate::sifr_type_to_rust_type(&param.ty),
            })
            .chain(captures.iter().map(|(name, ty)| RustParam::Named {
                name: name.clone(),
                ty: crate::sifr_type_to_rust_type(ty),
            }))
            .collect::<Vec<_>>();
        let ret = if matches!(func.return_type, Type::None) {
            None
        } else {
            Some(crate::sifr_type_to_rust_type(&func.return_type))
        };
        return Some(vec![RustStmt::LocalFn {
            name: func.name.clone(),
            params: fn_params,
            ret,
            body: lowered_body,
            is_async: func.is_async,
        }]);
    }

    let lowered_params = func
        .params
        .iter()
        .map(|param| RustParam::Named {
            name: param.name.clone(),
            ty: crate::sifr_type_to_rust_type(&param.ty),
        })
        .collect::<Vec<_>>();
    let mutates_captures = nested_mutated_vars
        .iter()
        .any(|name| !param_names.contains(name) && !locally_defined.contains(name));
    let nested_binding_mutable =
        outer_bindings.mutated_vars.contains(&func.name) || mutates_captures;

    Some(vec![RustStmt::Let {
        mutable: nested_binding_mutable,
        name: func.name.clone(),
        ty: None,
        value: crate::retained_callback_closure::closure_with_capture_clones(
            lowered_params,
            lowered_body,
            func.is_async || move_captures,
            func.is_async,
            capture_clones,
        ),
    }])
}

pub(super) fn append_recursive_capture_args_to_stmts(
    stmts: &mut [RustStmt],
    fn_name: &str,
    capture_names: &[String],
) {
    for stmt in stmts {
        match stmt {
            RustStmt::Verbatim(_) => {}
            RustStmt::Let { value, .. } | RustStmt::LetPattern { value, .. } => {
                append_recursive_capture_args_to_expr(value, fn_name, capture_names);
            }
            RustStmt::LetElse {
                value, else_body, ..
            } => {
                append_recursive_capture_args_to_expr(value, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(else_body, fn_name, capture_names);
            }
            RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
                append_recursive_capture_args_to_expr(target, fn_name, capture_names);
                append_recursive_capture_args_to_expr(value, fn_name, capture_names);
            }
            RustStmt::Expr(expr) | RustStmt::TailExpr(expr) | RustStmt::Return(Some(expr)) => {
                append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            }
            RustStmt::Assert { cond, msg } => {
                append_recursive_capture_args_to_expr(cond, fn_name, capture_names);
                if let Some(msg) = msg {
                    append_recursive_capture_args_to_expr(msg, fn_name, capture_names);
                }
            }
            RustStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                append_recursive_capture_args_to_expr(cond, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(then_body, fn_name, capture_names);
                if let Some(else_body) = else_body {
                    append_recursive_capture_args_to_stmts(else_body, fn_name, capture_names);
                }
            }
            RustStmt::IfLet {
                expr,
                then_body,
                else_body,
                ..
            } => {
                append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(then_body, fn_name, capture_names);
                if let Some(else_body) = else_body {
                    append_recursive_capture_args_to_stmts(else_body, fn_name, capture_names);
                }
            }
            RustStmt::Match { expr, arms } => {
                append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        append_recursive_capture_args_to_expr(guard, fn_name, capture_names);
                    }
                    append_recursive_capture_args_to_stmts(&mut arm.body, fn_name, capture_names);
                }
            }
            RustStmt::For { iter, body, .. } => {
                append_recursive_capture_args_to_expr(iter, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::With { items, body } => {
                for item in items {
                    append_recursive_capture_args_to_expr(&mut item.value, fn_name, capture_names);
                }
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::While { cond, body } => {
                append_recursive_capture_args_to_expr(cond, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::Loop { body } | RustStmt::Block(body) => {
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::LocalFn { body, .. } => {
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => {}
        }
    }
}

pub(super) fn append_recursive_capture_args_to_expr(
    expr: &mut RustExpr,
    fn_name: &str,
    capture_names: &[String],
) {
    match expr {
        RustExpr::FnCall { func, args } => {
            append_recursive_capture_args_to_expr(func, fn_name, capture_names);
            for arg in args.iter_mut() {
                append_recursive_capture_args_to_expr(arg, fn_name, capture_names);
            }
            if matches!(func.as_ref(), RustExpr::Ident(name) if name == fn_name) {
                for capture_name in capture_names {
                    args.push(RustExpr::Ident(capture_name.clone()));
                }
            }
        }
        RustExpr::MethodCall { receiver, args, .. } => {
            append_recursive_capture_args_to_expr(receiver, fn_name, capture_names);
            for arg in args {
                append_recursive_capture_args_to_expr(arg, fn_name, capture_names);
            }
        }
        RustExpr::Field { expr, .. } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
        }
        RustExpr::Index { expr, index } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            append_recursive_capture_args_to_expr(index, fn_name, capture_names);
        }
        RustExpr::Slice { expr, start, stop } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            if let Some(start) = start {
                append_recursive_capture_args_to_expr(start, fn_name, capture_names);
            }
            if let Some(stop) = stop {
                append_recursive_capture_args_to_expr(stop, fn_name, capture_names);
            }
        }
        RustExpr::BinOp { left, right, .. } => {
            append_recursive_capture_args_to_expr(left, fn_name, capture_names);
            append_recursive_capture_args_to_expr(right, fn_name, capture_names);
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Try(operand)
        | RustExpr::Await(operand)
        | RustExpr::Paren(operand) => {
            append_recursive_capture_args_to_expr(operand, fn_name, capture_names);
        }
        RustExpr::Cast { expr, .. } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
        }
        RustExpr::Ref { expr, .. } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
        }
        RustExpr::Block { stmts, expr } => {
            append_recursive_capture_args_to_stmts(stmts, fn_name, capture_names);
            if let Some(expr) = expr {
                append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            }
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            append_recursive_capture_args_to_expr(cond, fn_name, capture_names);
            append_recursive_capture_args_to_expr(then_expr, fn_name, capture_names);
            if let Some(else_expr) = else_expr {
                append_recursive_capture_args_to_expr(else_expr, fn_name, capture_names);
            }
        }
        RustExpr::Match { expr, arms } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    append_recursive_capture_args_to_expr(guard, fn_name, capture_names);
                }
                append_recursive_capture_args_to_stmts(&mut arm.body, fn_name, capture_names);
            }
        }
        RustExpr::Closure { body, .. } => {
            append_recursive_capture_args_to_expr(body, fn_name, capture_names);
        }
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
        }
        RustExpr::StructInit { fields, .. } => {
            for (_, field_value) in fields {
                append_recursive_capture_args_to_expr(field_value, fn_name, capture_names);
            }
        }
        RustExpr::Tuple(items)
        | RustExpr::Array(items)
        | RustExpr::Vec(items)
        | RustExpr::MacroCall { args: items, .. } => {
            for item in items {
                append_recursive_capture_args_to_expr(item, fn_name, capture_names);
            }
        }
        RustExpr::TimeoutAwait {
            duration,
            future,
            error,
        } => {
            append_recursive_capture_args_to_expr(duration, fn_name, capture_names);
            append_recursive_capture_args_to_expr(future, fn_name, capture_names);
            append_recursive_capture_args_to_expr(error, fn_name, capture_names);
        }
        RustExpr::FormatMacro { args, .. } => {
            for arg in args {
                append_recursive_capture_args_to_expr(arg, fn_name, capture_names);
            }
        }
        RustExpr::Range { start, end } => {
            append_recursive_capture_args_to_expr(start, fn_name, capture_names);
            append_recursive_capture_args_to_expr(end, fn_name, capture_names);
        }
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) | RustExpr::Verbatim(_) => {}
    }
}
