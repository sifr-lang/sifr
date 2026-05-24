use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::task_scope_calls::non_send_reason;
use super::task_scope_calls::{
    is_task_scope_type, lower_task_scope_spawn_from_object, mark_task_handle_observed,
};
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprAttribute, ExprCall};
use sifr_type_system::Type;

pub(in crate::lower) enum TaskCallLowering {
    Lowered(HirExpr),
    Rejected,
    NoMatch,
}

pub(in crate::lower) fn lower_asyncio_compat_call(
    func_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> TaskCallLowering {
    let Some(member_name) = ctx.asyncio_compat_imports.get(func_name).cloned() else {
        return TaskCallLowering::NoMatch;
    };
    match member_name.as_str() {
        "sleep" => lower_task_sleep_call(call, ctx),
        "wait_for" => lower_task_timeout_call(call, ctx),
        "gather" => lower_task_gather_call(call, ctx),
        "create_task" => lower_asyncio_create_task_call(call, ctx),
        "run" => lower_asyncio_run_call(call, ctx),
        _ => TaskCallLowering::NoMatch,
    }
}

fn lower_asyncio_run_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if ctx.current_owner.as_deref() != Some("main") {
        expression_diagnostics::type_mismatch(
            ctx,
            "asyncio.run() is only supported as a main() entrypoint compatibility shim; call and await the coroutine directly inside async code".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "asyncio.run() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "asyncio.run() takes exactly one coroutine argument".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }

    let Some(coroutine) = lower_expr(&call.arguments.args[0], ctx) else {
        return TaskCallLowering::Rejected;
    };
    let Type::Coroutine(ok_ty, err_ty) = coroutine.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "asyncio.run() requires a coroutine returned by an async function, got '{}'",
                coroutine.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    let ty = if matches!(err_ty.resolve_alias(), Type::Never) {
        ok_ty.as_ref().clone()
    } else {
        Type::Result(ok_ty.clone(), err_ty.clone())
    };
    TaskCallLowering::Lowered(HirExpr::Await {
        value: Box::new(coroutine),
        ty,
    })
}

fn lower_asyncio_create_task_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    let active_scopes: Vec<(String, Type)> = ctx
        .scope
        .active_bindings()
        .into_iter()
        .filter(|(_, ty)| is_task_scope_type(ty))
        .collect();
    let [(scope_name, scope_ty)] = active_scopes.as_slice() else {
        expression_diagnostics::type_mismatch(
            ctx,
            "asyncio.create_task() requires exactly one active task scope; use it inside async with task.scope() or task.TaskGroup()".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    };
    let scope_object = HirExpr::Name {
        name: scope_name.clone(),
        ty: scope_ty.clone(),
    };
    lower_task_scope_spawn_from_object(scope_object, call, ctx)
        .map_or(TaskCallLowering::Rejected, TaskCallLowering::Lowered)
}

pub(in crate::lower) fn lower_task_module_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> TaskCallLowering {
    let Expr::Name(module_name) = attr.value.as_ref() else {
        return TaskCallLowering::NoMatch;
    };
    if module_name.id.as_str() != "task" {
        return TaskCallLowering::NoMatch;
    }
    match attr.attr.as_str() {
        "sleep" => lower_task_sleep_call(call, ctx),
        "timeout" => lower_task_timeout_call(call, ctx),
        "gather" => lower_task_gather_call(call, ctx),
        "race" => lower_task_race_call(call, ctx),
        "select" => lower_task_select_call(call, ctx),
        "spawn_blocking" => lower_task_spawn_blocking_call(call, ctx),
        "spawn" => {
            expression_diagnostics::type_mismatch(
                ctx,
                "task.spawn() is not available in v1; use scope.spawn(...) inside async with task.scope()".to_string(),
                call.range(),
            );
            TaskCallLowering::Rejected
        }
        _ => TaskCallLowering::NoMatch,
    }
}

fn lower_task_spawn_blocking_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.spawn_blocking() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.spawn_blocking() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.spawn_blocking() takes exactly one sync function argument".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }

    let worker = lower_expr(&call.arguments.args[0], ctx);
    let Some(worker) = worker else {
        return TaskCallLowering::Rejected;
    };
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.spawn_blocking() requires a sync function argument, got '{}'",
                worker.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    if !ft.params.is_empty() {
        expression_diagnostics::type_mismatch(
            ctx,
            "task.spawn_blocking() v1 requires a zero-argument function; wrap owned inputs in a dedicated helper before offloading".to_string(),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    }
    if super::workload_annotations::reject_unclassified_offload_target(
        ctx,
        &call.arguments.args[0],
        "task.spawn_blocking()",
    ) {
        return TaskCallLowering::Rejected;
    }

    let (ok_ty, err_ty, result_func) = match ft.return_type.resolve_alias() {
        Type::Result(ok, err) => (
            ok.as_ref().clone(),
            err.as_ref().clone(),
            "__sifr_spawn_blocking_result",
        ),
        other => (
            other.clone(),
            Type::Never,
            "__sifr_spawn_blocking_infallible",
        ),
    };
    if let Some(reason) = non_send_reason(&ok_ty) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.spawn_blocking() cannot return non-send value type '{}': {reason}",
                ok_ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !matches!(err_ty.resolve_alias(), Type::Never) {
        if let Some(reason) = non_send_reason(&err_ty) {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "task.spawn_blocking() cannot return non-send error type '{}': {reason}",
                    err_ty.display_name()
                ),
                call.arguments.args[0].range(),
            );
            return TaskCallLowering::Rejected;
        }
    }

    TaskCallLowering::Lowered(HirExpr::Call {
        func: result_func.to_string(),
        args: vec![worker],
        ty: Type::BlockingTask(Box::new(ok_ty), Box::new(err_ty)),
    })
}

