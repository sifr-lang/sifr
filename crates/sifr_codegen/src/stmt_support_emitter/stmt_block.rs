use super::{
    HirExpr, HirStmt, RustEmitter, RustStmt, Type, is_none_like_result_value,
    is_result_int_division_error_type, result_int_to_sifr_int_rust_type,
    should_force_mutable_binding, should_omit_local_type_annotation, type_contains_any_or_unknown,
};
impl RustEmitter {
    pub(crate) fn try_lower_stmt_block_for_ir_inner(
        &mut self,
        stmts: &[HirStmt],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let mut lowered_block = Vec::new();
        for (stmt_index, stmt) in stmts.iter().enumerate() {
            if let Some(lowered) =
                self.try_lower_checked_place_mutation_tail_for_ir(stmt, &stmts[stmt_index + 1..])?
            {
                lowered_block.extend(lowered);
                return Ok(Some(lowered_block));
            }
            if let Some(lowered) =
                self.try_lower_nonempty_pop_tail_for_ir(stmt, &stmts[stmt_index + 1..])?
            {
                lowered_block.push(lowered);
                return Ok(Some(lowered_block));
            }
            if let Some(lowered_guard) = self.try_lower_checked_dict_exit_guard_for_ir(stmt)? {
                lowered_block.push(lowered_guard);
                continue;
            }
            if let Some(lowered_guards) = self.try_lower_checked_sequence_exit_guards_for_ir(
                stmt,
                Some(&stmts[stmt_index + 1..]),
            )? {
                lowered_block.extend(lowered_guards);
                continue;
            }
            if let Some(lowered) = self.try_lower_checked_place_if_for_ir(stmt)? {
                lowered_block.push(lowered);
                continue;
            }
            if let Some(lowered) = self.try_lower_atomic_checked_read_stmt_for_ir(stmt)? {
                lowered_block.extend(lowered);
                continue;
            }
            let maybe_simple_lowered = self.try_lower_simple_block_stmt_for_ir(stmt)?;
            let should_bypass_simple_lowering = self.should_bypass_simple_block_lowering(stmt);
            let maybe_simple_lowered = if should_bypass_simple_lowering {
                None
            } else {
                maybe_simple_lowered
            };
            let (lowered_stmts, skip_rewrite) = if let Some(mut lowered_stmts) =
                maybe_simple_lowered
            {
                if let HirStmt::Let { name, ty, .. } = stmt {
                    let effective_ty = if type_contains_any_or_unknown(ty) {
                        self.local_binding_types
                            .get(name)
                            .cloned()
                            .unwrap_or_else(|| ty.clone())
                    } else {
                        ty.clone()
                    };
                    if let Some(cache_stmt) =
                        self.string_char_cache_init_stmt_for_local(name, &effective_ty)
                    {
                        lowered_stmts.push(cache_stmt);
                    }
                }
                (lowered_stmts, false)
            } else if let Some(lowered) = self.try_lower_nested_function_stmt_for_block(stmt) {
                (lowered, true)
            } else if let Some(lowered) = self.try_lower_tuple_unpack_stmt_for_block(stmt)? {
                (lowered, false)
            } else if let HirStmt::Let {
                name, ty, value, ..
            } = stmt
            {
                let effective_ty = if type_contains_any_or_unknown(ty) {
                    self.local_binding_types
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| ty.clone())
                } else {
                    ty.clone()
                };
                let generic_class_needs_inference = matches!(
                    &effective_ty,
                    Type::Class {
                        name: class_name,
                        type_args,
                        ..
                    } if self.generic_classes.contains(class_name) && type_args.is_empty()
                );
                let borrowed_dict_get = None;
                let nonempty_list_value =
                    self.lower_nonempty_list_binding_value_for_ir(name, value)?;
                let checked_option_value = if nonempty_list_value.is_none() {
                    self.lower_checked_place_option_value_for_target(&effective_ty, value)?
                } else {
                    None
                };
                let value_is_nonempty_list = nonempty_list_value.is_some();
                let recursive_borrowed_view = !self.mutated_vars.contains(name)
                    && self.recursive_option_borrowed_type(&effective_ty).is_some()
                    && self.expr_is_recursive_option_borrowed_view(value);
                let lowered_value = if let Some(lowered) = nonempty_list_value {
                    lowered
                } else if let Some(lowered) = checked_option_value {
                    lowered
                } else if let Some(lowered) = borrowed_dict_get.clone() {
                    lowered
                } else if !recursive_borrowed_view
                    && let Some(clone_expr) =
                        self.try_lower_borrowed_move_name_clone_for_ir(&effective_ty, value)
                {
                    clone_expr
                } else {
                    let lowered = if recursive_borrowed_view {
                        let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
                            return Ok(None);
                        };
                        lowered
                    } else if !self.body_analysis.aggregate_statement_has_last_use(stmt)
                        && let Some(lowered) = self.lower_rendered_expr_for_ir(value)?
                    {
                        lowered
                    } else {
                        let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
                            return Ok(None);
                        };
                        lowered
                    };
                    if recursive_borrowed_view {
                        lowered
                    } else {
                        self.coerce_local_value_for_target_type_for_ir(
                            &effective_ty,
                            value,
                            lowered,
                        )?
                    }
                };
                let lowered_value = self.rewrite_stdlib_constant_idents_in_expr(lowered_value);
                let lowered_ty = if recursive_borrowed_view {
                    self.recursive_option_borrowed_type(&effective_ty)
                } else if name == "_"
                    || generic_class_needs_inference
                    || borrowed_dict_get.is_some()
                    || value_is_nonempty_list
                    || Self::is_borrowed_empty_list_get_expr_for_ir(&lowered_value)
                    || should_omit_local_type_annotation(&effective_ty, value)
                {
                    None
                } else if matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_ty),
                    Type::Int | Type::LiteralInt(_)
                ) && self.is_sifr_int_expr(&lowered_value)
                {
                    Some(crate::RustType::Named("SifrInt".to_string()))
                } else if is_result_int_division_error_type(&effective_ty)
                    && self.is_sifr_int_result_expr(&lowered_value)
                {
                    Some(result_int_to_sifr_int_rust_type(&effective_ty))
                } else {
                    Some(self.rust_ir_type_with_generics(&effective_ty))
                };
                let mut lowered = if name == "_"
                    && matches!(
                        crate::resolve_alias_type_for_plain_call(&effective_ty),
                        Type::None
                    ) {
                    vec![RustStmt::Expr(lowered_value)]
                } else {
                    vec![RustStmt::Let {
                        mutable: self.mutated_vars.contains(name)
                            || should_force_mutable_binding(&effective_ty, &self.recursive_fields),
                        name: name.clone(),
                        ty: lowered_ty,
                        value: lowered_value,
                    }]
                };
                if recursive_borrowed_view {
                    self.recursive_option_borrowed_views.insert(name.clone());
                }
                if let Some(cache_stmt) =
                    self.string_char_cache_init_stmt_for_local(name, &effective_ty)
                {
                    lowered.push(cache_stmt);
                }
                (lowered, true)
            } else if let HirStmt::Assign { name, value } = stmt {
                if let Some(lowered) =
                    self.try_lower_self_string_concat_assign_for_ir(name, value)?
                {
                    (lowered, true)
                } else {
                    let target_ty = self.local_binding_types.get(name).cloned();
                    let checked_option_value = if let Some(target_ty) = target_ty.as_ref() {
                        self.lower_checked_place_option_value_for_target(target_ty, value)?
                    } else {
                        None
                    };
                    let value_is_target_typed = checked_option_value.is_some();
                    let lowered_value = if let Some(lowered) = checked_option_value {
                        lowered
                    } else if !self.body_analysis.aggregate_statement_has_last_use(stmt)
                        && let Some(lowered) = self.lower_rendered_expr_for_ir(value)?
                    {
                        lowered
                    } else {
                        let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
                            return Ok(None);
                        };
                        lowered
                    };
                    let lowered_value = self.rewrite_stdlib_constant_idents_in_expr(lowered_value);
                    let lowered_value = if value_is_target_typed {
                        lowered_value
                    } else if let Some(target_ty) = target_ty {
                        Self::validate_assignment_source_type_for_ir(name, &target_ty, value)?;
                        self.coerce_local_value_for_target_type_for_ir(
                            &target_ty,
                            value,
                            lowered_value,
                        )?
                    } else {
                        lowered_value
                    };
                    let mut lowered = vec![RustStmt::Assign {
                        target: crate::RustExpr::Ident(name.clone()),
                        value: lowered_value,
                    }];
                    if let Some(cache_stmt) = self.string_char_cache_rebuild_stmt_for_local(name) {
                        lowered.push(cache_stmt);
                    }
                    (lowered, true)
                }
            } else if let HirStmt::AugAssign { name, op, value } = stmt {
                let value_ty = Self::resolve_alias_type_for_loop_iter(value.ty());
                if self.is_registered_sifr_int_local(name) {
                    let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(None);
                    };
                    let value_expr = self.rewrite_stdlib_constant_idents_in_expr(value_expr);
                    let Some(lowered) =
                        self.lower_exact_int_augassign_stmt_for_ir(name, op, value, value_expr)
                    else {
                        return Ok(None);
                    };
                    (vec![lowered], true)
                } else if op == "+=" {
                    match value_ty {
                        Type::Str => {
                            let cache_name = self.string_char_cache_vars.get(name).cloned();
                            let mut stmts = Vec::new();
                            let (arg_expr, cache_chars_source) = if cache_name.is_some()
                                && !matches!(value, HirExpr::StringLiteral(_))
                            {
                                let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                                    return Ok(None);
                                };
                                let temp_name = format!("__sifr_string_augassign_{name}");
                                stmts.push(crate::RustStmt::Let {
                                    mutable: false,
                                    name: temp_name.clone(),
                                    ty: None,
                                    value: value_expr,
                                });
                                let as_str =
                                    self.string_view_expr(value, crate::RustExpr::Ident(temp_name));
                                (as_str.clone(), as_str)
                            } else if let HirExpr::StringLiteral(val) = value {
                                let literal = crate::RustExpr::Verbatim(format!("{val:?}"));
                                (literal.clone(), literal)
                            } else {
                                let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                                    return Ok(None);
                                };
                                let as_str = self.string_view_expr(value, value_expr);
                                (as_str.clone(), as_str)
                            };
                            stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                                method: "push_str".to_string(),
                                args: vec![arg_expr],
                            }));
                            if let Some(cache_name) = cache_name {
                                stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(cache_name)),
                                    method: "extend".to_string(),
                                    args: vec![crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                            cache_chars_source,
                                        ))),
                                        method: "chars".to_string(),
                                        args: vec![],
                                    }],
                                }));
                            }
                            (stmts, true)
                        }
                        Type::List(_) => {
                            let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                                return Ok(None);
                            };
                            (
                                vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                                    method: "extend".to_string(),
                                    args: vec![value_expr],
                                })],
                                true,
                            )
                        }
                        _ => {
                            let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                                return Ok(None);
                            };
                            let lowered_value = if self.is_registered_sifr_int_local(name) {
                                self.coerce_expr_to_sifr_int_value(lowered_value)
                            } else {
                                lowered_value
                            };
                            (
                                vec![RustStmt::AugAssign {
                                    target: crate::RustExpr::Ident(name.clone()),
                                    op: "+".to_string(),
                                    value: lowered_value,
                                }],
                                true,
                            )
                        }
                    }
                } else if op == "**=" {
                    let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(None);
                    };
                    (
                        vec![crate::RustStmt::Assign {
                            target: crate::RustExpr::Ident(name.clone()),
                            value: crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                    crate::RustExpr::Ident(name.clone()),
                                ))),
                                method: "pow".to_string(),
                                args: vec![crate::RustExpr::Cast {
                                    expr: Box::new(value_expr),
                                    ty: crate::RustType::Named("u32".to_string()),
                                }],
                            },
                        }],
                        true,
                    )
                } else {
                    let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(None);
                    };
                    let lowered_value = if self.is_registered_sifr_int_local(name) {
                        self.coerce_expr_to_sifr_int_value(lowered_value)
                    } else {
                        lowered_value
                    };
                    let normalized_op = if op == "//=" {
                        "/".to_string()
                    } else {
                        op.strip_suffix('=').unwrap_or(op).to_string()
                    };
                    (
                        vec![RustStmt::AugAssign {
                            target: crate::RustExpr::Ident(name.clone()),
                            op: normalized_op,
                            value: lowered_value,
                        }],
                        true,
                    )
                }
            } else if let HirStmt::SubscriptAssign {
                object,
                index,
                value,
                object_ty,
                failure,
            } = stmt
            {
                let Some(lowered_stmt) = self.lower_subscript_assign_stmt_for_ir(
                    object,
                    index,
                    value,
                    object_ty,
                    failure.as_ref(),
                )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::NestedSubscriptAssign {
                object,
                outer_index,
                inner_index,
                value,
                object_ty,
                outer_failure,
                inner_failure,
                operation,
            } = stmt
            {
                let lowered_stmt = self.lower_checked_nested_mutation_for_ir(
                    crate::checked_place_mutation::CheckedNestedMutationPlan {
                        root: crate::RustExpr::Ident(object.clone()),
                        root_ty: object_ty,
                        outer_index,
                        inner_index,
                        value,
                        operation,
                        outer_failure: outer_failure.as_ref(),
                        inner_failure: inner_failure.as_ref(),
                    },
                )?;
                let Some(lowered_stmt) = lowered_stmt else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::AttributeNestedSubscriptAssign {
                object,
                field,
                outer_index,
                inner_index,
                value,
                field_ty,
                outer_failure,
                inner_failure,
                operation,
            } = stmt
            {
                let Some(lowered_stmt) = self.lower_checked_nested_mutation_for_ir(
                    crate::checked_place_mutation::CheckedNestedMutationPlan {
                        root: crate::RustExpr::Field {
                            expr: Box::new(Self::object_name_expr_for_ir(object)),
                            field: field.clone(),
                        },
                        root_ty: field_ty,
                        outer_index,
                        inner_index,
                        value,
                        operation,
                        outer_failure: outer_failure.as_ref(),
                        inner_failure: inner_failure.as_ref(),
                    },
                )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::SubscriptAugAssign {
                object,
                index,
                op,
                value,
                object_ty,
                failure,
            } = stmt
            {
                let Some(lowered_stmt) = self.lower_subscript_augassign_stmt_for_ir(
                    object,
                    index,
                    op,
                    value,
                    object_ty,
                    failure.as_ref(),
                )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::Delete {
                object,
                index,
                failure,
            } = stmt
            {
                let Some(lowered_stmt) =
                    self.lower_delete_stmt_for_ir(object, index, failure.as_ref())?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::AttributeSubscriptAssign {
                object,
                field,
                index,
                value,
                field_ty,
                failure,
                operation,
            } = stmt
            {
                let receiver = crate::RustExpr::Field {
                    expr: Box::new(Self::object_name_expr_for_ir(object)),
                    field: field.clone(),
                };
                let Some(lowered_stmt) = self.lower_checked_single_mutation_for_ir(
                    receiver,
                    field_ty,
                    index,
                    value,
                    operation,
                    failure.as_ref(),
                )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::FieldAssign {
                object,
                field,
                field_ty,
                value,
            } = stmt
            {
                let Some(lowered_stmt) =
                    self.lower_field_assign_stmt_for_block(object, field, field_ty, value)?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::NestedFieldAssign {
                object,
                field,
                field_ty,
                nested_field,
                nested_field_ty,
                value,
            } = stmt
            {
                let Some(lowered_stmt) = self.lower_nested_field_assign_stmt_for_ir(
                    object,
                    field,
                    field_ty,
                    nested_field,
                    nested_field_ty,
                    value,
                )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::Assert { test, msg } = stmt {
                let Some(lowered_test) = self.lower_condition_expr_for_ir(test)? else {
                    return Ok(None);
                };
                let lowered_msg = if let Some(msg_expr) = msg {
                    let Some(lowered) = self.lower_rendered_expr_for_ir(msg_expr)? else {
                        return Ok(None);
                    };
                    Some(lowered)
                } else {
                    None
                };
                (
                    vec![RustStmt::Assert {
                        cond: lowered_test,
                        msg: lowered_msg,
                    }],
                    true,
                )
            } else if let HirStmt::Expr { expr } = stmt {
                let lowered_expr =
                    if let Some(lowered) = self.try_lower_stmt_expr_statement_only(expr)? {
                        lowered
                    } else if let Some(lowered) = self.lower_rendered_expr_for_ir(expr)? {
                        lowered
                    } else {
                        let Some(lowered) = self.lower_stmt_expr_for_ir(expr)? else {
                            return Ok(None);
                        };
                        lowered
                    };
                (vec![RustStmt::Expr(lowered_expr)], true)
            } else if let HirStmt::Return { value } = stmt {
                let return_ty_snapshot = self.current_return_type.clone();
                let lowered_return_stmt = if let Some(value) = value {
                    if self.emission_ctx.in_display_impl && self.try_closure_depth == 0 {
                        let Some(display_expr) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };
                        RustStmt::Return(Some(crate::RustExpr::MacroCall {
                            name: "write".to_string(),
                            args: vec![
                                crate::RustExpr::Ident("f".to_string()),
                                crate::RustExpr::Literal(crate::RustLiteral::Str("{}".to_string())),
                                display_expr,
                            ],
                        }))
                    } else if self.try_closure_depth > 0 {
                        let Some(lowered_return_value) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };
                        RustStmt::Return(Some(self.try_closure_return_value_for_ir(
                            lowered_return_value,
                            is_none_like_result_value(value),
                            return_ty_snapshot.as_ref(),
                        )))
                    } else {
                        let Some(lowered_return_value) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };
                        RustStmt::Return(Some(lowered_return_value))
                    }
                } else if self.try_closure_depth > 0 {
                    RustStmt::Return(Some(
                        self.try_closure_unit_return_for_ir(return_ty_snapshot.as_ref()),
                    ))
                } else if self.emission_ctx.in_display_impl
                    || (self.emission_ctx.in_generator_closure
                        && return_ty_snapshot.as_ref().is_some_and(|ty| {
                            matches!(ty.resolve_alias(), Type::Result(ok, _) if matches!(ok.resolve_alias(), Type::None))
                        }))
                {
                    RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                    }))
                } else {
                    RustStmt::Return(None)
                };
                (vec![lowered_return_stmt], true)
            } else if let HirStmt::Yield { value } = stmt {
                let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
                (vec![crate::lower_suspended_yield_stmt(lowered_value)], true)
            } else if let HirStmt::Raise { value } = stmt {
                let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                let lowered = self.coerce_raised_error_for_ir(value, lowered);
                (
                    vec![RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
                        args: vec![lowered],
                    }))],
                    true,
                )
            } else if let HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } = stmt
            {
                if let Some(lowered) = self.try_lower_checked_dict_if_for_ir(
                    condition,
                    then_body,
                    elif_clauses,
                    else_body.as_deref(),
                )? {
                    (vec![lowered], true)
                } else if let Some(lowered) = self.try_lower_checked_sequence_if_for_ir(
                    condition,
                    then_body,
                    elif_clauses,
                    else_body.as_deref(),
                )? {
                    (vec![lowered], true)
                } else {
                    let Some(lowered_if_stmt) = self.try_lower_if_stmt_for_ir(
                        condition,
                        then_body,
                        elif_clauses,
                        else_body.as_deref(),
                    )?
                    else {
                        return Ok(None);
                    };
                    (vec![lowered_if_stmt], true)
                }
            } else if let HirStmt::While {
                condition,
                body,
                else_body,
            } = stmt
            {
                let has_else = else_body.is_some();
                self.loop_else_stack.push(has_else);
                let missing = self.lower_loop_break_for_ir();
                let (condition_refresh_keys, condition_refreshes) =
                    self.checked_place_loop_condition_refreshes_for_ir(condition, body, &missing);
                let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
                    let _ = self.loop_else_stack.pop();
                    return Ok(None);
                };
                let checked_read_guards = self
                    .checked_sequence_loop_guards_for_ir(condition, body)?
                    .into_iter()
                    .filter(|guard| !condition_refresh_keys.contains(&guard.key))
                    .collect::<Vec<_>>();
                let lowered_body = self.lower_checked_sequence_loop_body_for_ir(
                    body,
                    &checked_read_guards,
                    &missing,
                    &condition_refresh_keys,
                )?;
                let lowered_loop = lowered_body.map(|body| {
                    Self::checked_place_while_stmt_for_ir(lowered_cond, body, condition_refreshes)
                });
                let Some(lowered_loop) = lowered_loop else {
                    let popped = self.loop_else_stack.pop();
                    debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                    return Ok(None);
                };
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                if let Some(else_body) = else_body {
                    let Some(lowered_else_body) =
                        self.try_lower_scoped_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(None);
                    };
                    (
                        vec![Self::loop_else_scaffold_for_ir(
                            lowered_loop,
                            lowered_else_body,
                        )],
                        true,
                    )
                } else {
                    (vec![lowered_loop], true)
                }
            } else if let HirStmt::For {
                target,
                target_ty,
                iter,
                body,
                else_body,
                ..
            } = stmt
            {
                if else_body.is_some() {
                    return Ok(None);
                }
                let char_set_loop = Self::should_lower_string_set_loop_target_as_char(
                    target, target_ty, iter, body,
                );
                let Some(lowered_iter) = (if char_set_loop {
                    self.lower_string_chars_for_iter_expr_for_ir(iter)?
                } else {
                    self.try_lower_for_iter_expr_for_ir(iter, target_ty)?
                }) else {
                    return Ok(None);
                };
                let target_cache_init = if char_set_loop || target.contains(',') {
                    None
                } else {
                    self.string_char_cache_init_stmt_for_loop_target(target, target_ty)
                };
                let checked_read_guards = if char_set_loop {
                    Vec::new()
                } else {
                    self.checked_sequence_for_guards_for_ir(target, iter, body)?
                };
                self.loop_else_stack.push(false);
                let lowered_body_result = self.lower_checked_sequence_loop_body_for_ir(
                    body,
                    &checked_read_guards,
                    &RustStmt::Continue,
                    &[],
                );
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                let lowered_body_result = lowered_body_result?;
                let Some(mut lowered_body) = lowered_body_result else {
                    return Ok(None);
                };
                if let Some(cache_stmt) = target_cache_init {
                    lowered_body.insert(0, cache_stmt);
                }
                let var = if target.contains(',') {
                    let names = target
                        .split(',')
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .collect::<Vec<_>>();
                    if names.is_empty() {
                        return Ok(None);
                    }
                    format!("({})", names.join(", "))
                } else {
                    target.clone()
                };
                (
                    vec![RustStmt::For {
                        var,
                        iter: lowered_iter,
                        body: lowered_body,
                    }],
                    true,
                )
            } else if let HirStmt::AsyncFor {
                target,
                iter,
                iter_error_ty,
                close_error_ty,
                active_error_ty,
                body,
                else_body,
                ..
            } = stmt
            {
                self.loop_else_stack.push(else_body.is_some());
                let lowered_async_for_result = self.try_lower_async_for_stmt_for_ir(
                    target,
                    iter,
                    iter_error_ty,
                    close_error_ty.as_ref(),
                    active_error_ty,
                    body,
                );
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                let Some(lowered_async_for) = lowered_async_for_result? else {
                    return Ok(None);
                };
                if let Some(else_body) = else_body {
                    let Some(lowered_else_body) =
                        self.try_lower_scoped_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(None);
                    };
                    (
                        vec![Self::loop_else_scaffold_for_ir(
                            lowered_async_for,
                            lowered_else_body,
                        )],
                        true,
                    )
                } else {
                    (vec![lowered_async_for], true)
                }
            } else if let HirStmt::With { items, body } = stmt {
                let Some(lowered_with) = self.try_lower_with_stmt_for_ir(items, body)? else {
                    return Ok(None);
                };
                (vec![lowered_with], true)
            } else if let HirStmt::AsyncWith { kind, target, body } = stmt {
                let Some(lowered_async_with) =
                    self.try_lower_async_with_stmt_for_ir(kind, target.as_deref(), body)?
                else {
                    return Ok(None);
                };
                (vec![lowered_async_with], true)
            } else if matches!(stmt, HirStmt::TryExcept { .. }) {
                let Some(lowered_try_except) = self
                    .try_lower_try_except_hir_stmt_for_ir_with_following(
                        stmt,
                        Some(&stmts[stmt_index + 1..]),
                    )?
                else {
                    return Ok(None);
                };
                (lowered_try_except, true)
            } else if let HirStmt::TryFinally { body, finalbody } = stmt {
                let Some(lowered_try_finally) =
                    self.try_lower_try_finally_stmt_for_ir(body, finalbody)?
                else {
                    return Ok(None);
                };
                (lowered_try_finally, true)
            } else if matches!(stmt, HirStmt::Pass) {
                (Vec::new(), true)
            } else if let Some(lowered) = self.lower_loop_control_stmt_for_ir(stmt) {
                (vec![lowered], true)
            } else {
                return Ok(None);
            };
            if skip_rewrite {
                lowered_block.extend(lowered_stmts);
            } else {
                lowered_block.extend(
                    lowered_stmts
                        .into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt)),
                );
            }
        }
        Ok(Some(lowered_block))
    }
}
