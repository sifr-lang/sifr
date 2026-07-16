use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::offload_worker_captures::validate_offload_worker_captures;
use super::task_scope_calls::non_send_reason;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::ExprCall;
use sifr_type_system::{FunctionType, OwnershipKind, Type};

pub(in crate::lower) fn lower_parallel_imported_call(
    func_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    if ctx.parallel_map_bindings.contains(func_name) {
        return Some(lower_parallel_map_like_call(
            "__sifr_parallel_map",
            "parallel.map()",
            false,
            call,
            ctx,
        ));
    }
    if ctx.parallel_try_map_bindings.contains(func_name) {
        return Some(lower_parallel_map_like_call(
            "__sifr_parallel_try_map",
            "parallel.try_map()",
            true,
            call,
            ctx,
        ));
    }
    None
}

pub(in crate::lower) fn lower_parallel_pool_method_call(
    object: &HirExpr,
    method_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    if !is_pool_type(object.ty()) {
        return None;
    }
    match method_name {
        "map" => Some(lower_parallel_pool_map_like_call(
            object,
            "__sifr_pool_map",
            "Pool.map()",
            false,
            call,
            ctx,
        )),
        "try_map" => Some(lower_parallel_pool_map_like_call(
            object,
            "__sifr_pool_try_map",
            "Pool.try_map()",
            true,
            call,
            ctx,
        )),
        _ => None,
    }
}

fn lower_parallel_pool_map_like_call(
    object: &HirExpr,
    helper: &str,
    api_name: &str,
    expects_result: bool,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let (items, worker, ok_ty, _err_ty) =
        validate_parallel_map_like_call(api_name, expects_result, call, ctx)?;
    let mut args = vec![object.clone(), items, worker];
    let ty = parallel_result_type(ok_ty, expects_result, ctx);
    Some(HirExpr::Call {
        func: helper.to_string(),
        args: std::mem::take(&mut args),
        ty,
    })
}

fn lower_parallel_map_like_call(
    helper: &str,
    api_name: &str,
    expects_result: bool,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let (items, worker, ok_ty, _err_ty) =
        validate_parallel_map_like_call(api_name, expects_result, call, ctx)?;
    let ty = parallel_result_type(ok_ty, expects_result, ctx);
    Some(HirExpr::Call {
        func: helper.to_string(),
        args: vec![items, worker],
        ty,
    })
}

fn validate_parallel_map_like_call(
    api_name: &str,
    expects_result: bool,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<(HirExpr, HirExpr, Type, Type)> {
    if ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::ASYNC_DIRECT_CPU_HEAVY_CALL,
            format!("{api_name} is CPU-heavy synchronous work; call it from sync code or offload the caller with task.spawn_cpu()"),
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
    if call.arguments.args.len() != 2 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            format!("{api_name} takes exactly an owned list and a named sync function"),
            call_arity_range(call),
        );
        return None;
    }

    let items = lower_expr(&call.arguments.args[0], ctx)?;
    let Type::List(item_ty) = items.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} first argument must be list[T], got '{}'",
                items.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if let Some(reason) = non_send_reason(item_ty.as_ref()) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} cannot process non-send item type '{}': {reason}",
                item_ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }

    let worker = lower_expr(&call.arguments.args[1], ctx)?;
    validate_offload_worker_captures(api_name, &worker, call.arguments.args[1].range(), ctx)?;
    let Type::Function(ft) = worker.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} requires a named sync function argument, got '{}'",
                worker.ty().display_name()
            ),
            call.arguments.args[1].range(),
        );
        return None;
    };
    let (param_ty, ok_ty, err_ty) =
        validate_parallel_worker_signature(api_name, expects_result, ft, call, ctx)?;
    if !item_ty.is_assignable_to(&param_ty) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} worker parameter expects '{}', got list item '{}'",
                param_ty.display_name(),
                item_ty.display_name()
            ),
            call.arguments.args[1].range(),
        );
        return None;
    }
    if let Some(reason) = non_send_reason(&ok_ty) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{api_name} cannot return non-send value type '{}': {reason}",
                ok_ty.display_name()
            ),
            call.arguments.args[1].range(),
        );
        return None;
    }
    if expects_result && !matches!(err_ty.resolve_alias(), Type::Never) {
        if let Some(reason) = non_send_reason(&err_ty) {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "{api_name} cannot return non-send error type '{}': {reason}",
                    err_ty.display_name()
                ),
                call.arguments.args[1].range(),
            );
            return None;
        }
    }

    if let HirExpr::Name { name, ty } = &items {
        if ty.ownership() == OwnershipKind::Move {
            ctx.mark_moved_with_flow(name);
        }
    }

    Some((items, worker, ok_ty, err_ty))
}

fn validate_parallel_worker_signature(
    api_name: &str,
    expects_result: bool,
    ft: &FunctionType,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<(Type, Type, Type)> {
    if ft.params.len() != 1 {
        expression_diagnostics::type_mismatch(
            ctx,
            format!("{api_name} worker must take exactly one argument"),
            call.arguments.args[1].range(),
        );
        return None;
    }
    let param_ty = ft.params[0].1.clone();
    match (expects_result, ft.return_type.resolve_alias()) {
        (false, Type::Result(_, _)) => {
            expression_diagnostics::type_mismatch(
                ctx,
                format!("{api_name} worker must return a plain value; use try_map for Result-returning workers"),
                call.arguments.args[1].range(),
            );
            None
        }
        (false, other) => Some((param_ty, other.clone(), Type::Never)),
        (true, Type::Result(ok, err)) => {
            Some((param_ty, ok.as_ref().clone(), err.as_ref().clone()))
        }
        (true, other) => {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "{api_name} worker must return Result[U, E], got '{}'",
                    other.display_name()
                ),
                call.arguments.args[1].range(),
            );
            None
        }
    }
}

fn parallel_result_type(ok_ty: Type, expects_result: bool, ctx: &LowerCtx) -> Type {
    let values = Type::List(Box::new(ok_ty));
    let error_ty = if expects_result {
        parallel_error_type("WorkerError", ctx)
    } else {
        parallel_error_type("WorkerRuntimeError", ctx)
    };
    Type::Result(Box::new(values), Box::new(error_ty))
}

fn parallel_error_type(name: &str, ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get(name)
        .cloned()
        .unwrap_or_else(|| Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        })
}

fn is_pool_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if name == "Pool")
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