fn lower_task_gather_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.gather() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.gather() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.gather() takes exactly one list of task handles".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }

    let Some(handles) = lower_expr(&call.arguments.args[0], ctx) else {
        return TaskCallLowering::Rejected;
    };
    let handles_ty = handles.ty().clone();
    let Type::List(element_ty) = handles_ty.resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.gather() argument must be list[Task[T, E]], got '{}'",
                handles.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    let Type::Task(ok_ty, err_ty) = element_ty.resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.gather() argument must be list[Task[T, E]], got '{}'",
                handles.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    let result_ok_ty = ok_ty.clone();
    let result_err_ty = err_ty.clone();
    mark_task_handle_names_moved(&handles, ctx);
    TaskCallLowering::Lowered(HirExpr::Call {
        func: "__sifr_task_gather".to_string(),
        args: vec![handles],
        ty: Type::Awaitable(Box::new(Type::TaskResult(
            Box::new(Type::List(result_ok_ty)),
            result_err_ty,
        ))),
    })
}

fn lower_task_select_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.select() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.select() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 2 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.select() takes exactly two task handles".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }

    let Some(first) = lower_expr(&call.arguments.args[0], ctx) else {
        return TaskCallLowering::Rejected;
    };
    let Type::Task(first_ok_ty, first_err_ty) = first.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.select() first argument must be a task handle, got '{}'",
                first.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    let (first_ok_ty, first_err_ty) = (first_ok_ty.clone(), first_err_ty.clone());

    let Some(second) = lower_expr(&call.arguments.args[1], ctx) else {
        return TaskCallLowering::Rejected;
    };
    let Type::Task(second_ok_ty, second_err_ty) = second.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.select() second argument must be a task handle, got '{}'",
                second.ty().display_name()
            ),
            call.arguments.args[1].range(),
        );
        return TaskCallLowering::Rejected;
    };
    let (second_ok_ty, second_err_ty) = (second_ok_ty.clone(), second_err_ty.clone());

    mark_task_handle_names_moved(&first, ctx);
    mark_task_handle_names_moved(&second, ctx);

    let first_result_ty = Type::TaskResult(first_ok_ty, first_err_ty);
    let second_result_ty = Type::TaskResult(second_ok_ty, second_err_ty);
    TaskCallLowering::Lowered(HirExpr::Call {
        func: "__sifr_task_select".to_string(),
        args: vec![first, second],
        ty: Type::Awaitable(Box::new(Type::Select2(
            Box::new(first_result_ty),
            Box::new(second_result_ty),
        ))),
    })
}

