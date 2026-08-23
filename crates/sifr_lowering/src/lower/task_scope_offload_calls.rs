use super::LowerCtx;
use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::offload_worker_captures::validate_offload_worker_captures;
use super::task_scope_calls::{
    enforce_task_group_error_type, enforce_task_group_is_open, is_task_group_type,
    is_task_scope_type, non_send_reason,
};
use super::workload_annotations::WorkloadKind;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;

pub(in crate::lower) fn lower_task_scope_offload_method_call(
    object: HirExpr,
    method_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    if !is_task_scope_type(object.ty()) {
        return None;
    }
    match method_name {
        "spawn_blocking" => Some(lower_scope_spawn_blocking(object, call, ctx)),
        "spawn_cpu" => Some(lower_scope_spawn_cpu(object, call, ctx)),
        "spawn_process" => Some(lower_scope_spawn_process(object, call, ctx)),
        _ => None,
    }
}

fn lower_scope_spawn_blocking(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let worker = validate_sync_worker(call, ctx, "scope.spawn_blocking()")?;
    if super::workload_annotations::reject_unclassified_offload_target(
        ctx,
        &call.arguments.args[0],
        "scope.spawn_blocking()",
    ) {
        return None;
    }
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        return None;
    };
    let (ok_ty, err_ty, method) = match ft.return_type.resolve_alias() {
        Type::Result(ok, err) => (
            ok.as_ref().clone(),
            err.as_ref().clone(),
            "__sifr_scope_spawn_blocking_result",
        ),
        other => (
            other.clone(),
            Type::Never,
            "__sifr_scope_spawn_blocking_infallible",
        ),
    };
    validate_sendable_result(&ok_ty, &err_ty, call, ctx, "scope.spawn_blocking()")?;
    enforce_group_rules(&object, &err_ty, call, ctx)?;
    let receiver_convention =
        super::mutating_methods::receiver_convention_for_non_class_method(object.ty(), method);
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method.to_string(),
        args: vec![worker],
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(super::method_call_metadata::source_method_call(call)),
        ty: Type::Task(Box::new(ok_ty), Box::new(err_ty)),
    })
}

fn lower_scope_spawn_cpu(object: HirExpr, call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let worker = validate_sync_worker(call, ctx, "scope.spawn_cpu()")?;
    if super::workload_annotations::reject_offload_target_without_kind(
        ctx,
        &call.arguments.args[0],
        "scope.spawn_cpu()",
        WorkloadKind::CpuHeavy,
    ) {
        return None;
    }
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        return None;
    };
    let (ok_ty, source_err_ty, task_err_ty, method) = match ft.return_type.resolve_alias() {
        Type::Result(ok, err) => (
            ok.as_ref().clone(),
            err.as_ref().clone(),
            worker_error_type("WorkerError", ctx),
            "__sifr_scope_spawn_cpu_result",
        ),
        other => (
            other.clone(),
            Type::Never,
            worker_error_type("WorkerRuntimeError", ctx),
            "__sifr_scope_spawn_cpu_infallible",
        ),
    };
    validate_sendable_result(&ok_ty, &source_err_ty, call, ctx, "scope.spawn_cpu()")?;
    enforce_group_rules(&object, &task_err_ty, call, ctx)?;
    let receiver_convention =
        super::mutating_methods::receiver_convention_for_non_class_method(object.ty(), method);
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method.to_string(),
        args: vec![worker],
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(super::method_call_metadata::source_method_call(call)),
        ty: Type::Task(Box::new(ok_ty), Box::new(task_err_ty)),
    })
}

fn lower_scope_spawn_process(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            "scope.spawn_process() is only valid inside async functions".to_string(),
            call.range(),
        );
        return None;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "scope.spawn_process() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "scope.spawn_process() takes exactly one Command argument".to_string(),
            call_arity_range(call),
        );
        return None;
    }
    let command = lower_expr(&call.arguments.args[0], ctx)?;
    if !matches!(command.ty().resolve_alias(), Type::Class { name, .. } if name == "Command") {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "scope.spawn_process() requires a Command argument, got '{}'",
                command.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    if is_task_group_type(object.ty()) {
        enforce_task_group_is_open(&object, call, ctx)?;
    }
    let receiver_convention = super::mutating_methods::receiver_convention_for_non_class_method(
        object.ty(),
        "__sifr_scope_spawn_process",
    );
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: "__sifr_scope_spawn_process".to_string(),
        args: vec![command],
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(super::method_call_metadata::source_method_call(call)),
        ty: Type::Result(
            Box::new(process_class_type("ProcessHandle", ctx)),
            Box::new(process_error_type(ctx)),
        ),
    })
}

fn process_class_type(name: &str, ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get(name)
        .cloned()
        .unwrap_or_else(|| Type::Class {
            identity: Some(format!("sifr.parallel.{name}")),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: vec![("_handle".to_string(), Type::Int)],
            methods: Vec::new(),
            parent_class: None,
        })
}

fn process_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("ProcessError")
        .cloned()
        .unwrap_or_else(|| Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "ProcessError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        })
}

fn validate_sync_worker(call: &ExprCall, ctx: &mut LowerCtx, api_name: &str) -> Option<HirExpr> {
    if !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            format!("{api_name} is only valid inside async functions"),
            call.range(),
        );
        return None;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            format!("{api_name} does not accept keyword arguments"),
            first_call_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            format!("{api_name} takes exactly one sync function argument"),
            call_arity_range(call),
        );
        return None;
    }
    let worker = lower_expr(&call.arguments.args[0], ctx)?;
    validate_offload_worker_captures(api_name, &worker, call.arguments.args[0].range(), ctx)?;
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} requires a sync function argument, got '{}'",
                worker.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if !ft.params.is_empty() {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} v1 requires a zero-argument function; wrap owned inputs in a dedicated helper before offloading"
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    Some(worker)
}

fn validate_sendable_result(
    ok_ty: &Type,
    err_ty: &Type,
    call: &ExprCall,
    ctx: &mut LowerCtx,
    api_name: &str,
) -> Option<()> {
    if let Some(reason) = non_send_reason(ok_ty) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} cannot return non-send value type '{}': {reason}",
                ok_ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    if !matches!(err_ty.resolve_alias(), Type::Never) {
        if let Some(reason) = non_send_reason(err_ty) {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "{api_name} cannot return non-send error type '{}': {reason}",
                    err_ty.display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        }
    }
    Some(())
}

fn enforce_group_rules(
    object: &HirExpr,
    task_err_ty: &Type,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<()> {
    if !is_task_group_type(object.ty()) {
        return Some(());
    }
    enforce_task_group_is_open(object, call, ctx)?;
    enforce_task_group_error_type(object, task_err_ty, call, ctx)
}

fn worker_error_type(name: &str, ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get(name)
        .cloned()
        .unwrap_or_else(|| Type::Class {
            identity: Some(format!("sifr.parallel.{name}")),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_worker_errors_keep_canonical_parallel_identity() {
        let ctx = LowerCtx::new();
        for name in ["WorkerError", "WorkerRuntimeError"] {
            assert!(matches!(
                worker_error_type(name, &ctx),
                Type::Class { identity: Some(identity), .. }
                    if identity == format!("sifr.parallel.{name}")
            ));
        }
    }
}
