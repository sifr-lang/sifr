use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprAttribute, ExprCall};
use sifr_type_system::Type;

pub(super) enum TaskModuleCall {
    Lowered(HirExpr),
    Rejected,
    NotTaskModuleCall,
}

pub(super) fn lower_task_module_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> TaskModuleCall {
    let Expr::Name(module_name) = attr.value.as_ref() else {
        return TaskModuleCall::NotTaskModuleCall;
    };
    if module_name.id.as_str() != "task" {
        return TaskModuleCall::NotTaskModuleCall;
    }
    if attr.attr.as_str() != "sleep" {
        return TaskModuleCall::NotTaskModuleCall;
    }
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "task.sleep() is only valid inside async functions".to_string(),
            call.range(),
        );
        return TaskModuleCall::Rejected;
    }
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "task.sleep() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return TaskModuleCall::Rejected;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "task.sleep() takes exactly one duration argument".to_string(),
            call_arity_range(call),
        );
        return TaskModuleCall::Rejected;
    }
    let Some(duration) = lower_expr(&call.arguments.args[0], ctx) else {
        return TaskModuleCall::Rejected;
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
        return TaskModuleCall::Rejected;
    }
    TaskModuleCall::Lowered(HirExpr::Call {
        func: "__sifr_task_sleep".to_string(),
        args: vec![duration],
        ty: Type::Awaitable(Box::new(Type::None)),
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
        .first()
        .map_or_else(|| call.func.range(), Ranged::range)
}
