use super::{
    HirExpr, RustExpr, RustType,
    collections_and_comprehensions::{
        is_fixed_width_int_like_simple, is_int_like_simple, is_numeric_simple,
    },
    try_lower_leaf_expr,
};

pub(super) fn try_lower_simple_range_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty, .. } = expr {
        if is_int_like_simple(ty) {
            return Some(RustExpr::Ident(name.clone()));
        }
        return None;
    }
    if matches!(expr, HirExpr::RangeLiteral { .. }) {
        return None;
    }
    try_lower_leaf_expr(expr)
}

pub(super) fn try_lower_mixed_float_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    let lowered = try_lower_simple_binop_operand_expr(expr)?;
    if is_int_like_simple(expr.ty()) {
        return Some(RustExpr::Cast {
            expr: Box::new(lowered),
            ty: RustType::F64,
        });
    }
    Some(lowered)
}

pub(super) fn try_lower_promoted_integer_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    let lowered = match expr {
        HirExpr::Name { name, ty, .. }
            if is_int_like_simple(ty) || is_fixed_width_int_like_simple(ty) =>
        {
            RustExpr::Ident(name.clone())
        }
        _ => try_lower_simple_binop_operand_expr(expr)?,
    };
    if is_fixed_width_int_like_simple(expr.ty()) {
        return Some(RustExpr::Cast {
            expr: Box::new(lowered),
            ty: RustType::I64,
        });
    }
    Some(lowered)
}

pub(super) fn try_lower_simple_binop_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty, .. } = expr {
        if is_numeric_simple(ty) {
            return Some(RustExpr::Ident(name.clone()));
        }
    }
    try_lower_leaf_expr(expr)
}
