use super::assignment_widening::reconcile_optional_reassignment;
use super::async_with::lower_async_with;
use super::aug_assign_lowering::lower_aug_assign as lower_aug_assign_impl;
use super::binding_mutability::ensure_mutable_parameter_binding;
use super::builtin_calls::callable_builtin_element_type;
use super::container_literal_specialization::{
    apply_container_specialization_patches, type_contains_unknown_or_any,
    validate_subscript_assignment_target,
};
use super::control_flow_conditions::validate_control_flow_condition;
use super::diagnostics::{collect_raise_error_types, format_type_name, is_valid_error_type};
use super::expressions::{lower_expr, lower_star_unpack_assign, lower_tuple_unpack_assign};
use super::fixed_width_class_payload::class_specialization_payload_conflicts;
use super::fixed_width_fitting::{validate_fixed_width_initializer, FixedWidthInitializerFit};
use super::flow_helpers::then_body_always_exits;
use super::for_loop_safety::{is_collection_backed_iter_source, loop_body_mutates_iter_source};
use super::function_flow::infer_function_return_type;
use super::if_branch_bindings::{
    predeclare_exhaustive_if_assigned_names, seed_exhaustive_if_bindings,
};
use super::integer_nonzero_guards::{
    detect_false_nonzero_integer_guards, detect_true_nonzero_integer_guards,
};
use super::len_aliases::record_len_alias_fact;
use super::match_diagnostics;
use super::match_lowering::lower_match;
use super::name_diagnostics;
use super::narrowing::{apply_narrowing, detect_narrowing_condition};
use super::nonlocal_support::{
    collect_declared_nonlocals, hir_body_calls_function, lower_nonlocal, should_rebind_simple_name,
};
use super::numeric_sentinels::{
    apply_numeric_sentinel_patches, domain_typed_sentinel_expr, numeric_domain_for_type,
    numeric_sentinel_kind,
};
use super::ownership_diagnostics;
use super::protocol_diagnostics;
use super::sequence_guard_detection::{
    detect_false_exit_sequence_guards, detect_range_sequence_guards, detect_true_sequence_guards,
    detect_while_sequence_guards,
};
use super::sequence_guard_updates::{
    maybe_record_dict_assignment_guard, merge_exhaustive_branch_sequence_guards,
};
use super::sequence_pointers::record_sequence_pointer_fact;
use super::sequence_shapes::sequence_shape_fact;
use super::statement_diagnostics;
use super::task_scope_calls::task_group_spawn_owner;
use super::typing_and_functions::{
    ast_convention_to_param, register_local_function_signature, register_local_function_symbol,
    resolve_annotation_expr,
};
use super::LowerCtx;
use crate::hir_nodes::{
    HirExceptHandler, HirExpr, HirFunction, HirIteratorOp, HirParam, HirPattern, HirStmt,
    MethodKind,
};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{
    ExceptHandler, Expr, Pattern, Singleton, Stmt, StmtAnnAssign, StmtAssign, StmtAugAssign,
    StmtFor, StmtIf, StmtReturn, StmtWhile,
};
use sifr_type_system::{make_union, FunctionType, NarrowingCondition, OwnershipKind, Type};

fn empty_collection_literal_kind(expr: &Expr) -> Option<&'static str> {
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

