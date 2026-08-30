use super::{
    DiagnosticCode, Expr, FunctionType, HirExpr, HirIteratorOp, HirStmt, LowerCtx,
    NarrowingCondition, Ranged, StmtAssign, StmtFor, StmtIf, TextRange, Type, apply_narrowing,
    body_always_leaves_current_path, callable_builtin_element_type,
    detect_false_exit_sequence_guards, detect_false_nonzero_integer_guards,
    detect_narrowing_condition, detect_range_sequence_guards, detect_true_nonzero_integer_guards,
    detect_true_sequence_guards, empty_collection_literal_kind, ensure_mutable_parameter_binding,
    failed_initializer_taint, finish_async_generator_advance_for_expr,
    invalidate_rebound_binding_facts, is_collection_backed_iter_source,
    loop_body_mutates_iter_source, lower_expr, lower_star_unpack_assign, lower_stmts,
    lower_tuple_unpack_assign, maybe_record_dict_assignment_guard,
    merge_exhaustive_branch_sequence_guards, name_diagnostics, numeric_domain_for_type,
    numeric_sentinel_kind, ownership_diagnostics, predeclare_exhaustive_if_assigned_names,
    reconcile_optional_reassignment, record_async_generator_advance_binding,
    record_const_integer_binding, record_len_alias_fact, record_sequence_pointer_fact,
    resolve_field_type_from_type, resolve_object_field_type,
    restore_const_integer_state_after_branches, seed_binding_after_failed_initializer,
    seed_exhaustive_if_bindings, sequence_shape_fact, should_adopt_inferred_binding_hint,
    should_rebind_simple_name, statement_diagnostics, str, task_group_spawn_owner,
    then_body_always_exits, validate_control_flow_condition, validate_subscript_assignment_target,
};
use crate::lower::defaultdict_refinement::order_independent_defaultdict_hint;
use crate::lower::expressions::{consume_affine_value_name, consume_owned_value};
use crate::lower::must_use_obligations;
use crate::lower::python_interop as pyinterop;
use crate::lower::task_join_set_calls::record_join_set_terminal_awaitable;

