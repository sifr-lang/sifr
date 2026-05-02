use super::assignment_widening::reconcile_optional_reassignment;
use super::aug_assign_lowering::lower_aug_assign as lower_aug_assign_impl;
use super::binding_mutability::ensure_mutable_parameter_binding;
use super::builtin_calls::callable_builtin_element_type;
use super::classes::collect_literal_coverage;
use super::container_literal_specialization::{
    apply_container_specialization_patches, type_contains_unknown_or_any,
    validate_subscript_assignment_target,
};
use super::control_flow_conditions::validate_control_flow_condition;
use super::diagnostics::{collect_raise_error_types, format_type_name, is_valid_error_type};
use super::expressions::{lower_expr, lower_star_unpack_assign, lower_tuple_unpack_assign};
use super::flow_helpers::{expr_to_literal_value, then_body_always_exits};
use super::for_loop_safety::{is_collection_backed_iter_source, loop_body_mutates_iter_source};
use super::function_flow::infer_function_return_type;
use super::if_branch_bindings::{
    predeclare_exhaustive_if_assigned_names, seed_exhaustive_if_bindings,
};
use super::len_aliases::record_len_alias_fact;
use super::name_diagnostics;
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
use super::typing_and_functions::{
    ast_convention_to_param, register_local_function_signature, register_local_function_symbol,
    resolve_annotation_expr,
};
use super::LowerCtx;
use crate::hir_nodes::{
    HirExceptHandler, HirExpr, HirFunction, HirIteratorOp, HirMatchArm, HirParam, HirPattern,
    HirStmt, MethodKind,
};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{
    BoolOp, CmpOp, ExceptHandler, Expr, Pattern, Singleton, Stmt, StmtAnnAssign, StmtAssign,
    StmtAugAssign, StmtFor, StmtIf, StmtMatch, StmtReturn, StmtWhile, UnaryOp,
};
use sifr_type_system::infer::resolve_type_annotation;
use sifr_type_system::{
    make_union, narrow_type, FunctionType, NarrowingCondition, OwnershipKind, Type,
};

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