fn hint_matches_empty_collection_shape(value_expr: &Expr, hint: &Type) -> bool {
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

fn should_adopt_inferred_binding_hint(
    value_expr: &Expr,
    value_ty: &Type,
    hint: &Type,
    allow_empty_collection_hint: bool,
) -> bool {
    if !type_contains_unknown_or_any(value_ty) {
        return false;
    }
    if empty_collection_literal_kind(value_expr).is_some() {
        return allow_empty_collection_hint
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

pub(super) fn lower_stmts(
    stmts: &[Stmt],
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Vec<HirStmt> {
    let nested_inference =
        super::nested_function_inference::infer_nested_function_types(stmts, ctx);
    let can_adopt_empty_collection_hints = !nested_inference.function_types.is_empty();
    ctx.inferred_binding_hints
        .push(nested_inference.binding_hints.clone());
    ctx.push_empty_collection_hint_adoption(can_adopt_empty_collection_hints);
    predeclare_nested_function_symbols(stmts, &nested_inference.function_types, ctx);

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
        if let Some(hir_stmt) = lower_stmt(stmt, func_type, ctx) {
            result.push(hir_stmt);
        }
        apply_numeric_sentinel_patches(&mut result, &mut ctx.pending_numeric_sentinel_patches);
        apply_container_specialization_patches(
            &mut result,
            &mut ctx.pending_container_specialization_patches,
        );
    }
    let _ = ctx.inferred_binding_hints.pop();
    ctx.pop_empty_collection_hint_adoption();
    result
}
fn predeclare_nested_function_symbols(
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

pub(super) fn lower_stmt(
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
            Some(HirStmt::Expr { expr })
        }
        Stmt::If(if_stmt) => lower_if(if_stmt, func_type, ctx),
        Stmt::While(while_stmt) => lower_while(while_stmt, func_type, ctx),
        Stmt::For(for_stmt) => lower_for(for_stmt, func_type, ctx),
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
                for item in &with_stmt.items {
                    let value = lower_expr(&item.context_expr, ctx)?;
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
                    let val_ty = value.ty().clone();
                    let context_range = item.context_expr.range();
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
                            // If the return type is a class, look up the fully-defined version
                            if let Type::Class { name: ret_name, .. } = &ret_ty {
                                ctx.class_types.get(ret_name).cloned().unwrap_or(ret_ty)
                            } else {
                                ret_ty
                            }
                        } else {
                            val_ty.clone()
                        }
                    } else {
                        val_ty.clone()
                    };
                    ctx.scope.define(var_name.clone(), bound_ty);
                    items.push((var_name, value, has_context_manager));
                }
                let body = lower_stmts(&with_stmt.body, func_type, ctx);
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

                // Check if this is a catch-all (except Error) or a specific handler
                if invalid_error_type_form {
                    super::result_diagnostics::invalid_except_type(
                        ctx,
                        "except type must be a simple error class name",
                        error_type_range.unwrap_or_else(|| h.range()),
                    );
                } else if let Some(ref et) = error_type {
                    if et == "Error" {
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
                        covered_types.insert(et.clone());
                    }
                } else {
                    // Bare except (no type) — acts as catch-all
                    has_catch_all = true;
                }

                // Define the error variable in scope if named
                ctx.scope.push();
                if let Some(ref var_name) = name {
                    let error_var_ty = if let Some(ref et) = error_type {
                        if et == "Error" {
                            // catch-all: bind as the base Error type
                            ctx.class_types
                                .get("Error")
                                .cloned()
                                .unwrap_or_else(|| Type::Class {
                                    name: "Error".to_string(),
                                    fields: vec![("message".to_string(), Type::Str)],
                                    methods: vec![],
                                    parent_class: None,
                                })
                        } else if let Some(class_ty) = ctx.class_types.get(et) {
                            class_ty.clone()
                        } else {
                            // Unknown error type — already reported above
                            Type::Class {
                                name: et.clone(),
                                fields: vec![("message".to_string(), Type::Str)],
                                methods: vec![],
                                parent_class: None,
                            }
                        }
                    } else {
                        // Bare except — error variable is base Error type
                        ctx.class_types
                            .get("Error")
                            .cloned()
                            .unwrap_or_else(|| Type::Class {
                                name: "Error".to_string(),
                                fields: vec![("message".to_string(), Type::Str)],
                                methods: vec![],
                                parent_class: None,
                            })
                    };
                    ctx.scope.define(var_name.clone(), error_var_ty);
                }
                let handler_body = lower_stmts(&h.body, func_type, ctx);
                ctx.scope.pop();

                // Resolve the error type for codegen
                let error_resolved_type = error_type
                    .as_ref()
                    .and_then(|et| ctx.class_types.get(et).cloned());
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
                // Expand covered_types: if a parent is covered, all its children are covered
                let mut expanded_covered = covered_types.clone();
                for covered in &covered_types {
                    if let Some(children) = ctx.error_hierarchy.get(covered) {
                        for child in children {
                            expanded_covered.insert(child.clone());
                        }
                    }
                }
                // Check if subclasses fully cover their parent
                // If all children of a parent are covered, the parent is covered
                for (parent, children) in &ctx.error_hierarchy {
                    if try_error_types.contains(parent) && !expanded_covered.contains(parent) {
                        let all_children_covered =
                            children.iter().all(|c| expanded_covered.contains(c));
                        if all_children_covered {
                            expanded_covered.insert(parent.clone());
                        }
                    }
                }
                let uncovered: Vec<String> = try_error_types
                    .iter()
                    .filter(|et| !expanded_covered.contains(*et))
                    .cloned()
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

            let body_error_types: Vec<String> = try_error_types.into_iter().collect();
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
            // Extract the function type (params + return type)
            let ft = ctx
                .functions
                .get(func.name.as_str())
                .cloned()
                .unwrap_or_else(|| register_local_function_symbol(func, ctx));

            // Lower the nested function body
            let declared_nonlocals = collect_declared_nonlocals(&func.body);
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
                    .define_parameter(name.clone(), ty.clone(), convention.is_mutable());
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
                    .define_parameter(name.clone(), ty.clone(), convention.is_mutable());
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
                    .define_parameter(name.clone(), ty.clone(), convention.is_mutable());
                let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));
                params.push(HirParam {
                    name,
                    ty,
                    default,
                    keyword_only: true,
                    convention,
                });
            }

            let body = lower_stmts(&func.body, &ft, ctx);
            ctx.exit_function_scope();

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
            let inferred_return_type = infer_function_return_type(
                func.name.as_ref(),
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
            );

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
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    decorators,
                    type_params: Vec::new(),
                },
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

