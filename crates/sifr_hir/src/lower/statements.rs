use crate::hir_nodes::{
    HirExceptHandler, HirExpr, HirFunction, HirMatchArm, HirParam, HirPattern, HirStmt, MethodKind,
};
use sifr_python_ast::{
    BoolOp, CmpOp, ExceptHandler, Expr, Number, Operator, Pattern, Singleton, Stmt, StmtAnnAssign,
    StmtAssign, StmtAugAssign, StmtFor, StmtIf, StmtMatch, StmtReturn, StmtWhile, UnaryOp,
};
use sifr_type_system::infer::resolve_type_annotation;
use sifr_type_system::{
    narrow_type, type_check_binary_op, FunctionType, NarrowingCondition, OwnershipKind,
    ParamConvention, Type,
};

use super::classes::collect_literal_coverage;
use super::diagnostics::{collect_raise_error_types, format_type_name, is_valid_error_type};
use super::expressions::{lower_expr, lower_star_unpack_assign, lower_tuple_unpack_assign};
use super::typing_and_functions::{extract_function_type, resolve_annotation_expr};
use super::LowerCtx;

pub(super) fn lower_stmts(
    stmts: &[Stmt],
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Vec<HirStmt> {
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
    }
    result
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
        Stmt::Return(ret) => lower_return(ret, func_type, ctx),
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
                ctx.error(format!(
                    "unused Result value of type '{}' must be used. Use 'let _ = expr' to explicitly discard",
                    expr_ty.display_name()
                ));
            }
            Some(HirStmt::Expr { expr })
        }
        Stmt::If(if_stmt) => lower_if(if_stmt, func_type, ctx),
        Stmt::While(while_stmt) => lower_while(while_stmt, func_type, ctx),
        Stmt::For(for_stmt) => lower_for(for_stmt, func_type, ctx),
        Stmt::Break(_) => {
            if !ctx.in_loop() {
                ctx.error("'break' outside of loop".to_string());
                return None;
            }
            Some(HirStmt::Break)
        }
        Stmt::Continue(_) => {
            if !ctx.in_loop() {
                ctx.error("'continue' outside of loop".to_string());
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
                    ctx.error("raise requires an Error class instance — `raise \"message\"` is not allowed, use e.g. `raise ValueError(\"message\")`".to_string());
                    return None;
                }
                let value = lower_expr(exc, ctx)?;
                // Verify the raised value is an error type
                let raised_ty = value.ty();
                if !is_valid_error_type(raised_ty, ctx) {
                    let ty_name = format_type_name(raised_ty);
                    ctx.error(format!(
                        "raise requires an Error class instance — `{ty_name}` is not an Error class"
                    ));
                    return None;
                }
                Some(HirStmt::Raise { value })
            } else {
                ctx.error("bare 'raise' without an expression is not supported".to_string());
                None
            }
        }
        Stmt::With(with_stmt) => {
            if with_stmt.items.is_empty() {
                ctx.error("with statement must have at least one item".to_string());
                return None;
            }
            let mut items = Vec::new();
            ctx.scope.push();
            for item in &with_stmt.items {
                let value = lower_expr(&item.context_expr, ctx)?;
                let var_name = if let Some(ref vars) = item.optional_vars {
                    if let Expr::Name(n) = vars.as_ref() {
                        n.id.clone()
                    } else {
                        ctx.error("with target must be a simple name".to_string());
                        return None;
                    }
                } else {
                    format!("_with_val_{}", items.len())
                };
                let val_ty = value.ty().clone();
                // Check if the type implements the ContextManager protocol (__enter__/__exit__)
                let has_context_manager = if let Type::Class { methods, .. } = &val_ty {
                    let has_enter = methods.iter().any(|(name, _)| name == "__enter__");
                    let has_exit = methods.iter().any(|(name, _)| name == "__exit__");
                    if has_enter && has_exit {
                        true
                    } else if has_enter || has_exit {
                        ctx.error("type used in 'with' statement must implement both __enter__ and __exit__ methods".to_string());
                        false
                    } else {
                        ctx.error(format!(
                            "type '{}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)",
                            match &val_ty { Type::Class { name, .. } => name.clone(), _ => "unknown".to_string() }
                        ));
                        false
                    }
                } else {
                    // Non-class types don't have methods — can't be context managers
                    ctx.error("type used in 'with' statement must implement the ContextManager protocol (__enter__/__exit__)".to_string());
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
            ctx.scope.pop();
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
                        Some(n.id.clone())
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
        Stmt::FunctionDef(func) => {
            // Nested function definition (def inside def)
            // Extract the function type (params + return type)
            let ft = extract_function_type(func, ctx);

            // Register the nested function in the current scope so it can be called
            ctx.functions.insert(func.name.to_string(), ft.clone());

            // Lower the nested function body
            ctx.scope.push();

            // Define parameters in scope
            let mut params = Vec::new();
            for (i, param_def) in func.parameters.args.iter().enumerate() {
                let name = param_def.parameter.name.to_string();
                let ty = ft
                    .params
                    .get(i)
                    .map(|(_, t, _)| t.clone())
                    .unwrap_or(Type::Any);
                ctx.scope.define(name.clone(), ty.clone());
                let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));
                params.push(HirParam {
                    name,
                    ty,
                    default,
                    keyword_only: false,
                    convention: ParamConvention::default(),
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
                ctx.scope.define(name.clone(), ty.clone());
                params.push(HirParam {
                    name,
                    ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
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
                ctx.scope.define(name.clone(), ty.clone());
                let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));
                params.push(HirParam {
                    name,
                    ty,
                    default,
                    keyword_only: true,
                    convention: ParamConvention::default(),
                });
            }

            let body = lower_stmts(&func.body, &ft, ctx);
            ctx.scope.pop();

            // Infer return type if not explicitly annotated
            let inferred_return_type = if *ft.return_type == Type::Any && func.returns.is_none() {
                let return_types = collect_return_types(&body);
                if return_types.is_empty() {
                    Type::None
                } else if return_types.len() == 1 {
                    return_types.into_iter().next().unwrap()
                } else {
                    let mut members: Vec<Type> = return_types.into_iter().collect();
                    members.sort_by_key(sifr_type_system::Type::display_name);
                    members.dedup();
                    if members.len() == 1 {
                        members.into_iter().next().unwrap()
                    } else {
                        Type::Union(members)
                    }
                }
            } else {
                *ft.return_type
            };

            // Collect user-defined decorators
            let decorators: Vec<String> = func
                .decorator_list
                .iter()
                .filter_map(|d| {
                    if let Expr::Name(n) = &d.expression {
                        let name = n.id.clone();
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
        ctx.scope.push();

        let pattern = lower_pattern(&case.pattern, &subject_ty, ctx)?;

        // Bind captured variables into scope
        bind_pattern_vars(&pattern, ctx);

        let guard = if let Some(ref g) = case.guard {
            let guard_expr = lower_expr(g, ctx)?;
            let guard_ty = guard_expr.ty();
            if *guard_ty != Type::Bool && *guard_ty != Type::Any {
                ctx.error(format!(
                    "match guard must be a bool expression, got '{}'",
                    guard_ty.display_name()
                ));
            }
            Some(guard_expr)
        } else {
            None
        };

        let body = lower_stmts(&case.body, func_type, ctx);

        ctx.scope.pop();

        arms.push(HirMatchArm {
            pattern,
            guard,
            body,
        });
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
                ctx.error(format!(
                    "non-exhaustive match: type '{}' has uncovered variants: {} — add matching case(s) or `case _:`",
                    subject_ty.display_name(),
                    uncovered.join(", ")
                ));
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
                ctx.error(format!(
                    "non-exhaustive match: enum '{}' has uncovered variants: {} — add matching case(s) or `case _:`",
                    name,
                    uncovered.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                ));
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
                ctx.error(format!(
                    "non-exhaustive match: type '{}' cannot be fully covered by literal patterns — add `case _:` to handle remaining values",
                    subject_ty.display_name()
                ));
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
                    n.id.clone()
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
                n.id.clone()
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
                    let found = class_fields
                        .iter()
                        .find(|(n, _)| n == &field_name)
                        .map(|(_, t)| t.clone());
                    if found.is_none() {
                        ctx.error(format!(
                            "class '{}' has no field '{}' — available fields: {}",
                            class_name,
                            field_name,
                            class_fields
                                .iter()
                                .map(|(n, _)| n.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                        return None;
                    }
                    found.unwrap()
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

pub(super) fn lower_ann_assign(ann: &StmtAnnAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let name = if let Expr::Name(n) = ann.target.as_ref() {
        n.id.clone()
    } else {
        ctx.error("annotated assignment target must be a simple name".to_string());
        return None;
    };

    let declared_type = resolve_annotation_expr(&ann.annotation, ctx);

    let value = if let Some(val) = &ann.value {
        let mut expr = lower_expr(val, ctx)?;
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
            ctx.error(format!(
                "type mismatch: expected '{}', got '{}'",
                declared_type.display_name(),
                final_ty.display_name()
            ));
        }
        expr
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
                ctx.error(format!(
                    "cannot store borrowed parameter `{src_name}`: it is borrowed by default -- use `own {src_name}` to take ownership, or store `{src_name}.clone()`"
                ));
            } else {
                ctx.scope.mark_moved(src_name);
            }
        }
    }

    ctx.scope.define(name.clone(), declared_type.clone());

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
            let name = n.id.clone();
            if i == 0 {
                // First (rightmost) target gets the actual value
                let existing = ctx.scope.lookup(&name);
                if existing.is_some() {
                    // Reassignment
                    result.push(HirStmt::Assign {
                        name: name.clone(),
                        value: value.clone(),
                    });
                } else {
                    // New variable
                    ctx.scope.define(name.clone(), val_ty.clone());
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
                    Some(Expr::Name(prev_n)) => prev_n.id.clone(),
                    _ => continue,
                };
                let name_expr = HirExpr::Name {
                    name: prev_target,
                    ty: val_ty.clone(),
                };
                let existing = ctx.scope.lookup(&name);
                if existing.is_some() {
                    result.push(HirStmt::Assign {
                        name: name.clone(),
                        value: name_expr,
                    });
                } else {
                    ctx.scope.define(name.clone(), val_ty.clone());
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

pub(super) fn lower_assign(assign: &StmtAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    if assign.targets.len() != 1 {
        ctx.error("multiple assignment targets not supported yet".to_string());
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
        let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
            n.id.clone()
        } else {
            ctx.error("attribute assignment target must be a simple name".to_string());
            return None;
        };
        let field_name = attr.attr.to_string();
        let value = lower_expr(&assign.value, ctx)?;
        return Some(HirStmt::FieldAssign {
            object: obj_name,
            field: field_name,
            value,
        });
    }

    // Handle subscript assignment: list[i] = val or dict[key] = val
    if let Expr::Subscript(sub) = &assign.targets[0] {
        // Handle nested subscript: matrix[i][j] = val
        if let Expr::Subscript(inner_sub) = sub.value.as_ref() {
            let obj_name = if let Expr::Name(n) = inner_sub.value.as_ref() {
                n.id.clone()
            } else {
                ctx.error("nested subscript assignment target must be a simple name".to_string());
                return None;
            };
            let obj_ty = ctx
                .scope
                .lookup(&obj_name)
                .map(|info| info.effective_type().clone())
                .unwrap_or(Type::Unknown);
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
                n.id.clone()
            } else {
                ctx.error("subscript assignment target must be a simple name".to_string());
                return None;
            };
            let field_name = attr.attr.to_string();
            // Look up field type from the object's class definition
            let field_ty = ctx
                .scope
                .lookup(&obj_name)
                .and_then(|info| {
                    let obj_ty = info.effective_type();
                    // The object may be typed as Type::Class directly (e.g. `self`)
                    // or as Type::Unknown for unresolved types.
                    if let Type::Class { fields, .. } = obj_ty {
                        fields
                            .iter()
                            .find(|(n, _)| n == &field_name)
                            .map(|(_, t)| t.clone())
                    } else if let Type::Class {
                        name: class_name, ..
                    } = obj_ty
                    {
                        // Class by name reference
                        ctx.class_types.get(class_name).and_then(|class_ty| {
                            if let Type::Class { fields, .. } = class_ty {
                                fields
                                    .iter()
                                    .find(|(n, _)| n == &field_name)
                                    .map(|(_, t)| t.clone())
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or(Type::Unknown);
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
            n.id.clone()
        } else {
            ctx.error("subscript assignment target must be a simple name".to_string());
            return None;
        };
        let obj_ty = ctx
            .scope
            .lookup(&obj_name)
            .map(|info| info.effective_type().clone())
            .unwrap_or(Type::Unknown);
        let index = lower_expr(&sub.slice, ctx)?;
        let value = lower_expr(&assign.value, ctx)?;
        return Some(HirStmt::SubscriptAssign {
            object: obj_name,
            index,
            value,
            object_ty: obj_ty,
        });
    }

    let name = if let Expr::Name(n) = &assign.targets[0] {
        n.id.clone()
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

    let value = lower_expr(&assign.value, ctx)?;
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
    if let Some(info) = ctx.scope.lookup(&name) {
        // Reassignment: check type compatibility
        if !value_ty.is_assignable_to(&info.ty) {
            ctx.error(format!(
                "type mismatch: cannot assign '{}' to variable '{}' of type '{}'",
                value_ty.display_name(),
                name,
                info.ty.display_name()
            ));
        }
        // Reset moved state on reassignment
        ctx.scope.reset_moved(&name);
        Some(HirStmt::Assign { name, value })
    } else {
        // New variable (type inferred)
        ctx.scope.define(name.clone(), value_ty.clone());
        Some(HirStmt::Let {
            name,
            ty: value_ty,
            value,
            is_mutable: true,
        })
    }
}

pub(super) fn lower_aug_assign(aug: &StmtAugAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    // Handle augmented assignment on attributes: self.field += val
    if let Expr::Attribute(attr) = aug.target.as_ref() {
        let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
            n.id.clone()
        } else {
            ctx.error("augmented attribute assignment target must be a simple name".to_string());
            return None;
        };
        let field_name = attr.attr.to_string();
        let value = lower_expr(&aug.value, ctx)?;
        let op_str = match aug.op {
            Operator::Add => "+=",
            Operator::Sub => "-=",
            Operator::Mult => "*=",
            Operator::Div => "/=",
            Operator::Mod => "%=",
            Operator::Pow => "**=",
            Operator::BitAnd => "&=",
            Operator::BitOr => "|=",
            Operator::BitXor => "^=",
            Operator::LShift => "<<=",
            Operator::RShift => ">>=",
            Operator::FloorDiv => "//=",
            Operator::MatMult => {
                ctx.error("matrix multiplication operator (@) is not supported".to_string());
                return None;
            }
        };
        return Some(HirStmt::AttributeAugAssign {
            object: obj_name,
            field: field_name,
            op: op_str.to_string(),
            value,
        });
    }

    // Handle augmented assignment on subscript: list[i] += val
    if let Expr::Subscript(sub) = aug.target.as_ref() {
        let obj_name = if let Expr::Name(n) = sub.value.as_ref() {
            n.id.clone()
        } else {
            ctx.error("augmented subscript assignment target must be a simple name".to_string());
            return None;
        };
        let obj_ty = ctx
            .scope
            .lookup(&obj_name)
            .map(|info| info.effective_type().clone())
            .unwrap_or(Type::Unknown);
        let index = lower_expr(&sub.slice, ctx)?;
        let value = lower_expr(&aug.value, ctx)?;
        let op_str = match aug.op {
            Operator::Add => "+=",
            Operator::Sub => "-=",
            Operator::Mult => "*=",
            Operator::Div => "/=",
            Operator::Mod => "%=",
            Operator::Pow => "**=",
            Operator::BitAnd => "&=",
            Operator::BitOr => "|=",
            Operator::BitXor => "^=",
            Operator::LShift => "<<=",
            Operator::RShift => ">>=",
            Operator::FloorDiv => "//=",
            Operator::MatMult => {
                ctx.error("matrix multiplication operator (@) is not supported".to_string());
                return None;
            }
        };
        return Some(HirStmt::SubscriptAugAssign {
            object: obj_name,
            index,
            op: op_str.to_string(),
            value,
            object_ty: obj_ty,
        });
    }

    let name = if let Expr::Name(n) = aug.target.as_ref() {
        n.id.clone()
    } else {
        ctx.error("augmented assignment target must be a simple name".to_string());
        return None;
    };

    let value = lower_expr(&aug.value, ctx)?;

    let op_str = match aug.op {
        Operator::Add => "+=",
        Operator::Sub => "-=",
        Operator::Mult => "*=",
        Operator::Div => "/=",
        Operator::FloorDiv => "//=",
        Operator::Mod => "%=",
        Operator::Pow => "**=",
        Operator::BitAnd => "&=",
        Operator::BitOr => "|=",
        Operator::BitXor => "^=",
        Operator::LShift => "<<=",
        Operator::RShift => ">>=",
        Operator::MatMult => {
            ctx.error("matrix multiplication operator (@) is not supported".to_string());
            return None;
        }
    };

    // Check that the variable exists
    let var_info = ctx.scope.lookup(&name);
    if var_info.is_none() {
        ctx.error(format!("undefined variable: '{name}'"));
        return None;
    }
    let var_ty = var_info.unwrap().ty.clone();

    // Type check the operation
    let base_op = &op_str[..op_str.len() - 1]; // Remove '='
                                               // For += on strings, allow str += str
                                               // For += on lists, allow list += list
    if base_op == "+" {
        match (&var_ty, value.ty()) {
            (Type::Str, Type::Str) => {}
            (Type::List(_), Type::List(_)) => {}
            _ => {
                if let Err(e) = type_check_binary_op(&var_ty, base_op, value.ty()) {
                    ctx.error(e.message);
                    return None;
                }
            }
        }
    } else if let Err(e) = type_check_binary_op(&var_ty, base_op, value.ty()) {
        ctx.error(e.message);
        return None;
    }

    Some(HirStmt::AugAssign {
        name,
        op: op_str.to_string(),
        value,
    })
}

pub(super) fn lower_return(
    ret: &StmtReturn,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let value = if let Some(val) = &ret.value {
        let expr = lower_expr(val, ctx)?;
        let expr_ty = expr.ty().clone();

        // Escape analysis: returning a borrowed parameter is a compile error.
        // The programmer must use `own` to transfer ownership, or call `.clone()` explicitly.
        if let HirExpr::Name { name, ty } = &expr {
            if ctx.borrowed_params.contains(name.as_str()) && ty.ownership() == OwnershipKind::Move
            {
                ctx.error(format!(
                    "cannot return borrowed parameter `{name}`: it is borrowed by default -- use `own {name}` to take ownership, or return `{name}.clone()`"
                ));
            }
        }

        // If the function returns Result[T, E] and the value is T (not Result), wrap in Ok()
        if let Type::Result(ref ok_ty, _) = *func_type.return_type {
            if expr_ty.is_assignable_to(ok_ty) && !matches!(expr_ty, Type::Result(_, _)) {
                // Wrap in Ok()
                return Some(HirStmt::Return {
                    value: Some(HirExpr::OkWrap {
                        ty: func_type.return_type.as_ref().clone(),
                        value: Box::new(expr),
                    }),
                });
            }
        }

        if !expr_ty.is_assignable_to(&func_type.return_type) {
            ctx.error(format!(
                "return type mismatch: expected '{}', got '{}'",
                func_type.return_type.display_name(),
                expr_ty.display_name()
            ));
        }
        Some(expr)
    } else {
        if *func_type.return_type != Type::None {
            // If function returns Result[(), E], wrap in Ok(())
            if let Type::Result(ref ok_ty, _) = *func_type.return_type {
                if **ok_ty == Type::None {
                    return Some(HirStmt::Return {
                        value: Some(HirExpr::OkWrap {
                            ty: func_type.return_type.as_ref().clone(),
                            value: Box::new(HirExpr::NoneLiteral),
                        }),
                    });
                }
            }
            ctx.error(format!(
                "function expects return type '{}', but returns nothing",
                func_type.return_type.display_name()
            ));
        }
        None
    };

    Some(HirStmt::Return { value })
}

pub(super) fn lower_if(
    if_stmt: &StmtIf,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    // Try to detect a narrowing condition from the test expression
    let narrowing_cond = detect_narrowing_condition(&if_stmt.test, ctx);

    let condition = lower_expr(&if_stmt.test, ctx)?;

    // Save narrowing state before branches
    let saved_state = ctx.scope.save_narrowing_state();
    // Save moved state before branches
    let saved_moved = ctx.scope.save_moved_state();

    // Apply narrowing for then-branch (condition is true)
    if let Some(ref cond) = narrowing_cond {
        apply_narrowing(ctx, cond, true);
    }

    ctx.scope.push();
    let then_body = lower_stmts(&if_stmt.body, func_type, ctx);
    ctx.scope.pop();

    // Record which vars were moved in then-branch
    let then_moved = ctx.scope.save_moved_state();

    // Restore state before processing elif/else
    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);

    // Collect all narrowing conditions (if + elifs) for cumulative negation
    let mut all_conditions: Vec<NarrowingCondition> = Vec::new();
    if let Some(ref cond) = narrowing_cond {
        all_conditions.push(cond.clone());
    }

    // Track moved state from each branch for merging
    let mut branch_moved_states: Vec<_> = vec![then_moved];

    let mut elif_clauses = Vec::new();
    for clause in &if_stmt.elif_else_clauses {
        if let Some(test) = &clause.test {
            // For elif, apply the negation of ALL previous conditions
            // This ensures cumulative narrowing: if A was Dog, elif B was Cat,
            // then in elif C the type is narrowed by removing both Dog and Cat
            ctx.scope.restore_narrowing_state(&saved_state);
            ctx.scope.restore_moved_state(&saved_moved);
            for prev_cond in &all_conditions {
                apply_narrowing(ctx, prev_cond, false);
            }

            let elif_narrowing = detect_narrowing_condition(test, ctx);
            let cond = lower_expr(test, ctx)?;

            let elif_saved = ctx.scope.save_narrowing_state();
            if let Some(ref elif_cond) = elif_narrowing {
                apply_narrowing(ctx, elif_cond, true);
            }

            ctx.scope.push();
            let body = lower_stmts(&clause.body, func_type, ctx);
            ctx.scope.pop();
            elif_clauses.push((cond, body));

            // Record moved state from this elif branch
            branch_moved_states.push(ctx.scope.save_moved_state());

            ctx.scope.restore_narrowing_state(&elif_saved);
            ctx.scope.restore_moved_state(&saved_moved);

            // Track this elif's condition for subsequent branches
            if let Some(elif_cond) = elif_narrowing {
                all_conditions.push(elif_cond);
            }
        }
    }

    // For else-branch, apply negation of ALL conditions (if + all elifs)
    let else_body = if_stmt
        .elif_else_clauses
        .iter()
        .find(|c| c.test.is_none())
        .map(|clause| {
            ctx.scope.restore_narrowing_state(&saved_state);
            ctx.scope.restore_moved_state(&saved_moved);
            for prev_cond in &all_conditions {
                apply_narrowing(ctx, prev_cond, false);
            }
            ctx.scope.push();
            let body = lower_stmts(&clause.body, func_type, ctx);
            ctx.scope.pop();
            // Record moved state from else branch
            branch_moved_states.push(ctx.scope.save_moved_state());
            body
        });

    // Restore original narrowing state after all branches
    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);

    // Merge moved state: if a variable was moved in ANY branch, mark it as moved
    // after the if/else (conservative, matches Rust behavior for partial moves)
    for branch_state in &branch_moved_states {
        for (name, was_moved) in branch_state {
            if *was_moved {
                ctx.scope.mark_moved(name);
            }
        }
    }

    // Early-return narrowing: if the then-body always exits (return/break/continue/raise),
    // apply the inverse narrowing after the if block.
    // e.g., `if x is None: return` -> after the if, x is not None
    if let Some(ref cond) = narrowing_cond {
        if then_body_always_exits(&then_body) && elif_clauses.is_empty() && else_body.is_none() {
            apply_narrowing(ctx, cond, false);
        }
    }

    Some(HirStmt::If {
        condition,
        then_body,
        elif_clauses,
        else_body,
    })
}

/// Check if a block of statements always exits (return, break, continue, raise).
/// Used for early-return narrowing: `if x is None: return` narrows x after the if.
pub(super) fn then_body_always_exits(stmts: &[HirStmt]) -> bool {
    crate::cfg::flow_facts(stmts).always_exits()
}

/// Collect all return types from a list of HIR statements (recursively).
pub(super) fn collect_return_types(stmts: &[HirStmt]) -> Vec<Type> {
    crate::cfg::flow_facts(stmts)
        .reachable_return_types()
        .to_vec()
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
                        let var_name = var.id.clone();
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
                            let var_name = var.id.clone();
                            if ctx.scope.lookup(&var_name).is_some() {
                                return Some(NarrowingCondition::IsNone(var_name));
                            }
                        }
                    }
                    CmpOp::IsNot => {
                        if let (Expr::Name(var), Expr::NoneLiteral(_)) =
                            (cmp.left.as_ref(), &cmp.comparators[0])
                        {
                            let var_name = var.id.clone();
                            if ctx.scope.lookup(&var_name).is_some() {
                                return Some(NarrowingCondition::IsNotNone(var_name));
                            }
                        }
                    }
                    // x == "value" -> Equality narrowing
                    CmpOp::Eq => {
                        if let Expr::Name(var) = cmp.left.as_ref() {
                            let var_name = var.id.clone();
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
            let var_name = name.id.clone();
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
                Some(conditions.into_iter().next().unwrap())
            } else {
                Some(NarrowingCondition::And(conditions))
            }
        }
        _ => None,
    }
}

/// Convert an AST expression to a `LiteralValue` (for equality narrowing).
pub(super) fn expr_to_literal_value(expr: &Expr) -> Option<sifr_type_system::LiteralValue> {
    match expr {
        Expr::StringLiteral(s) => Some(sifr_type_system::LiteralValue::Str(
            s.value.to_str().to_string(),
        )),
        Expr::NumberLiteral(num) => match &num.value {
            Number::Int(i) => i.as_i64().map(sifr_type_system::LiteralValue::Int),
            _ => None,
        },
        Expr::BooleanLiteral(b) => Some(sifr_type_system::LiteralValue::Bool(b.value)),
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
    let condition = lower_expr(&while_stmt.test, ctx)?;

    // Snapshot moved state before loop to detect moves inside the body
    let moved_before_loop = ctx.scope.save_moved_state();

    ctx.scope.push();
    ctx.loop_depth += 1;
    let body = lower_stmts(&while_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

    // Check for outer-scope variables moved inside the loop body
    let newly_moved = ctx.scope.moved_since(&moved_before_loop);
    for var_name in &newly_moved {
        ctx.error(format!(
            "value '{var_name}' is moved inside loop body; it would be unavailable on subsequent iterations"
        ));
    }

    let else_body = if while_stmt.orelse.is_empty() {
        None
    } else {
        ctx.scope.push();
        let else_stmts = lower_stmts(&while_stmt.orelse, func_type, ctx);
        ctx.scope.pop();
        Some(else_stmts)
    };

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
    // Lower the iterable expression
    let iter_expr = lower_expr(&for_stmt.iter, ctx)?;
    let iter_ty = iter_expr.ty().clone();

    // Determine the element type from the iterable
    let elem_ty = iter_ty.iterable_element_type().unwrap_or_else(|| {
        ctx.error(format!(
            "cannot iterate over type '{}'",
            iter_ty.display_name()
        ));
        Type::Any
    });

    // Extract the target variable name(s)
    let target_name = match for_stmt.target.as_ref() {
        Expr::Name(n) => n.id.clone(),
        Expr::Tuple(tup) => {
            // Tuple unpacking: for i, v in enumerate(lst)
            let names: Vec<String> = tup
                .elts
                .iter()
                .filter_map(|e| {
                    if let Expr::Name(n) = e {
                        Some(n.id.clone())
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
    if target_name.contains(',') {
        // Tuple unpacking: define each variable with its type from the tuple
        let names: Vec<&str> = target_name.split(',').collect();
        if let Type::Tuple(elem_types) = &elem_ty {
            if elem_types.len() != names.len() {
                ctx.error(format!(
                    "for loop tuple target expects {} element(s), iterable yields {}",
                    names.len(),
                    elem_types.len()
                ));
                ctx.scope.pop();
                return None;
            }
            for (i, name) in names.iter().enumerate() {
                let ty = elem_types[i].clone();
                ctx.scope.define((*name).to_string(), ty);
            }
        } else {
            ctx.error(format!(
                "for loop tuple target expects iterable elements of tuple type, got '{}'",
                elem_ty.display_name()
            ));
            ctx.scope.pop();
            return None;
        }
    } else {
        ctx.scope.define(target_name.clone(), elem_ty.clone());
    }
    ctx.loop_depth += 1;
    let body = lower_stmts(&for_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

    // Check for outer-scope variables moved inside the loop body
    let newly_moved = ctx.scope.moved_since(&moved_before_loop);
    for var_name in &newly_moved {
        ctx.error(format!(
            "value '{var_name}' is moved inside loop body; it would be unavailable on subsequent iterations"
        ));
    }

    let else_body = if for_stmt.orelse.is_empty() {
        None
    } else {
        ctx.scope.push();
        let else_stmts = lower_stmts(&for_stmt.orelse, func_type, ctx);
        ctx.scope.pop();
        Some(else_stmts)
    };

    Some(HirStmt::For {
        target: target_name,
        target_ty: elem_ty,
        iter: iter_expr,
        body,
        else_body,
    })
}
