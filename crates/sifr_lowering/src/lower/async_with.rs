use super::expressions::lower_expr;
use super::python_interop::try_lower_python_async_with;
use super::statement_diagnostics;
use super::statements::lower_stmts;
use super::task_context_keywords::lower_task_context_keyword;
use super::task_owner_scope_state::{
    enter_task_owner_scope, exit_task_owner_scope, task_group_type, task_scope_type,
};
use super::LowerCtx;
use crate::hir_nodes::{HirAsyncWithKind, HirStmt};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, StmtWith};
use sifr_type_system::{FunctionType, Type};

fn simple_with_target_name(optional_vars: Option<&Expr>, ctx: &mut LowerCtx) -> Option<String> {
    let vars = optional_vars?;
    if let Expr::Name(n) = vars {
        Some(n.id.to_string())
    } else {
        statement_diagnostics::unsupported_form(
            ctx,
            "with target must be a simple name",
            vars.range(),
        );
        None
    }
}

fn async_task_call_name(expr: &Expr) -> Option<(&str, &sifr_python_ast::ExprCall)> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Attribute(attr) = call.func.as_ref() else {
        return None;
    };
    let Expr::Name(module_name) = attr.value.as_ref() else {
        return None;
    };
    if module_name.id.as_str() == "task" {
        Some((attr.attr.as_str(), call))
    } else {
        None
    }
}

fn timeout_error_type() -> Type {
    Type::Class {
        name: "TimeoutError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: vec![],
        parent_class: Some("Error".to_string()),
    }
}

fn scope_failure_type() -> Type {
    Type::Class {
        name: "ScopeFailure".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: vec![],
        parent_class: Some("Error".to_string()),
    }
}

fn return_type_accepts_timeout_error(return_type: &Type) -> bool {
    let Type::Result(_, err) = return_type.resolve_alias() else {
        return false;
    };
    timeout_error_type().is_assignable_to(err)
}

fn return_type_accepts_scope_failure(return_type: &Type) -> bool {
    let Type::Result(_, err) = return_type.resolve_alias() else {
        return false;
    };
    scope_failure_type().is_assignable_to(err)
}

fn async_exit_cause_type() -> Type {
    Type::Class {
        name: "AsyncExitCause".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    }
}

fn return_type_accepts_error(return_type: &Type, error_ty: &Type) -> bool {
    let Type::Result(_, err) = return_type.resolve_alias() else {
        return false;
    };
    error_ty.is_assignable_to(err)
}

fn method_signature<'a>(
    methods: &'a [(String, FunctionType)],
    method_name: &str,
) -> Option<&'a FunctionType> {
    methods.iter().find_map(
        |(name, ft)| {
            if name == method_name {
                Some(ft)
            } else {
                None
            }
        },
    )
}

fn async_context_methods(ty: &Type) -> Option<(&FunctionType, &FunctionType)> {
    match ty.resolve_alias() {
        Type::Class { methods, .. } | Type::Protocol { methods, .. } => Some((
            method_signature(methods, "__aenter__")?,
            method_signature(methods, "__aexit__")?,
        )),
        _ => None,
    }
}

fn async_result_parts(ty: &Type) -> Option<(Type, Type)> {
    let Type::Coroutine(ok_ty, err_ty) = ty.resolve_alias() else {
        return None;
    };
    Some((ok_ty.as_ref().clone(), err_ty.as_ref().clone()))
}

fn lower_user_async_with(
    with_stmt: &StmtWith,
    item: &sifr_python_ast::WithItem,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    ctx.with_pushed_scope(|ctx| {
        let context = lower_expr(&item.context_expr, ctx)?;
        if let Some(lowered) =
            try_lower_python_async_with(with_stmt, item, func_type, &context, ctx)
        {
            return lowered;
        }
        lower_native_user_async_with(with_stmt, item, func_type, context, ctx)
    })
}