pub(in crate::lower) fn lower_assign(assign: &StmtAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    if assign.targets.len() != 1 {
        statement_diagnostics::invalid_assignment_target(
            ctx,
            "multiple assignment targets are not supported",
            assign.range(),
        );
        return None;
    }

    // Handle tuple unpacking: a, b = expr or a, *b = expr
    if let Expr::Tuple(tuple) = &assign.targets[0] {
        // Check if any element is a Starred expression (star unpacking)
        let has_star = tuple.elts.iter().any(|e| matches!(e, Expr::Starred(_)));
        if has_star {
            return lower_star_unpack_assign(tuple, &assign.value, ctx);
        }
        return lower_tuple_unpack_assign(tuple, &assign.value, ctx);
    }

    // Handle attribute assignment: self.field = value or obj.field = value
    if let Expr::Attribute(attr) = &assign.targets[0] {
        if let Expr::Attribute(inner_attr) = attr.value.as_ref() {
            let obj_name = if let Expr::Name(n) = inner_attr.value.as_ref() {
                n.id.to_string()
            } else {
                statement_diagnostics::invalid_assignment_target(
                    ctx,
                    "attribute assignment target must be a simple name",
                    inner_attr.value.range(),
                );
                return None;
            };
            let obj_range = inner_attr.value.range();
            if !ensure_mutable_parameter_binding(ctx, &obj_name, obj_range) {
                return None;
            }
            let field_name = inner_attr.attr.to_string();
            let field_ty = resolve_object_field_type(ctx, &obj_name, &field_name);
            let nested_field_name = attr.attr.to_string();
            let nested_field_ty = resolve_field_type_from_type(&field_ty, &nested_field_name)
                .unwrap_or(Type::Unknown);
            let value = pyinterop::lower_python_context_owned_expr(&assign.value, ctx)?;
            consume_affine_value_name(&value, assign.value.range(), ctx);
            return Some(HirStmt::NestedFieldAssign {
                object: obj_name,
                field: field_name,
                field_ty,
                nested_field: nested_field_name,
                nested_field_ty,
                value,
            });
        }
        let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
            n.id.to_string()
        } else {
            statement_diagnostics::invalid_assignment_target(
                ctx,
                "attribute assignment target must be a simple name",
                attr.value.range(),
            );
            return None;
        };
        let obj_range = attr.value.range();
        if !ensure_mutable_parameter_binding(ctx, &obj_name, obj_range) {
            return None;
        }
        let field_name = attr.attr.to_string();
        let field_ty = resolve_object_field_type(ctx, &obj_name, &field_name);
        let value = pyinterop::lower_python_context_owned_expr(&assign.value, ctx)?;
        consume_affine_value_name(&value, assign.value.range(), ctx);
        return Some(HirStmt::FieldAssign {
            object: obj_name,
            field: field_name,
            field_ty,
            value,
        });
    }

    // Handle subscript assignment: list[i] = val or dict[key] = val
    if let Expr::Subscript(sub) = &assign.targets[0] {
        // Handle nested subscript: matrix[i][j] = val
        if let Expr::Subscript(inner_sub) = sub.value.as_ref() {
            let (obj_name, field_name, obj_ty) = if let Expr::Name(n) = inner_sub.value.as_ref() {
                let obj_ty = ctx
                    .scope
                    .lookup(&n.id)
                    .map(|info| info.effective_type().clone())
                    .unwrap_or(Type::Unknown);
                (n.id.to_string(), None, obj_ty)
            } else if let Expr::Attribute(attr) = inner_sub.value.as_ref() {
                let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
                    n.id.to_string()
                } else {
                    statement_diagnostics::invalid_assignment_target(
                        ctx,
                        "nested subscript assignment target must be a simple name",
                        attr.value.range(),
                    );
                    return None;
                };
                let field_name = attr.attr.to_string();
                let field_ty = resolve_object_field_type(ctx, &obj_name, &field_name);
                (obj_name, Some(field_name), field_ty)
            } else {
                statement_diagnostics::invalid_assignment_target(
                    ctx,
                    "nested subscript assignment target must be a simple name",
                    inner_sub.value.range(),
                );
                return None;
            };
            let obj_range = inner_sub.value.range();
            if !ensure_mutable_parameter_binding(ctx, &obj_name, obj_range) {
                return None;
            }
            if matches!(obj_ty.resolve_alias(), Type::Bytes) {
                super::ownership_diagnostics::immutable_bytes_subscript_assignment(
                    ctx,
                    inner_sub.range(),
                );
                return None;
            }
            let outer_index = lower_expr(&inner_sub.slice, ctx)?;
            let inner_index = lower_expr(&sub.slice, ctx)?;
            let value = pyinterop::lower_python_context_owned_expr(&assign.value, ctx)?;
            consume_affine_value_name(&value, assign.value.range(), ctx);
            if let Some(field) = field_name {
                return Some(HirStmt::AttributeNestedSubscriptAssign {
                    object: obj_name,
                    field,
                    outer_index,
                    inner_index,
                    value,
                    field_ty: obj_ty,
                });
            }
            return Some(HirStmt::NestedSubscriptAssign {
                object: obj_name,
                outer_index,
                inner_index,
                value,
                object_ty: obj_ty,
            });
        }
        // Handle attribute subscript assignment: self.field[key] = val
        if let Expr::Attribute(attr) = sub.value.as_ref() {
            let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
                n.id.to_string()
            } else {
                statement_diagnostics::invalid_assignment_target(
                    ctx,
                    "subscript assignment target must be a simple name",
                    attr.value.range(),
                );
                return None;
            };
            let obj_range = attr.value.range();
            if !ensure_mutable_parameter_binding(ctx, &obj_name, obj_range) {
                return None;
            }
            let field_name = attr.attr.to_string();
            let field_ty = resolve_object_field_type(ctx, &obj_name, &field_name);
            if matches!(field_ty.resolve_alias(), Type::Bytes) {
                super::ownership_diagnostics::immutable_bytes_subscript_assignment(
                    ctx,
                    sub.range(),
                );
                return None;
            }
            let index = lower_expr(&sub.slice, ctx)?;
            let value = pyinterop::lower_python_context_owned_expr(&assign.value, ctx)?;
            consume_affine_value_name(&value, assign.value.range(), ctx);
            return Some(HirStmt::AttributeSubscriptAssign {
                object: obj_name,
                field: field_name,
                index,
                value,
                field_ty,
            });
        }
        let obj_name = if let Expr::Name(n) = sub.value.as_ref() {
            n.id.to_string()
        } else {
            statement_diagnostics::invalid_assignment_target(
                ctx,
                "subscript assignment target must be a simple name",
                sub.value.range(),
            );
            return None;
        };
        let obj_range = sub.value.range();
        if !ensure_mutable_parameter_binding(ctx, &obj_name, obj_range) {
            return None;
        }
        let obj_ty = ctx
            .scope
            .lookup(&obj_name)
            .map(|info| info.effective_type().clone())
            .unwrap_or(Type::Unknown);
        if matches!(obj_ty.resolve_alias(), Type::Bytes) {
            super::ownership_diagnostics::immutable_bytes_subscript_assignment(ctx, sub.range());
            return None;
        }
        let index = lower_expr(&sub.slice, ctx)?;
        let value = pyinterop::lower_python_context_owned_expr(&assign.value, ctx)?;
        let object_ty = validate_subscript_assignment_target(
            ctx,
            &obj_name,
            &obj_ty,
            index.ty(),
            value.ty(),
            sub.range(),
        );
        maybe_record_dict_assignment_guard(ctx, &object_ty, &obj_name, &sub.slice);
        consume_affine_value_name(&value, assign.value.range(), ctx);
        return Some(HirStmt::SubscriptAssign {
            object: obj_name,
            index,
            value,
            object_ty,
        });
    }

    let (name, name_range) = if let Expr::Name(n) = &assign.targets[0] {
        (n.id.to_string(), n.range())
    } else {
        statement_diagnostics::invalid_assignment_target(
            ctx,
            "assignment target must be a simple name",
            assign.targets[0].range(),
        );
        return None;
    };

    // Handle `_ = expr` as explicit discard (suppresses #[must_use] warnings)
    if name == "_" {
        let value = lower_expr(&assign.value, ctx)?;
        let value_ty = value.ty().clone();
        pyinterop::reject_python_context_borrow_discard(&value, assign.range(), ctx);
        if let Some(obligation) = ctx.must_use_obligation_for_type(&value_ty) {
            ctx.error_with_code_at(
                DiagnosticCode::OWN_USE_AFTER_MOVE,
                format!("cannot discard must-use resource `{obligation}`; transfer or close it"),
                assign.range(),
            );
        }
        finish_async_generator_advance_for_expr(ctx, &value);
        return Some(HirStmt::Let {
            name: "_".to_string(),
            ty: value_ty,
            value,
            is_mutable: false,
        });
    }

    let should_treat_as_existing_binding = if ctx.current_function_frame_start().is_some() {
        should_rebind_simple_name(ctx, &name)
    } else {
        ctx.scope.lookup(&name).is_some()
    };
    let error_count_before_initializer = ctx.begin_initializer_lowering();
    let Some(mut value) = lower_expr(&assign.value, ctx) else {
        if !should_treat_as_existing_binding {
            let error_taint = failed_initializer_taint(
                ctx,
                &name,
                assign.value.range(),
                error_count_before_initializer,
            )?;
            let fallback_ty = ctx
                .inferred_binding_hint(&name)
                .cloned()
                .unwrap_or(Type::Unknown);
            seed_binding_after_failed_initializer(ctx, &name, fallback_ty, false, error_taint);
        }
        return None;
    };
    let value_ty = value.ty().clone();
    pyinterop::reject_python_context_borrow_storage(&value, assign.value.range(), ctx);

    consume_owned_value(&value, assign.value.range(), ctx);

    // Check if variable already exists
    if should_treat_as_existing_binding {
        let Some(info) = ctx.scope.lookup(&name).cloned() else {
            name_diagnostics::undefined_variable(ctx, &name, name_range);
            return None;
        };
        if ownership_diagnostics::reject_borrowed_affine_parameter_reassignment(
            ctx,
            &name,
            info.is_parameter_binding(),
            &info.ty,
            name_range,
        ) {
            return None;
        }
        if info.is_parameter_binding() && !info.is_mutable_binding() {
            super::ownership_diagnostics::immutable_parameter_reassignment(ctx, &name, name_range);
            return None;
        }
        // Reassignment: check type compatibility
        let info_ty = info.ty.clone();
        let can_widen = info.is_inferred_local_binding();
        if !reconcile_optional_reassignment(ctx, &name, &info_ty, &value_ty, can_widen) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "type mismatch: cannot assign '{}' to variable '{}' of type '{}'",
                    value_ty.display_name(),
                    name,
                    info_ty.display_name()
                ),
                assign.value.range(),
            );
        }
        if ctx.live_join_set_bindings.contains(&name) && !ctx.scope.is_moved(&name) {
            ctx.error_with_code_at(
                DiagnosticCode::OWN_USE_AFTER_MOVE,
                format!(
                    "cannot reassign live JoinSet binding '{name}' before consuming it with await {name}.join_all() or await {name}.cancel_all()"
                ),
                name_range,
            );
        }
        if let Some(obligation) = ctx.live_must_use_bindings.get(&name).cloned() {
            if !ctx.scope.is_moved(&name) {
                ctx.error_with_code_at(
                    DiagnosticCode::OWN_USE_AFTER_MOVE,
                    format!(
                        "cannot reassign must-use binding '{name}' owning {obligation} before closing or transferring it"
                    ),
                    name_range,
                );
            }
        }
        // Reset moved state on reassignment
        ctx.reset_moved_with_flow(&name);
        ctx.record_must_use_binding(&name, &value_ty);
        ctx.task_handle_group_owners.remove(&name);
        ctx.live_join_set_bindings.remove(&name);
        record_join_set_terminal_awaitable(&name, &value, ctx);
        record_async_generator_advance_binding(ctx, &name, &value);
        invalidate_rebound_binding_facts(ctx, &name);
        if matches!(value.ty(), Type::Int | Type::LiteralInt(_))
            && value.ty().is_assignable_to(&info_ty)
        {
            record_const_integer_binding(ctx, &name, &value);
        }
        if ctx.numeric_sentinel_fact(&name).is_some() {
            if let Some(domain) = numeric_domain_for_type(&value_ty) {
                ctx.resolve_numeric_sentinel_domain(&name, domain);
            }
        }
        ctx.clear_sequence_shape_fact(&name);
        record_len_alias_fact(ctx, &name, &assign.value);
        record_sequence_pointer_fact(ctx, &name, &assign.value);
        ctx.empty_dict_specializations.remove(&name);
        ctx.pending_container_specialization_patches.remove(&name);
        Some(HirStmt::Assign { name, value })
    } else {
        // New variable (type inferred)
        let allow_general_hint = ctx.can_adopt_empty_collection_hints();
        let allow_order_independent_dict_hint = ctx.can_adopt_empty_plain_dict_hint(&name);
        let adopted_defaultdict_hint_ty = ctx
            .can_adopt_defaultdict_hint(&name)
            .then(|| ctx.inferred_binding_hint(&name))
            .flatten()
            .filter(|hint| order_independent_defaultdict_hint(&assign.value, hint))
            .cloned();
        let adopted_hint_ty = adopted_defaultdict_hint_ty.clone().or_else(|| {
            ctx.inferred_binding_hint(&name)
                .filter(|hint| {
                    should_adopt_inferred_binding_hint(
                        &assign.value,
                        &value_ty,
                        hint,
                        allow_general_hint,
                        allow_order_independent_dict_hint,
                    )
                })
                .cloned()
        });
        let binding_ty = adopted_hint_ty.clone().unwrap_or_else(|| value_ty.clone());
        let inferred_empty_dict_ty = (empty_collection_literal_kind(&assign.value) == Some("dict")
            && allow_order_independent_dict_hint)
            .then_some(adopted_hint_ty)
            .flatten();
        if let (Some(specialized_ty), HirExpr::DictLiteral { ty: literal_ty, .. }) =
            (&inferred_empty_dict_ty, &mut value)
        {
            *literal_ty = specialized_ty.clone();
        }
        if let (Some(specialized_ty), HirExpr::Call { ty: call_ty, .. }) =
            (&adopted_defaultdict_hint_ty, &mut value)
        {
            *call_ty = specialized_ty.clone();
        }
        ctx.scope.define(name.clone(), binding_ty.clone());
        ctx.record_must_use_binding(&name, &binding_ty);
        if matches!(value.ty(), Type::Int | Type::LiteralInt(_))
            && value.ty().is_assignable_to(&binding_ty)
        {
            record_const_integer_binding(ctx, &name, &value);
        }
        record_join_set_terminal_awaitable(&name, &value, ctx);
        record_async_generator_advance_binding(ctx, &name, &value);
        if let Some(group_name) = task_group_spawn_owner(&value) {
            ctx.task_handle_group_owners
                .insert(name.clone(), group_name);
        }
        if let Some(kind) = numeric_sentinel_kind(&assign.value) {
            ctx.record_numeric_sentinel_initializer(name.clone(), kind);
        } else {
            ctx.clear_numeric_sentinel_var(&name);
        }
        if let Some(fact) = sequence_shape_fact(&name, &assign.value) {
            ctx.record_sequence_shape_fact(fact);
        } else {
            ctx.clear_sequence_shape_fact(&name);
        }
        record_len_alias_fact(ctx, &name, &assign.value);
        ctx.pending_container_specialization_patches.remove(&name);
        if let Some(specialized_ty) = inferred_empty_dict_ty {
            ctx.empty_dict_specializations
                .insert(name.clone(), specialized_ty);
        } else {
            ctx.empty_dict_specializations.remove(&name);
        }
        record_sequence_pointer_fact(ctx, &name, &assign.value);
        Some(HirStmt::Let {
            name,
            ty: binding_ty,
            value,
            is_mutable: true,
        })
    }
}