pub(super) fn lower_pattern(
    pattern: &Pattern,
    subject_ty: &Type,
    ctx: &mut LowerCtx,
) -> Option<HirPattern> {
    match pattern {
        Pattern::MatchAs(pat_as) => {
            if pat_as.pattern.is_none() && pat_as.name.is_none() {
                // `case _:` — wildcard
                return Some(HirPattern::Wildcard);
            }
            if let Some(name) = &pat_as.name {
                let var_name = name.to_string();
                if let Some(inner_pat) = &pat_as.pattern {
                    // `case SomePattern as x:` — match inner pattern, bind to x
                    let inner = lower_pattern(inner_pat, subject_ty, ctx)?;
                    // For now, treat as capture with narrowed type
                    let narrowed_ty = pattern_narrowed_type(&inner, subject_ty, ctx);
                    let _ = inner; // inner pattern info embedded in capture
                    return Some(HirPattern::Capture {
                        name: var_name,
                        ty: narrowed_ty,
                    });
                }
                // `case x:` — capture pattern
                return Some(HirPattern::Capture {
                    name: var_name,
                    ty: subject_ty.clone(),
                });
            }
            if let Some(inner_pat) = &pat_as.pattern {
                return lower_pattern(inner_pat, subject_ty, ctx);
            }
            Some(HirPattern::Wildcard)
        }
        Pattern::MatchSingleton(singleton) => match &singleton.value {
            Singleton::None => Some(HirPattern::None),
            Singleton::True => Some(HirPattern::Literal {
                value: HirExpr::BoolLiteral(true),
            }),
            Singleton::False => Some(HirPattern::Literal {
                value: HirExpr::BoolLiteral(false),
            }),
        },
        Pattern::MatchValue(val_pat) => {
            // Could be a literal or an attribute access like Color.RED
            if let Expr::Attribute(attr) = val_pat.value.as_ref() {
                let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
                    n.id.to_string()
                } else {
                    match_diagnostics::invalid_pattern_form(
                        ctx,
                        "complex attribute pattern is not supported",
                        attr.value.range(),
                    );
                    return None;
                };
                let attr_name = attr.attr.to_string();
                Some(HirPattern::Value {
                    path: vec![obj_name, attr_name],
                })
            } else {
                // Try to lower as a literal expression
                let expr = lower_expr(val_pat.value.as_ref(), ctx)?;
                let value = match validate_fixed_width_initializer(
                    ctx,
                    subject_ty,
                    &expr,
                    val_pat.value.range(),
                ) {
                    FixedWidthInitializerFit::Fits(value) => value,
                    FixedWidthInitializerFit::Rejected => return None,
                    FixedWidthInitializerFit::NotConst => expr,
                };
                Some(HirPattern::Literal { value })
            }
        }
        Pattern::MatchOr(or_pat) => {
            let mut patterns = Vec::new();
            for p in &or_pat.patterns {
                patterns.push(lower_pattern(p, subject_ty, ctx)?);
            }
            Some(HirPattern::Or { patterns })
        }
        Pattern::MatchClass(class_pat) => {
            let class_name = if let Expr::Name(n) = class_pat.cls.as_ref() {
                n.id.to_string()
            } else {
                match_diagnostics::invalid_pattern_form(
                    ctx,
                    "class pattern class name must be a simple name",
                    class_pat.cls.range(),
                );
                return None;
            };

            // Resolve the class type to get field types
            let class_ty = ctx.class_types.get(&class_name).cloned();

            let mut fields = Vec::new();
            for kw in &class_pat.arguments.keywords {
                let field_name = kw.attr.to_string();
                let field_ty = if let Some(Type::Class {
                    fields: class_fields,
                    ..
                }) = &class_ty
                {
                    let Some(field_ty) = class_fields
                        .iter()
                        .find(|(n, _)| n == &field_name)
                        .map(|(_, t)| t.clone())
                    else {
                        match_diagnostics::invalid_class_pattern_field(
                            ctx,
                            &class_name,
                            &field_name,
                            &class_fields
                                .iter()
                                .map(|(n, _)| n.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                            kw.attr.range(),
                        );
                        return None;
                    };
                    field_ty
                } else {
                    Type::Any
                };
                let field_pattern = lower_pattern(&kw.pattern, &field_ty, ctx)?;
                fields.push((field_name, field_pattern));
            }

            Some(HirPattern::Class { class_name, fields })
        }
        Pattern::MatchSequence(seq_pat) => {
            if seq_pat.patterns.is_empty() {
                return Some(HirPattern::Tuple { elements: vec![] });
            }
            let elem_types: Vec<Type> = if let Type::Tuple(ref elems) = *subject_ty {
                elems.clone()
            } else {
                match_diagnostics::invalid_pattern_form(
                    ctx,
                    &format!(
                        "tuple pattern requires subject of tuple type, got '{}'",
                        subject_ty.display_name()
                    ),
                    seq_pat.range(),
                );
                return None;
            };
            if elem_types.len() != seq_pat.patterns.len() {
                match_diagnostics::invalid_pattern_form(
                    ctx,
                    &format!(
                        "tuple pattern expects {} element(s), subject has {}",
                        seq_pat.patterns.len(),
                        elem_types.len()
                    ),
                    seq_pat.range(),
                );
                return None;
            }
            let mut elements = Vec::new();
            for (i, pat) in seq_pat.patterns.iter().enumerate() {
                let elem_ty = elem_types[i].clone();
                if let Some(lowered) = lower_pattern(pat, &elem_ty, ctx) {
                    elements.push(lowered);
                } else {
                    return None;
                }
            }
            Some(HirPattern::Tuple { elements })
        }
        Pattern::MatchMapping(_) => {
            match_diagnostics::invalid_pattern_form(
                ctx,
                "mapping patterns are not yet supported",
                pattern.range(),
            );
            None
        }
        Pattern::MatchStar(_) => {
            match_diagnostics::invalid_pattern_form(
                ctx,
                "star patterns are not yet supported",
                pattern.range(),
            );
            None
        }
    }
}

