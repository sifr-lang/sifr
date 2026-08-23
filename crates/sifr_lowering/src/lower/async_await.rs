use super::LowerCtx;
use super::async_generator_advances::finish_async_generator_advance_for_expr;
use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::ownership_diagnostics;
use super::task_join_set_calls::consume_awaited_join_set_terminal;
use super::task_scope_calls::{mark_task_handle_observed, sync_guard_type_label};
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_python_ast::ExprAwait;
use sifr_type_system::Type;

pub(in crate::lower) fn lower_await(await_expr: &ExprAwait, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            "await is only valid inside async functions".to_string(),
            await_expr.range(),
        );
        return None;
    }

    for (name, ty) in ctx.scope.active_bindings() {
        if let Some(label) = sync_guard_type_label(&ty) {
            ownership_diagnostics::sync_guard_across_await(ctx, &name, label, await_expr.range());
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

    if let HirExpr::Call { func, .. } = &value {
        if ctx.async_functions.contains(func)
            && matches!(
                ctx.async_suspension_summaries.get(func),
                Some(super::async_effects::AsyncSuspensionSummary::NoSuspend)
            )
        {
            ctx.error_with_code_at(
                sifr_diagnostics::DiagnosticCode::ASYNC_AWAIT_NO_SUSPEND,
                format!(
                    "awaited async function '{func}' has no real suspension effect; remove await and make it a synchronous function"
                ),
                await_expr.value.range(),
            );
        }
    }

    if matches!(
        value.ty().resolve_alias(),
        Type::Task(_, _) | Type::BlockingTask(_, _)
    ) {
        if let HirExpr::Name { name, .. } = &value {
            mark_task_handle_observed(name, ctx);
            ctx.mark_moved_with_flow(name);
        }
    }
    consume_awaited_join_set_terminal(&value, ctx);
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

pub(in crate::lower) fn coroutine_result_type(surface_return_type: &Type) -> Type {
    match surface_return_type.resolve_alias() {
        Type::Result(ok, err) => Type::Coroutine(ok.clone(), err.clone()),
        other => Type::Coroutine(Box::new(other.clone()), Box::new(Type::Never)),
    }
}
