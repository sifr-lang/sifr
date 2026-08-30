use super::{Expr, HirExpr, LowerCtx, Ranged, expression_diagnostics, expression_operators};
use crate::lower::integer_failure_diagnostics;

pub(super) fn lower_proven_nonzero_slice_step(
    step: &Expr,
    ctx: &mut LowerCtx,
) -> Option<Box<HirExpr>> {
    let lowered = super::lower_expr(step, ctx)?;
    if !expression_operators::is_exact_or_fixed_int_like(lowered.ty()) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "slice step must be an integer, got '{}'",
                lowered.ty().display_name()
            ),
            step.range(),
        );
        return None;
    }
    if !integer_failure_diagnostics::is_proven_nonzero_integer_expr(&lowered, ctx) {
        integer_failure_diagnostics::emit_exact_int_requires_handling(ctx, step.range());
        return None;
    }
    Some(Box::new(lowered))
}