pub(in crate::lower) fn lower_if(
    if_stmt: &StmtIf,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let narrowing_cond = detect_narrowing_condition(&if_stmt.test, ctx);

    let condition = lower_expr(&if_stmt.test, ctx)?;
    validate_control_flow_condition(&condition, "if", if_stmt.test.range(), ctx);
    predeclare_exhaustive_if_assigned_names(if_stmt, ctx);

    let saved_state = ctx.scope.save_narrowing_state();
    let saved_moved = ctx.scope.save_moved_state();
    let saved_const_integer_state = ctx.scope.save_const_integer_state();
    let saved_sequence_guards = ctx.save_sequence_guards();
    let saved_nonzero_integer_bindings = ctx.save_proven_nonzero_integer_bindings();

    if let Some(ref cond) = narrowing_cond {
        apply_narrowing(ctx, cond, true);
    }
    for guard in detect_true_sequence_guards(&if_stmt.test, ctx) {
        ctx.add_sequence_guard(guard);
    }
    for name in detect_true_nonzero_integer_guards(&if_stmt.test, ctx) {
        ctx.add_proven_nonzero_integer_binding(name);
    }

    ctx.scope.push();
    let then_body = lower_stmts(&if_stmt.body, func_type, ctx);
    ctx.scope.pop();

    let then_moved = ctx.scope.save_moved_state();
    let then_const_integer_state = ctx.scope.save_const_integer_state();
    let then_sequence_guards = ctx.save_sequence_guards();

    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);
    ctx.scope
        .restore_const_integer_state(&saved_const_integer_state);
    ctx.restore_sequence_guards(&saved_sequence_guards);
    ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);

    let mut all_conditions: Vec<NarrowingCondition> = Vec::new();
    if let Some(ref cond) = narrowing_cond {
        all_conditions.push(cond.clone());
    }

    let mut branch_moved_states: Vec<_> = vec![then_moved];
    let mut branch_const_integer_states =
        vec![(then_const_integer_state, then_body_always_exits(&then_body))];
    let mut branch_sequence_states = vec![then_sequence_guards];
    let mut post_if_false_nonzero_guards = Vec::new();
    let mut all_previous_branches_exit = then_body_always_exits(&then_body);
    if all_previous_branches_exit {
        post_if_false_nonzero_guards
            .extend(detect_false_nonzero_integer_guards(&if_stmt.test, ctx));
    }

    let mut elif_clauses = Vec::new();
    for clause in &if_stmt.elif_else_clauses {
        if let Some(test) = &clause.test {
            ctx.scope.restore_narrowing_state(&saved_state);
            ctx.scope.restore_moved_state(&saved_moved);
            ctx.scope
                .restore_const_integer_state(&saved_const_integer_state);
            ctx.restore_sequence_guards(&saved_sequence_guards);
            ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);
            for prev_cond in &all_conditions {
                apply_narrowing(ctx, prev_cond, false);
            }

            let elif_narrowing = detect_narrowing_condition(test, ctx);
            let cond = lower_expr(test, ctx)?;
            validate_control_flow_condition(&cond, "elif", test.range(), ctx);

            let elif_saved = ctx.scope.save_narrowing_state();
            if let Some(ref elif_cond) = elif_narrowing {
                apply_narrowing(ctx, elif_cond, true);
            }
            for guard in detect_true_sequence_guards(test, ctx) {
                ctx.add_sequence_guard(guard);
            }
            for name in detect_true_nonzero_integer_guards(test, ctx) {
                ctx.add_proven_nonzero_integer_binding(name);
            }

            ctx.scope.push();
            let body = lower_stmts(&clause.body, func_type, ctx);
            ctx.scope.pop();
            let elif_body_exits = then_body_always_exits(&body);
            if all_previous_branches_exit && elif_body_exits {
                post_if_false_nonzero_guards.extend(detect_false_nonzero_integer_guards(test, ctx));
            }
            all_previous_branches_exit &= elif_body_exits;
            elif_clauses.push((cond, body));

            branch_moved_states.push(ctx.scope.save_moved_state());
            branch_const_integer_states
                .push((ctx.scope.save_const_integer_state(), elif_body_exits));
            branch_sequence_states.push(ctx.save_sequence_guards());

            ctx.scope.restore_narrowing_state(&elif_saved);
            ctx.scope.restore_moved_state(&saved_moved);
            ctx.scope
                .restore_const_integer_state(&saved_const_integer_state);
            ctx.restore_sequence_guards(&saved_sequence_guards);
            ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);

            if let Some(elif_cond) = elif_narrowing {
                all_conditions.push(elif_cond);
            }
        }
    }

    let else_body = if_stmt
        .elif_else_clauses
        .iter()
        .find(|c| c.test.is_none())
        .map(|clause| {
            ctx.scope.restore_narrowing_state(&saved_state);
            ctx.scope.restore_moved_state(&saved_moved);
            ctx.scope
                .restore_const_integer_state(&saved_const_integer_state);
            ctx.restore_sequence_guards(&saved_sequence_guards);
            ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);
            for prev_cond in &all_conditions {
                apply_narrowing(ctx, prev_cond, false);
            }
            ctx.scope.push();
            let body = lower_stmts(&clause.body, func_type, ctx);
            ctx.scope.pop();
            branch_moved_states.push(ctx.scope.save_moved_state());
            branch_const_integer_states.push((
                ctx.scope.save_const_integer_state(),
                then_body_always_exits(&body),
            ));
            branch_sequence_states.push(ctx.save_sequence_guards());
            body
        });

    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);
    restore_const_integer_state_after_branches(
        ctx,
        &saved_const_integer_state,
        &branch_const_integer_states,
    );
    ctx.restore_sequence_guards(&saved_sequence_guards);
    ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);

    let branch_exits = branch_const_integer_states
        .iter()
        .map(|(_, exits)| *exits)
        .collect::<Vec<_>>();
    must_use_obligations::validate_branch_join(
        ctx,
        &branch_moved_states,
        &branch_exits,
        &saved_moved,
        else_body.is_some(),
        if_stmt.range(),
    );

    for branch_state in &branch_moved_states {
        for (name, was_moved) in branch_state {
            if *was_moved {
                ctx.mark_moved_with_flow(name);
            }
        }
    }

    seed_exhaustive_if_bindings(
        ctx,
        &then_body,
        &elif_clauses,
        else_body.as_ref(),
        if_stmt.range(),
    );
    merge_exhaustive_branch_sequence_guards(ctx, else_body.is_some(), &branch_sequence_states);

    // Early-return narrowing: if the then-body always exits (return/break/continue/raise),
    // apply the inverse narrowing after the if block.
    // e.g., `if x is None: return` -> after the if, x is not None
    if let Some(ref cond) = narrowing_cond {
        if body_always_leaves_current_path(&then_body)
            && elif_clauses.is_empty()
            && else_body.is_none()
        {
            apply_narrowing(ctx, cond, false);
        }
    }
    if body_always_leaves_current_path(&then_body) && elif_clauses.is_empty() && else_body.is_none()
    {
        for guard in detect_false_exit_sequence_guards(&if_stmt.test, ctx) {
            ctx.add_sequence_guard(guard);
        }
    }
    for name in post_if_false_nonzero_guards {
        ctx.add_proven_nonzero_integer_binding(name);
    }
    ctx.clear_sequence_pointers();
    Some(HirStmt::If {
        condition,
        then_body,
        elif_clauses,
        else_body,
    })
}