fn lower_task_race_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.race() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.race() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.race() takes exactly one list of task handles".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }

    let Some(handles) = lower_expr(&call.arguments.args[0], ctx) else {
        return TaskCallLowering::Rejected;
    };
    let handles_ty = handles.ty().clone();
    let Type::List(element_ty) = handles_ty.resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.race() argument must be list[Task[T, E]], got '{}'",
                handles.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    let Type::Task(ok_ty, err_ty) = element_ty.resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.race() argument must be list[Task[T, E]], got '{}'",
                handles.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    let result_ok_ty = ok_ty.clone();
    let result_err_ty = err_ty.clone();
    mark_task_handle_names_moved(&handles, ctx);
    TaskCallLowering::Lowered(HirExpr::Call {
        func: "__sifr_task_race".to_string(),
        args: vec![handles],
        ty: Type::Awaitable(Box::new(Type::TaskResult(result_ok_ty, result_err_ty))),
    })
}

fn lower_task_sleep_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.sleep() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.sleep() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.sleep() takes exactly one duration argument".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    let Some(duration) = lower_expr(&call.arguments.args[0], ctx) else {
        return TaskCallLowering::Rejected;
    };
    if !matches!(duration.ty().resolve_alias(), Type::Int | Type::Float) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.sleep() duration must be int or float, got '{}'",
                duration.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    }
    TaskCallLowering::Lowered(HirExpr::Call {
        func: "__sifr_task_sleep".to_string(),
        args: vec![duration],
        ty: Type::Awaitable(Box::new(Type::None)),
    })
}

fn lower_task_timeout_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.timeout() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.timeout() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 2 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.timeout() takes exactly one task handle and one duration argument".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }

    let Some(handle) = lower_expr(&call.arguments.args[0], ctx) else {
        return TaskCallLowering::Rejected;
    };
    let Type::Task(ok_ty, err_ty) = handle.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.timeout() first argument must be a task handle, got '{}'",
                handle.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    let (ok_ty, err_ty) = (ok_ty.clone(), err_ty.clone());
    let Some(duration) = lower_expr(&call.arguments.args[1], ctx) else {
        return TaskCallLowering::Rejected;
    };
    if !matches!(duration.ty().resolve_alias(), Type::Int | Type::Float) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.timeout() duration must be int or float, got '{}'",
                duration.ty().display_name()
            ),
            call.arguments.args[1].range(),
        );
        return TaskCallLowering::Rejected;
    }
    if let HirExpr::Name { name, .. } = &handle {
        mark_task_handle_observed(name, ctx);
        ctx.scope.mark_moved(name);
    }
    TaskCallLowering::Lowered(HirExpr::MethodCall {
        object: Box::new(handle),
        method: "__sifr_timeout".to_string(),
        args: vec![duration],
        ty: Type::Awaitable(Box::new(Type::TaskResult(
            ok_ty,
            Box::new(Type::TimeoutResult(err_ty)),
        ))),
    })
}

fn mark_task_handle_names_moved(expr: &HirExpr, ctx: &mut LowerCtx) {
    match expr {
        HirExpr::Name { name, .. } if matches!(expr.ty().resolve_alias(), Type::Task(_, _)) => {
            mark_task_handle_observed(name, ctx);
            ctx.scope.mark_moved(name);
        }
        HirExpr::Name { name, .. } if matches!(expr.ty().resolve_alias(), Type::List(_)) => {
            ctx.scope.mark_moved(name);
        }
        HirExpr::ListLiteral { elements, .. } | HirExpr::TupleLiteral { elements, .. } => {
            for element in elements {
                mark_task_handle_names_moved(element, ctx);
            }
        }
        _ => {}
    }
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
        .first()
        .map_or_else(|| call.func.range(), Ranged::range)
}