pub(super) fn pattern_narrowed_type(
    pattern: &HirPattern,
    subject_ty: &Type,
    ctx: &LowerCtx,
) -> Type {
    match pattern {
        HirPattern::None => Type::None,
        HirPattern::Class { class_name, .. } => {
            // Look up the class type
            if let Some(class_ty) = ctx.class_types.get(class_name) {
                class_ty.clone()
            } else {
                subject_ty.clone()
            }
        }
        _ => subject_ty.clone(),
    }
}

pub(super) fn bind_pattern_vars(pattern: &HirPattern, ctx: &mut LowerCtx) {
    match pattern {
        HirPattern::Capture { name, ty } => {
            ctx.scope.define(name.clone(), ty.clone());
        }
        HirPattern::Class { fields, .. } => {
            for (_, field_pat) in fields {
                bind_pattern_vars(field_pat, ctx);
            }
        }
        HirPattern::Or { patterns } => {
            // Bind from first pattern (all OR alternatives should bind same names)
            if let Some(first) = patterns.first() {
                bind_pattern_vars(first, ctx);
            }
        }
        HirPattern::Tuple { elements } => {
            for elem in elements {
                bind_pattern_vars(elem, ctx);
            }
        }
        _ => {}
    }
}

fn seed_binding_after_failed_initializer(
    ctx: &mut LowerCtx,
    name: &str,
    ty: Type,
    is_explicit_local: bool,
    error_taint: crate::scope::ErrorTaint,
) {
    ctx.scope
        .define_poisoned_local(name.to_string(), ty, is_explicit_local, error_taint);
    ctx.empty_dict_specializations.remove(name);
    ctx.pending_container_specialization_patches.remove(name);
    ctx.clear_numeric_sentinel_var(name);
    ctx.clear_sequence_shape_fact(name);
}

fn failed_initializer_taint(
    ctx: &mut LowerCtx,
    name: &str,
    range: ruff_text_size::TextRange,
    error_count_before_initializer: usize,
) -> Option<crate::scope::ErrorTaint> {
    let taint = ctx.error_taint_since(error_count_before_initializer);
    if taint.is_none() {
        ctx.error_with_code_at(
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
            format!(
                "internal compiler error: failed initializer for '{name}' did not emit a diagnostic"
            ),
            range,
        );
    }
    taint
}

fn invalidate_rebound_binding_facts(ctx: &mut LowerCtx, name: &str) {
    ctx.scope.clear_narrowing(name);
    ctx.clear_sequence_guards_for_binding(name);
    ctx.clear_proven_nonzero_integer_binding(name);
}

