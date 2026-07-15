use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::offload_worker_captures::validate_offload_worker_captures;
use super::task_scope_calls::lower_task_scope_spawn_from_object_allowing_reserved_ctx;
use super::task_scope_calls::mark_task_handle_observed;
use super::task_scope_calls::non_send_reason;
use super::workload_annotations::WorkloadKind;
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
        "spawn_scoped" => lower_task_spawn_scoped_call(call, ctx),
        "spawn_blocking" => lower_task_spawn_blocking_call(call, ctx),
        "spawn_cpu" => lower_task_spawn_cpu_call(call, ctx),
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

fn lower_task_spawn_scoped_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.spawn_scoped() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if ctx.active_task_owner_depth == 0 {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.spawn_scoped() requires an active structured task owner; use it inside async with task.TaskGroup() as group"
                .to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.spawn_scoped() takes exactly one coroutine argument".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    let Some((owner_name, owner_ty)) = ctx.active_task_owner_bindings.last().cloned() else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.spawn_scoped() requires a named active task owner; use async with task.TaskGroup() as group"
                .to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    };
    let owner = HirExpr::Name {
        name: owner_name,
        ty: owner_ty,
    };
    match lower_task_scope_spawn_from_object_allowing_reserved_ctx(owner, call, ctx) {
        Some(expr) => TaskCallLowering::Lowered(expr),
        None => TaskCallLowering::Rejected,
    }
}

fn lower_task_spawn_cpu_call(call: &ExprCall, ctx: &mut LowerCtx) -> TaskCallLowering {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.spawn_cpu() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskCallLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.spawn_cpu() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.spawn_cpu() takes exactly one sync function argument".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }

    let worker = lower_expr(&call.arguments.args[0], ctx);
    let Some(worker) = worker else {
        return TaskCallLowering::Rejected;
    };
    if validate_offload_worker_captures(
        "task.spawn_cpu()",
        &worker,
        call.arguments.args[0].range(),
        ctx,
    )
    .is_none()
    {
        return TaskCallLowering::Rejected;
    }
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.spawn_cpu() requires a sync function argument, got '{}'",
                worker.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    };
    if !ft.params.is_empty() {
        expression_diagnostics::type_mismatch(
            ctx,
            "task.spawn_cpu() v1 requires a zero-argument function; wrap owned inputs in a dedicated helper before offloading".to_string(),
            call.arguments.args[0].range(),
        );
        return TaskCallLowering::Rejected;
    }
    if super::workload_annotations::reject_offload_target_without_kind(
        ctx,
        &call.arguments.args[0],
        "task.spawn_cpu()",
        WorkloadKind::CpuHeavy,
    ) {
        return TaskCallLowering::Rejected;
    }

    let (ok_ty, err_ty, result_func) = match ft.return_type.resolve_alias() {
        Type::Result(ok, err) => (
            ok.as_ref().clone(),
            err.as_ref().clone(),
            "__sifr_spawn_cpu_result",
        ),
        other => (other.clone(), Type::Never, "__sifr_spawn_cpu_infallible"),
    };
    if let Some(reason) = non_send_reason(&ok_ty) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.spawn_cpu() cannot return non-send value type '{}': {reason}",
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
                    "task.spawn_cpu() cannot return non-send error type '{}': {reason}",
                    err_ty.display_name()
                ),
                call.arguments.args[0].range(),
            );
            return TaskCallLowering::Rejected;
        }
    }

    let task_error_ty = if matches!(err_ty.resolve_alias(), Type::Never) {
        worker_error_type("WorkerRuntimeError", ctx)
    } else {
        worker_error_type("WorkerError", ctx)
    };
    TaskCallLowering::Lowered(HirExpr::Call {
        func: result_func.to_string(),
        args: vec![worker],
        ty: Type::BlockingTask(Box::new(ok_ty), Box::new(task_error_ty)),
    })
}

fn worker_error_type(name: &str, ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get(name)
        .cloned()
        .unwrap_or_else(|| Type::Class {
            identity: None,
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        })
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
    if validate_offload_worker_captures(
        "task.spawn_blocking()",
        &worker,
        call.arguments.args[0].range(),
        ctx,
    )
    .is_none()
    {
        return TaskCallLowering::Rejected;
    }
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
    if !call.arguments.args.is_empty() {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.select() takes named task branches, for example task.select(first=a, second=b)"
                .to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    if call.arguments.keywords.len() != 2 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.select() takes exactly two named task branches".to_string(),
            call_arity_range(call),
        );
        return TaskCallLowering::Rejected;
    }
    for keyword in &call.arguments.keywords {
        if keyword.arg.is_none() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "task.select() does not support unpacked keyword branches".to_string(),
                keyword.range,
            );
            return TaskCallLowering::Rejected;
        }
    }
    let first_keyword = &call.arguments.keywords[0];
    let second_keyword = &call.arguments.keywords[1];
    if first_keyword.arg == second_keyword.arg {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.select() branch names must be unique".to_string(),
            second_keyword.range,
        );
        return TaskCallLowering::Rejected;
    }

    let Some(first) = lower_expr(&first_keyword.value, ctx) else {
        return TaskCallLowering::Rejected;
    };
    let Type::Task(first_ok_ty, first_err_ty) = first.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.select() branch '{}' must be a task handle, got '{}'",
                first_keyword
                    .arg
                    .as_ref()
                    .map_or("<unpacked>", |name| name.as_str()),
                first.ty().display_name()
            ),
            first_keyword.value.range(),
        );
        return TaskCallLowering::Rejected;
    };
    let (first_ok_ty, first_err_ty) = (first_ok_ty.clone(), first_err_ty.clone());

    let Some(second) = lower_expr(&second_keyword.value, ctx) else {
        return TaskCallLowering::Rejected;
    };
    let Type::Task(second_ok_ty, second_err_ty) = second.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "task.select() branch '{}' must be a task handle, got '{}'",
                second_keyword
                    .arg
                    .as_ref()
                    .map_or("<unpacked>", |name| name.as_str()),
                second.ty().display_name()
            ),
            second_keyword.value.range(),
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
        ctx.mark_moved_with_flow(name);
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
            ctx.mark_moved_with_flow(name);
        }
        HirExpr::Name { name, .. } if matches!(expr.ty().resolve_alias(), Type::List(_)) => {
            ctx.mark_moved_with_flow(name);
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
