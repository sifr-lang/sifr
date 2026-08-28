use super::LowerCtx;
use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::offload_worker_captures::validate_offload_worker_captures;
use super::task_scope_calls::{mark_task_handle_observed, non_send_reason};
use super::typing_and_functions::resolve_annotation_expr;
use super::workload_annotations::WorkloadKind;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::{Expr, ExprCall, ExprSubscript};
use sifr_type_system::Type;

pub(in crate::lower) enum JoinSetConstructorLowering {
    Lowered(HirExpr),
    Rejected,
    NoMatch,
}

pub(in crate::lower) fn lower_task_join_set_constructor(
    subscript: &ExprSubscript,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> JoinSetConstructorLowering {
    if !is_task_join_set_subscript(subscript) {
        return JoinSetConstructorLowering::NoMatch;
    }
    if !call.arguments.args.is_empty() {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.JoinSet[T, E]() takes no positional arguments".to_string(),
            call_arity_range(call),
        );
        return JoinSetConstructorLowering::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.JoinSet[T, E]() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return JoinSetConstructorLowering::Rejected;
    }
    let Expr::Tuple(tuple) = subscript.slice.as_ref() else {
        expression_diagnostics::type_mismatch(
            ctx,
            "task.JoinSet type constructor requires [T, E] syntax".to_string(),
            subscript.slice.range(),
        );
        return JoinSetConstructorLowering::Rejected;
    };
    if tuple.elts.len() != 2 {
        expression_diagnostics::type_mismatch(
            ctx,
            "task.JoinSet type constructor requires exactly 2 type parameters".to_string(),
            subscript.slice.range(),
        );
        return JoinSetConstructorLowering::Rejected;
    }
    let ok_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
    let err_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
    JoinSetConstructorLowering::Lowered(HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "__sifr_join_set_new".to_string(),
        args: vec![],
        ty: Type::JoinSet(Box::new(ok_ty), Box::new(err_ty)),
    })
}

pub(in crate::lower) fn lower_join_set_method_call(
    object: HirExpr,
    method_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    let join_set_ty = object.ty().resolve_alias().clone();
    let Type::JoinSet(ok_ty, err_ty) = join_set_ty else {
        return None;
    };
    let ok_ty = ok_ty.as_ref();
    let err_ty = err_ty.as_ref();
    match method_name {
        "add" => Some(lower_join_set_add(object, call, ctx, ok_ty, err_ty)),
        "spawn_blocking" => Some(lower_join_set_spawn_blocking(
            object, call, ctx, ok_ty, err_ty,
        )),
        "spawn_cpu" => Some(lower_join_set_spawn_cpu(object, call, ctx, ok_ty, err_ty)),
        "join_all" => Some(lower_join_set_terminal(
            object,
            call,
            ctx,
            "__sifr_join_all",
            Type::Awaitable(Box::new(Type::List(Box::new(Type::TaskResult(
                Box::new(ok_ty.clone()),
                Box::new(err_ty.clone()),
            ))))),
        )),
        "cancel_all" => Some(lower_join_set_terminal(
            object,
            call,
            ctx,
            "__sifr_cancel_all",
            Type::Awaitable(Box::new(Type::List(Box::new(cancel_outcome_type())))),
        )),
        _ => None,
    }
}

pub(in crate::lower) fn consume_awaited_join_set_terminal(value: &HirExpr, ctx: &mut LowerCtx) {
    if let Some(owner) = join_set_terminal_owner(value) {
        consume_join_set_owner(&owner, ctx);
        return;
    }
    if let HirExpr::Name { name, .. } = value {
        if let Some(owner) = ctx.join_set_terminal_awaitables.remove(name) {
            consume_join_set_owner(&owner, ctx);
        }
    }
}

pub(in crate::lower) fn record_join_set_terminal_awaitable(
    binding: &str,
    value: &HirExpr,
    ctx: &mut LowerCtx,
) {
    if let Some(owner) = join_set_terminal_owner(value) {
        ctx.join_set_terminal_awaitables
            .insert(binding.to_string(), owner);
    } else {
        ctx.join_set_terminal_awaitables.remove(binding);
    }
}

