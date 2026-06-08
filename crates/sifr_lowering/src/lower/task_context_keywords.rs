use super::expressions::lower_expr;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprCall};
use sifr_type_system::Type;

pub(in crate::lower) fn lower_task_context_keyword(
    ctx: &mut LowerCtx,
    call: &ExprCall,
    callable_name: &str,
) -> Option<Option<HirExpr>> {
    if call.arguments.keywords.is_empty() {
        return Some(None);
    }
    if call.arguments.keywords.len() != 1 {
        ctx.error_with_code_at(
            DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
            format!("{callable_name} accepts only the reserved ctx keyword"),
            call.arguments.keywords[1].range(),
        );
        return None;
    }
    let keyword = &call.arguments.keywords[0];
    let Some(name) = keyword.arg.as_ref() else {
        ctx.error_with_code_at(
            DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
            format!("{callable_name} does not support unpacked keyword arguments"),
            keyword.range,
        );
        return None;
    };
    if name.as_str() != "ctx" {
        ctx.error_with_code_at(
            DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
            format!("{callable_name} got an unexpected keyword argument '{name}'"),
            keyword.range,
        );
        return None;
    }
    if matches!(keyword.value, Expr::NoneLiteral(_)) {
        return Some(None);
    }
    let context = lower_expr(&keyword.value, ctx)?;
    let is_context = matches!(
        context.ty().resolve_alias(),
        Type::Class { name, fields, .. }
            if name == "Context"
                && fields
                    .iter()
                    .any(|(field_name, field_ty)| field_name == "name" && matches!(field_ty, Type::Str))
    );
    if !is_context {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "{callable_name} ctx must be sifr.task.Context or None, got '{}'",
                context.ty().display_name()
            ),
            keyword.value.range(),
        );
        return None;
    }
    Some(Some(context))
}