pub(super) fn lower_ann_assign(ann: &StmtAnnAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let name = if let Expr::Name(n) = ann.target.as_ref() {
        n.id.to_string()
    } else {
        statement_diagnostics::invalid_assignment_target(
            ctx,
            "annotated assignment target must be a simple name",
            ann.target.range(),
        );
        return None;
    };
    let declared_type = resolve_annotation_expr(&ann.annotation, ctx);

    let (value, initializer_range) = if let Some(val) = &ann.value {
        let initializer_range = val.range();
        let error_count_before_initializer = ctx.error_count();
        let mut expr = if let Some(kind) = numeric_sentinel_kind(val) {
            if let Some(domain) = numeric_domain_for_type(&declared_type) {
                domain_typed_sentinel_expr(kind, domain)
            } else if let Some(expr) = lower_expr(val, ctx) {
                expr
            } else {
                let error_taint = failed_initializer_taint(
                    ctx,
                    &name,
                    initializer_range,
                    error_count_before_initializer,
                )?;
                seed_binding_after_failed_initializer(
                    ctx,
                    &name,
                    declared_type.clone(),
                    true,
                    error_taint,
                );
                return None;
            }
        } else if let Some(expr) = lower_expr(val, ctx) {
            expr
        } else {
            let error_taint = failed_initializer_taint(
                ctx,
                &name,
                initializer_range,
                error_count_before_initializer,
            )?;
            seed_binding_after_failed_initializer(
                ctx,
                &name,
                declared_type.clone(),
                true,
                error_taint,
            );
            return None;
        };
        let expr_ty = expr.ty().clone();
        // Inside try blocks, auto-unwrap Result[T, E] when declared type is T
        if ctx.in_try_block {
            if let Type::Result(ref ok_ty, ref err_ty) = expr_ty {
                if ok_ty.as_ref().is_assignable_to(&declared_type) {
                    // Track the error type for exhaustiveness checking
                    if let Type::Class { name, .. } = err_ty.as_ref() {
                        ctx.try_block_error_types.insert(name.clone());
                    }
                    expr = HirExpr::QuestionMark {
                        expr: Box::new(expr),
                        ty: declared_type.clone(),
                    };
                }
            }
        }
        // Type check: value must be assignable to declared type
        let final_ty = expr.ty().clone();
        // int literals are assignable to bigint (coercion: 42 -> BigInt::from(42))
        let is_int_to_bigint = final_ty == Type::Int && declared_type == Type::BigInt;
        let fixed_width_fit =
            validate_fixed_width_initializer(ctx, &declared_type, &expr, initializer_range);
        let fixed_width_not_const = matches!(fixed_width_fit, FixedWidthInitializerFit::NotConst);
        if let FixedWidthInitializerFit::Fits(folded_expr) = fixed_width_fit {
            expr = folded_expr;
        }
        let class_specialization_conflict =
            class_specialization_payload_conflicts(&final_ty, &declared_type);
        if !is_int_to_bigint
            && ((fixed_width_not_const && !final_ty.is_assignable_to(&declared_type))
                || class_specialization_conflict)
        {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "type mismatch: expected '{}', got '{}'",
                    declared_type.display_name(),
                    final_ty.display_name()
                ),
                initializer_range,
            );
        }
        (expr, initializer_range)
    } else {
        name_diagnostics::uninitialized_variable(ctx, &name, ann.target.range());
        return None;
    };

    // Track move: if RHS is a variable name with Move ownership, mark it as moved.
    // Also check escape analysis: storing a borrowed parameter into a local variable
    // would allow it to outlive the borrow, which is not allowed.
    if let HirExpr::Name {
        name: ref src_name,
        ref ty,
    } = value
    {
        if ty.ownership() == sifr_type_system::OwnershipKind::Move {
            // Escape analysis: cannot store a borrowed parameter into a new binding
            if ctx.borrowed_params.contains(src_name.as_str()) {
                ownership_diagnostics::borrowed_parameter_store_escape(
                    ctx,
                    src_name,
                    initializer_range,
                );
            } else {
                ctx.scope.mark_moved(src_name);
            }
        }
    }

    ctx.empty_dict_specializations.remove(&name);
    ctx.pending_container_specialization_patches.remove(&name);
    ctx.scope
        .define_explicit_local(name.clone(), declared_type.clone());
    if let Some(kind) = ann
        .value
        .as_ref()
        .and_then(|value| numeric_sentinel_kind(value))
    {
        ctx.record_numeric_sentinel_initializer(name.clone(), kind);
        if let Some(domain) = numeric_domain_for_type(&declared_type) {
            ctx.resolve_numeric_sentinel_domain(&name, domain);
        }
    } else {
        ctx.clear_numeric_sentinel_var(&name);
    }
    if let Some(fact) = ann
        .value
        .as_ref()
        .and_then(|value| sequence_shape_fact(&name, value))
    {
        ctx.record_sequence_shape_fact(fact);
    } else {
        ctx.clear_sequence_shape_fact(&name);
    }
    let initializer = ann.value.as_ref()?;
    record_len_alias_fact(ctx, &name, initializer);
    record_sequence_pointer_fact(ctx, &name, initializer);
    Some(HirStmt::Let {
        name,
        ty: declared_type,
        value,
        is_mutable: true,
    })
}
/// Handle chained assignment: x = y = z = 0
/// Expands into: z = 0; y = z; x = y (right-to-left, last target gets the value first)
pub(super) fn lower_chained_assign(assign: &StmtAssign, ctx: &mut LowerCtx) -> Vec<HirStmt> {
    let mut result = Vec::new();
    // Lower the value expression once
    let Some(value) = lower_expr(&assign.value, ctx) else {
        return result;
    };
    let val_ty = value.ty().clone();

    // Process targets in reverse order (rightmost gets the value first)
    let targets: Vec<_> = assign.targets.iter().collect();
    for (i, target) in targets.iter().rev().enumerate() {
        if let Expr::Name(n) = target {
            let name = n.id.to_string();
            if i == 0 {
                // First (rightmost) target gets the actual value
                let existing = ctx.scope.lookup(&name);
                if existing.is_some() {
                    // Reassignment
                    invalidate_rebound_binding_facts(ctx, &name);
                    ctx.empty_dict_specializations.remove(&name);
                    ctx.pending_container_specialization_patches.remove(&name);
                    result.push(HirStmt::Assign {
                        name: name.clone(),
                        value: value.clone(),
                    });
                } else {
                    // New variable
                    ctx.scope.define(name.clone(), val_ty.clone());
                    ctx.empty_dict_specializations.remove(&name);
                    ctx.pending_container_specialization_patches.remove(&name);
                    result.push(HirStmt::Let {
                        name: name.clone(),
                        ty: val_ty.clone(),
                        value: value.clone(),
                        is_mutable: true,
                    });
                }
            } else {
                // Subsequent targets get a reference to the previous target
                let prev_target = match targets.get(targets.len() - i) {
                    Some(Expr::Name(prev_n)) => prev_n.id.to_string(),
                    _ => continue,
                };
                let name_expr = HirExpr::Name {
                    name: prev_target.clone(),
                    ty: val_ty.clone(),
                };
                let existing = ctx.scope.lookup(&name);
                if existing.is_some() {
                    invalidate_rebound_binding_facts(ctx, &name);
                    ctx.empty_dict_specializations.remove(&name);
                    ctx.pending_container_specialization_patches.remove(&name);
                    result.push(HirStmt::Assign {
                        name: name.clone(),
                        value: name_expr,
                    });
                } else {
                    ctx.scope.define(name.clone(), val_ty.clone());
                    ctx.empty_dict_specializations.remove(&name);
                    ctx.pending_container_specialization_patches.remove(&name);
                    result.push(HirStmt::Let {
                        name: name.clone(),
                        ty: val_ty.clone(),
                        value: name_expr,
                        is_mutable: true,
                    });
                }
            }
        } else {
            statement_diagnostics::invalid_assignment_target(
                ctx,
                "chained assignment targets must be simple names",
                target.range(),
            );
        }
    }

    result
}

