use super::async_generator_advances::finish_async_generator_advance_for_expr;
use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::ownership_diagnostics;
use super::task_scope_calls::{is_lock_guard_type, mark_task_handle_observed};
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_python_ast::ExprAwait;
use sifr_type_system::Type;

pub(super) fn lower_await(await_expr: &ExprAwait, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            "await is only valid inside async functions".to_string(),
            await_expr.range(),
        );
        return None;
    }

    for (name, ty) in ctx.scope.active_bindings() {
        if is_lock_guard_type(&ty) {
            ownership_diagnostics::lock_guard_across_await(ctx, &name, await_expr.range());
        }
    }

    let value = lower_expr(await_expr.value.as_ref(), ctx)?;
    let result_ty = await_result_type(value.ty());
    let Some(ty) = result_ty else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "await requires an awaitable value, got '{}'",
                value.ty().display_name()
            ),
            await_expr.value.range(),
        );
        return None;
    };

    if matches!(
        value.ty().resolve_alias(),
        Type::Task(_, _) | Type::BlockingTask(_, _)
    ) {
        if let HirExpr::Name { name, .. } = &value {
            mark_task_handle_observed(name, ctx);
            ctx.scope.mark_moved(name);
        }
    }
    finish_async_generator_advance_for_expr(ctx, &value);

    Some(HirExpr::Await {
        value: Box::new(value),
        ty,
    })
}

fn await_result_type(ty: &Type) -> Option<Type> {
    match ty.resolve_alias() {
        Type::Coroutine(ok, err) if matches!(err.resolve_alias(), Type::Never) => {
            Some(ok.as_ref().clone())
        }
        Type::Coroutine(ok, err) => Some(Type::Result(ok.clone(), err.clone())),
        Type::Task(ok, err) => Some(Type::TaskResult(ok.clone(), err.clone())),
        Type::BlockingTask(ok, err) => Some(Type::TaskResult(ok.clone(), err.clone())),
        Type::Awaitable(result) => Some(result.as_ref().clone()),
        _ => None,
    }
}

pub(super) fn coroutine_result_type(surface_return_type: &Type) -> Type {
    match surface_return_type.resolve_alias() {
        Type::Result(ok, err) => Type::Coroutine(ok.clone(), err.clone()),
        other => Type::Coroutine(Box::new(other.clone()), Box::new(Type::Never)),
    }
}