fn lower_join_set_add(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
    ok_ty: &Type,
    err_ty: &Type,
) -> Option<HirExpr> {
    validate_no_keywords(call, ctx, "JoinSet.add()")?;
    validate_exact_arg_count(call, ctx, "JoinSet.add()", 1)?;
    let handle = lower_expr(&call.arguments.args[0], ctx)?;
    let method = match handle.ty().resolve_alias() {
        Type::Task(handle_ok, handle_err)
            if handle_ok.is_assignable_to(ok_ty) && handle_err.is_assignable_to(err_ty) =>
        {
            "__sifr_add_task"
        }
        Type::BlockingTask(handle_ok, handle_err)
            if handle_ok.is_assignable_to(ok_ty) && handle_err.is_assignable_to(err_ty) =>
        {
            "__sifr_add_blocking_task"
        }
        Type::Task(_, _) | Type::BlockingTask(_, _) => {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "JoinSet.add() handle type must match JoinSet[{}, {}], got '{}'",
                    ok_ty.display_name(),
                    err_ty.display_name(),
                    handle.ty().display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        }
        _ => {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "JoinSet.add() requires a task handle, got '{}'",
                    handle.ty().display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        }
    };
    mark_join_set_live(&object, ctx);
    mark_handle_consumed(&handle, ctx);
    let receiver_convention =
        super::mutating_methods::receiver_convention_for_non_class_method(object.ty(), method);
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method.to_string(),
        args: vec![handle],
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(super::method_call_metadata::source_method_call(call)),
        ty: join_item_id_type(),
    })
}

fn lower_join_set_spawn_blocking(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
    ok_ty: &Type,
    err_ty: &Type,
) -> Option<HirExpr> {
    let worker = validate_worker(call, ctx, "JoinSet.spawn_blocking()")?;
    if super::workload_annotations::reject_unclassified_offload_target(
        ctx,
        &call.arguments.args[0],
        "JoinSet.spawn_blocking()",
    ) {
        return None;
    }
    validate_worker_result_type(
        &worker,
        ok_ty,
        err_ty,
        call,
        ctx,
        "JoinSet.spawn_blocking()",
    )?;
    mark_join_set_live(&object, ctx);
    let receiver_convention = super::mutating_methods::receiver_convention_for_non_class_method(
        object.ty(),
        "__sifr_spawn_blocking",
    );
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: "__sifr_spawn_blocking".to_string(),
        args: vec![worker],
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(super::method_call_metadata::source_method_call(call)),
        ty: join_item_id_type(),
    })
}

fn lower_join_set_spawn_cpu(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
    ok_ty: &Type,
    err_ty: &Type,
) -> Option<HirExpr> {
    let worker = validate_worker(call, ctx, "JoinSet.spawn_cpu()")?;
    if super::workload_annotations::reject_offload_target_without_kind(
        ctx,
        &call.arguments.args[0],
        "JoinSet.spawn_cpu()",
        WorkloadKind::CpuHeavy,
    ) {
        return None;
    }
    let worker_error = worker_error_type("WorkerError", ctx);
    if !worker_error.is_assignable_to(err_ty) || !err_ty.is_assignable_to(&worker_error) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "JoinSet.spawn_cpu() requires JoinSet[T, WorkerError] in the current task runtime rules, got JoinSet[{}, {}]",
                ok_ty.display_name(),
                err_ty.display_name()
            ),
            call.func.range(),
        );
        return None;
    }
    validate_worker_result_ok_type(&worker, ok_ty, call, ctx, "JoinSet.spawn_cpu()")?;
    if let Type::Function(ft) = worker.ty().resolve_alias() {
        if let Type::Result(_, worker_err) = ft.return_type.resolve_alias() {
            if let Some(reason) = non_send_reason(worker_err) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "JoinSet.spawn_cpu() cannot return non-send error type '{}': {reason}",
                        worker_err.display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            }
        }
    }
    mark_join_set_live(&object, ctx);
    let receiver_convention = super::mutating_methods::receiver_convention_for_non_class_method(
        object.ty(),
        "__sifr_spawn_cpu",
    );
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: "__sifr_spawn_cpu".to_string(),
        args: vec![worker],
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(super::method_call_metadata::source_method_call(call)),
        ty: join_item_id_type(),
    })
}

