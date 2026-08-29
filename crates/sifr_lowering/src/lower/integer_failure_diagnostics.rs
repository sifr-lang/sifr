use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

use crate::hir_nodes::HirExpr;

use super::LowerCtx;

const EXACT_INT_DIVISION_REQUIRES_HANDLING: &str = "integer division, modulo, exponentiation, shift, or range step requires handling a typed integer failure unless the compiler can prove this operation is safe";

pub(in crate::lower) fn exact_int_division_requires_handling(
    _left: &HirExpr,
    op: &str,
    _right: &HirExpr,
    _ctx: &mut LowerCtx,
    _range: TextRange,
) -> bool {
    if !is_integer_failure_op(op) {
        return false;
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
    if base_op == "/" && is_exact_int_like(target_ty) && is_exact_int_like(value.ty()) {
        emit_exact_int_division_requires_handling(ctx, range);
        return true;
    }
    if !is_integer_failure_op(base_op) {
        return false;
    }
    if involves_fixed_width_integer(target_ty, value.ty()) {
        emit_exact_int_division_requires_handling(ctx, range);
        return true;
    }
    if (is_integer_exponentiation(base_op) || is_integer_shift(base_op))
        && is_exact_int_like(target_ty)
        && is_exact_int_like(value.ty())
        && !super::expression_operators::statically_safe_bounded_integer_augassign(base_op, value)
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

fn is_integer_shift(op: &str) -> bool {
    matches!(op, "<<" | ">>")
}

fn is_integer_failure_op(op: &str) -> bool {
    is_exact_int_division_or_modulo(op) || is_integer_exponentiation(op) || is_integer_shift(op)
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

pub(in crate::lower) fn is_proven_nonzero_integer_expr(expr: &HirExpr, ctx: &LowerCtx) -> bool {
    if let Some(value) = super::expression_operators::proven_exact_integer_value(expr, ctx) {
        return value != num_bigint::BigInt::from(0_u8);
    }
    match expr {
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            is_proven_nonzero_integer_expr(operand, ctx)
        }
        HirExpr::Name { name, .. } => ctx.is_proven_nonzero_integer_binding(name),
        HirExpr::Call { func, args, .. } if func == "len" && args.len() == 1 => {
            super::sequence_guards::hir_sequence_guard_target_name(&args[0])
                .is_some_and(|name| ctx.min_length_guard(&name) > 0)
        }
        _ => false,
    }
}
