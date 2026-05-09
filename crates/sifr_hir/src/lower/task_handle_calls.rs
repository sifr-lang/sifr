use super::expression_diagnostics;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;

pub(super) fn is_task_handle_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Task(_, _))
}

pub(super) fn lower_task_handle_method_call(
    object: HirExpr,
    method_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if method_name != "join" && method_name != "cancel" {
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
    let Type::Task(ok_ty, err_ty) = object.ty().resolve_alias() else {
        return None;
    };
    if method_name == "cancel" {
        return Some(HirExpr::MethodCall {
            object: Box::new(object),
            method: method_name.to_string(),
            args: vec![],
            ty: Type::None,
        });
    }
    let result_ok_ty = ok_ty.clone();
    let result_err_ty = err_ty.clone();
    if let HirExpr::Name { name, .. } = &object {
        ctx.scope.mark_moved(name);
    }
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name.to_string(),
        args: vec![],
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
