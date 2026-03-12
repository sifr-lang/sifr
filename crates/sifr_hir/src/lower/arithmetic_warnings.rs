use super::LowerCtx;
use crate::HirExpr;

pub(super) fn check_int_overflow_risk(
    op: &str,
    left: &HirExpr,
    right: &HirExpr,
    ctx: &mut LowerCtx,
) {
    let is_left_const = matches!(left, HirExpr::IntLiteral(_));
    let is_right_const = matches!(right, HirExpr::IntLiteral(_));

    match op {
        "**" => {
            if let HirExpr::IntLiteral(exp) = right {
                if *exp > 40 {
                    ctx.warn(format!(
                        "warning: int exponentiation with large exponent ({exp}) may overflow i64; consider using bigint"
                    ));
                }
            } else {
                ctx.warn(
                    "warning: int exponentiation (**) with non-constant exponent may overflow i64 at runtime; consider using bigint".to_string()
                );
            }
        }
        "*" => {
            if !is_left_const && !is_right_const {
                ctx.warn(
                    "warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values".to_string()
                );
            }
        }
        "<<" => {
            if !is_right_const {
                ctx.warn(
                    "warning: int left shift (<<) with non-constant shift amount may overflow i64 at runtime; consider using bigint".to_string()
                );
            } else if let HirExpr::IntLiteral(shift) = right {
                if *shift >= 63 {
                    ctx.warn(format!(
                        "warning: int left shift by {shift} exceeds i64 range; consider using bigint"
                    ));
                }
            }
        }
        _ => {}
    }
}