fn lower_native_user_async_with(
    with_stmt: &StmtWith,
    item: &sifr_python_ast::WithItem,
    func_type: &FunctionType,
    context: crate::hir_nodes::HirExpr,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let context_ty = context.ty().clone();
    let Some((enter_ft, exit_ft)) = async_context_methods(&context_ty) else {
        ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "async with requires an async context manager with __aenter__() and __aexit__(AsyncExitCause), got '{}'",
                    context_ty.display_name()
                ),
                item.context_expr.range(),
            );
        return None;
    };
    if !enter_ft.params.is_empty() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "__aenter__ for async with must take no arguments".to_string(),
            item.context_expr.range(),
        );
        return None;
    }
    if exit_ft.params.len() != 1 || !async_exit_cause_type().is_assignable_to(&exit_ft.params[0].1)
    {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "__aexit__ for async with must take exactly one AsyncExitCause argument".to_string(),
            item.context_expr.range(),
        );
        return None;
    }
    let Some((enter_value_ty, enter_error_ty)) = async_result_parts(&enter_ft.return_type) else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "__aenter__ for async with must be async and return Result[T, E]".to_string(),
            item.context_expr.range(),
        );
        return None;
    };
    let Some((exit_value_ty, exit_error_ty)) = async_result_parts(&exit_ft.return_type) else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "__aexit__ for async with must be async and return Result[None, E]".to_string(),
            item.context_expr.range(),
        );
        return None;
    };
    if !matches!(exit_value_ty.resolve_alias(), Type::None) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "__aexit__ for async with must return Result[None, E]".to_string(),
            item.context_expr.range(),
        );
        return None;
    }
    if !return_type_accepts_error(&func_type.return_type, &enter_error_ty)
        || !return_type_accepts_error(&func_type.return_type, &exit_error_ty)
    {
        ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "fallible async context manager enter/exit requires the enclosing function to return a compatible Result error type".to_string(),
                item.context_expr.range(),
            );
        return None;
    }
    let target = simple_with_target_name(item.optional_vars.as_deref(), ctx);
    if let Some(name) = &target {
        ctx.scope.define(name.clone(), enter_value_ty.clone());
    }
    let body = lower_stmts(&with_stmt.body, func_type, ctx);
    if body.iter().any(stmt_contains_user_async_with_blocked_exit) {
        ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "user-defined async context managers cannot use raise or yield inside the body until abnormal-exit cleanup lowering is implemented".to_string(),
                item.context_expr.range(),
            );
        return None;
    }
    Some(HirStmt::AsyncWith {
        kind: HirAsyncWithKind::UserDefined {
            context,
            enter_value_ty,
            enter_error_ty,
            exit_error_ty,
        },
        target,
        body,
    })
}

fn expr_contains_await(expr: &crate::hir_nodes::HirExpr) -> bool {
    use crate::hir_nodes::HirExpr;

    match expr {
        HirExpr::Await { .. } => true,
        HirExpr::QuestionMark { expr, .. }
        | HirExpr::OkWrap { value: expr, .. }
        | HirExpr::ErrWrap { value: expr, .. }
        | HirExpr::FieldAccess { object: expr, .. } => expr_contains_await(expr),
        HirExpr::Call { args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::IteratorCall { args, .. } => args.iter().any(expr_contains_await),
        HirExpr::FString { parts, .. } => parts.iter().any(|part| match part {
            crate::hir_nodes::HirFStringPart::Literal(_) => false,
            crate::hir_nodes::HirFStringPart::Expr(expr) => expr_contains_await(expr),
        }),
        HirExpr::MethodCall { object, args, .. } => {
            expr_contains_await(object) || args.iter().any(expr_contains_await)
        }
        HirExpr::BinOp { left, right, .. } => {
            expr_contains_await(left) || expr_contains_await(right)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => expr_contains_await(left) || comparators.iter().any(expr_contains_await),
        HirExpr::BoolOp { values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        } => values.iter().any(expr_contains_await),
        HirExpr::UnaryOp { operand, .. } => expr_contains_await(operand),
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_contains_await(condition)
                || expr_contains_await(then_expr)
                || expr_contains_await(else_expr)
        }
        HirExpr::Index { object, index, .. } => {
            expr_contains_await(object) || expr_contains_await(index)
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            expr_contains_await(object)
                || start.as_deref().is_some_and(expr_contains_await)
                || stop.as_deref().is_some_and(expr_contains_await)
                || step.as_deref().is_some_and(expr_contains_await)
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            keys.iter().any(expr_contains_await) || values.iter().any(expr_contains_await)
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => expr_contains_await(element) || expr_contains_await(collection),
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            expr_contains_await(start)
                || expr_contains_await(end)
                || step.as_deref().is_some_and(expr_contains_await)
        }
        HirExpr::WalrusExpr { value, .. } => expr_contains_await(value),
        HirExpr::SuperCall { args, .. } => args.iter().any(expr_contains_await),
        HirExpr::Lambda { body, .. } => expr_contains_await(body),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            expr_contains_await(expr)
                || generators.iter().any(|(_, iter, filter)| {
                    expr_contains_await(iter) || filter.as_ref().is_some_and(expr_contains_await)
                })
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            expr_contains_await(key_expr)
                || expr_contains_await(val_expr)
                || generators.iter().any(|(_, iter, filter)| {
                    expr_contains_await(iter) || filter.as_ref().is_some_and(expr_contains_await)
                })
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            expr_contains_await(expr)
                || expr_contains_await(iter)
                || filter.as_deref().is_some_and(expr_contains_await)
        }
        _ => false,
    }
}

