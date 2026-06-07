use super::LowerCtx;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprCall};

pub(in crate::lower) fn validate_reserved_task_context_keyword(
    ctx: &mut LowerCtx,
    call: &ExprCall,
    callable_name: &str,
) -> Option<()> {
    if call.arguments.keywords.is_empty() {
        return Some(());
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
    if !matches!(keyword.value, Expr::NoneLiteral(_)) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "{callable_name} ctx must be None until sifr.task.Context propagation lands in M5"
            ),
            keyword.value.range(),
        );
        return None;
    }
    Some(())
}