fn lower_join_set_terminal(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
    method: &str,
    ty: Type,
) -> Option<HirExpr> {
    validate_no_keywords(call, ctx, "JoinSet terminal method")?;
    validate_exact_arg_count(call, ctx, "JoinSet terminal method", 0)?;
    let receiver_convention =
        super::mutating_methods::receiver_convention_for_non_class_method(object.ty(), method);
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method.to_string(),
        args: vec![],
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(super::method_call_metadata::source_method_call(call)),
        ty,
    })
}

fn validate_worker(call: &ExprCall, ctx: &mut LowerCtx, api_name: &str) -> Option<HirExpr> {
    validate_no_keywords(call, ctx, api_name)?;
    validate_exact_arg_count(call, ctx, api_name, 1)?;
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

fn validate_worker_result_type(
    worker: &HirExpr,
    ok_ty: &Type,
    err_ty: &Type,
    call: &ExprCall,
    ctx: &mut LowerCtx,
    api_name: &str,
) -> Option<()> {
    validate_worker_result_ok_type(worker, ok_ty, call, ctx, api_name)?;
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        return None;
    };
    let Type::Result(_, worker_err) = ft.return_type.resolve_alias() else {
        return None;
    };
    if !worker_err.is_assignable_to(err_ty) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} worker error type '{}' must match JoinSet error type '{}'",
                worker_err.display_name(),
                err_ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    if let Some(reason) = non_send_reason(worker_err) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} cannot return non-send error type '{}': {reason}",
                worker_err.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    Some(())
}

fn validate_worker_result_ok_type(
    worker: &HirExpr,
    ok_ty: &Type,
    call: &ExprCall,
    ctx: &mut LowerCtx,
    api_name: &str,
) -> Option<()> {
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        return None;
    };
    let Type::Result(worker_ok, _) = ft.return_type.resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!("{api_name} requires a worker returning Result[T, E]"),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if !worker_ok.is_assignable_to(ok_ty) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} worker value type '{}' must match JoinSet value type '{}'",
                worker_ok.display_name(),
                ok_ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    if let Some(reason) = non_send_reason(worker_ok) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} cannot return non-send value type '{}': {reason}",
                worker_ok.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    Some(())
}

fn mark_handle_consumed(handle: &HirExpr, ctx: &mut LowerCtx) {
    if let HirExpr::Name { name, .. } = handle {
        mark_task_handle_observed(name, ctx);
        ctx.mark_moved_with_flow(name);
    }
}

fn mark_join_set_live(object: &HirExpr, ctx: &mut LowerCtx) {
    if let HirExpr::Name { name, .. } = object {
        ctx.live_join_set_bindings.insert(name.clone());
    }
}

fn consume_join_set_owner(owner: &str, ctx: &mut LowerCtx) {
    ctx.live_join_set_bindings.remove(owner);
    ctx.mark_moved_with_flow(owner);
}

fn join_set_terminal_owner(value: &HirExpr) -> Option<String> {
    let HirExpr::MethodCall { object, method, .. } = value else {
        return None;
    };
    if method != "__sifr_join_all" && method != "__sifr_cancel_all" {
        return None;
    }
    let HirExpr::Name { name, .. } = object.as_ref() else {
        return None;
    };
    Some(name.clone())
}

fn is_task_join_set_subscript(subscript: &ExprSubscript) -> bool {
    let Expr::Attribute(attr) = subscript.value.as_ref() else {
        return false;
    };
    let Expr::Name(module) = attr.value.as_ref() else {
        return false;
    };
    module.id.as_str() == "task" && attr.attr.as_str() == "JoinSet"
}

fn join_item_id_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "JoinItemId".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    }
}

fn cancel_outcome_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "CancelOutcome".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    }
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

fn validate_no_keywords(call: &ExprCall, ctx: &mut LowerCtx, api_name: &str) -> Option<()> {
    if call.arguments.keywords.is_empty() {
        return Some(());
    }
    expression_diagnostics::call_unexpected_keyword(
        ctx,
        format!("{api_name} does not accept keyword arguments"),
        first_call_keyword_range(call),
    );
    None
}

fn validate_exact_arg_count(
    call: &ExprCall,
    ctx: &mut LowerCtx,
    api_name: &str,
    expected: usize,
) -> Option<()> {
    if call.arguments.args.len() == expected {
        return Some(());
    }
    expression_diagnostics::call_wrong_positional_count(
        ctx,
        format!("{api_name} takes exactly {expected} positional arguments"),
        call_arity_range(call),
    );
    None
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