fn expr_contains_task_spawn(expr: &crate::hir_nodes::HirExpr) -> bool {
    use crate::hir_nodes::HirExpr;

    match expr {
        HirExpr::MethodCall { method, .. }
            if method == "__sifr_spawn_infallible"
                || method == "__sifr_spawn_infallible_with_context"
                || method == "__sifr_spawn_result"
                || method == "__sifr_spawn_result_with_context" =>
        {
            true
        }
        HirExpr::Await { value, .. }
        | HirExpr::QuestionMark { expr: value, .. }
        | HirExpr::OkWrap { value, .. }
        | HirExpr::ErrWrap { value, .. }
        | HirExpr::FieldAccess { object: value, .. } => expr_contains_task_spawn(value),
        HirExpr::Call { args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::IteratorCall { args, .. } => args.iter().any(expr_contains_task_spawn),
        HirExpr::FString { parts, .. } => parts.iter().any(|part| match part {
            crate::hir_nodes::HirFStringPart::Literal(_) => false,
            crate::hir_nodes::HirFStringPart::Expr(expr) => expr_contains_task_spawn(expr),
        }),
        HirExpr::MethodCall { object, args, .. } => {
            expr_contains_task_spawn(object) || args.iter().any(expr_contains_task_spawn)
        }
        HirExpr::BinOp { left, right, .. } => {
            expr_contains_task_spawn(left) || expr_contains_task_spawn(right)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => expr_contains_task_spawn(left) || comparators.iter().any(expr_contains_task_spawn),
        HirExpr::BoolOp { values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        } => values.iter().any(expr_contains_task_spawn),
        HirExpr::UnaryOp { operand, .. } => expr_contains_task_spawn(operand),
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_contains_task_spawn(condition)
                || expr_contains_task_spawn(then_expr)
                || expr_contains_task_spawn(else_expr)
        }
        HirExpr::Index { object, index, .. } => {
            expr_contains_task_spawn(object) || expr_contains_task_spawn(index)
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            expr_contains_task_spawn(object)
                || start.as_deref().is_some_and(expr_contains_task_spawn)
                || stop.as_deref().is_some_and(expr_contains_task_spawn)
                || step.as_deref().is_some_and(expr_contains_task_spawn)
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            keys.iter().any(expr_contains_task_spawn) || values.iter().any(expr_contains_task_spawn)
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => expr_contains_task_spawn(element) || expr_contains_task_spawn(collection),
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            expr_contains_task_spawn(start)
                || expr_contains_task_spawn(end)
                || step.as_deref().is_some_and(expr_contains_task_spawn)
        }
        HirExpr::WalrusExpr { value, .. } => expr_contains_task_spawn(value),
        HirExpr::SuperCall { args, .. } => args.iter().any(expr_contains_task_spawn),
        HirExpr::Lambda { body, .. } => expr_contains_task_spawn(body),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            expr_contains_task_spawn(expr)
                || generators.iter().any(|(_, iter, filter)| {
                    expr_contains_task_spawn(iter)
                        || filter.as_ref().is_some_and(expr_contains_task_spawn)
                })
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            expr_contains_task_spawn(key_expr)
                || expr_contains_task_spawn(val_expr)
                || generators.iter().any(|(_, iter, filter)| {
                    expr_contains_task_spawn(iter)
                        || filter.as_ref().is_some_and(expr_contains_task_spawn)
                })
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            expr_contains_task_spawn(expr)
                || expr_contains_task_spawn(iter)
                || filter.as_deref().is_some_and(expr_contains_task_spawn)
        }
        _ => false,
    }
}

