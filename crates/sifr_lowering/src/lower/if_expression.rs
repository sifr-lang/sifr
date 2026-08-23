use super::LowerCtx;
use super::expressions::lower_expr;
use super::sequence_guard_detection::detect_true_sequence_guards;
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::ExprIf;

pub(in crate::lower) fn lower_if_expr(if_expr: &ExprIf, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let condition = lower_expr(&if_expr.test, ctx)?;

    let saved_moved = ctx.scope.save_moved_state();
    let saved_sequence_guards = ctx.save_sequence_guards();

    for guard in detect_true_sequence_guards(&if_expr.test, ctx) {
        ctx.add_sequence_guard(guard);
    }
    let then_expr = lower_expr(&if_expr.body, ctx)?;
    let then_moved = ctx.scope.save_moved_state();

    ctx.scope.restore_moved_state(&saved_moved);
    ctx.restore_sequence_guards(&saved_sequence_guards);

    let else_expr = lower_expr(&if_expr.orelse, ctx)?;
    let else_moved = ctx.scope.save_moved_state();

    ctx.scope.restore_moved_state(&saved_moved);
    ctx.restore_sequence_guards(&saved_sequence_guards);

    for (name, was_moved) in then_moved.iter().chain(else_moved.iter()) {
        if *was_moved {
            ctx.mark_moved_with_flow(name);
        }
    }

    let then_ty = then_expr.ty().clone();
    let else_ty = else_expr.ty().clone();

    if !then_ty.is_assignable_to(&else_ty) && !else_ty.is_assignable_to(&then_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_IF_BRANCH_MISMATCH,
            format!(
                "if expression branches have incompatible types: '{}' and '{}'",
                then_ty.display_name(),
                else_ty.display_name()
            ),
            if_expr.orelse.range(),
        );
        return None;
    }

    Some(HirExpr::IfExpr {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
        ty: then_ty,
    })
}
