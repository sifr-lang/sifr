use super::{
    analyze_block, infer_expr_type, merge_env_types, Expr, FunctionEnv, HashMap,
    LocalFunctionState, LowerCtx, Type,
};
use sifr_python_ast::{ExceptHandler, Pattern, Stmt, StmtMatch, StmtTry, StmtWith};
use std::collections::HashSet;

pub(super) fn function_has_value_return(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Return(ret) => ret.value.is_some(),
        Stmt::If(stmt) => {
            function_has_value_return(&stmt.body)
                || stmt
                    .elif_else_clauses
                    .iter()
                    .any(|clause| function_has_value_return(&clause.body))
        }
        Stmt::While(stmt) => {
            function_has_value_return(&stmt.body) || function_has_value_return(&stmt.orelse)
        }
        Stmt::For(stmt) => {
            function_has_value_return(&stmt.body) || function_has_value_return(&stmt.orelse)
        }
        Stmt::With(stmt) => function_has_value_return(&stmt.body),
        Stmt::Try(stmt) => {
            function_has_value_return(&stmt.body)
                || function_has_value_return(&stmt.orelse)
                || function_has_value_return(&stmt.finalbody)
                || stmt.handlers.iter().any(|handler| {
                    let ExceptHandler::ExceptHandler(handler) = handler;
                    function_has_value_return(&handler.body)
                })
        }
        Stmt::Match(stmt) => stmt
            .cases
            .iter()
            .any(|case| function_has_value_return(&case.body)),
        Stmt::FunctionDef(_) | Stmt::ClassDef(_) => false,
        _ => false,
    })
}

fn block_always_exits(stmts: &[Stmt]) -> bool {
    stmts.iter().any(inference_stmt_always_exits)
}

fn pattern_is_unconditional(pattern: &Pattern) -> bool {
    matches!(pattern, Pattern::MatchAs(pattern) if pattern.pattern.is_none())
}

fn matched_class_type<'a>(subject_ty: &'a Type, class_name: &str) -> Option<&'a Type> {
    match subject_ty.resolve_alias() {
        Type::Class { name, .. } if name == class_name => Some(subject_ty.resolve_alias()),
        Type::Union(members) => members
            .iter()
            .find_map(|member| matched_class_type(member, class_name)),
        _ => None,
    }
}

pub(super) fn inference_stmt_always_exits(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return(_) | Stmt::Raise(_) => true,
        Stmt::If(stmt) => {
            block_always_exits(&stmt.body)
                && stmt
                    .elif_else_clauses
                    .last()
                    .is_some_and(|clause| clause.test.is_none())
                && stmt
                    .elif_else_clauses
                    .iter()
                    .all(|clause| block_always_exits(&clause.body))
        }
        Stmt::With(stmt) => block_always_exits(&stmt.body),
        Stmt::Match(stmt) => {
            stmt.cases.iter().all(|case| block_always_exits(&case.body))
                && stmt
                    .cases
                    .iter()
                    .any(|case| case.guard.is_none() && pattern_is_unconditional(&case.pattern))
        }
        Stmt::Try(stmt) => {
            if block_always_exits(&stmt.finalbody) {
                return true;
            }
            let normal_path_exits = block_always_exits(&stmt.body)
                || (!stmt.orelse.is_empty() && block_always_exits(&stmt.orelse));
            let handlers_exit = stmt.handlers.iter().all(|handler| {
                let ExceptHandler::ExceptHandler(handler) = handler;
                block_always_exits(&handler.body)
            });
            normal_path_exits && handlers_exit
        }
        _ => false,
    }
}

fn bind_pattern_names(pattern: &Pattern, subject_ty: &Type, env: &mut FunctionEnv, ctx: &LowerCtx) {
    match pattern {
        Pattern::MatchAs(pattern) => {
            if let Some(name) = &pattern.name {
                env.bind_var(name.as_str(), subject_ty.clone());
            }
            if let Some(inner) = &pattern.pattern {
                bind_pattern_names(inner, subject_ty, env, ctx);
            }
        }
        Pattern::MatchOr(pattern) => {
            for alternative in &pattern.patterns {
                bind_pattern_names(alternative, subject_ty, env, ctx);
            }
        }
        Pattern::MatchSequence(pattern) => {
            let element_types = match subject_ty.resolve_alias() {
                Type::Tuple(items) => items.clone(),
                _ => vec![Type::Unknown; pattern.patterns.len()],
            };
            for (index, element) in pattern.patterns.iter().enumerate() {
                let element_ty = element_types.get(index).cloned().unwrap_or(Type::Unknown);
                bind_pattern_names(element, &element_ty, env, ctx);
            }
        }
        Pattern::MatchClass(pattern) => {
            let class_name = match pattern.cls.as_ref() {
                Expr::Name(name) => Some(name.id.as_str()),
                _ => None,
            };
            let class_ty = class_name
                .and_then(|name| matched_class_type(subject_ty, name))
                .or_else(|| class_name.and_then(|name| ctx.class_types.get(name)));
            for keyword in &pattern.arguments.keywords {
                let field_ty = class_ty
                    .and_then(|ty| match ty.resolve_alias() {
                        Type::Class { fields, .. } => fields
                            .iter()
                            .find(|(name, _)| name == keyword.attr.as_str())
                            .map(|(_, ty)| ty.clone()),
                        _ => None,
                    })
                    .unwrap_or(Type::Unknown);
                bind_pattern_names(&keyword.pattern, &field_ty, env, ctx);
            }
        }
        _ => {}
    }
}

