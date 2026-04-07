use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::union_contains_none;

pub(super) fn validate_two_arg_min_max_operands(
    func_name: &str,
    left: &HirExpr,
    right: &HirExpr,
    ctx: &mut LowerCtx,
) -> bool {
    if union_contains_none(left.ty()) || union_contains_none(right.ty()) {
        ctx.error(format!(
            "{func_name}() with 2 arguments does not accept optional operands; got '{}' and '{}' (guard or unwrap first)",
            left.ty().display_name(),
            right.ty().display_name()
        ));
        return false;
    }
    if !left.ty().is_assignable_to(right.ty()) && !right.ty().is_assignable_to(left.ty()) {
        ctx.error(format!(
            "{func_name}() arguments must be comparable and type-compatible; got '{}' and '{}'",
            left.ty().display_name(),
            right.ty().display_name()
        ));
        return false;
    }
    true
}

pub(super) fn validate_variadic_min_max_operands(
    func_name: &str,
    operands: &[HirExpr],
    ctx: &mut LowerCtx,
) -> bool {
    if operands.len() < 2 {
        return true;
    }

    for window in operands.windows(2) {
        let [left, right] = window else {
            continue;
        };
        if !validate_two_arg_min_max_operands(func_name, left, right, ctx) {
            return false;
        }
    }

    true
}
