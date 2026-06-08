use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::ownership_diagnostics;
use super::task_context_keywords::validate_reserved_task_context_keyword;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::{ExprAttribute, ExprCall};
use sifr_type_system::Type;
use std::collections::HashSet;

pub(in crate::lower) fn is_task_scope_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if name == "TaskScope" || name == "TaskGroup")
}

pub(in crate::lower) fn is_task_group_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if name == "TaskGroup")
}

pub(in crate::lower) fn lower_task_scope_spawn_call(
    object: HirExpr,
    _attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    lower_task_scope_spawn_from_object(object, call, ctx)
}

pub(in crate::lower) fn lower_task_scope_spawn_from_object(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    lower_task_scope_spawn_from_object_impl(object, call, ctx, "scope.spawn()", false)
}

pub(in crate::lower) fn lower_task_scope_spawn_from_object_allowing_reserved_ctx(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    lower_task_scope_spawn_from_object_impl(object, call, ctx, "task.spawn_scoped()", true)
}

fn lower_task_scope_spawn_from_object_impl(
    object: HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
    callable_name: &str,
    allow_reserved_ctx: bool,
) -> Option<HirExpr> {
    if !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            format!("{callable_name} is only valid inside async functions"),
            call.range(),
        );
        return None;
    }
    validate_spawn_keywords(call, ctx, callable_name, allow_reserved_ctx)?;
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            format!("{callable_name} takes exactly one coroutine argument"),
            call_arity_range(call),
        );
        return None;
    }

    let coroutine = lower_expr(&call.arguments.args[0], ctx)?;
    let Type::Coroutine(ok_ty, err_ty) = coroutine.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{callable_name} requires a coroutine argument, got '{}'",
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
            format!("{callable_name} requires a direct coroutine call in v1"),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if let Some(name) = borrowed_task_boundary_argument(args, ctx) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "{callable_name} cannot move borrowed parameter '{name}' across a task boundary; pass an owned value or clone it before spawning"
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    if let Some(non_send) = non_send_task_boundary_argument(args) {
        ownership_diagnostics::non_send_task_capture(
            ctx,
            &non_send.value,
            &non_send.ty,
            &non_send.reason,
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

fn validate_spawn_keywords(
    call: &ExprCall,
    ctx: &mut LowerCtx,
    callable_name: &str,
    allow_reserved_ctx: bool,
) -> Option<()> {
    if call.arguments.keywords.is_empty() {
        return Some(());
    }
    if !allow_reserved_ctx {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            format!("{callable_name} does not accept keyword arguments"),
            first_call_keyword_range(call),
        );
        return None;
    }
    validate_reserved_task_context_keyword(ctx, call, callable_name)
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

struct NonSendTaskBoundaryArgument {
    value: String,
    ty: String,
    reason: String,
}

fn non_send_task_boundary_argument(args: &[HirExpr]) -> Option<NonSendTaskBoundaryArgument> {
    args.iter()
        .find_map(non_send_task_boundary_argument_in_expr)
}

fn non_send_task_boundary_argument_in_expr(expr: &HirExpr) -> Option<NonSendTaskBoundaryArgument> {
    let reason = non_send_reason(expr.ty())?;
    Some(NonSendTaskBoundaryArgument {
        value: task_boundary_expr_label(expr),
        ty: expr.ty().display_name(),
        reason,
    })
}

pub(in crate::lower) fn validate_channel_send_element(
    object_ty: &Type,
    method_name: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    call: &ExprCall,
    ctx: &mut LowerCtx,
) {
    if method_name != "send" || !is_channel_sender_type(object_ty) {
        return;
    }
    let Some(arg) = args.first() else {
        return;
    };
    let Some(reason) = non_send_reason(arg.ty()) else {
        return;
    };
    ownership_diagnostics::non_send_channel_element(
        ctx,
        &channel_send_arg_label(arg),
        &arg.ty().display_name(),
        &reason,
        arg_ranges.first().copied().unwrap_or_else(|| call.range()),
    );
}

pub(in crate::lower) fn validate_shared_constructor(
    func_name: &str,
    args: &[HirExpr],
    arg_ranges: &[Option<TextRange>],
    call: &ExprCall,
    ctx: &mut LowerCtx,
) {
    if public_type_name(func_name) != "Shared" {
        return;
    }
    let Some(arg) = args.first() else {
        return;
    };
    let Some(reason) = non_share_safe_reason(arg.ty()) else {
        return;
    };
    ownership_diagnostics::non_share_safe_shared_value(
        ctx,
        &channel_send_arg_label(arg),
        &arg.ty().display_name(),
        &reason,
        arg_ranges
            .first()
            .copied()
            .flatten()
            .unwrap_or_else(|| call.range()),
    );
}

fn task_boundary_expr_label(expr: &HirExpr) -> String {
    match expr {
        HirExpr::Name { name, .. } => name.clone(),
        HirExpr::FieldAccess { field, .. } => format!("field `{field}`"),
        _ => "spawn argument".to_string(),
    }
}

fn channel_send_arg_label(expr: &HirExpr) -> String {
    match expr {
        HirExpr::Name { name, .. } => name.clone(),
        HirExpr::FieldAccess { field, .. } => format!("field `{field}`"),
        _ => "value".to_string(),
    }
}

fn is_channel_sender_type(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Class { name, .. } if public_type_name(name) == "ChannelSender"
    )
}

pub(in crate::lower) fn non_send_reason(ty: &Type) -> Option<String> {
    non_send_reason_inner(ty.resolve_alias(), &mut HashSet::new())
}

fn non_share_safe_reason(ty: &Type) -> Option<String> {
    non_share_safe_reason_inner(ty.resolve_alias(), &mut HashSet::new())
}

fn non_share_safe_reason_inner(ty: &Type, visiting: &mut HashSet<String>) -> Option<String> {
    match ty {
        Type::List(_) => {
            Some("list values are mutable and require explicit synchronization".to_string())
        }
        Type::Dict(_, _) => {
            Some("dict values are mutable and require explicit synchronization".to_string())
        }
        Type::Set(_) => {
            Some("set values are mutable and require explicit synchronization".to_string())
        }
        Type::Class {
            name,
            fields,
            parent_class,
            ..
        } => {
            if is_share_safe_sync_wrapper(name) {
                return None;
            }
            if let Some(label) = process_owned_handle_type_label_by_name(name) {
                return Some(format!(
                    "`{}` is a {label} and must stay in its owning task",
                    public_type_name(name)
                ));
            }
            if class_has_non_send_marker(name, parent_class.as_deref()) {
                return Some(format!("`{name}` inherits the `NonSend` marker"));
            }
            if !visiting.insert(name.clone()) {
                return None;
            }
            let field_reason = fields.iter().find_map(|(field, field_ty)| {
                non_share_safe_reason_inner(field_ty.resolve_alias(), visiting)
                    .map(|reason| format!("field `{field}` is not share-safe: {reason}"))
            });
            visiting.remove(name);
            field_reason.or_else(|| {
                Some(format!(
                    "`{}` is a mutable class without an explicit synchronization wrapper",
                    public_type_name(name)
                ))
            })
        }
        Type::Tuple(elems) | Type::Union(elems) | Type::Intersection(elems) => elems
            .iter()
            .find_map(|elem| non_share_safe_reason_inner(elem.resolve_alias(), visiting)),
        Type::Iterable(elem)
        | Type::Iterator(elem)
        | Type::Awaitable(elem)
        | Type::Failure(elem)
        | Type::TimeoutResult(elem) => non_share_safe_reason_inner(elem.resolve_alias(), visiting),
        Type::Result(key, value)
        | Type::Select2(key, value)
        | Type::BlockingTask(key, value)
        | Type::Task(key, value)
        | Type::TaskResult(key, value)
        | Type::Coroutine(key, value)
        | Type::AsyncIterator(key, value)
        | Type::AsyncGenerator(key, value) => {
            non_share_safe_reason_inner(key.resolve_alias(), visiting)
                .or_else(|| non_share_safe_reason_inner(value.resolve_alias(), visiting))
        }
        Type::Alias { body, .. } => non_share_safe_reason_inner(body.resolve_alias(), visiting),
        Type::Newtype { inner, .. } => non_share_safe_reason_inner(inner.resolve_alias(), visiting),
        other => non_send_reason(other),
    }
}

fn non_send_reason_inner(ty: &Type, visiting: &mut HashSet<String>) -> Option<String> {
    match ty {
        Type::Class {
            name,
            fields,
            parent_class,
            ..
        } => {
            if let Some(label) = sync_guard_type_label_by_name(name) {
                return Some(format!("`{}` is a {label}", public_type_name(name)));
            }
            if let Some(label) = process_owned_handle_type_label_by_name(name) {
                return Some(format!(
                    "`{}` is a {label} and must stay in its owning task",
                    public_type_name(name)
                ));
            }
            if class_has_non_send_marker(name, parent_class.as_deref()) {
                return Some(format!("`{name}` inherits the `NonSend` marker"));
            }
            if !visiting.insert(name.clone()) {
                return None;
            }
            let found = fields.iter().find_map(|(field, field_ty)| {
                non_send_reason_inner(field_ty.resolve_alias(), visiting)
                    .map(|reason| format!("field `{field}` is not sendable: {reason}"))
            });
            visiting.remove(name);
            found
        }
        Type::List(elem)
        | Type::Set(elem)
        | Type::Iterable(elem)
        | Type::Iterator(elem)
        | Type::Awaitable(elem)
        | Type::Failure(elem)
        | Type::TimeoutResult(elem) => non_send_reason_inner(elem.resolve_alias(), visiting),
        Type::Dict(key, value)
        | Type::Result(key, value)
        | Type::Select2(key, value)
        | Type::BlockingTask(key, value)
        | Type::Task(key, value)
        | Type::TaskResult(key, value)
        | Type::Coroutine(key, value)
        | Type::AsyncIterator(key, value)
        | Type::AsyncGenerator(key, value) => non_send_reason_inner(key.resolve_alias(), visiting)
            .or_else(|| non_send_reason_inner(value.resolve_alias(), visiting)),
        Type::Tuple(elems) | Type::Union(elems) | Type::Intersection(elems) => elems
            .iter()
            .find_map(|elem| non_send_reason_inner(elem.resolve_alias(), visiting)),
        Type::Alias { body, .. } => non_send_reason_inner(body.resolve_alias(), visiting),
        Type::Newtype { inner, .. } => non_send_reason_inner(inner.resolve_alias(), visiting),
        Type::Callable(params, _, ret) => params
            .iter()
            .find_map(|param| non_send_reason_inner(param.resolve_alias(), visiting))
            .or_else(|| non_send_reason_inner(ret.resolve_alias(), visiting)),
        _ => None,
    }
}

fn class_has_non_send_marker(name: &str, parent_chain: Option<&str>) -> bool {
    name == "NonSend"
        || parent_chain.is_some_and(|parents| parents.split('|').any(|parent| parent == "NonSend"))
}

pub(in crate::lower) fn sync_guard_type_label(ty: &Type) -> Option<&'static str> {
    let Type::Class { name, .. } = ty.resolve_alias() else {
        return None;
    };
    sync_guard_type_label_by_name(name)
}