fn stmt_contains_await(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Expr { expr }
        | HirStmt::Let { value: expr, .. }
        | HirStmt::Assign { value: expr, .. }
        | HirStmt::AugAssign { value: expr, .. }
        | HirStmt::AttributeAugAssign { value: expr, .. }
        | HirStmt::FieldAssign { value: expr, .. }
        | HirStmt::NestedFieldAssign { value: expr, .. }
        | HirStmt::Return { value: Some(expr) }
        | HirStmt::Assert { test: expr, .. }
        | HirStmt::Raise { value: expr }
        | HirStmt::TupleUnpack { value: expr, .. }
        | HirStmt::StarUnpack { value: expr, .. }
        | HirStmt::SubscriptAssign { value: expr, .. }
        | HirStmt::NestedSubscriptAssign { value: expr, .. }
        | HirStmt::AttributeNestedSubscriptAssign { value: expr, .. }
        | HirStmt::SubscriptAugAssign { value: expr, .. }
        | HirStmt::AttributeSubscriptAssign { value: expr, .. }
        | HirStmt::Yield { value: expr } => expr_contains_await(expr),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            expr_contains_await(condition)
                || then_body.iter().any(stmt_contains_await)
                || elif_clauses.iter().any(|(condition, body)| {
                    expr_contains_await(condition) || body.iter().any(stmt_contains_await)
                })
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(stmt_contains_await))
        }
        HirStmt::While {
            condition, body, ..
        } => expr_contains_await(condition) || body.iter().any(stmt_contains_await),
        HirStmt::For {
            iter,
            body,
            else_body,
            ..
        } => {
            expr_contains_await(iter)
                || body.iter().any(stmt_contains_await)
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(stmt_contains_await))
        }
        HirStmt::AsyncFor { .. } => true,
        HirStmt::AsyncWith { kind, body, .. } => {
            matches!(
                kind,
                HirAsyncWithKind::TaskTimeout { .. } | HirAsyncWithKind::UserDefined { .. }
            ) || body.iter().any(stmt_contains_await)
        }
        HirStmt::Delete { object, index } => {
            expr_contains_await(object) || expr_contains_await(index)
        }
        HirStmt::With { items, body } => {
            items.iter().any(|item| expr_contains_await(&item.context))
                || body.iter().any(stmt_contains_await)
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            body.iter().any(stmt_contains_await)
                || handlers
                    .iter()
                    .any(|handler| handler.body.iter().any(stmt_contains_await))
        }
        HirStmt::TryFinally { body, finalbody } => {
            body.iter().any(stmt_contains_await) || finalbody.iter().any(stmt_contains_await)
        }
        HirStmt::Match { subject, arms, .. } => {
            expr_contains_await(subject)
                || arms.iter().any(|arm| {
                    arm.guard.as_ref().is_some_and(expr_contains_await)
                        || arm.body.iter().any(stmt_contains_await)
                })
        }
        _ => false,
    }
}

