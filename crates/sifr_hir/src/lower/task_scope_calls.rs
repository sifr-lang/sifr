use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::{ExprAttribute, ExprCall};
use sifr_type_system::Type;

pub(super) fn is_task_scope_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if name == "TaskScope" || name == "TaskGroup")
}

fn is_task_group_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if name == "TaskGroup")
}

pub(super) fn lower_task_scope_spawn_call(
    object: HirExpr,
    _attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            "scope.spawn() is only valid inside async functions".to_string(),
            call.range(),
        );
        return None;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "scope.spawn() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "scope.spawn() takes exactly one coroutine argument".to_string(),
            call_arity_range(call),
        );
        return None;
    }

    let coroutine = lower_expr(&call.arguments.args[0], ctx)?;
    let Type::Coroutine(ok_ty, err_ty) = coroutine.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "scope.spawn() requires a coroutine argument, got '{}'",
                coroutine.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };
    let task_ok_ty = ok_ty.clone();
    let task_err_ty = err_ty.clone();
    if is_task_group_type(object.ty()) {
        enforce_task_group_is_open(&object, call, ctx)?;
        enforce_task_group_error_type(&object, &task_err_ty, call, ctx)?;
    }
    let HirExpr::Call { args, .. } = &coroutine else {
        expression_diagnostics::type_mismatch(
            ctx,
            "scope.spawn() requires a direct coroutine call in v1".to_string(),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if let Some(name) = borrowed_task_boundary_argument(args, ctx) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "scope.spawn() cannot move borrowed parameter '{name}' across a task boundary; pass an owned value or clone it before spawning"
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }

    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: if matches!(task_err_ty.resolve_alias(), Type::Never) {
            "__sifr_spawn_infallible".to_string()
        } else {
            "__sifr_spawn_result".to_string()
        },
        args: vec![coroutine],
        ty: Type::Task(task_ok_ty, task_err_ty),
    })
}

fn borrowed_task_boundary_argument(args: &[HirExpr], ctx: &LowerCtx) -> Option<String> {
    args.iter()
        .find_map(|arg| borrowed_task_boundary_argument_in_expr(arg, ctx))
}

fn borrowed_task_boundary_argument_in_expr(expr: &HirExpr, ctx: &LowerCtx) -> Option<String> {
    match expr {
        HirExpr::Name { name, ty }
            if ctx.borrowed_params.contains(name.as_str())
                && matches!(ty.ownership(), sifr_type_system::OwnershipKind::Move) =>
        {
            Some(name.clone())
        }
        HirExpr::TupleLiteral { elements, .. }
        | HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. } => borrowed_task_boundary_argument(elements, ctx),
        HirExpr::DictLiteral { keys, values, .. } => keys
            .iter()
            .chain(values.iter())
            .find_map(|expr| borrowed_task_boundary_argument_in_expr(expr, ctx)),
        _ => None,
    }
}

pub(super) fn task_group_spawn_owner(expr: &HirExpr) -> Option<String> {
    let HirExpr::MethodCall { object, method, .. } = expr else {
        return None;
    };
    if method != "__sifr_spawn_infallible" && method != "__sifr_spawn_result" {
        return None;
    }
    let HirExpr::Name { name, ty } = object.as_ref() else {
        return None;
    };
    is_task_group_type(ty).then(|| name.clone())
}

pub(super) fn mark_task_handle_observed(name: &str, ctx: &mut LowerCtx) {
    if let Some(group_name) = ctx.task_handle_group_owners.get(name).cloned() {
        ctx.task_groups_not_proven_open.insert(group_name);
    }
}

fn enforce_task_group_is_open(object: &HirExpr, call: &ExprCall, ctx: &mut LowerCtx) -> Option<()> {
    let HirExpr::Name { name, .. } = object else {
        return Some(());
    };
    if !ctx.task_groups_not_proven_open.contains(name) {
        return Some(());
    }
    expression_diagnostics::type_mismatch(
        ctx,
        format!(
            "task.TaskGroup() binding '{name}' is no longer proven Open after observing a child task; spawn before observation or use a new group"
        ),
        call.range(),
    );
    None
}

fn enforce_task_group_error_type(
    object: &HirExpr,
    task_err_ty: &Type,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<()> {
    let HirExpr::Name { name, .. } = object else {
        return Some(());
    };
    if matches!(task_err_ty.resolve_alias(), Type::Never) {
        return Some(());
    }
    if let Some(existing) = ctx.task_group_error_types.get(name) {
        if task_err_ty.is_assignable_to(existing) && existing.is_assignable_to(task_err_ty) {
            return Some(());
        }
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.TaskGroup() children must share one error type in v1; expected '{}', got '{}'",
                existing.display_name(),
                task_err_ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    ctx.task_group_error_types
        .insert(name.clone(), task_err_ty.clone());
    Some(())
}

fn first_call_keyword_range(call: &ExprCall) -> TextRange {
    call.arguments
        .keywords
        .first()
        .map_or_else(|| call.func.range(), |keyword| keyword.range)
}

fn call_arity_range(call: &ExprCall) -> TextRange {
    call.arguments
        .args
        .last()
        .map_or_else(|| call.func.range(), Ranged::range)
}
