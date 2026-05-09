use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprAttribute, ExprCall};
use sifr_type_system::Type;

pub(super) enum TaskCallLowering {
    Lowered(HirExpr),
    Rejected,
    NoMatch,
}

pub(super) fn lower_task_module_call(
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
