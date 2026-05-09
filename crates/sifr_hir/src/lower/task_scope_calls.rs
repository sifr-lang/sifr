use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::{ExprAttribute, ExprCall};
use sifr_type_system::Type;

pub(super) fn is_task_scope_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if name == "TaskScope" || name == "TaskGroup")
}

pub(super) fn lower_task_scope_spawn_call(
    object: HirExpr,
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            "scope.spawn() is only valid inside async functions".to_string(),
            call.range(),
        );
        return None;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "scope.spawn() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "scope.spawn() takes exactly one coroutine argument".to_string(),
            call_arity_range(call),
        );
        return None;
    }

    let coroutine = lower_expr(&call.arguments.args[0], ctx)?;
    let Type::Coroutine(ok_ty, err_ty) = coroutine.ty().resolve_alias() else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "scope.spawn() requires a coroutine argument, got '{}'",
                coroutine.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };
    let task_ok_ty = ok_ty.clone();
    let task_err_ty = err_ty.clone();
    if !matches!(task_err_ty.resolve_alias(), Type::Never) {
        expression_diagnostics::type_mismatch(
            ctx,
            "scope.spawn() currently accepts only infallible coroutine arguments until task error plumbing lands".to_string(),
            call.arguments.args[0].range(),
        );
        return None;
    }
    if !matches!(&coroutine, HirExpr::Call { args, .. } if args.is_empty()) {
        expression_diagnostics::type_mismatch(
            ctx,
            "scope.spawn() currently accepts only no-argument coroutine calls until task-boundary checking lands".to_string(),
            call.arguments.args[0].range(),
        );
        return None;
    }

    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: attr.attr.to_string(),
        args: vec![coroutine],
        ty: Type::Task(task_ok_ty, task_err_ty),
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
