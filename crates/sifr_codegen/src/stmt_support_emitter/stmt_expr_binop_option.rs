use crate::{RustExpr, Type};

pub(crate) fn unwrap_optional_binop_operand(
    expr: RustExpr,
    ty: &Type,
    binding_name: &str,
) -> Option<RustExpr> {
    if !crate::helpers::is_option_type(ty) {
        return Some(expr);
    }
    let _ = (expr, binding_name);
    None
}

pub(crate) fn unwrap_optional_binop_operands(
    left: RustExpr,
    right: RustExpr,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> Option<(RustExpr, RustExpr)> {
    if crate::helpers::is_option_type(result_ty) {
        return Some((left, right));
    }
    Some((
        unwrap_optional_binop_operand(left, left_ty, "__sifr_left_value")?,
        unwrap_optional_binop_operand(right, right_ty, "__sifr_right_value")?,
    ))
}

pub(crate) fn binop_with_optional_operands(
    left: RustExpr,
    right: RustExpr,
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> Option<RustExpr> {
    let (left, right) = unwrap_optional_binop_operands(left, right, left_ty, right_ty, result_ty)?;
    Some(RustExpr::BinOp {
        left: Box::new(left),
        op: if op == "//" { "/" } else { op }.to_string(),
        right: Box::new(right),
    })
}
