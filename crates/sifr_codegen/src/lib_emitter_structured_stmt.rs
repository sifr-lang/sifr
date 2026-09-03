use super::{
    ClassScope, HirExpr, HirStmt, RustEmitter, RustExpr, RustStmt, ScopeContext, Type,
    body_contains_await, resolve_alias_type_for_plain_call, stmt_needs_performance_lowering,
    try_lower_simple_stmt_with_scope_result_and_bindings,
};

impl RustEmitter {
    pub(crate) fn try_lower_structured_stmt_with_following(
        &mut self,
        stmt: &HirStmt,
        following_stmts: Option<&[HirStmt]>,
    ) -> Result<bool, crate::CodegenError> {
        let scope_ctx = ScopeContext {
            function_return_type: self.current_return_type.clone(),
            in_generator_closure: self.emission_ctx.in_generator_closure,
            in_display_impl: self.emission_ctx.in_display_impl,
            in_loop_with_else: self.current_loop_has_else(),
            class_scope: if self.current_class_name.is_some() {
                ClassScope::Inside
            } else {
                ClassScope::Outside
            },
        };

        if let HirStmt::NestedFunction { func, .. } = stmt {
            let captures = self.collect_recursive_nested_fn_captures(func);
            if captures.is_empty() {
                self.nested_fn_captures.remove(&func.name);
            } else {
                self.nested_fn_captures.insert(func.name.clone(), captures);
            }
        }

        if self.try_capture_checked_place_control_stmt(stmt, following_stmts)? {
            return Ok(true);
        }

        let should_bypass_simple_lowering = matches!(
            stmt,
            HirStmt::NestedFunction { .. }
                | HirStmt::Assign { .. }
                | HirStmt::Delete { .. }
                | HirStmt::SubscriptAssign { .. }
                | HirStmt::NestedSubscriptAssign { .. }
                | HirStmt::AttributeNestedSubscriptAssign { .. }
                | HirStmt::AttributeSubscriptAssign { .. }
                | HirStmt::SubscriptAugAssign { .. }
                | HirStmt::StarUnpack { .. }
        ) || self.stmt_uses_checked_place_read_witness(stmt)
            || Self::stmt_defines_nonempty_list(stmt)
            || self.stmt_uses_checked_option_target(stmt)
            || matches!(
                stmt,
                HirStmt::AsyncWith {
                    kind: sifr_ir::HirAsyncWithKind::TaskTimeout { .. }
                        | sifr_ir::HirAsyncWithKind::UserDefined { .. }
                        | sifr_ir::HirAsyncWithKind::Python { .. },
                    body,
                    ..
                } if body_contains_await(body)
            )
            || matches!(
                stmt,
                HirStmt::AsyncWith {
                    kind: sifr_ir::HirAsyncWithKind::UserDefined { .. }
                        | sifr_ir::HirAsyncWithKind::Python { .. },
                    ..
                }
            )
            || matches!(stmt, HirStmt::Let { ty, .. } if self.type_contains_generic_class(ty))
            || matches!(stmt, HirStmt::Let { name, .. } if self.hoistable_static_dict_locals.contains(name))
            || stmt_needs_performance_lowering(stmt);
        if !should_bypass_simple_lowering {
            if let Some(lowered_stmts) = try_lower_simple_stmt_with_scope_result_and_bindings(
                stmt,
                &self.mutated_vars,
                &self.borrowed_params,
                &self.mut_borrowed_params,
                &self.local_binding_types,
                &self.recursive_fields,
                &scope_ctx,
            )? {
                self.lowering_stats.expr_candidate_total += 1;
                self.lowering_stats.expr_candidate_structured += 1;
                let rewritten_stmts = lowered_stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect::<Vec<_>>();
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                self.emit_lowered_stmts(&rewritten_stmts);
                return Ok(true);
            }
        }

        if self.try_lower_structured_nested_function_stmt(stmt) {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }

        if let HirStmt::AsyncWith { kind, target, body } = stmt {
            if let Some(lowered_stmt) =
                self.try_lower_async_with_stmt_for_ir(kind, target.as_deref(), body)?
            {
                self.push_captured_stmt(&self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt));
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
        }

        if let HirStmt::AsyncFor {
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
            if let Some(lowered_async_for) = lowered_async_for_result? {
                let lowered_stmt = if let Some(else_body) = else_body {
                    let Some(lowered_else_body) =
                        self.try_lower_scoped_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(false);
                    };
                    Self::loop_else_scaffold_for_ir(lowered_async_for, lowered_else_body)
                } else {
                    lowered_async_for
                };
                self.push_captured_stmt(&self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt));
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
        }