fn should_adopt_inferred_binding_hint(value_expr: &Expr, value_ty: &Type, hint: &Type) -> bool {
    if !type_contains_unknown_or_any(value_ty) {
        return false;
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
    ctx.inferred_binding_hints
        .push(nested_inference.binding_hints.clone());
    predeclare_nested_function_symbols(stmts, &nested_inference.function_types, ctx);

    let mut result = Vec::new();
    for (index, stmt) in stmts.iter().enumerate() {
        if crate::cfg::flow_facts(&result).always_exits() {
            ctx.warn(format!(
                "unreachable statement at block index {index} was ignored"
            ));
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
                ctx.error("yield without a value is not supported".to_string());
                return None;
            }
            let expr = lower_expr(&expr_stmt.value, ctx)?;
            // #[must_use] enforcement: Result values must not be silently discarded
            let expr_ty = expr.ty();
            if matches!(expr_ty, Type::Result(_, _)) {
                ctx.error_with_code(
                    DiagnosticCode::RESULT_UNUSED_VALUE,
                    format!(
                        "unused Result value of type '{}' must be used. Use 'let _ = expr' to explicitly discard",
                        expr_ty.display_name()
                    ),
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
                ctx.error("del with multiple targets not supported".to_string());
                return None;
            }
            if let Expr::Subscript(sub) = &del_stmt.targets[0] {
                let object = lower_expr(&sub.value, ctx)?;
                let index = lower_expr(&sub.slice, ctx)?;
                Some(HirStmt::Delete { object, index })
            } else {
                ctx.error(
                    "del is only supported for collection items (del d[key], del a[i])".to_string(),
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
                    super::result_diagnostics::invalid_raise_string(ctx);
                    return None;
                }
                let value = lower_expr(exc, ctx)?;
                // Verify the raised value is an error type
                let raised_ty = value.ty();
                if !is_valid_error_type(raised_ty, ctx) {
                    let ty_name = format_type_name(raised_ty);
                    super::result_diagnostics::invalid_raise_non_error(ctx, ty_name.as_str());
                    return None;
                }
                Some(HirStmt::Raise { value })
            } else {
                super::result_diagnostics::invalid_bare_raise(ctx);
                None
            }
        }
        Stmt::With(with_stmt) => {
            if with_stmt.items.is_empty() {
                ctx.error("with statement must have at least one item".to_string());
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
                            ctx.error("with target must be a simple name".to_string());
                            return None;
                        }
                    } else {
                        format!("_with_val_{}", items.len())
                    };
                    let val_ty = value.ty().clone();
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
                            protocol_diagnostics::context_manager_incomplete(ctx, name);
                            false
                        } else {
                            protocol_diagnostics::context_manager_missing(ctx, name);
                            false
                        }
                    } else {
                        // Non-class types don't have methods — can't be context managers
                        let type_name = val_ty.display_name();
                        protocol_diagnostics::context_manager_missing(ctx, &type_name);
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
                let error_type = if let Some(ref type_expr) = h.type_ {
                    if let Expr::Name(n) = type_expr.as_ref() {
                        Some(n.id.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                let name = h.name.as_ref().map(std::string::ToString::to_string);

                // Check if this is a catch-all (except Error) or a specific handler
                if let Some(ref et) = error_type {
                    if et == "Error" {
                        has_catch_all = true;
                    } else {
                        // Validate the except type is a known error class
                        if !ctx.error_types.contains(et) {
                            ctx.error(format!(
                                "`{et}` in except arm is not a known error class — use a class extending Error"
                            ));
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
                    ctx.error(format!(
                        "except arms do not cover all error types from try body — uncovered: {}. Add `except Error as e` as a catch-all or add specific except arms",
                        sorted.join(", ")
                    ));
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
                super::flow_diagnostics::recursive_nonlocal_nested_function(ctx, &func.name);
            }

            let inferred_return_type = infer_function_return_type(
                func.name.as_ref(),
                ft.return_type.as_ref(),
                func.returns.is_some(),
                &body,
                |message| ctx.error(message),
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
                    method_kind: MethodKind::Regular,
                    decorators,
                    type_params: Vec::new(),
                },
            })
        }
        Stmt::Match(match_stmt) => lower_match(match_stmt, func_type, ctx),
        _ => {
            ctx.error("unsupported statement type".to_string());
            None
        }
    }
}

pub(super) fn lower_match(
    match_stmt: &StmtMatch,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let subject = lower_expr(&match_stmt.subject, ctx)?;
    let subject_ty = subject.ty().clone();

    let mut arms = Vec::new();
    for case in &match_stmt.cases {
        let arm = ctx.with_pushed_scope(|ctx| {
            let pattern = lower_pattern(&case.pattern, &subject_ty, ctx)?;

            // Bind captured variables into scope
            bind_pattern_vars(&pattern, ctx);

            let guard = if let Some(ref g) = case.guard {
                let guard_expr = lower_expr(g, ctx)?;
                let guard_ty = guard_expr.ty();
                if *guard_ty != Type::Bool && *guard_ty != Type::Any {
                    super::match_diagnostics::guard_not_bool(ctx, &guard_ty.display_name());
                }
                Some(guard_expr)
            } else {
                None
            };

            let body = lower_stmts(&case.body, func_type, ctx);
            Some(HirMatchArm {
                pattern,
                guard,
                body,
            })
        })?;

        arms.push(arm);
    }

    // Exhaustiveness check: verify all variants of the subject type are covered
    let has_wildcard = arms
        .iter()
        .any(|arm| matches!(arm.pattern, HirPattern::Wildcard));
    let has_capture_without_guard = arms
        .iter()
        .any(|arm| matches!(arm.pattern, HirPattern::Capture { .. }) && arm.guard.is_none());

    if !has_wildcard && !has_capture_without_guard {
        if let Type::Union(members) = &subject_ty {
            // Collect covered types from arms
            let mut covered_none = false;
            let mut covered_classes: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut covered_types: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut covered_literal_strs: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut covered_literal_ints: std::collections::HashSet<i64> =
                std::collections::HashSet::new();
            let mut covered_literal_bools: std::collections::HashSet<bool> =
                std::collections::HashSet::new();

            for arm in &arms {
                match &arm.pattern {
                    HirPattern::None => {
                        covered_none = true;
                    }
                    HirPattern::Class { class_name, .. } => {
                        covered_classes.insert(class_name.clone());
                    }
                    HirPattern::Capture { ty, .. } if arm.guard.is_none() => {
                        covered_types.insert(ty.display_name());
                    }
                    HirPattern::Literal { .. } => {
                        collect_literal_coverage(
                            &arm.pattern,
                            &mut covered_literal_strs,
                            &mut covered_literal_ints,
                            &mut covered_literal_bools,
                        );
                    }
                    HirPattern::Or { patterns } => {
                        for p in patterns {
                            match p {
                                HirPattern::None => {
                                    covered_none = true;
                                }
                                HirPattern::Class { class_name, .. } => {
                                    covered_classes.insert(class_name.clone());
                                }
                                HirPattern::Literal { .. } => {
                                    collect_literal_coverage(
                                        p,
                                        &mut covered_literal_strs,
                                        &mut covered_literal_ints,
                                        &mut covered_literal_bools,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check each union member is covered
            let mut uncovered: Vec<String> = Vec::new();
            for member in members {
                match member {
                    Type::None => {
                        if !covered_none {
                            uncovered.push("None".to_string());
                        }
                    }
                    Type::Class { name, .. } => {
                        if !covered_classes.contains(name) && !covered_types.contains(name) {
                            uncovered.push(name.clone());
                        }
                    }
                    Type::Int => {
                        if !covered_types.contains("int") && !covered_classes.contains("int") {
                            uncovered.push("int".to_string());
                        }
                    }
                    Type::Str => {
                        if !covered_types.contains("str") && !covered_classes.contains("str") {
                            uncovered.push("str".to_string());
                        }
                    }
                    Type::Float => {
                        if !covered_types.contains("float") && !covered_classes.contains("float") {
                            uncovered.push("float".to_string());
                        }
                    }
                    Type::Bool => {
                        if !covered_types.contains("bool") && !covered_classes.contains("bool") {
                            uncovered.push("bool".to_string());
                        }
                    }
                    Type::LiteralStr(s) => {
                        if !covered_literal_strs.contains(s) {
                            uncovered.push(format!("\"{s}\""));
                        }
                    }
                    Type::LiteralInt(n) => {
                        if !covered_literal_ints.contains(n) {
                            uncovered.push(n.to_string());
                        }
                    }
                    Type::LiteralBool(b) => {
                        if !covered_literal_bools.contains(b) {
                            uncovered.push(b.to_string());
                        }
                    }
                    _ => {}
                }
            }

            if !uncovered.is_empty() {
                super::match_diagnostics::non_exhaustive_union(
                    ctx,
                    &subject_ty.display_name(),
                    &uncovered.join(", "),
                );
            }
        }

        // Check enum exhaustiveness
        if let Type::Enum {
            ref name,
            ref variants,
        } = subject_ty
        {
            let mut covered_variants: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for arm in &arms {
                if let HirPattern::Value { path } = &arm.pattern {
                    if path.len() == 2 {
                        covered_variants.insert(path[1].clone());
                    }
                }
                if let HirPattern::Or { patterns } = &arm.pattern {
                    for p in patterns {
                        if let HirPattern::Value { path } = p {
                            if path.len() == 2 {
                                covered_variants.insert(path[1].clone());
                            }
                        }
                    }
                }
            }
            let uncovered: Vec<&String> = variants
                .iter()
                .map(|(v, _)| v)
                .filter(|v| !covered_variants.contains(*v))
                .collect();
            if !uncovered.is_empty() {
                super::match_diagnostics::non_exhaustive_enum(
                    ctx,
                    name,
                    &uncovered
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            }
        }

        // For non-union, non-enum types with only literal/guarded patterns, require a wildcard
        if !matches!(subject_ty, Type::Union(_)) && !matches!(subject_ty, Type::Enum { .. }) {
            let all_literal_or_guarded = arms.iter().all(|arm| {
                matches!(arm.pattern, HirPattern::Literal { .. })
                    || matches!(arm.pattern, HirPattern::Or { .. })
                    || arm.guard.is_some()
            });
            if all_literal_or_guarded {
                super::match_diagnostics::non_exhaustive_literal(ctx, &subject_ty.display_name());
            }
        }
    }

    Some(HirStmt::Match {
        subject,
        subject_ty,
        arms,
    })
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
                    ctx.error("complex attribute pattern not supported".to_string());
                    return None;
                };
                let attr_name = attr.attr.to_string();
                Some(HirPattern::Value {
                    path: vec![obj_name, attr_name],
                })
            } else {
                // Try to lower as a literal expression
                let expr = lower_expr(val_pat.value.as_ref(), ctx)?;
                Some(HirPattern::Literal { value: expr })
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
                ctx.error("class pattern class name must be a simple name".to_string());
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
                        super::match_diagnostics::invalid_class_pattern_field(
                            ctx,
                            &class_name,
                            &field_name,
                            &class_fields
                                .iter()
                                .map(|(n, _)| n.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
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
                ctx.error(format!(
                    "tuple pattern requires subject of tuple type, got '{}'",
                    subject_ty.display_name()
                ));
                return None;
            };
            if elem_types.len() != seq_pat.patterns.len() {
                ctx.error(format!(
                    "tuple pattern expects {} element(s), subject has {}",
                    seq_pat.patterns.len(),
                    elem_types.len()
                ));
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
            ctx.error("mapping patterns in match are not yet supported".to_string());
            None
        }
        Pattern::MatchStar(_) => {
            ctx.error("star patterns in match are not yet supported".to_string());
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
) {
    if is_explicit_local {
        ctx.scope.define_explicit_local(name.to_string(), ty);
    } else {
        ctx.scope.define(name.to_string(), ty);
    }
    ctx.empty_dict_specializations.remove(name);
    ctx.pending_container_specialization_patches.remove(name);
    ctx.clear_numeric_sentinel_var(name);
    ctx.clear_sequence_shape_fact(name);
}

fn invalidate_rebound_binding_facts(ctx: &mut LowerCtx, name: &str) {
    ctx.scope.clear_narrowing(name);
    ctx.clear_sequence_guards_for_binding(name);
}

pub(super) fn lower_ann_assign(ann: &StmtAnnAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let name = if let Expr::Name(n) = ann.target.as_ref() {
        n.id.to_string()
    } else {
        ctx.error("annotated assignment target must be a simple name".to_string());
        return None;
    };
    let declared_type = resolve_annotation_expr(&ann.annotation, ctx);

    let (value, initializer_range) = if let Some(val) = &ann.value {
        let initializer_range = val.range();
        let mut expr = if let Some(kind) = numeric_sentinel_kind(val) {
            if let Some(domain) = numeric_domain_for_type(&declared_type) {
                domain_typed_sentinel_expr(kind, domain)
            } else if let Some(expr) = lower_expr(val, ctx) {
                expr
            } else {
                seed_binding_after_failed_initializer(ctx, &name, declared_type.clone(), true);
                return None;
            }
        } else if let Some(expr) = lower_expr(val, ctx) {
            expr
        } else {
            seed_binding_after_failed_initializer(ctx, &name, declared_type.clone(), true);
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
        if !is_int_to_bigint && !final_ty.is_assignable_to(&declared_type) {
            ctx.error_with_code(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "type mismatch: expected '{}', got '{}'",
                    declared_type.display_name(),
                    final_ty.display_name()
                ),
            );
        }
        (expr, initializer_range)
    } else {
        ctx.error(format!("variable '{name}' must be initialized"));
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
            ctx.error("chained assignment targets must be simple names".to_string());
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
        ctx.error("multiple assignment targets are not supported".to_string());
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
                ctx.error("attribute assignment target must be a simple name".to_string());
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
            ctx.error("attribute assignment target must be a simple name".to_string());
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
                ctx.error("nested subscript assignment target must be a simple name".to_string());
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
                ctx.error("subscript assignment target must be a simple name".to_string());
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
            ctx.error("subscript assignment target must be a simple name".to_string());
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
        let object_ty =
            validate_subscript_assignment_target(ctx, &obj_name, &obj_ty, index.ty(), value.ty());
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
        ctx.error("assignment target must be a simple name".to_string());
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
    let Some(value) = lower_expr(&assign.value, ctx) else {
        if !should_treat_as_existing_binding {
            let fallback_ty = ctx
                .inferred_binding_hint(&name)
                .cloned()
                .unwrap_or(Type::Unknown);
            seed_binding_after_failed_initializer(ctx, &name, fallback_ty, false);
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
            ctx.error_with_code(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "type mismatch: cannot assign '{}' to variable '{}' of type '{}'",
                    value_ty.display_name(),
                    name,
                    info_ty.display_name()
                ),
            );
        }
        // Reset moved state on reassignment
        ctx.scope.reset_moved(&name);
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
            .filter(|hint| should_adopt_inferred_binding_hint(&assign.value, &value_ty, hint))
            .cloned()
            .unwrap_or_else(|| value_ty.clone());
        ctx.scope.define(name.clone(), binding_ty.clone());
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
            ctx.error_with_code(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "return type mismatch: expected '{}', got '{}'",
                    func_type.return_type.display_name(),
                    expr_ty.display_name()
                ),
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
            ctx.error(format!(
                "function expects return type '{}', but returns nothing",
                func_type.return_type.display_name()
            ));
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
    validate_control_flow_condition(&condition, "if", Some(if_stmt.test.range()), ctx);
    predeclare_exhaustive_if_assigned_names(if_stmt, ctx);

    let saved_state = ctx.scope.save_narrowing_state();
    let saved_moved = ctx.scope.save_moved_state();
    let saved_sequence_guards = ctx.save_sequence_guards();

    if let Some(ref cond) = narrowing_cond {
        apply_narrowing(ctx, cond, true);
    }
    for guard in detect_true_sequence_guards(&if_stmt.test, ctx) {
        ctx.add_sequence_guard(guard);
    }

    ctx.scope.push();
    let then_body = lower_stmts(&if_stmt.body, func_type, ctx);
    ctx.scope.pop();

    let then_moved = ctx.scope.save_moved_state();
    let then_sequence_guards = ctx.save_sequence_guards();

    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);
    ctx.restore_sequence_guards(&saved_sequence_guards);

    let mut all_conditions: Vec<NarrowingCondition> = Vec::new();
    if let Some(ref cond) = narrowing_cond {
        all_conditions.push(cond.clone());
    }

    let mut branch_moved_states: Vec<_> = vec![then_moved];
    let mut branch_sequence_states = vec![then_sequence_guards];

    let mut elif_clauses = Vec::new();
    for clause in &if_stmt.elif_else_clauses {
        if let Some(test) = &clause.test {
            ctx.scope.restore_narrowing_state(&saved_state);
            ctx.scope.restore_moved_state(&saved_moved);
            ctx.restore_sequence_guards(&saved_sequence_guards);
            for prev_cond in &all_conditions {
                apply_narrowing(ctx, prev_cond, false);
            }

            let elif_narrowing = detect_narrowing_condition(test, ctx);
            let cond = lower_expr(test, ctx)?;
            validate_control_flow_condition(&cond, "elif", Some(test.range()), ctx);

            let elif_saved = ctx.scope.save_narrowing_state();
            if let Some(ref elif_cond) = elif_narrowing {
                apply_narrowing(ctx, elif_cond, true);
            }
            for guard in detect_true_sequence_guards(test, ctx) {
                ctx.add_sequence_guard(guard);
            }

            ctx.scope.push();
            let body = lower_stmts(&clause.body, func_type, ctx);
            ctx.scope.pop();
            elif_clauses.push((cond, body));

            branch_moved_states.push(ctx.scope.save_moved_state());
            branch_sequence_states.push(ctx.save_sequence_guards());

            ctx.scope.restore_narrowing_state(&elif_saved);
            ctx.scope.restore_moved_state(&saved_moved);
            ctx.restore_sequence_guards(&saved_sequence_guards);

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
    ctx.clear_sequence_pointers();
    Some(HirStmt::If {
        condition,
        then_body,
        elif_clauses,
        else_body,
    })
}

/// Detect a narrowing condition from an if-test expression.
pub(super) fn detect_narrowing_condition(
    expr: &Expr,
    ctx: &LowerCtx,
) -> Option<NarrowingCondition> {
    match expr {
        // isinstance(x, Type) -> IsInstance narrowing
        Expr::Call(call) => {
            if let Expr::Name(func_name) = call.func.as_ref() {
                if func_name.id.as_str() == "isinstance" && call.arguments.args.len() == 2 {
                    if let Expr::Name(var) = &call.arguments.args[0] {
                        let var_name = var.id.to_string();
                        // Check that the variable exists and has a union/Unknown type
                        if ctx.scope.lookup(&var_name).is_some() {
                            if let Expr::Name(type_name) = &call.arguments.args[1] {
                                // Try built-in types first, then class types
                                let target_ty =
                                    resolve_type_annotation(&type_name.id).or_else(|| {
                                        ctx.class_types.get(type_name.id.as_str()).cloned()
                                    });
                                if let Some(target_ty) = target_ty {
                                    return Some(NarrowingCondition::IsInstance(
                                        var_name, target_ty,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        // x is None / x is not None
        Expr::Compare(cmp) => {
            if cmp.ops.len() == 1 && cmp.comparators.len() == 1 {
                match &cmp.ops[0] {
                    CmpOp::Is => {
                        if let (Expr::Name(var), Expr::NoneLiteral(_)) =
                            (cmp.left.as_ref(), &cmp.comparators[0])
                        {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                return Some(NarrowingCondition::IsNone(var_name));
                            }
                        }
                    }
                    CmpOp::IsNot => {
                        if let (Expr::Name(var), Expr::NoneLiteral(_)) =
                            (cmp.left.as_ref(), &cmp.comparators[0])
                        {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                return Some(NarrowingCondition::IsNotNone(var_name));
                            }
                        }
                    }
                    // x == "value" -> Equality narrowing
                    CmpOp::Eq => {
                        if let Expr::Name(var) = cmp.left.as_ref() {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                if let Some(lit_val) = expr_to_literal_value(&cmp.comparators[0]) {
                                    return Some(NarrowingCondition::Equality(var_name, lit_val));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        // Simple variable name -> Truthiness narrowing
        Expr::Name(name) => {
            let var_name = name.id.to_string();
            if ctx.scope.lookup(&var_name).is_some() {
                Some(NarrowingCondition::Truthiness(var_name))
            } else {
                None
            }
        }
        // not expr -> negate the inner condition
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            let inner = detect_narrowing_condition(&unary.operand, ctx)?;
            Some(NarrowingCondition::Not(Box::new(inner)))
        }
        // a and b -> And narrowing (both conditions must be true)
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::And) => {
            let conditions: Vec<NarrowingCondition> = boolop
                .values
                .iter()
                .filter_map(|v| detect_narrowing_condition(v, ctx))
                .collect();
            if conditions.is_empty() {
                None
            } else if conditions.len() == 1 {
                conditions.into_iter().next()
            } else {
                Some(NarrowingCondition::And(conditions))
            }
        }
        // a or b -> Or narrowing (at least one condition must be true)
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::Or) => {
            let conditions: Vec<NarrowingCondition> = boolop
                .values
                .iter()
                .filter_map(|v| detect_narrowing_condition(v, ctx))
                .collect();
            if conditions.is_empty() {
                None
            } else if conditions.len() == 1 {
                conditions.into_iter().next()
            } else {
                Some(NarrowingCondition::Or(conditions))
            }
        }
        _ => None,
    }
}

/// Apply narrowing to the scope based on a condition.
pub(super) fn apply_narrowing(ctx: &mut LowerCtx, condition: &NarrowingCondition, is_true: bool) {
    match condition {
        NarrowingCondition::And(conditions) => {
            if is_true {
                // All conditions are true: apply each narrowing
                for cond in conditions {
                    apply_narrowing(ctx, cond, true);
                }
            } else {
                // At least one is false: can't narrow precisely, skip
            }
        }
        NarrowingCondition::Or(conditions) => {
            if !is_true {
                // All conditions are false: apply each false-narrowing
                for cond in conditions {
                    apply_narrowing(ctx, cond, false);
                }
            }
        }
        _ => {
            if let Some(var_name) = condition.var_name() {
                if let Some(info) = ctx.scope.lookup(var_name) {
                    let current_ty = info.effective_type().clone();
                    let narrowed = narrow_type(&current_ty, condition, is_true);
                    ctx.scope.narrow_var(var_name, narrowed);
                }
            }
        }
    }
}

pub(super) fn lower_while(
    while_stmt: &StmtWhile,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let narrowing_cond = detect_narrowing_condition(&while_stmt.test, ctx);
    let condition = lower_expr(&while_stmt.test, ctx)?;
    validate_control_flow_condition(&condition, "while", Some(while_stmt.test.range()), ctx);
    let saved_narrowing_state = ctx.scope.save_narrowing_state();
    let saved_sequence_guards = ctx.save_sequence_guards();
    if let Some(ref cond) = narrowing_cond {
        apply_narrowing(ctx, cond, true);
    }
    for guard in detect_while_sequence_guards(while_stmt, ctx) {
        ctx.add_sequence_guard(guard);
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
        ctx.error(format!(
            "for-loop iterable must have a statically-known element type, got '{}'",
            iter_source_ty.display_name()
        ));
        return None;
    }
    let Some(elem_ty) = callable_builtin_element_type(&iter_source_ty) else {
        if matches!(iter_source_ty.resolve_alias(), Type::Tuple(_)) {
            ctx.error(
                "for-loop tuple iteration requires one statically provable element type"
                    .to_string(),
            );
            return None;
        }
        ctx.error(format!(
            "cannot iterate over type '{}'",
            iter_source_ty.display_name()
        ));
        return None;
    };
    let iter_expr = HirExpr::IteratorCall {
        op: HirIteratorOp::Iter,
        args: vec![iterable_expr],
        ty: Type::Iterator(Box::new(elem_ty.clone())),
    };

    // Extract the target variable name(s)
    let target_name = match for_stmt.target.as_ref() {
        Expr::Name(n) => n.id.to_string(),
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
                ctx.error("for loop tuple target must contain only simple names".to_string());
                return None;
            }
            names.join(",")
        }
        _ => {
            ctx.error("for loop target must be a simple name or tuple".to_string());
            return None;
        }
    };

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
                ctx.error_with_code(
                    DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                    format!(
                        "for loop tuple target expects {} element(s), iterable yields {}",
                        names.len(),
                        elem_types.len()
                    ),
                );
                ctx.scope.pop();
                return None;
            }
            for (i, name) in names.iter().enumerate() {
                let ty = elem_types[i].clone();
                ctx.scope.define((*name).to_string(), ty);
            }
        } else {
            ctx.error_with_code(
                DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                format!(
                    "for loop tuple target expects iterable elements of tuple type, got '{}'",
                    elem_ty.display_name()
                ),
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
            ctx.error(format!(
                "cannot mutate '{source_name}' while iterating over it in a for loop"
            ));
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
