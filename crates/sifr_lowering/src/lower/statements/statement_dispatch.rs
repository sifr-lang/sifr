use super::async_for::lower_async_for;
use super::async_generator_advances::finish_async_generator_advance_for_expr;
use super::async_with::lower_async_with;
use super::container_literal_specialization::{
    apply_container_specialization_patches, type_contains_unknown_or_any,
};
use super::diagnostics::{
    collect_raise_error_types, format_type_name, has_decorator, is_valid_error_type,
};
use super::expressions::consume_affine_value_name;
use super::expressions::lower_expr;
use super::function_flow::infer_function_return_type;
use super::match_lowering::lower_match;
use super::nested_function_state::{
    push_nested_function_captures, push_nested_function_mutations,
    restore_nested_function_captures, restore_nested_function_mutations,
};
use super::nonlocal_support::{
    collect_declared_nonlocals, hir_body_calls_function, lower_nonlocal,
};
use super::numeric_sentinels::apply_numeric_sentinel_patches;
use super::protocol_diagnostics;
use super::return_lowering::lower_return;
use super::statement_diagnostics;
use super::typing_and_functions::{
    ast_convention_to_param, register_local_function_signature, register_local_function_symbol,
    reject_unsupported_nested_async_generator,
};
use super::LowerCtx;
use super::{
    lower_ann_assign, lower_assign, lower_aug_assign, lower_chained_assign, lower_for, lower_if,
    lower_while, str,
};
use crate::hir_nodes::{
    HirExceptHandler, HirExpr, HirFunction, HirParam, HirStmt, HirWithItem, HirWithItemKind,
    MethodKind,
};
use crate::lower::python_interop;
use crate::lower::rust_interop::{
    classify_rust_interop_stub_body, collect_rust_interop_declarations,
    has_rust_interop_decorator_syntax, RustInteropOwner,
};
use crate::lower::type_bounds::reject_unavailable_dict_hash_key;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{ExceptHandler, Expr, Stmt};
use sifr_type_system::{FunctionType, Type};