fn sync_guard_type_label_by_name(name: &str) -> Option<&'static str> {
    matches!(
        public_type_name(name),
        "LockGuard" | "RwLockReadGuard" | "RwLockWriteGuard"
    )
    .then_some("lock guard")
    .or_else(|| (public_type_name(name) == "SemaphorePermit").then_some("semaphore permit"))
}

fn is_share_safe_sync_wrapper(name: &str) -> bool {
    matches!(
        public_type_name(name),
        "Shared" | "Lock" | "RwLock" | "Semaphore" | "Notify" | "ChannelSender" | "ChannelReceiver"
    )
}

fn process_owned_handle_type_label_by_name(name: &str) -> Option<&'static str> {
    match public_type_name(name) {
        "Child" | "AsyncChild" => Some("process child handle"),
        "PipeReader" | "PipeWriter" | "AsyncPipeReader" | "AsyncPipeWriter" => {
            Some("process pipe handle")
        }
        _ => None,
    }
}

pub(in crate::lower) fn public_type_name(name: &str) -> &str {
    name.strip_prefix("__compat_sifr_sync_")
        .or_else(|| name.strip_prefix("__compat_sifr_process_"))
        .unwrap_or(name)
}

pub(in crate::lower) fn task_group_spawn_owner(expr: &HirExpr) -> Option<String> {
    let HirExpr::MethodCall { object, method, .. } = expr else {
        return None;
    };
    if !matches!(
        method.as_str(),
        "__sifr_spawn_infallible"
            | "__sifr_spawn_result"
            | "__sifr_scope_spawn_blocking_infallible"
            | "__sifr_scope_spawn_blocking_result"
            | "__sifr_scope_spawn_cpu_infallible"
            | "__sifr_scope_spawn_cpu_result"
    ) {
        return None;
    }
    let HirExpr::Name { name, ty } = object.as_ref() else {
        return None;
    };
    is_task_group_type(ty).then(|| name.clone())
}

pub(in crate::lower) fn mark_task_handle_observed(name: &str, ctx: &mut LowerCtx) {
    if let Some(group_name) = ctx.task_handle_group_owners.get(name).cloned() {
        ctx.task_groups_not_proven_open.insert(group_name);
    }
}

pub(in crate::lower) fn enforce_task_group_is_open(
    object: &HirExpr,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<()> {
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

pub(in crate::lower) fn enforce_task_group_error_type(
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
