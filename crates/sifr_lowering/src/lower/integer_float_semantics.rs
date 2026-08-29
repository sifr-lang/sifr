use super::expression_operators::{
    builtin_error_type, exact_integer_expr_is_proven_float_representable,
    is_exact_or_fixed_int_like,
};
use super::{HirExpr, LowerCtx, Type};
use sifr_type_system::make_union;

pub(super) fn mixed_float_integer_arithmetic_result_type(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if !matches!(op, "+" | "-" | "*" | "/" | "//" | "%" | "**") {
        return None;
    }
    let integer = match (left.ty().resolve_alias(), right.ty().resolve_alias()) {
        (Type::Float, right_ty) if is_exact_or_fixed_int_like(right_ty) => right,
        (left_ty, Type::Float) if is_exact_or_fixed_int_like(left_ty) => left,
        _ => return None,
    };
    if exact_integer_expr_is_proven_float_representable(integer, ctx) {
        return Some(Type::Float);
    }
    Some(Type::Result(
        Box::new(Type::Float),
        Box::new(make_union(vec![
            builtin_error_type(ctx, "FloatOverflowError", "OverflowError", vec![]),
            builtin_error_type(ctx, "FloatPrecisionLossError", "OverflowError", vec![]),
        ])),
    ))
}