pub(super) fn analyze_match_stmt(
    stmt: &StmtMatch,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    let subject_ty = infer_expr_type(&stmt.subject, env, states, current_function, ctx);
    for case in &stmt.cases {
        let mut case_env = env.clone();
        bind_pattern_names(&case.pattern, &subject_ty, &mut case_env, ctx);
        if let Some(guard) = &case.guard {
            let _ = infer_expr_type(guard, &mut case_env, states, current_function, ctx);
        }
        analyze_block(&case.body, &mut case_env, states, current_function, ctx);
        merge_env_types(env, &case_env);
    }
}

pub(super) fn analyze_try_stmt(
    stmt: &StmtTry,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    let mut body_env = env.clone();
    analyze_block(&stmt.body, &mut body_env, states, current_function, ctx);
    if !stmt.orelse.is_empty() {
        analyze_block(&stmt.orelse, &mut body_env, states, current_function, ctx);
    }
    merge_env_types(env, &body_env);
    for handler in &stmt.handlers {
        let ExceptHandler::ExceptHandler(handler) = handler;
        let mut handler_env = env.clone();
        if let Some(name) = &handler.name {
            handler_env.bind_var(name.as_str(), Type::Any);
        }
        analyze_block(
            &handler.body,
            &mut handler_env,
            states,
            current_function,
            ctx,
        );
        merge_env_types(env, &handler_env);
    }
    if !stmt.finalbody.is_empty() {
        let mut final_env = env.clone();
        analyze_block(
            &stmt.finalbody,
            &mut final_env,
            states,
            current_function,
            ctx,
        );
        merge_env_types(env, &final_env);
    }
}

pub(super) fn analyze_with_stmt(
    stmt: &StmtWith,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    let mut body_env = env.clone();
    for item in &stmt.items {
        let context_ty = infer_expr_type(
            &item.context_expr,
            &mut body_env,
            states,
            current_function,
            ctx,
        );
        if let Some(target) = &item.optional_vars {
            if let Expr::Name(name) = target.as_ref() {
                let entered_ty = match context_ty.resolve_alias() {
                    Type::Class { methods, .. } => methods
                        .iter()
                        .find(|(method, _)| method == "__enter__")
                        .map_or_else(
                            || context_ty.clone(),
                            |(_, function)| (*function.return_type).clone(),
                        ),
                    _ => context_ty,
                };
                body_env.bind_var(name.id.as_str(), entered_ty);
            }
        }
    }
    analyze_block(&stmt.body, &mut body_env, states, current_function, ctx);
    merge_env_types(env, &body_env);
}

fn collect_pattern_names(pattern: &Pattern, bindings: &mut HashSet<String>) {
    match pattern {
        Pattern::MatchAs(pattern) => {
            if let Some(name) = &pattern.name {
                bindings.insert(name.to_string());
            }
            if let Some(inner) = &pattern.pattern {
                collect_pattern_names(inner, bindings);
            }
        }
        Pattern::MatchOr(pattern) => {
            for alternative in &pattern.patterns {
                collect_pattern_names(alternative, bindings);
            }
        }
        Pattern::MatchSequence(pattern) => {
            for element in &pattern.patterns {
                collect_pattern_names(element, bindings);
            }
        }
        Pattern::MatchClass(pattern) => {
            for keyword in &pattern.arguments.keywords {
                collect_pattern_names(&keyword.pattern, bindings);
            }
        }
        _ => {}
    }
}

pub(super) fn collect_compound_local_bindings(stmt: &Stmt, bindings: &mut HashSet<String>) -> bool {
    match stmt {
        Stmt::Match(stmt) => {
            for case in &stmt.cases {
                collect_pattern_names(&case.pattern, bindings);
                super::collect_current_function_local_bindings(&case.body, bindings);
            }
        }
        Stmt::Try(stmt) => {
            super::collect_current_function_local_bindings(&stmt.body, bindings);
            super::collect_current_function_local_bindings(&stmt.orelse, bindings);
            super::collect_current_function_local_bindings(&stmt.finalbody, bindings);
            for handler in &stmt.handlers {
                let ExceptHandler::ExceptHandler(handler) = handler;
                if let Some(name) = &handler.name {
                    bindings.insert(name.to_string());
                }
                super::collect_current_function_local_bindings(&handler.body, bindings);
            }
        }
        _ => return false,
    }
    true
}

pub(super) fn collect_compound_nonlocals(stmt: &Stmt, names: &mut HashSet<String>) -> bool {
    match stmt {
        Stmt::Match(stmt) => {
            for case in &stmt.cases {
                super::collect_nonlocal_names(&case.body, names);
            }
        }
        Stmt::Try(stmt) => {
            super::collect_nonlocal_names(&stmt.body, names);
            super::collect_nonlocal_names(&stmt.orelse, names);
            super::collect_nonlocal_names(&stmt.finalbody, names);
            for handler in &stmt.handlers {
                let ExceptHandler::ExceptHandler(handler) = handler;
                super::collect_nonlocal_names(&handler.body, names);
            }
        }
        _ => return false,
    }
    true
}