fn resolve_field_type_from_type(object_ty: &Type, field_name: &str) -> Option<Type> {
    let resolved = object_ty.resolve_alias();
    if let Type::Class { fields, .. } = resolved {
        return fields
            .iter()
            .find(|(name, _)| name == field_name)
            .map(|(_, ty)| ty.clone());
    }
    if let Type::Union(members) = resolved {
        let mut field_members = Vec::new();
        let mut has_none = false;
        for member in members {
            match member.resolve_alias() {
                Type::None => {
                    has_none = true;
                }
                Type::Class { fields, .. } => {
                    let (_, member_field_ty) =
                        fields.iter().find(|(name, _)| name == field_name)?;
                    field_members.push(member_field_ty.clone());
                }
                _ => return None,
            }
        }
        if field_members.is_empty() {
            return None;
        }
        if has_none {
            field_members.push(Type::None);
        }
        return Some(make_union(field_members));
    }
    None
}

pub(super) fn resolve_object_field_type(
    ctx: &LowerCtx,
    object_name: &str,
    field_name: &str,
) -> Type {
    ctx.scope
        .lookup(object_name)
        .and_then(|info| resolve_field_type_from_type(info.effective_type(), field_name))
        .unwrap_or(Type::Unknown)
}

pub(super) fn lower_assign(assign: &StmtAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
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
            let value = lower_expr(&assign.value, ctx)?;
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
        let value = lower_expr(&assign.value, ctx)?;
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
            let obj_name = if let Expr::Name(n) = inner_sub.value.as_ref() {
                n.id.to_string()
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
            let obj_ty = ctx
                .scope
                .lookup(&obj_name)
                .map(|info| info.effective_type().clone())
                .unwrap_or(Type::Unknown);
            if matches!(obj_ty.resolve_alias(), Type::Bytes) {
                super::ownership_diagnostics::immutable_bytes_subscript_assignment(
                    ctx,
                    inner_sub.range(),
                );
                return None;
            }
            let outer_index = lower_expr(&inner_sub.slice, ctx)?;
            let inner_index = lower_expr(&sub.slice, ctx)?;
            let value = lower_expr(&assign.value, ctx)?;
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
            let value = lower_expr(&assign.value, ctx)?;
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
        let value = lower_expr(&assign.value, ctx)?;
        let object_ty = validate_subscript_assignment_target(
            ctx,
            &obj_name,
            &obj_ty,
            index.ty(),
            value.ty(),
            sub.range(),
        );
        maybe_record_dict_assignment_guard(ctx, &object_ty, &obj_name, &sub.slice);
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
    let error_count_before_initializer = ctx.error_count();
    let Some(value) = lower_expr(&assign.value, ctx) else {
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

    // Track move: if RHS is a variable name with Move ownership, mark it as moved
    if let HirExpr::Name {
        name: ref src_name,
        ref ty,
    } = value
    {
        if ty.ownership() == sifr_type_system::OwnershipKind::Move {
            ctx.scope.mark_moved(src_name);
        }
    }

    // Check if variable already exists
    if should_treat_as_existing_binding {
        let Some(info) = ctx.scope.lookup(&name) else {
            name_diagnostics::undefined_variable(ctx, &name, name_range);
            return None;
        };
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
        // Reset moved state on reassignment
        ctx.scope.reset_moved(&name);
        ctx.task_handle_group_owners.remove(&name);
        invalidate_rebound_binding_facts(ctx, &name);
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
        let binding_ty = ctx
            .inferred_binding_hint(&name)
            .filter(|hint| {
                should_adopt_inferred_binding_hint(
                    &assign.value,
                    &value_ty,
                    hint,
                    ctx.can_adopt_empty_collection_hints(),
                )
            })
            .cloned()
            .unwrap_or_else(|| value_ty.clone());
        ctx.scope.define(name.clone(), binding_ty.clone());
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
        ctx.empty_dict_specializations.remove(&name);
        ctx.pending_container_specialization_patches.remove(&name);
        record_sequence_pointer_fact(ctx, &name, &assign.value);
        Some(HirStmt::Let {
            name,
            ty: binding_ty,
            value,
            is_mutable: true,
        })
    }
}

pub(super) fn lower_aug_assign(aug: &StmtAugAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    lower_aug_assign_impl(aug, ctx)
}

pub(super) fn lower_return(
    ret: &StmtReturn,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> HirStmt {
    let value = if let Some(val) = &ret.value {
        let Some(expr) = lower_expr(val, ctx) else {
            // Keep control-flow shape intact after expression diagnostics so
            // return-completeness analysis does not emit a cascade error.
            return HirStmt::Return {
                value: Some(HirExpr::NoneLiteral),
            };
        };
        let expr_ty = expr.ty().clone();

        // Escape analysis: returning a borrowed parameter is a compile error.
        // The programmer must add `own` at the signature boundary, or call `.clone()` explicitly.
        if let HirExpr::Name { name, ty } = &expr {
            if ctx.borrowed_params.contains(name.as_str()) && ty.ownership() == OwnershipKind::Move
            {
                let range = val.range();
                ownership_diagnostics::borrowed_parameter_return_escape(ctx, name, range);
            }
        }

        // If the function returns Result[T, E] and the value is T (not Result), wrap in Ok()
        if let Type::Result(ref ok_ty, _) = *func_type.return_type {
            if expr_ty.is_assignable_to(ok_ty) && !matches!(expr_ty, Type::Result(_, _)) {
                // Wrap in Ok()
                return HirStmt::Return {
                    value: Some(HirExpr::OkWrap {
                        ty: func_type.return_type.as_ref().clone(),
                        value: Box::new(expr),
                    }),
                };
            }
        }

        if !expr_ty.is_assignable_to(&func_type.return_type) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "return type mismatch: expected '{}', got '{}'",
                    func_type.return_type.display_name(),
                    expr_ty.display_name()
                ),
                val.range(),
            );
        }
        Some(expr)
    } else {
        if *func_type.return_type != Type::None {
            // If function returns Result[(), E], wrap in Ok(())
            if let Type::Result(ref ok_ty, _) = *func_type.return_type {
                if **ok_ty == Type::None {
                    return HirStmt::Return {
                        value: Some(HirExpr::OkWrap {
                            ty: func_type.return_type.as_ref().clone(),
                            value: Box::new(HirExpr::NoneLiteral),
                        }),
                    };
                }
            }
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "type mismatch: expected '{}', got 'None'",
                    func_type.return_type.display_name()
                ),
                ret.range(),
            );
        }
        None
    };

    HirStmt::Return { value }
}