fn stmt_contains_task_spawn(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Expr { expr }
        | HirStmt::Let { value: expr, .. }
        | HirStmt::Assign { value: expr, .. }
        | HirStmt::AugAssign { value: expr, .. }
        | HirStmt::AttributeAugAssign { value: expr, .. }
        | HirStmt::FieldAssign { value: expr, .. }
        | HirStmt::NestedFieldAssign { value: expr, .. }
        | HirStmt::Return { value: Some(expr) }
        | HirStmt::Assert { test: expr, .. }
        | HirStmt::Raise { value: expr }
        | HirStmt::TupleUnpack { value: expr, .. }
        | HirStmt::StarUnpack { value: expr, .. }
        | HirStmt::SubscriptAssign { value: expr, .. }
        | HirStmt::NestedSubscriptAssign { value: expr, .. }
        | HirStmt::AttributeNestedSubscriptAssign { value: expr, .. }
        | HirStmt::SubscriptAugAssign { value: expr, .. }
        | HirStmt::AttributeSubscriptAssign { value: expr, .. }
        | HirStmt::Yield { value: expr } => expr_contains_task_spawn(expr),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            expr_contains_task_spawn(condition)
                || then_body.iter().any(stmt_contains_task_spawn)
                || elif_clauses.iter().any(|(condition, body)| {
                    expr_contains_task_spawn(condition) || body.iter().any(stmt_contains_task_spawn)
                })
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(stmt_contains_task_spawn))
        }
        HirStmt::While {
            condition, body, ..
        } => expr_contains_task_spawn(condition) || body.iter().any(stmt_contains_task_spawn),
        HirStmt::For {
            iter,
            body,
            else_body,
            ..
        }
        | HirStmt::AsyncFor {
            iter,
            body,
            else_body,
            ..
        } => {
            expr_contains_task_spawn(iter)
                || body.iter().any(stmt_contains_task_spawn)
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(stmt_contains_task_spawn))
        }
        HirStmt::AsyncWith { body, .. } => body.iter().any(stmt_contains_task_spawn),
        HirStmt::Delete { object, index } => {
            expr_contains_task_spawn(object) || expr_contains_task_spawn(index)
        }
        HirStmt::With { items, body } => {
            items
                .iter()
                .any(|item| expr_contains_task_spawn(&item.context))
                || body.iter().any(stmt_contains_task_spawn)
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            body.iter().any(stmt_contains_task_spawn)
                || handlers
                    .iter()
                    .any(|handler| handler.body.iter().any(stmt_contains_task_spawn))
        }
        HirStmt::TryFinally { body, finalbody } => {
            body.iter().any(stmt_contains_task_spawn)
                || finalbody.iter().any(stmt_contains_task_spawn)
        }
        HirStmt::Match { subject, arms, .. } => {
            expr_contains_task_spawn(subject)
                || arms.iter().any(|arm| {
                    arm.guard.as_ref().is_some_and(expr_contains_task_spawn)
                        || arm.body.iter().any(stmt_contains_task_spawn)
                })
        }
        _ => false,
    }
}

fn stmt_contains_user_async_with_blocked_exit(stmt: &HirStmt) -> bool {
    stmt_contains_scope_exit(stmt, false)
}

fn stmt_contains_scope_early_exit(stmt: &HirStmt) -> bool {
    stmt_contains_scope_exit(stmt, true)
}

fn stmt_contains_scope_exit(stmt: &HirStmt, include_return: bool) -> bool {
    match stmt {
        HirStmt::Return { .. } => include_return,
        HirStmt::Raise { .. } | HirStmt::Yield { .. } => true,
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            then_body
                .iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                || elif_clauses.iter().any(|(_, body)| {
                    body.iter()
                        .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                })
                || else_body.as_ref().is_some_and(|body| {
                    body.iter()
                        .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                })
        }
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        }
        | HirStmt::AsyncFor {
            body, else_body, ..
        } => {
            body.iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                || else_body.as_ref().is_some_and(|body| {
                    body.iter()
                        .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                })
        }
        HirStmt::AsyncWith { body, .. } | HirStmt::With { body, .. } => body
            .iter()
            .any(|stmt| stmt_contains_scope_exit(stmt, include_return)),
        HirStmt::TryExcept { body, handlers, .. } => {
            body.iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                || handlers.iter().any(|handler| {
                    handler
                        .body
                        .iter()
                        .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                })
        }
        HirStmt::TryFinally { body, finalbody } => {
            body.iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                || finalbody
                    .iter()
                    .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
        }
        HirStmt::Match { arms, .. } => arms.iter().any(|arm| {
            arm.body
                .iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
        }),
        _ => false,
    }
}

