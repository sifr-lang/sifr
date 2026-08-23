use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

use crate::hir_nodes::HirExpr;

use super::LowerCtx;

const EXACT_INT_DIVISION_REQUIRES_HANDLING: &str = "integer division, modulo, or exponentiation requires handling a typed integer failure unless the compiler can prove this operation is safe";

pub(in crate::lower) fn exact_int_division_requires_handling(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &mut LowerCtx,
    range: TextRange,
) -> bool {
    if ctx.is_stdlib_lowering() || !is_integer_failure_op(op) {
        return false;
    }
    if involves_fixed_width_integer(left.ty(), right.ty()) {
        emit_exact_int_division_requires_handling(ctx, range);
        return true;
    }
    if is_integer_exponentiation(op)
        && is_exact_int_like(left.ty())
        && is_exact_int_like(right.ty())
        && !is_proven_nonnegative_integer_expr(right)
    {
        emit_exact_int_division_requires_handling(ctx, range);
        return true;
    }
    false
}

pub(in crate::lower) fn exact_int_augassign_requires_handling(
    target_ty: &Type,
    base_op: &str,
    value: &HirExpr,
    ctx: &mut LowerCtx,
    range: TextRange,
) -> bool {
    if ctx.is_stdlib_lowering() || !is_integer_failure_op(base_op) {
        return false;
    }
    if involves_fixed_width_integer(target_ty, value.ty()) {
        emit_exact_int_division_requires_handling(ctx, range);
        return true;
    }
    if is_integer_exponentiation(base_op)
        && is_exact_int_like(target_ty)
        && is_exact_int_like(value.ty())
        && !is_proven_nonnegative_integer_expr(value)
    {
        emit_exact_int_division_requires_handling(ctx, range);
        return true;
    }
    if is_exact_int_like(target_ty)
        && is_exact_int_like(value.ty())
        && is_exact_int_division_or_modulo(base_op)
        && !is_proven_nonzero_integer_expr(value, ctx)
    {
        emit_exact_int_division_requires_handling(ctx, range);
        return true;
    }
    false
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

fn is_integer_exponentiation(op: &str) -> bool {
    op == "**"
}

fn is_integer_failure_op(op: &str) -> bool {
    is_exact_int_division_or_modulo(op) || is_integer_exponentiation(op)
}

fn is_exact_int_like(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::LiteralInt(_))
}

fn involves_fixed_width_integer(left: &Type, right: &Type) -> bool {
    is_exact_or_fixed_integer_like(left)
        && is_exact_or_fixed_integer_like(right)
        && (is_fixed_width_integer(left) || is_fixed_width_integer(right))
}

fn is_exact_or_fixed_integer_like(ty: &Type) -> bool {
    is_exact_int_like(ty) || is_fixed_width_integer(ty)
}

fn is_fixed_width_integer(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::FixedInt(_))
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

fn is_proven_nonnegative_integer_expr(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::IntLiteral(value) => *value >= 0,
        HirExpr::LargeIntLiteral(value) => !value.starts_with('-'),
        _ => false,
    }
}