pub(in crate::lower) fn empty_collection_literal_kind(expr: &Expr) -> Option<&'static str> {
    match expr {
        Expr::List(list) if list.elts.is_empty() => Some("list"),
        Expr::Dict(dict) if dict.items.is_empty() => Some("dict"),
        Expr::Set(set) if set.elts.is_empty() => Some("set"),
        Expr::Call(call)
            if call.arguments.args.is_empty() && call.arguments.keywords.is_empty() =>
        {
            let Expr::Name(name) = call.func.as_ref() else {
                return None;
            };
            (name.id.as_str() == "set").then_some("set")
        }
        Expr::Call(call)
            if call.arguments.args.is_empty() && call.arguments.keywords.is_empty() =>
        {
            let Expr::Attribute(attr) = call.func.as_ref() else {
                return None;
            };
            match (attr.value.as_ref(), attr.attr.as_str()) {
                (Expr::Name(module), "deque") if module.id.as_str() == "collections" => {
                    Some("deque")
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn hint_matches_empty_collection_shape(value_expr: &Expr, hint: &Type) -> bool {
    let Some(kind) = empty_collection_literal_kind(value_expr) else {
        return false;
    };
    match (kind, hint.resolve_alias()) {
        ("list", Type::List(_)) => true,
        ("dict", Type::Dict(_, _)) => true,
        ("set", Type::Set(_)) => true,
        ("deque", Type::Class { name, .. }) => name == "deque",
        _ => false,
    }
}

pub(super) fn should_adopt_inferred_binding_hint(
    value_expr: &Expr,
    value_ty: &Type,
    hint: &Type,
    allow_empty_collection_hint: bool,
    allow_empty_plain_dict_hint: bool,
) -> bool {
    if !type_contains_unknown_or_any(value_ty) {
        return false;
    }
    let empty_collection_kind = empty_collection_literal_kind(value_expr);
    if empty_collection_kind.is_some() {
        return (allow_empty_collection_hint
            || (allow_empty_plain_dict_hint && empty_collection_kind == Some("dict")))
            && !type_contains_unknown_or_any(hint)
            && hint_matches_empty_collection_shape(value_expr, hint);
    }
    if value_ty.is_assignable_to(hint) {
        return true;
    }
    if type_contains_unknown_or_any(hint) {
        return false;
    }
    hint_matches_empty_collection_shape(value_expr, hint)
}

pub(in crate::lower) fn lower_stmts(
    stmts: &[Stmt],
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Vec<HirStmt> {
    let nested_inference =
        super::nested_function_inference::infer_nested_function_types(stmts, ctx);
    let can_adopt_empty_collection_hints = !nested_inference.function_types.is_empty();
    let mut empty_plain_dict_hint_names =
        crate::lower::empty_plain_dict_inference::safe_hint_names_for_block(stmts);
    empty_plain_dict_hint_names.retain(|name| {
        nested_inference.binding_hints.get(name)
            == nested_inference.exact_dict_write_hints.get(name)
    });
    ctx.inferred_binding_hints
        .push(nested_inference.binding_hints.clone());
    ctx.push_empty_collection_hint_adoption(can_adopt_empty_collection_hints);
    ctx.push_empty_plain_dict_hint_adoption(empty_plain_dict_hint_names);
    predeclare_nested_function_symbols(stmts, &nested_inference.function_types, ctx);
    let previous_nested_captures =
        push_nested_function_captures(&nested_inference.function_captures, ctx);
    let previous_nested_mutations =
        push_nested_function_mutations(&nested_inference.function_mutated_captures, ctx);

    let mut result = Vec::new();
    for stmt in stmts {
        if crate::cfg::flow_facts(&result).always_exits() {
            ctx.warn_unreachable_statement(stmt.range());
            continue;
        }
        // Handle chained assignment (x = y = z = 0) by expanding into multiple statements
        if let Stmt::Assign(assign) = stmt {
            if assign.targets.len() > 1 {
                let expanded = lower_chained_assign(assign, ctx);
                result.extend(expanded);
                continue;
            }
        }
        if let Stmt::Try(try_stmt) = stmt {
            if !try_stmt.finalbody.is_empty() {
                let has_handlers = !try_stmt.handlers.is_empty();
                let mut body = if has_handlers {
                    lower_stmt(stmt, func_type, ctx).into_iter().collect()
                } else {
                    lower_stmts(&try_stmt.body, func_type, ctx)
                };
                if !has_handlers && !try_stmt.orelse.is_empty() {
                    let orelse = lower_stmts(&try_stmt.orelse, func_type, ctx);
                    body.extend(orelse);
                }
                let finalbody = lower_stmts(&try_stmt.finalbody, func_type, ctx);
                result.push(HirStmt::TryFinally { body, finalbody });
                apply_numeric_sentinel_patches(
                    &mut result,
                    &mut ctx.pending_numeric_sentinel_patches,
                );
                apply_container_specialization_patches(
                    &mut result,
                    &mut ctx.pending_container_specialization_patches,
                );
                continue;
            }
        }
        if let Some(hir_stmt) = lower_stmt(stmt, func_type, ctx) {
            result.push(hir_stmt);
        }
        apply_numeric_sentinel_patches(&mut result, &mut ctx.pending_numeric_sentinel_patches);
        apply_container_specialization_patches(
            &mut result,
            &mut ctx.pending_container_specialization_patches,
        );
    }
    super::function_body::mark_threadsafe_callback_move_handlers(&mut result, ctx);
    let _ = ctx.inferred_binding_hints.pop();
    ctx.pop_empty_collection_hint_adoption();
    ctx.pop_empty_plain_dict_hint_adoption();
    restore_nested_function_mutations(previous_nested_mutations, ctx);
    restore_nested_function_captures(previous_nested_captures, ctx);
    result
}

pub(super) fn predeclare_nested_function_symbols(
    stmts: &[Stmt],
    inferred_types: &std::collections::HashMap<String, FunctionType>,
    ctx: &mut LowerCtx,
) {
    for stmt in stmts {
        if let Stmt::FunctionDef(func) = stmt {
            if let Some(function_type) = inferred_types.get(func.name.as_str()).cloned() {
                register_local_function_signature(func, function_type, ctx);
            } else {
                register_local_function_symbol(func, ctx);
            }
        }
    }
}

pub(in crate::lower) fn lower_stmt(
    stmt: &Stmt,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    match stmt {
        Stmt::AnnAssign(ann) => lower_ann_assign(ann, ctx),
        Stmt::Assign(assign) => lower_assign(assign, ctx),
        Stmt::AugAssign(aug) => lower_aug_assign(aug, ctx),
        Stmt::Return(ret) => Some(lower_return(ret, func_type, ctx)),
        Stmt::Expr(expr_stmt) => {
            // Check if this is a yield expression used as a statement
            if let Expr::Yield(yield_expr) = expr_stmt.value.as_ref() {
                if let Some(ref val) = yield_expr.value {
                    let value = lower_expr(val, ctx)?;
                    if let Some(borrowed) =
                        python_interop::python_context_borrow_in_owned_expr(&value, ctx)
                    {
                        ctx.error_with_code_at(
                            DiagnosticCode::PYCTX_INVALID_DECLARATION,
                            format!(
                                "invalid Python context declaration: entered binding '{borrowed}' is a context-scoped borrow and cannot escape by yield"
                            ),
                            val.range(),
                        );
                    }
                    consume_affine_value_name(&value, val.range(), ctx);
                    return Some(HirStmt::Yield { value });
                }
                statement_diagnostics::unsupported_form(
                    ctx,
                    "yield without a value",
                    yield_expr.range(),
                );
                return None;
            }
            let expr = lower_expr(&expr_stmt.value, ctx)?;
            python_interop::reject_python_context_borrow_discard(
                &expr,
                expr_stmt.value.range(),
                ctx,
            );
            // #[must_use] enforcement: Result values must not be silently discarded
            let expr_ty = expr.ty();
            if matches!(expr_ty, Type::Result(_, _)) {
                ctx.error_with_code_at(
                    DiagnosticCode::RESULT_UNUSED_VALUE,
                    format!(
                        "unused Result value of type '{}' must be used. Use 'let _ = expr' to explicitly discard",
                        expr_ty.display_name()
                    ),
                    expr_stmt.value.range(),
                );
            }
            finish_async_generator_advance_for_expr(ctx, &expr);
            Some(HirStmt::Expr { expr })
        }
        Stmt::If(if_stmt) => lower_if(if_stmt, func_type, ctx),
        Stmt::While(while_stmt) => lower_while(while_stmt, func_type, ctx),
        Stmt::For(for_stmt) => {
            if for_stmt.is_async {
                lower_async_for(for_stmt, func_type, ctx)
            } else {
                lower_for(for_stmt, func_type, ctx)
            }
        }
        Stmt::Break(break_stmt) => {
            if !ctx.in_loop() {
                super::flow_diagnostics::break_outside_loop(ctx, break_stmt.range());
                return None;
            }
            Some(HirStmt::Break)
        }
        Stmt::Continue(continue_stmt) => {
            if !ctx.in_loop() {
                super::flow_diagnostics::continue_outside_loop(ctx, continue_stmt.range());
                return None;
            }
            Some(HirStmt::Continue)
        }
        Stmt::Pass(_) => Some(HirStmt::Pass),
        Stmt::Delete(del_stmt) => {
            if del_stmt.targets.len() != 1 {
                statement_diagnostics::unsupported_form(
                    ctx,
                    "del with multiple targets",
                    del_stmt.range(),
                );
                return None;
            }
            if let Expr::Subscript(sub) = &del_stmt.targets[0] {
                let object = lower_expr(&sub.value, ctx)?;
                let index = lower_expr(&sub.slice, ctx)?;
                if reject_unavailable_dict_hash_key(
                    object.ty(),
                    index.ty(),
                    "dict item deletion",
                    sub.range(),
                    ctx,
                ) {
                    return None;
                }
                Some(HirStmt::Delete { object, index })
            } else {
                statement_diagnostics::unsupported_form(
                    ctx,
                    "del is only supported for collection items (del d[key], del a[i])",
                    del_stmt.targets[0].range(),
                );
                None
            }
        }
        Stmt::Assert(assert_stmt) => {
            let test = lower_expr(&assert_stmt.test, ctx)?;
            let msg = if let Some(ref msg_expr) = assert_stmt.msg {
                Some(lower_expr(msg_expr, ctx)?)
            } else {
                None
            };
            Some(HirStmt::Assert { test, msg })
        }
        Stmt::Raise(raise_stmt) => {
            if let Some(ref exc) = raise_stmt.exc {
                // Check if the raise expression is a string literal — disallow raise "message"
                if matches!(exc.as_ref(), Expr::StringLiteral(_) | Expr::FString(_)) {
                    super::result_diagnostics::invalid_raise_string(ctx, exc.range());
                    return None;
                }
                let value = lower_expr(exc, ctx)?;
                // Verify the raised value is an error type
                let raised_ty = value.ty();
                if !is_valid_error_type(raised_ty, ctx) {
                    let ty_name = format_type_name(raised_ty);
                    super::result_diagnostics::invalid_raise_non_error(
                        ctx,
                        ty_name.as_str(),
                        exc.range(),
                    );
                    return None;
                }
                Some(HirStmt::Raise { value })
            } else {
                super::result_diagnostics::invalid_bare_raise(ctx, raise_stmt.range());
                None
            }
        }
        Stmt::With(with_stmt) => {
            if with_stmt.is_async {
                return lower_async_with(with_stmt, func_type, ctx);
            }
            if with_stmt.items.is_empty() {
                statement_diagnostics::unsupported_form(
                    ctx,
                    "with statement must have at least one item",
                    with_stmt.range(),
                );
                return None;
            }
            let (items, body) = ctx.with_pushed_scope(|ctx| {
                let mut items = Vec::new();
                let mut previous_context_borrows = Vec::new();
                for item in &with_stmt.items {
                    let mut value = lower_expr(&item.context_expr, ctx)?;
                    let var_name = if let Some(ref vars) = item.optional_vars {
                        if let Expr::Name(n) = vars.as_ref() {
                            n.id.to_string()
                        } else {
                            statement_diagnostics::unsupported_form(
                                ctx,
                                "with target must be a simple name",
                                vars.range(),
                            );
                            return None;
                        }
                    } else {
                        format!("_with_val_{}", items.len())
                    };
                    let context_range = item.context_expr.range();
                    let context_owner = match &value {
                        HirExpr::Name { name, .. } => Some(name.clone()),
                        _ => None,
                    };
                    let mut python_kind = None;
                    if let Type::Result(ok_type, error_type) = value.ty().resolve_alias() {
                        let ok_type = ok_type.as_ref().clone();
                        let error_type = error_type.as_ref().clone();
                        if let Some(kind) = python_interop::python_context_item_kind(
                            &ok_type,
                            ctx,
                            context_range,
                        ) {
                            if !ctx.in_try_block {
                                ctx.error_with_code_at(
                                    DiagnosticCode::PYCTX_INVALID_DECLARATION,
                                    "invalid Python context declaration: fallible Python context construction requires an enclosing try block"
                                        .to_string(),
                                    context_range,
                                );
                                return None;
                            }
                            super::record_try_error_types(ctx, &error_type);
                            value = HirExpr::QuestionMark {
                                expr: Box::new(value),
                                ty: ok_type,
                            };
                            python_kind = Some(kind);
                        }
                    }
                    let val_ty = value.ty().clone();
                    if let Some(kind) = python_kind.or_else(|| {
                        python_interop::python_context_item_kind(&val_ty, ctx, context_range)
                    }) {
                        let HirWithItemKind::Python {
                            entered_type,
                            entered_is_opaque_borrow,
                            ..
                        } = &kind
                        else {
                            return None;
                        };
                        if let Some(name) = context_owner.as_deref() {
                            ctx.mark_moved_with_flow(name);
                        }
                        ctx.scope.define(var_name.clone(), entered_type.clone());
                        if *entered_is_opaque_borrow {
                            previous_context_borrows.push((
                                var_name.clone(),
                                ctx.python_context_borrows
                                    .insert(var_name.clone(), context_range),
                            ));
                        }
                        items.push(HirWithItem {
                            target: var_name,
                            context: value,
                            kind,
                        });
                        continue;
                    }
                    // Check if the type implements the ContextManager protocol (__enter__/__exit__)
                    let has_context_manager = if let Type::Class { name, methods, .. } = &val_ty {
                        let has_enter = methods
                            .iter()
                            .any(|(method_name, _)| method_name == "__enter__");
                        let has_exit = methods
                            .iter()
                            .any(|(method_name, _)| method_name == "__exit__");
                        if has_enter && has_exit {
                            true
                        } else if has_enter || has_exit {
                            protocol_diagnostics::context_manager_incomplete(
                                ctx,
                                name,
                                context_range,
                            );
                            false
                        } else {
                            protocol_diagnostics::context_manager_missing(ctx, name, context_range);
                            false
                        }
                    } else {
                        // Non-class types don't have methods — can't be context managers
                        let type_name = val_ty.display_name();
                        protocol_diagnostics::context_manager_missing(
                            ctx,
                            &type_name,
                            context_range,
                        );
                        false
                    };
                    // If the type has __enter__, the variable is bound to __enter__()'s return type
                    // We resolve the actual class type from ctx.class_types to get full fields/methods
                    let bound_ty = if has_context_manager {
                        if let Type::Class { methods, .. } = &val_ty {
                            let ret_ty = methods
                                .iter()
                                .find(|(name, _)| name == "__enter__")
                                .map(|(_, ft)| (*ft.return_type).clone())
                                .unwrap_or(val_ty.clone());
                            super::super::imported_class_identity::complete_context_enter_return_type(
                                &ctx.class_types,
                                &val_ty,
                                &ret_ty,
                            )
                        } else {
                            val_ty.clone()
                        }
                    } else {
                        val_ty.clone()
                    };
                    ctx.scope.define(var_name.clone(), bound_ty);
                    items.push(HirWithItem {
                        target: var_name,
                        context: value,
                        kind: HirWithItemKind::Native {
                            has_context_manager_protocol: has_context_manager,
                        },
                    });
                }
                let body = lower_stmts(&with_stmt.body, func_type, ctx);
                for (name, previous) in previous_context_borrows.into_iter().rev() {
                    if let Some(range) = previous {
                        ctx.python_context_borrows.insert(name, range);
                    } else {
                        ctx.python_context_borrows.remove(&name);
                    }
                }
                Some((items, body))
            })?;
            Some(HirStmt::With { items, body })
        }
        Stmt::Try(try_stmt) => {
            let prev_in_try = ctx.in_try_block;
            let prev_try_errors = std::mem::take(&mut ctx.try_block_error_types);
            ctx.in_try_block = true;
            let body = lower_stmts(&try_stmt.body, func_type, ctx);
            ctx.in_try_block = prev_in_try;
            let mut try_error_types =
                std::mem::replace(&mut ctx.try_block_error_types, prev_try_errors);

            // Also collect error types from raise statements in the body
            collect_raise_error_types(&body, &mut try_error_types);

            let mut handlers = Vec::new();
            let mut has_catch_all = false;
            let mut covered_types = std::collections::HashSet::new();

            for handler in &try_stmt.handlers {
                let ExceptHandler::ExceptHandler(h) = handler;
                let (error_type, error_type_range, invalid_error_type_form) =
                    if let Some(ref type_expr) = h.type_ {
                        if let Expr::Name(n) = type_expr.as_ref() {
                            (Some(n.id.to_string()), Some(n.range()), false)
                        } else {
                            (None, Some(type_expr.range()), true)
                        }
                    } else {
                        (None, None, false)
                    };
                let name = h.name.as_ref().map(std::string::ToString::to_string);
                let error_resolved_type = error_type
                    .as_ref()
                    .and_then(|error_name| ctx.class_types.get(error_name).cloned());
                let is_builtin_catch_all = error_resolved_type
                    .as_ref()
                    .is_some_and(Type::is_builtin_error_base);

                // Check if this is a catch-all (except Error) or a specific handler
                if invalid_error_type_form {
                    super::result_diagnostics::invalid_except_type(
                        ctx,
                        "except type must be a simple error class name",
                        error_type_range.unwrap_or_else(|| h.range()),
                    );
                } else if let Some(ref et) = error_type {
                    if is_builtin_catch_all {
                        has_catch_all = true;
                    } else {
                        // Validate the except type is a known error class
                        if !ctx.error_types.contains(et) {
                            super::result_diagnostics::unknown_except_type(
                                ctx,
                                et,
                                error_type_range.unwrap_or_else(|| h.range()),
                            );
                        }
                        if let Some(class_ty) = ctx.class_types.get(et) {
                            covered_types.insert(class_ty.clone());
                        }
                    }
                } else {
                    // Bare except (no type) — acts as catch-all
                    has_catch_all = true;
                }

                // Define the error variable in scope if named
                ctx.scope.push();
                if let Some(ref var_name) = name {
                    let error_var_ty = if let Some(ref et) = error_type {
                        if is_builtin_catch_all {
                            // catch-all: bind as the base Error type
                            error_resolved_type
                                .clone()
                                .unwrap_or_else(|| super::fallback_error_type("Error"))
                        } else if let Some(class_ty) = &error_resolved_type {
                            class_ty.clone()
                        } else {
                            // Unknown error type — already reported above
                            super::fallback_error_type(et)
                        }
                    } else {
                        // Bare except — error variable is base Error type
                        ctx.class_types
                            .get("Error")
                            .cloned()
                            .unwrap_or_else(|| super::fallback_error_type("Error"))
                    };
                    ctx.scope.define(var_name.clone(), error_var_ty);
                }
                let handler_body = lower_stmts(&h.body, func_type, ctx);
                ctx.scope.pop();

                handlers.push(HirExceptHandler {
                    error_type,
                    error_resolved_type,
                    name,
                    body: handler_body,
                });
            }

            // Exhaustiveness checking: if no catch-all, all error types must be covered
            // A parent type covers all its children (e.g., except IOError covers FileNotFoundError)
            // Subclasses partially cover their parent (e.g., except FileNotFoundError covers IOError::FileNotFound)
            if !has_catch_all && !try_error_types.is_empty() {
                let uncovered: Vec<String> = try_error_types
                    .iter()
                    .filter(|error_ty| {
                        if covered_types
                            .iter()
                            .any(|covered| error_ty.is_assignable_to(covered))
                        {
                            return false;
                        }
                        let error_name = error_ty.display_name();
                        let Some(children) = ctx.error_hierarchy.get(&error_name) else {
                            return true;
                        };
                        !children.iter().all(|child| {
                            ctx.class_types.get(child).is_some_and(|child_ty| {
                                covered_types
                                    .iter()
                                    .any(|covered| child_ty.is_assignable_to(covered))
                            })
                        })
                    })
                    .map(Type::display_name)
                    .collect();
                if !uncovered.is_empty() {
                    let mut sorted = uncovered;
                    sorted.sort();
                    super::result_diagnostics::uncovered_try_errors(
                        ctx,
                        &sorted.join(", "),
                        try_stmt.range(),
                    );
                }
            }

            let mut body_error_types: Vec<Type> = try_error_types.into_iter().collect();
            body_error_types.sort_by_key(Type::union_variant_name);
            Some(HirStmt::TryExcept {
                body,
                handlers,
                body_error_types,
            })
        }
        Stmt::Nonlocal(nonlocal) => {
            lower_nonlocal(nonlocal, ctx);
            None
        }
        Stmt::FunctionDef(func) => {
            // Nested function definition (def inside def)
            if let Some(decorator) = func.decorator_list.iter().find(|decorator| {
                python_interop::has_python_interop_decorator_syntax(std::slice::from_ref(decorator))
            }) {
                ctx.error_with_code_at(
                    DiagnosticCode::PYCALL_INVALID_SHAPE,
                    "invalid Python declaration call shape: nested Python declarations are not supported; declare the Python boundary at module or class scope"
                        .to_string(),
                    decorator.range,
                );
                return None;
            }
            // Extract the function type (params + return type)
            let ft = ctx
                .functions
                .get(func.name.as_str())
                .cloned()
                .unwrap_or_else(|| register_local_function_symbol(func, ctx));

            reject_unsupported_nested_async_generator(func, ft.return_type.as_ref(), ctx);
            super::ownership_diagnostics::reject_affine_nested_function_capture(
                ctx,
                func.name.as_str(),
                func.name.range(),
            );

            // Lower the nested function body
            let declared_nonlocals = collect_declared_nonlocals(&func.body);
            // Deferred container patches belong to the lexical function that
            // discovered them. A same-named binding in this nested function
            // must neither consume nor clear an enclosing function's patch.
            let enclosing_container_patches =
                std::mem::take(&mut ctx.pending_container_specialization_patches);
            ctx.enter_function_scope(declared_nonlocals.clone());

            // Define parameters in scope
            let mut params = Vec::new();
            for (i, param_def) in func.parameters.args.iter().enumerate() {
                let name = param_def.parameter.name.to_string();
                let ty = ft
                    .params
                    .get(i)
                    .map(|(_, t, _)| t.clone())
                    .unwrap_or(Type::Any);
                let convention = ft
                    .params
                    .get(i)
                    .map(|(_, _, convention)| *convention)
                    .unwrap_or_else(|| {
                        ast_convention_to_param(param_def.parameter.convention, &ty)
                    });
                ctx.scope
                    .define_parameter(name.clone(), ty.clone(), convention);
                let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));
                params.push(HirParam {
                    name,
                    ty,
                    default,
                    keyword_only: false,
                    convention,
                });
            }

            // Vararg parameter (*args)
            if let Some(ref vararg) = func.parameters.vararg {
                let name = vararg.name.to_string();
                let regular_count = func.parameters.args.len();
                let ty = ft
                    .params
                    .get(regular_count)
                    .map(|(_, t, _)| t.clone())
                    .unwrap_or(Type::Any);
                let convention = ft
                    .params
                    .get(regular_count)
                    .map(|(_, _, convention)| *convention)
                    .unwrap_or_else(|| ast_convention_to_param(vararg.convention, &ty));
                ctx.scope
                    .define_parameter(name.clone(), ty.clone(), convention);
                params.push(HirParam {
                    name,
                    ty,
                    default: None,
                    keyword_only: false,
                    convention,
                });
            }

            // Keyword-only args
            let regular_count =
                func.parameters.args.len() + usize::from(func.parameters.vararg.is_some());
            for (i, param_def) in func.parameters.kwonlyargs.iter().enumerate() {
                let name = param_def.parameter.name.to_string();
                let ty = ft
                    .params
                    .get(regular_count + i)
                    .map(|(_, t, _)| t.clone())
                    .unwrap_or(Type::Any);
                let convention = ft
                    .params
                    .get(regular_count + i)
                    .map(|(_, _, convention)| *convention)
                    .unwrap_or_else(|| {
                        ast_convention_to_param(param_def.parameter.convention, &ty)
                    });
                ctx.scope
                    .define_parameter(name.clone(), ty.clone(), convention);
                let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));
                params.push(HirParam {
                    name,
                    ty,
                    default,
                    keyword_only: true,
                    convention,
                });
            }

            let rust_interop = collect_rust_interop_declarations(
                &func.decorator_list,
                RustInteropOwner::Function,
                ctx,
                has_decorator(func, "blocking_io"),
                has_decorator(func, "cpu_heavy"),
                func.is_async,
            );
            let stub_body = classify_rust_interop_stub_body(
                &func.body,
                has_rust_interop_decorator_syntax(&func.decorator_list),
                ctx,
            );

            let previous_dynamic_python = ctx.current_function_trusts_dynamic_python;
            ctx.current_function_trusts_dynamic_python =
                has_decorator(func, "trust_python_dynamic");
            let body = if stub_body.skips_normal_body_lowering() {
                Vec::new()
            } else {
                super::function_body::lower_function_stmts(&func.body, &ft, ctx)
            };
            ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
            ctx.exit_function_scope();
            ctx.pending_container_specialization_patches = enclosing_container_patches;

            if !declared_nonlocals.is_empty() && hir_body_calls_function(&body, func.name.as_str())
            {
                super::flow_diagnostics::recursive_nonlocal_nested_function(
                    ctx,
                    &func.name,
                    func.name.range(),
                );
            }

            let return_annotation_range = func
                .returns
                .as_ref()
                .map_or_else(|| func.name.range(), |returns| returns.range());
            let inferred_return_type = if stub_body.skips_normal_body_lowering() {
                ft.return_type.as_ref().clone()
            } else {
                infer_function_return_type(
                    func.name.as_ref(),
                    func.is_async,
                    ft.return_type.as_ref(),
                    func.returns.is_some(),
                    &body,
                    |message| {
                        ctx.error_with_code_at(
                            DiagnosticCode::TYPE_MISMATCH,
                            message,
                            return_annotation_range,
                        );
                    },
                )
            };

            // Collect user-defined decorators
            let decorators: Vec<String> = func
                .decorator_list
                .iter()
                .filter_map(|d| {
                    if let Expr::Name(n) = &d.expression {
                        let name = n.id.to_string();
                        if name != "classmethod" && name != "staticmethod" {
                            Some(name)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            Some(HirStmt::NestedFunction {
                func: HirFunction {
                    name: func.name.to_string(),
                    params,
                    return_type: inferred_return_type,
                    body,
                    is_async: func.is_async,
                    method_kind: MethodKind::Regular,
                    receiver: None,
                    decorators,
                    rust_interop,
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: Vec::new(),
                },
                move_captures: false,
                capture_clones: Vec::new(),
            })
        }
        Stmt::Match(match_stmt) => lower_match(match_stmt, func_type, ctx),
        _ => {
            statement_diagnostics::unsupported_form(
                ctx,
                "unsupported statement type",
                stmt.range(),
            );
            None
        }
    }
}