pub(in crate::lower) fn lower_for(
    for_stmt: &StmtFor,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    // Lower the iterable expression and normalize protocol usage through `iter(...)`.
    let iterable_expr = pyinterop::lower_python_context_owned_expr(&for_stmt.iter, ctx)?;
    let iter_source_name = match &iterable_expr {
        HirExpr::Name { name, .. } => Some(name.clone()),
        _ => None,
    };
    let iter_source_ty = iterable_expr.ty().clone();
    if matches!(iter_source_ty.resolve_alias(), Type::Any | Type::Unknown) {
        statement_diagnostics::invalid_iteration(
            ctx,
            &format!(
                "for-loop iterable must have a statically-known element type, got '{}'",
                iter_source_ty.display_name()
            ),
            for_stmt.iter.range(),
        );
        return None;
    }
    let Some(elem_ty) = callable_builtin_element_type(&iter_source_ty) else {
        if matches!(iter_source_ty.resolve_alias(), Type::Tuple(_)) {
            statement_diagnostics::invalid_iteration(
                ctx,
                "for-loop tuple iteration requires one statically provable element type",
                for_stmt.iter.range(),
            );
            return None;
        }
        statement_diagnostics::invalid_iteration(
            ctx,
            &format!(
                "cannot iterate over type '{}'",
                iter_source_ty.display_name()
            ),
            for_stmt.iter.range(),
        );
        return None;
    };
    if statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, for_stmt.iter.range()) {
        return None;
    }
    let iter_expr = HirExpr::IteratorCall {
        op: HirIteratorOp::Iter,
        args: vec![iterable_expr],
        mutable_arg_places: Vec::new(),
        ty: Type::Iterator(Box::new(elem_ty.clone())),
    };
    let consumes_task_handle_collection = iter_source_name.is_some()
        && matches!(iter_source_ty.resolve_alias(), Type::List(_))
        && matches!(elem_ty.resolve_alias(), Type::Task(_, _));

    // Extract the target variable name(s)
    let (target_name, target_tuple_range): (String, Option<TextRange>) =
        match for_stmt.target.as_ref() {
            Expr::Name(n) => (n.id.to_string(), None),
            Expr::Tuple(tup) => {
                // Tuple unpacking: for i, v in enumerate(lst)
                let names: Vec<String> = tup
                    .elts
                    .iter()
                    .filter_map(|e| {
                        if let Expr::Name(n) = e {
                            Some(n.id.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                if names.len() != tup.elts.len() {
                    statement_diagnostics::invalid_iteration(
                        ctx,
                        "for loop tuple target must contain only simple names",
                        tup.range(),
                    );
                    return None;
                }
                (names.join(","), Some(tup.range()))
            }
            _ => {
                statement_diagnostics::invalid_iteration(
                    ctx,
                    "for loop target must be a simple name or tuple",
                    for_stmt.target.range(),
                );
                return None;
            }
        };

    if consumes_task_handle_collection {
        if let Some(source_name) = iter_source_name.as_deref() {
            ctx.mark_moved_with_flow(source_name);
        }
    }

    // Snapshot moved state before loop to detect moves inside the body
    let moved_before_loop = ctx.scope.save_moved_state();
    let saved_const_integer_state = ctx.scope.save_const_integer_state();

    // Create a new scope for the loop body, define the loop variable(s)
    ctx.scope.push();
    let saved_sequence_guards = ctx.save_sequence_guards();
    if target_name.contains(',') {
        // Tuple unpacking: define each variable with its type from the tuple
        let names: Vec<&str> = target_name.split(',').collect();
        if let Type::Tuple(elem_types) = &elem_ty {
            if elem_types.len() != names.len() {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                    format!(
                        "for loop tuple target expects {} element(s), iterable yields {}",
                        names.len(),
                        elem_types.len()
                    ),
                    target_tuple_range.unwrap_or_else(|| for_stmt.target.range()),
                );
                ctx.scope.pop();
                return None;
            }
            for (i, name) in names.iter().enumerate() {
                let ty = elem_types[i].clone();
                ctx.scope.define_ephemeral(
                    (*name).to_string(),
                    ty,
                    crate::scope::EphemeralOrigin::Iteration,
                );
            }
        } else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                format!(
                    "for loop tuple target expects iterable elements of tuple type, got '{}'",
                    elem_ty.display_name()
                ),
                target_tuple_range.unwrap_or_else(|| for_stmt.target.range()),
            );
            ctx.scope.pop();
            return None;
        }
    } else {
        ctx.scope.define_ephemeral(
            target_name.clone(),
            elem_ty.clone(),
            crate::scope::EphemeralOrigin::Iteration,
        );
    }
    for guard in detect_range_sequence_guards(for_stmt, &target_name, ctx) {
        ctx.add_sequence_guard(guard);
    }
    ctx.loop_depth += 1;
    let body = lower_stmts(&for_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();
    let body_const_integer_state = ctx.scope.save_const_integer_state();
    restore_const_integer_state_after_branches(
        ctx,
        &saved_const_integer_state,
        &[(body_const_integer_state, false)],
    );
    ctx.restore_sequence_guards(&saved_sequence_guards);
    super::append_growth_shapes::record_append_growth_sequence_shape_fact(
        for_stmt,
        &target_name,
        ctx,
    );
    if let Some(source_name) = iter_source_name.as_deref() {
        if is_collection_backed_iter_source(&iter_source_ty)
            && loop_body_mutates_iter_source(&body, source_name)
        {
            statement_diagnostics::mutation_during_iteration(ctx, source_name, for_stmt.range());
            return None;
        }
    }

    // Check for outer-scope variables moved inside the loop body
    let newly_moved = ctx.scope.moved_since(&moved_before_loop);
    for var_name in &newly_moved {
        ownership_diagnostics::moved_across_loop(ctx, var_name, for_stmt.range());
    }

    let else_body = if for_stmt.orelse.is_empty() {
        None
    } else {
        ctx.scope.push();
        let else_stmts = lower_stmts(&for_stmt.orelse, func_type, ctx);
        ctx.scope.pop();
        Some(else_stmts)
    };

    ctx.clear_sequence_pointers();

    Some(HirStmt::For {
        target: target_name,
        target_ty: elem_ty,
        iter: iter_expr,
        body,
        else_body,
    })
}
