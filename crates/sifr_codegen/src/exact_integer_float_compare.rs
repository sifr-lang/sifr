use crate::{RustExpr, Type};

pub(crate) fn lower_exact_integer_float_compare(
    left_ty: &Type,
    right_ty: &Type,
    op: &str,
    left: RustExpr,
    right: RustExpr,
) -> Option<RustExpr> {
    let left_is_float = matches!(left_ty.resolve_alias(), Type::Float);
    let right_is_float = matches!(right_ty.resolve_alias(), Type::Float);
    let left_is_integer = is_integer(left_ty);
    let right_is_integer = is_integer(right_ty);
    let (integer_ty, integer, float, integer_is_left) = match (
        left_is_integer,
        right_is_integer,
        left_is_float,
        right_is_float,
    ) {
        (true, false, false, true) => (left_ty, left, right, true),
        (false, true, true, false) => (right_ty, right, left, false),
        _ => return None,
    };
    let integer = if matches!(integer_ty.resolve_alias(), Type::FixedInt(_)) {
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "SifrInt".to_string(),
                "from".to_string(),
            ])),
            args: vec![integer],
        }
    } else {
        integer
    };
    let exact_op = if integer_is_left {
        op
    } else {
        reverse_comparison(op)?
    };
    let (method, negate) = match exact_op {
        "==" => ("eq_f64", false),
        "!=" => ("eq_f64", true),
        "<" => ("lt_f64", false),
        "<=" => ("le_f64", false),
        ">" => ("gt_f64", false),
        ">=" => ("ge_f64", false),
        _ => return None,
    };
    let comparison = RustExpr::MethodCall {
        receiver: Box::new(integer),
        method: method.to_string(),
        args: vec![float],
    };
    Some(if negate {
        RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(comparison),
        }
    } else {
        comparison
    })
}

fn is_integer(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Int | Type::LiteralInt(_) | Type::FixedInt(_)
    )
}

fn reverse_comparison(op: &str) -> Option<&'static str> {
    match op {
        "==" => Some("=="),
        "!=" => Some("!="),
        "<" => Some(">"),
        "<=" => Some(">="),
        ">" => Some("<"),
        ">=" => Some("<="),
        _ => None,
    }
}
