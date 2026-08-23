use super::LowerCtx;
use super::expression_diagnostics;
use super::task_scope_calls::non_send_reason;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

pub(in crate::lower) fn validate_offload_worker_captures(
    api_name: &str,
    worker: &HirExpr,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<()> {
    let HirExpr::Name { name, .. } = worker else {
        return Some(());
    };
    let Some(captures) = ctx.nested_function_captures.get(name).cloned() else {
        return Some(());
    };
    if captures.is_empty() {
        return Some(());
    }
    for (capture_name, capture_ty) in &captures {
        if let Some(reason) = non_send_reason(capture_ty) {
            ctx.error_with_code_at(
                DiagnosticCode::OWN_NON_SEND_TASK_CAPTURE,
                format!(
                    "{api_name} cannot move captured value `{capture_name}` of type `{}` across a worker boundary because {reason}; use an explicit synchronization primitive or pass owned sendable data through the worker input",
                    capture_ty.display_name()
                ),
                range,
            );
            return None;
        }
    }
    expression_diagnostics::type_mismatch(
        ctx,
        format!(
            "{api_name} does not accept nested worker functions with captures yet; use a top-level named sync function and pass data through the owned input"
        ),
        range,
    );
    None
}
