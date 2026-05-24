use super::LowerCtx;
use crate::HirExpr;
use ruff_text_size::TextRange;

pub(in crate::lower) fn check_int_overflow_risk(
    op: &str,
    left: &HirExpr,
    right: &HirExpr,
    ctx: &mut LowerCtx,
    range: TextRange,
) {
    let is_left_const = matches!(left, HirExpr::IntLiteral(_));
    let is_right_const = matches!(right, HirExpr::IntLiteral(_));

    match op {
        "**" => {
            if let HirExpr::IntLiteral(exp) = right {
                if *exp > 40 {
                    ctx.warn_arithmetic_overflow_risk("exponentiation", range);
                }
            } else {
                ctx.warn_arithmetic_overflow_risk("exponentiation", range);
            }
        }
        "*" => {
            if !is_left_const && !is_right_const {
                ctx.warn_arithmetic_overflow_risk("multiplication", range);
            }
        }
        "<<" => {
            if !is_right_const {
                ctx.warn_arithmetic_overflow_risk("left shift", range);
            } else if let HirExpr::IntLiteral(shift) = right {
                if *shift >= 63 {
                    ctx.warn_arithmetic_overflow_risk("left shift", range);
                }
            }
        }
        _ => {}
    }
}