        if let Some(lowered_stmts) = self.try_lower_tuple_unpack_stmt_for_block(stmt)? {
            if matches!(
                stmt,
                HirStmt::TupleUnpack { .. } | HirStmt::StarUnpack { .. }
            ) {
                for lowered_stmt in lowered_stmts {
                    let lowered_stmt = self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt);
                    self.push_captured_stmt(&lowered_stmt);
                }
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
        }

        if let HirStmt::Let {
            name, ty, value, ..
        } = stmt
        {
            let effective_ty = self
                .local_binding_types
                .get(name)
                .cloned()
                .unwrap_or_else(|| ty.clone());
            if let Some(hoisted_stmt) =
                self.try_hoist_static_readonly_dict_literal(name, &effective_ty, value)?
            {
                self.push_captured_stmt(&hoisted_stmt);
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
            let generic_class_needs_inference = matches!(
                &effective_ty,
                Type::Class {
                    name: class_name,
                    type_args,
                    ..
                } if self.generic_classes.contains(class_name) && type_args.is_empty()
            );
            let recursive_borrowed_view = !self.mutated_vars.contains(name)
                && self.recursive_option_borrowed_type(&effective_ty).is_some()
                && self.expr_is_recursive_option_borrowed_view(value);
            let lowered_value = if crate::helpers::is_copy_type_for_codegen(&effective_ty) {
                None
            } else {
                if let HirExpr::Name {
                    name: value_name, ..
                } = value
                {
                    if !recursive_borrowed_view
                        && (self.borrowed_params.contains(value_name)
                            || self.mut_borrowed_params.contains(value_name))
                    {
                        Some(crate::ownership_plan::materialize_owned_value(
                            &effective_ty,
                            crate::RustExpr::Ident(value_name.clone()),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            let borrowed_dict_get = None;
            let nonempty_list_value = self.lower_nonempty_list_binding_value_for_ir(name, value)?;
            let checked_option_value = if nonempty_list_value.is_none() {
                self.lower_checked_place_option_value_for_target(&effective_ty, value)?
            } else {
                None
            };
            let value_is_nonempty_list = nonempty_list_value.is_some();
            let lowered_value = if let Some(lowered) = nonempty_list_value {
                lowered
            } else if let Some(lowered) = checked_option_value {
                lowered
            } else if let Some(lowered) = borrowed_dict_get.clone() {
                lowered
            } else if let Some(clone_expr) = lowered_value {
                clone_expr
            } else {
                if let Some(lowered) = self.lower_rendered_expr_for_ir(value)? {
                    self.coerce_local_value_for_target_type_for_ir(&effective_ty, value, lowered)?
                } else if let Some(lowered) = self.lower_stmt_expr_for_ir(value)? {
                    self.coerce_local_value_for_target_type_for_ir(&effective_ty, value, lowered)?
                } else {
                    return Ok(false);
                }
            };

            let lowered_stmt = if name == "_"
                && matches!(resolve_alias_type_for_plain_call(&effective_ty), Type::None)
            {
                RustStmt::Expr(lowered_value)
            } else {
                RustStmt::Let {
                    mutable: self.mutated_vars.contains(name)
                        || matches!(
                            &effective_ty,
                            Type::Alias { name: alias_name, .. }
                                if alias_name.starts_with("__sifr_defaultdict_")
                        )
                        || matches!(effective_ty.resolve_alias(), Type::Iterator(_)),
                    name: name.clone(),
                    ty: if recursive_borrowed_view {
                        self.recursive_option_borrowed_type(&effective_ty)
                    } else if generic_class_needs_inference
                        || borrowed_dict_get.is_some()
                        || value_is_nonempty_list
                        || Self::is_borrowed_empty_list_get_expr_for_ir(&lowered_value)
                        || match (&effective_ty, value) {
                            (resolved_ty, HirExpr::Call { func, args, .. })
                                if matches!(
                                    resolve_alias_type_for_plain_call(resolved_ty),
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
                                if let Type::Dict(key_ty, value_ty) = body.resolve_alias() {
                                    matches!(key_ty.as_ref(), Type::Any | Type::Unknown)
                                        || matches!(value_ty.as_ref(), Type::List(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
                                        || matches!(value_ty.as_ref(), Type::Set(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        }
                    {
                        None
                    } else {
                        Some(self.rust_ir_type_with_generics(&effective_ty))
                    },
                    value: lowered_value,
                }
            };
            let lowered_stmt = self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt);
            self.push_captured_stmt(&lowered_stmt);
            if recursive_borrowed_view {
                self.recursive_option_borrowed_views.insert(name.clone());
            }
            if let Some(cache_stmt) =
                self.string_char_cache_init_stmt_for_local(name, &effective_ty)
            {
                let cache_stmt = self.rewrite_stdlib_constant_idents_in_stmt(cache_stmt);
                self.push_captured_stmt(&cache_stmt);
            }
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }

        if let HirStmt::Assign { name, value } = stmt {
            if let Some(lowered_stmts) =
                self.try_lower_self_string_concat_assign_for_ir(name, value)?
            {
                let lowered_stmts = lowered_stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect::<Vec<_>>();
                self.emit_lowered_stmts(&lowered_stmts);
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
            let target_ty = self.local_binding_types.get(name).cloned();
            let checked_option_value = if let Some(target_ty) = target_ty.as_ref() {
                self.lower_checked_place_option_value_for_target(target_ty, value)?
            } else {
                None
            };
            let lowered_value = if let Some(lowered) = checked_option_value {
                lowered
            } else if let Some(lowered) = self.lower_rendered_expr_for_ir(value)? {
                if let Some(target_ty) = target_ty.clone() {
                    Self::validate_assignment_source_type_for_ir(name, &target_ty, value)?;
                    self.coerce_local_value_for_target_type_for_ir(&target_ty, value, lowered)?
                } else {
                    lowered
                }
            } else if let Some(lowered) = self.lower_stmt_expr_for_ir(value)? {
                if let Some(target_ty) = target_ty {
                    Self::validate_assignment_source_type_for_ir(name, &target_ty, value)?;
                    self.coerce_local_value_for_target_type_for_ir(&target_ty, value, lowered)?
                } else {
                    lowered
                }
            } else {
                return Ok(false);
            };
            let lowered_stmt = RustStmt::Assign {
                target: RustExpr::Ident(name.clone()),
                value: lowered_value,
            };
            let lowered_stmt = self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt);
            self.push_captured_stmt(&lowered_stmt);
            if let Some(cache_stmt) = self.string_char_cache_rebuild_stmt_for_local(name) {
                let cache_stmt = self.rewrite_stdlib_constant_idents_in_stmt(cache_stmt);
                self.push_captured_stmt(&cache_stmt);
            }
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if let HirStmt::Yield { value } = stmt {
            let lowered_value = if let Some(lowered) = self.lower_rendered_expr_for_ir(value)? {
                lowered
            } else if let Some(lowered) = self.lower_stmt_expr_for_ir(value)? {
                lowered
            } else {
                return Ok(false);
            };
            let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
            self.push_captured_stmt(&crate::lower_suspended_yield_stmt(lowered_value));
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_field_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_nested_field_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_subscript_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_nested_subscript_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_attribute_nested_subscript_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_subscript_augassign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_delete_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_attribute_subscript_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_return_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_raise_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_if_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_while_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_for_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_with_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_try_except_stmt_with_following(stmt, following_stmts) {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if let HirStmt::TryFinally { body, finalbody } = stmt {
            let Some(lowered_stmts) = self.try_lower_try_finally_stmt_for_ir(body, finalbody)?
            else {
                return Ok(false);
            };
            for lowered_stmt in lowered_stmts {
                self.push_captured_stmt(&lowered_stmt);
            }
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_assert_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_aug_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if let HirStmt::Expr { expr } = stmt {
            if let Some(lowered_expr) = self.try_lower_stmt_expr_statement_only(expr)? {
                self.lowering_stats.expr_total += 1;
                self.lowering_stats.expr_candidate_total += 1;
                self.lowering_stats.expr_structured += 1;
                self.lowering_stats.expr_candidate_structured += 1;
                let rewritten = self.rewrite_stdlib_constant_idents_in_expr(lowered_expr);
                self.push_captured_stmt(&RustStmt::Expr(rewritten));
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
            if let Some(lowered_expr) = self.lower_stmt_expr_for_ir(expr)? {
                self.lowering_stats.expr_total += 1;
                self.lowering_stats.expr_candidate_total += 1;
                self.lowering_stats.expr_structured += 1;
                self.lowering_stats.expr_candidate_structured += 1;
                let rewritten = self.rewrite_stdlib_constant_idents_in_expr(lowered_expr);
                self.push_captured_stmt(&RustStmt::Expr(rewritten));
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
