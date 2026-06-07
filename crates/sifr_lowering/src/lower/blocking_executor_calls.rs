use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::offload_worker_captures::validate_offload_worker_captures;
use super::task_scope_calls::non_send_reason;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;

pub(in crate::lower) fn lower_thread_pool_submit_call(
    object: &HirExpr,
    method_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    if !is_thread_pool_executor_type(object.ty()) || method_name != "submit" {
        return None;
    }
    if !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            "ThreadPoolExecutor.submit() is only valid inside async functions in v1".to_string(),
            call.range(),
        );
        return Some(None);
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "ThreadPoolExecutor.submit() does not accept keyword arguments in v1".to_string(),
            first_call_keyword_range(call),
        );
        return Some(None);
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "ThreadPoolExecutor.submit() takes exactly one sync function argument".to_string(),
            call_arity_range(call),
        );
        return Some(None);
    }

    let worker = lower_expr(&call.arguments.args[0], ctx)?;
    if validate_offload_worker_captures(
        "ThreadPoolExecutor.submit()",
        &worker,
        call.arguments.args[0].range(),
        ctx,
    )
    .is_none()
    {
        return Some(None);
    }
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "ThreadPoolExecutor.submit() requires a sync function argument, got '{}'",
                worker.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return Some(None);
    };
    if !ft.params.is_empty() {
        expression_diagnostics::type_mismatch(
            ctx,
            "ThreadPoolExecutor.submit() v1 requires a zero-argument function; wrap owned inputs in a dedicated helper before offloading".to_string(),
            call.arguments.args[0].range(),
        );
        return Some(None);
    }
    if super::workload_annotations::reject_unclassified_offload_target(
        ctx,
        &call.arguments.args[0],
        "ThreadPoolExecutor.submit()",
    ) {
        return Some(None);
    }

    let (ok_ty, err_ty, submit_func) = match ft.return_type.resolve_alias() {
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
                "ThreadPoolExecutor.submit() cannot return non-send value type '{}': {reason}",
                ok_ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return Some(None);
    }
    if !matches!(err_ty.resolve_alias(), Type::Never) {
        if let Some(reason) = non_send_reason(&err_ty) {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "ThreadPoolExecutor.submit() cannot return non-send error type '{}': {reason}",
                    err_ty.display_name()
                ),
                call.arguments.args[0].range(),
            );
            return Some(None);
        }
    }

    Some(Some(HirExpr::Call {
        func: submit_func.to_string(),
        args: vec![worker],
        ty: Type::BlockingTask(Box::new(ok_ty), Box::new(err_ty)),
    }))
}

fn is_thread_pool_executor_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if public_type_name(name) == "ThreadPoolExecutor")
}

fn public_type_name(name: &str) -> &str {
    name.strip_prefix("__compat_sifr_concurrent_")
        .unwrap_or(name)
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