pub(in crate::lower) fn lower_async_with(
    with_stmt: &StmtWith,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "async with is only valid inside async functions".to_string(),
            with_stmt.range(),
        );
        return None;
    }
    if with_stmt.items.len() != 1 {
        statement_diagnostics::unsupported_form(
            ctx,
            "async with supports exactly one context item in v1",
            with_stmt.range(),
        );
        return None;
    }

    let item = &with_stmt.items[0];
    let task_call = async_task_call_name(&item.context_expr);
    let Some((task_fn, call)) = task_call else {
        return lower_user_async_with(with_stmt, item, func_type, ctx);
    };

    ctx.with_pushed_scope(|ctx| {
        let (kind, target) = match task_fn {
            "scope" => {
                if !call.arguments.keywords.is_empty() {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
                        "task.scope() does not accept keyword arguments".to_string(),
                        call.arguments.keywords[0].range(),
                    );
                    return None;
                }
                if !call.arguments.args.is_empty() {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
                        "task.scope() takes no arguments".to_string(),
                        item.context_expr.range(),
                    );
                    return None;
                }
                let target = simple_with_target_name(item.optional_vars.as_deref(), ctx);
                if let Some(name) = &target {
                    ctx.scope.define(name.clone(), task_scope_type());
                }
                (HirAsyncWithKind::TaskScope, target)
            }
            "TaskGroup" => {
                let context = lower_task_context_keyword(ctx, call, "task.TaskGroup()")?;
                if !call.arguments.args.is_empty() {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
                        "task.TaskGroup() takes no positional arguments".to_string(),
                        item.context_expr.range(),
                    );
                    return None;
                }
                let target = simple_with_target_name(item.optional_vars.as_deref(), ctx);
                if let Some(name) = &target {
                    ctx.scope.define(name.clone(), task_group_type());
                }
                (HirAsyncWithKind::TaskGroup { context }, target)
            }
            "timeout" => {
                if !call.arguments.keywords.is_empty() {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
                        "task.timeout() does not accept keyword arguments".to_string(),
                        call.arguments.keywords[0].range(),
                    );
                    return None;
                }
                if call.arguments.args.len() != 1 {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
                        "task.timeout() takes exactly one duration argument".to_string(),
                        item.context_expr.range(),
                    );
                    return None;
                }
                if item.optional_vars.is_some() {
                    statement_diagnostics::unsupported_form(
                        ctx,
                        "async with task.timeout(duration) does not bind a target in v1",
                        item.optional_vars
                            .as_ref()
                            .map_or(item.context_expr.range(), |vars| vars.range()),
                    );
                    return None;
                }
                let duration = lower_expr(&call.arguments.args[0], ctx)?;
                if !matches!(duration.ty().resolve_alias(), Type::Int | Type::Float) {
                    ctx.error_with_code_at(
                        DiagnosticCode::TYPE_MISMATCH,
                        format!(
                            "task.timeout() duration must be int or float, got '{}'",
                            duration.ty().display_name()
                        ),
                        call.arguments.args[0].range(),
                    );
                    return None;
                }
                (HirAsyncWithKind::TaskTimeout { duration }, None)
            }
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    "async with only supports task.scope(), task.TaskGroup(), and task.timeout(duration) in v1"
                        .to_string(),
                    item.context_expr.range(),
                );
                return None;
            }
        };
        let task_owner_snapshot = enter_task_owner_scope(ctx, &kind, target.as_ref());
        let body = lower_stmts(&with_stmt.body, func_type, ctx);
        exit_task_owner_scope(ctx, task_owner_snapshot);
        if matches!(kind, HirAsyncWithKind::TaskTimeout { .. })
            && body.iter().any(stmt_contains_await)
            && !return_type_accepts_timeout_error(&func_type.return_type)
        {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "async with task.timeout(duration) can time out at await points; enclosing function must return Result[..., TimeoutError]"
                    .to_string(),
                item.context_expr.range(),
            );
            return None;
        }
        if matches!(
            kind,
            HirAsyncWithKind::TaskScope | HirAsyncWithKind::TaskGroup { .. }
        )
            && body.iter().any(stmt_contains_task_spawn)
            && !return_type_accepts_scope_failure(&func_type.return_type)
        {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "async task scopes that spawn children can fail at exit; enclosing function must return Result[..., ScopeFailure] or Result[..., Error]"
                    .to_string(),
                item.context_expr.range(),
            );
            return None;
        }
        if matches!(
            kind,
            HirAsyncWithKind::TaskScope | HirAsyncWithKind::TaskGroup { .. }
        )
            && body.iter().any(stmt_contains_task_spawn)
            && body.iter().any(stmt_contains_scope_early_exit)
        {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "async task scopes that spawn children cannot use return, raise, or yield inside the scope until abnormal-exit cleanup lowering is implemented"
                    .to_string(),
                item.context_expr.range(),
            );
            return None;
        }
        Some(HirStmt::AsyncWith { kind, target, body })
    })
}
