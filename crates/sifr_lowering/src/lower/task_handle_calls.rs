use super::LowerCtx;
use super::expression_diagnostics;
use super::task_scope_calls::mark_task_handle_observed;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;

pub(in crate::lower) fn is_task_handle_type(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Task(_, _) | Type::BlockingTask(_, _)
    )
}

pub(in crate::lower) fn lower_task_handle_method_call(
    object: HirExpr,
    method_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if method_name != "join" && method_name != "cancel" && method_name != "cancel_and_join" {
        return None;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            format!("Task.{method_name}() does not accept keyword arguments"),
            first_call_keyword_range(call),
        );
        return None;
    }
    if !call.arguments.args.is_empty() {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            format!("Task.{method_name}() takes no arguments"),
            call_arity_range(call),
        );
        return None;
    }
    let (ok_ty, err_ty) = match object.ty().resolve_alias() {
        Type::Task(ok_ty, err_ty) | Type::BlockingTask(ok_ty, err_ty) => (ok_ty, err_ty),
        _ => return None,
    };
    let receiver_convention =
        super::mutating_methods::receiver_convention_for_non_class_method(object.ty(), method_name);
    if method_name == "cancel" {
        return Some(HirExpr::MethodCall {
            object: Box::new(object),
            method: method_name.to_string(),
            args: vec![],
            receiver_convention: Some(receiver_convention),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: Some(super::method_call_metadata::source_method_call(call)),
            ty: Type::None,
        });
    }
    let result_ok_ty = ok_ty.clone();
    let result_err_ty = err_ty.clone();
    if let HirExpr::Name { name, .. } = &object {
        mark_task_handle_observed(name, ctx);
        ctx.mark_moved_with_flow(name);
    }
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name.to_string(),
        args: vec![],
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(super::method_call_metadata::source_method_call(call)),
        ty: Type::Awaitable(Box::new(Type::TaskResult(result_ok_ty, result_err_ty))),
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