pub(super) fn lower_if(
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
    let then_sequence_guards = ctx.save_sequence_guards();

    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);
    ctx.restore_sequence_guards(&saved_sequence_guards);
    ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);

    let mut all_conditions: Vec<NarrowingCondition> = Vec::new();
    if let Some(ref cond) = narrowing_cond {
        all_conditions.push(cond.clone());
    }

    let mut branch_moved_states: Vec<_> = vec![then_moved];
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
            branch_sequence_states.push(ctx.save_sequence_guards());

            ctx.scope.restore_narrowing_state(&elif_saved);
            ctx.scope.restore_moved_state(&saved_moved);
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
            ctx.restore_sequence_guards(&saved_sequence_guards);
            ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);
            for prev_cond in &all_conditions {
                apply_narrowing(ctx, prev_cond, false);
            }
            ctx.scope.push();
            let body = lower_stmts(&clause.body, func_type, ctx);
            ctx.scope.pop();
            branch_moved_states.push(ctx.scope.save_moved_state());
            branch_sequence_states.push(ctx.save_sequence_guards());
            body
        });

    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);
    ctx.restore_sequence_guards(&saved_sequence_guards);
    ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);

    for branch_state in &branch_moved_states {
        for (name, was_moved) in branch_state {
            if *was_moved {
                ctx.scope.mark_moved(name);
            }
        }
    }

    seed_exhaustive_if_bindings(ctx, &then_body, &elif_clauses, else_body.as_ref());
    merge_exhaustive_branch_sequence_guards(ctx, else_body.is_some(), &branch_sequence_states);

    // Early-return narrowing: if the then-body always exits (return/break/continue/raise),
    // apply the inverse narrowing after the if block.
    // e.g., `if x is None: return` -> after the if, x is not None
    if let Some(ref cond) = narrowing_cond {
        if then_body_always_exits(&then_body) && elif_clauses.is_empty() && else_body.is_none() {
            apply_narrowing(ctx, cond, false);
        }
    }
    if then_body_always_exits(&then_body) && elif_clauses.is_empty() && else_body.is_none() {
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

pub(super) fn lower_while(
    while_stmt: &StmtWhile,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let narrowing_cond = detect_narrowing_condition(&while_stmt.test, ctx);
    let condition = lower_expr(&while_stmt.test, ctx)?;
    validate_control_flow_condition(&condition, "while", while_stmt.test.range(), ctx);
    let saved_narrowing_state = ctx.scope.save_narrowing_state();
    let saved_sequence_guards = ctx.save_sequence_guards();
    let saved_nonzero_integer_bindings = ctx.save_proven_nonzero_integer_bindings();
    if let Some(ref cond) = narrowing_cond {
        apply_narrowing(ctx, cond, true);
    }
    for guard in detect_while_sequence_guards(while_stmt, ctx) {
        ctx.add_sequence_guard(guard);
    }
    for name in detect_true_nonzero_integer_guards(&while_stmt.test, ctx) {
        ctx.add_proven_nonzero_integer_binding(name);
    }

    // Snapshot moved state before loop to detect moves inside the body
    let moved_before_loop = ctx.scope.save_moved_state();

    ctx.scope.push();
    ctx.loop_depth += 1;
    let body = lower_stmts(&while_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();
    ctx.scope.restore_narrowing_state(&saved_narrowing_state);
    ctx.restore_sequence_guards(&saved_sequence_guards);
    ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);

    // Check for outer-scope variables moved inside the loop body
    let newly_moved = ctx.scope.moved_since(&moved_before_loop);
    for var_name in &newly_moved {
        ownership_diagnostics::moved_across_loop(ctx, var_name, while_stmt.range());
    }

    let else_body = if while_stmt.orelse.is_empty() {
        None
    } else {
        ctx.scope.push();
        let else_stmts = lower_stmts(&while_stmt.orelse, func_type, ctx);
        ctx.scope.pop();
        Some(else_stmts)
    };

    ctx.clear_sequence_pointers();

    Some(HirStmt::While {
        condition,
        body,
        else_body,
    })
}

pub(super) fn lower_for(
    for_stmt: &StmtFor,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    // Lower the iterable expression and normalize protocol usage through `iter(...)`.
    let iterable_expr = lower_expr(&for_stmt.iter, ctx)?;
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
    let iter_expr = HirExpr::IteratorCall {
        op: HirIteratorOp::Iter,
        args: vec![iterable_expr],
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
            ctx.scope.mark_moved(source_name);
        }
    }

    // Snapshot moved state before loop to detect moves inside the body
    let moved_before_loop = ctx.scope.save_moved_state();

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
                ctx.scope.define((*name).to_string(), ty);
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
        ctx.scope.define(target_name.clone(), elem_ty.clone());
    }
    for guard in detect_range_sequence_guards(for_stmt, &target_name, ctx) {
        ctx.add_sequence_guard(guard);
    }
    ctx.loop_depth += 1;
    let body = lower_stmts(&for_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();
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
