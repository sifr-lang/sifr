use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

use crate::hir_nodes::HirExpr;

use super::LowerCtx;

const EXACT_INT_DIVISION_REQUIRES_HANDLING: &str =
    "exact integer division or modulo requires handling Result[int, DivisionError] unless the divisor is proven non-zero";

pub(super) fn exact_int_division_requires_handling(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &mut LowerCtx,
    range: TextRange,
) -> bool {
    if ctx.is_stdlib_lowering()
        || !is_exact_int_division_or_modulo(op)
        || !is_exact_int_like(left.ty())
        || !is_exact_int_like(right.ty())
        || is_proven_nonzero_integer_expr(right, ctx)
    {
        return false;
    }
    emit_exact_int_division_requires_handling(ctx, range);
    true
}

pub(super) fn exact_int_augassign_requires_handling(
    target_ty: &Type,
    base_op: &str,
    value: &HirExpr,
    ctx: &mut LowerCtx,
    range: TextRange,
) -> bool {
    if ctx.is_stdlib_lowering()
        || !is_exact_int_division_or_modulo(base_op)
        || !is_exact_int_like(target_ty)
        || !is_exact_int_like(value.ty())
        || is_proven_nonzero_integer_expr(value, ctx)
    {
        return false;
    }
    emit_exact_int_division_requires_handling(ctx, range);
    true
}

fn emit_exact_int_division_requires_handling(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING,
        EXACT_INT_DIVISION_REQUIRES_HANDLING.to_string(),
        range,
    );
}

fn is_exact_int_division_or_modulo(op: &str) -> bool {
    matches!(op, "//" | "%")
}

fn is_exact_int_like(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::LiteralInt(_))
}

fn is_proven_nonzero_integer_expr(expr: &HirExpr, ctx: &LowerCtx) -> bool {
    match expr {
        HirExpr::IntLiteral(value) => *value != 0,
        HirExpr::LargeIntLiteral(value) => value != "0",
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            is_proven_nonzero_integer_expr(operand, ctx)
        }
        HirExpr::Name { name, .. } => ctx.is_proven_nonzero_integer_binding(name),
        _ => false,
    }
}
