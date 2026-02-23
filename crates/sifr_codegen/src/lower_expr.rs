//! Expression lowering scaffolds for the IR migration.

use crate::{CodegenError, RustExpr, RustLiteral, RustType};
use sifr_hir::HirExpr;
use sifr_type_system::Type;

pub fn lower_expr_raw(raw: &str) -> Result<RustExpr, CodegenError> {
    Ok(RustExpr::RawCode(raw.to_string()))
}

/// Lowers leaf expressions that don't require emitter state.
/// This is the first incremental bridge from `emit_expr` string writes
/// to IR + renderer output.
pub fn try_lower_leaf_expr(expr: &HirExpr) -> Option<RustExpr> {
    match expr {
        HirExpr::IntLiteral(v) => Some(RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(RustLiteral::Int(*v))),
            ty: RustType::I64,
        }),
        HirExpr::FloatLiteral(v) => {
            Some(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Float(*v))),
                ty: RustType::F64,
            })
        }
        HirExpr::StringLiteral(s) => Some(RustExpr::Literal(RustLiteral::Str(s.clone()))),
        HirExpr::BoolLiteral(v) => Some(RustExpr::Literal(RustLiteral::Bool(*v))),
        HirExpr::NoneLiteral => Some(RustExpr::Literal(RustLiteral::None)),
        HirExpr::Name { name, ty }
            if is_bool_like_simple(ty)
                || is_numeric_simple(ty)
                || is_string_like_simple(ty)
                || is_enum_like_simple(ty) =>
        {
            Some(RustExpr::Ident(name.clone()))
        }
        HirExpr::EnumVariant { enum_name, variant, .. } => {
            Some(RustExpr::Path(vec![enum_name.clone(), variant.clone()]))
        }
        HirExpr::UnaryOp { op, operand, .. } => {
            match op.as_str() {
                "-" => Some(RustExpr::UnaryOp {
                    op: "-".to_string(),
                    operand: Box::new(try_lower_leaf_expr(operand)?),
                }),
                "+" => Some(try_lower_leaf_expr(operand)?),
                "~" if is_int_like_simple(operand.ty()) => {
                    let lowered_operand = try_lower_leaf_expr(operand).or_else(|| {
                        if let HirExpr::Name { name, .. } = operand.as_ref() {
                            return Some(RustExpr::Ident(name.clone()));
                        }
                        None
                    })?;
                    Some(RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(lowered_operand),
                    })
                }
                "not" if is_bool_like_simple(operand.ty()) => {
                    let lowered_operand = try_lower_leaf_expr(operand).or_else(|| {
                        if let HirExpr::Name { name, .. } = operand.as_ref() {
                            return Some(RustExpr::Ident(name.clone()));
                        }
                        None
                    })?;
                    Some(RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(lowered_operand),
                    })
                }
                "not" if is_option_like_simple(operand.ty()) => {
                    if let HirExpr::Name { name, .. } = operand.as_ref() {
                        return Some(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident(name.clone())),
                            method: "is_none".to_string(),
                            args: vec![],
                        });
                    }
                    None
                }
                _ => None,
            }
        }
        HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } => {
            if !is_safe_simple_binop(op, left.ty(), right.ty(), ty) {
                return None;
            }
            if is_mixed_simple_float_binop(op, left.ty(), right.ty(), ty)
                || is_mixed_simple_float_floor_division_binop(op, left.ty(), right.ty(), ty)
                || is_simple_int_true_division_binop(op, left.ty(), right.ty(), ty)
            {
                return Some(RustExpr::BinOp {
                    left: Box::new(try_lower_mixed_float_operand_expr(left)?),
                    op: normalize_binop_op(op).to_string(),
                    right: Box::new(try_lower_mixed_float_operand_expr(right)?),
                });
            }
            Some(RustExpr::BinOp {
                left: Box::new(try_lower_simple_binop_operand_expr(left)?),
                op: normalize_binop_op(op).to_string(),
                right: Box::new(try_lower_simple_binop_operand_expr(right)?),
            })
        }
        HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } if !ops.is_empty() && ops.len() == comparators.len() => {
            if ops.len() == 1 {
                let right = comparators.first()?;
                if let Some(lowered) = try_lower_option_none_compare_expr(left, &ops[0], right) {
                    return Some(lowered);
                }
                if let Some(lowered) = try_lower_none_identity_compare_expr(left, &ops[0], right) {
                    return Some(lowered);
                }
            }

            let mut lhs_expr = left.as_ref();
            let mut lowered_chain: Option<RustExpr> = None;

            for (idx, op) in ops.iter().enumerate() {
                let rhs_expr = comparators.get(idx)?;
                let normalized_op = normalize_compare_op(op);
                if !is_safe_simple_compare(normalized_op, lhs_expr.ty(), rhs_expr.ty()) {
                    return None;
                }

                let cmp = RustExpr::BinOp {
                    left: Box::new(try_lower_simple_compare_operand_expr(lhs_expr)?),
                    op: normalized_op.to_string(),
                    right: Box::new(try_lower_simple_compare_operand_expr(rhs_expr)?),
                };

                lowered_chain = Some(if let Some(existing) = lowered_chain {
                    RustExpr::BinOp {
                        left: Box::new(existing),
                        op: "&&".to_string(),
                        right: Box::new(cmp),
                    }
                } else {
                    cmp
                });

                lhs_expr = rhs_expr;
            }

            lowered_chain
        }
        HirExpr::BoolOp { op, values, .. } if values.len() >= 2 => {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return None,
            };

            let mut iter = values.iter();
            let mut lowered = try_lower_leaf_expr(iter.next()?)?;
            for value in iter {
                lowered = RustExpr::BinOp {
                    left: Box::new(lowered),
                    op: lowered_op.to_string(),
                    right: Box::new(try_lower_leaf_expr(value)?),
                };
            }
            Some(lowered)
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => Some(RustExpr::If {
            cond: Box::new(try_lower_leaf_expr(condition)?),
            then_expr: Box::new(try_lower_leaf_expr(then_expr)?),
            else_expr: Some(Box::new(try_lower_leaf_expr(else_expr)?)),
        }),
        HirExpr::TupleLiteral { elements, .. } => Some(RustExpr::Tuple(
            elements
                .iter()
                .map(try_lower_leaf_expr)
                .collect::<Option<Vec<_>>>()?,
        )),
        HirExpr::ListLiteral { elements, .. } => Some(RustExpr::Vec(
            elements
                .iter()
                .map(try_lower_leaf_expr)
                .collect::<Option<Vec<_>>>()?,
        )),
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            let lowered_range = RustExpr::Range {
                start: Box::new(try_lower_simple_range_operand_expr(start)?),
                end: Box::new(try_lower_simple_range_operand_expr(end)?),
            };

            if let Some(step_expr) = step.as_ref() {
                Some(RustExpr::MethodCall {
                    receiver: Box::new(lowered_range),
                    method: "step_by".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(try_lower_simple_range_operand_expr(step_expr)?),
                        ty: RustType::Named("usize".to_string()),
                    }],
                })
            } else {
                Some(lowered_range)
            }
        }
        _ => None,
    }
}

fn is_numeric_simple(ty: &Type) -> bool {
    normalize_simple_numeric_scalar_type(ty).is_some()
}

fn is_int_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_numeric_scalar_type(ty), Some("int"))
}

fn is_float_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_numeric_scalar_type(ty), Some("float"))
}

fn is_bool_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_compare_scalar_type(ty), Some("bool"))
}

fn is_string_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_compare_scalar_type(ty), Some("str"))
}

fn resolve_alias_type(ty: &Type) -> &Type {
    match ty {
        Type::Alias(_, inner) => resolve_alias_type(inner),
        _ => ty,
    }
}

fn is_enum_like_simple(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::Enum { .. })
}

fn is_option_like_simple(ty: &Type) -> bool {
    if let Type::Union(members) = resolve_alias_type(ty) {
        let non_none = members.iter().filter(|m| !matches!(m, Type::None)).count();
        let has_none = members.iter().any(|m| matches!(m, Type::None));
        has_none && non_none == 1
    } else {
        false
    }
}

fn normalize_compare_op(op: &str) -> &str {
    match op {
        "is" => "==",
        "is not" => "!=",
        _ => op,
    }
}

fn normalize_binop_op(op: &str) -> &str {
    match op {
        "//" => "/",
        _ => op,
    }
}

fn is_mixed_simple_float_binop(op: &str, left_ty: &Type, right_ty: &Type, result_ty: &Type) -> bool {
    if !matches!(op, "/" | "+" | "-" | "*" | "%") {
        return false;
    }
    if !is_float_like_simple(result_ty) {
        return false;
    }
    (is_int_like_simple(left_ty) && is_float_like_simple(right_ty))
        || (is_float_like_simple(left_ty) && is_int_like_simple(right_ty))
}

fn is_mixed_simple_float_floor_division_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    op == "//"
        && is_float_like_simple(result_ty)
        && ((is_int_like_simple(left_ty) && is_float_like_simple(right_ty))
            || (is_float_like_simple(left_ty) && is_int_like_simple(right_ty)))
}

fn is_simple_int_true_division_binop(op: &str, left_ty: &Type, right_ty: &Type, result_ty: &Type) -> bool {
    op == "/"
        && is_float_like_simple(result_ty)
        && is_int_like_simple(left_ty)
        && is_int_like_simple(right_ty)
}

fn is_safe_simple_compare(op: &str, left_ty: &Type, right_ty: &Type) -> bool {
    if !matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=") {
        return false;
    }
    let left_unaliased = resolve_alias_type(left_ty);
    let right_unaliased = resolve_alias_type(right_ty);
    if left_unaliased == right_unaliased && matches!(left_unaliased, Type::Enum { .. }) {
        return matches!(op, "==" | "!=");
    }
    let left_norm = normalize_simple_compare_scalar_type(left_ty);
    let right_norm = normalize_simple_compare_scalar_type(right_ty);
    left_norm.is_some() && left_norm == right_norm
}

fn is_safe_simple_binop(op: &str, left_ty: &Type, right_ty: &Type, result_ty: &Type) -> bool {
    if op == "//" {
        if is_mixed_simple_float_floor_division_binop(op, left_ty, right_ty, result_ty) {
            return true;
        }
        return is_same_simple_numeric_kind(left_ty, right_ty)
            && is_same_simple_numeric_kind(left_ty, result_ty)
            && (is_int_like_simple(left_ty) || is_float_like_simple(left_ty));
    }
    if op == "/" {
        if is_mixed_simple_float_binop(op, left_ty, right_ty, result_ty)
            || is_simple_int_true_division_binop(op, left_ty, right_ty, result_ty)
        {
            return true;
        }
        return is_same_simple_numeric_kind(left_ty, right_ty)
            && is_same_simple_numeric_kind(left_ty, result_ty)
            && is_float_like_simple(left_ty);
    }
    if matches!(op, "+" | "-" | "*" | "%")
        && is_mixed_simple_float_binop(op, left_ty, right_ty, result_ty)
    {
        return true;
    }
    if !matches!(op, "+" | "-" | "*" | "%") {
        return false;
    }
    is_same_simple_numeric_kind(left_ty, right_ty)
        && is_same_simple_numeric_kind(left_ty, result_ty)
        && is_numeric_simple(left_ty)
}

fn is_same_simple_numeric_kind(left: &Type, right: &Type) -> bool {
    let Some(left_kind) = normalize_simple_numeric_scalar_type(left) else {
        return false;
    };
    normalize_simple_numeric_scalar_type(right).is_some_and(|right_kind| right_kind == left_kind)
}

fn try_lower_option_none_compare_expr(left: &HirExpr, op: &str, right: &HirExpr) -> Option<RustExpr> {
    let name_expr = if matches!(right, HirExpr::NoneLiteral) {
        left
    } else if matches!(left, HirExpr::NoneLiteral) {
        right
    } else {
        return None;
    };
    let HirExpr::Name { name, ty } = name_expr else {
        return None;
    };
    if !is_option_like_simple(ty) {
        return None;
    }
    let method = match op {
        "is" => "is_none",
        "is not" => "is_some",
        _ => return None,
    };
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(name.clone())),
        method: method.to_string(),
        args: vec![],
    })
}

fn try_lower_none_identity_compare_expr(left: &HirExpr, op: &str, right: &HirExpr) -> Option<RustExpr> {
    if !matches!(op, "is" | "is not") {
        return None;
    }
    let other = if matches!(right, HirExpr::NoneLiteral) {
        left
    } else if matches!(left, HirExpr::NoneLiteral) {
        right
    } else {
        return None;
    };
    if !(matches!(other, HirExpr::NoneLiteral) || matches!(resolve_alias_type(other.ty()), Type::None)) {
        return None;
    }
    Some(RustExpr::Literal(RustLiteral::Bool(op == "is")))
}

fn try_lower_simple_range_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty } = expr {
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

fn try_lower_mixed_float_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    let lowered = try_lower_simple_binop_operand_expr(expr)?;
    if is_int_like_simple(expr.ty()) {
        return Some(RustExpr::Cast {
            expr: Box::new(lowered),
            ty: RustType::F64,
        });
    }
    Some(lowered)
}

fn try_lower_simple_binop_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty } = expr {
        if is_numeric_simple(ty) {
            return Some(RustExpr::Ident(name.clone()));
        }
    }
    try_lower_leaf_expr(expr)
}

fn try_lower_simple_compare_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty } = expr {
        if normalize_simple_compare_scalar_type(ty).is_some() || is_enum_like_simple(ty) {
            return Some(RustExpr::Ident(name.clone()));
        }
    }
    try_lower_leaf_expr(expr)
}

fn normalize_simple_compare_scalar_type(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Alias(_, inner) => normalize_simple_compare_scalar_type(inner),
        Type::Int | Type::LiteralInt(_) => Some("int"),
        Type::Float => Some("float"),
        Type::Bool | Type::LiteralBool(_) => Some("bool"),
        Type::Str | Type::LiteralStr(_) => Some("str"),
        _ => None,
    }
}

fn normalize_simple_numeric_scalar_type(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Alias(_, inner) => normalize_simple_numeric_scalar_type(inner),
        Type::Int | Type::LiteralInt(_) => Some("int"),
        Type::Float => Some("float"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_raw_expr_placeholder() {
        let expr = lower_expr_raw("a + b").expect("placeholder lower should succeed");
        assert!(matches!(expr, RustExpr::RawCode(_)));
    }

    #[test]
    fn lowers_leaf_expr_variants() {
        let int_expr = try_lower_leaf_expr(&HirExpr::IntLiteral(7)).expect("int lowered");
        let str_expr =
            try_lower_leaf_expr(&HirExpr::StringLiteral("ok".to_string())).expect("str lowered");
        let bool_expr = try_lower_leaf_expr(&HirExpr::BoolLiteral(true)).expect("bool lowered");
        let bool_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "ok".to_string(),
            ty: Type::Bool,
        })
        .expect("bool name lowered");
        let none_expr = try_lower_leaf_expr(&HirExpr::NoneLiteral).expect("none lowered");
        let enum_expr = try_lower_leaf_expr(&HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "RED".to_string(),
            ty: sifr_type_system::Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1))],
            },
        })
        .expect("enum variant lowered");

        assert!(matches!(
            int_expr,
            RustExpr::Cast {
                ty: RustType::I64,
                ..
            }
        ));
        assert!(matches!(str_expr, RustExpr::Literal(RustLiteral::Str(_))));
        assert!(matches!(bool_expr, RustExpr::Literal(RustLiteral::Bool(true))));
        assert!(matches!(bool_name_expr, RustExpr::Ident(ref name) if name == "ok"));
        assert!(matches!(none_expr, RustExpr::Literal(RustLiteral::None)));
        assert!(matches!(enum_expr, RustExpr::Path(_)));
    }

    #[test]
    fn lowers_numeric_name_leaf_expr_variants() {
        let int_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "count".to_string(),
            ty: Type::Int,
        })
        .expect("int name lowered");
        let float_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "ratio".to_string(),
            ty: Type::Float,
        })
        .expect("float name lowered");
        let alias_int_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "index".to_string(),
            ty: Type::Alias("Index".to_string(), Box::new(Type::Int)),
        })
        .expect("alias-int name lowered");
        let alias_float_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "weight".to_string(),
            ty: Type::Alias("Weight".to_string(), Box::new(Type::Float)),
        })
        .expect("alias-float name lowered");

        assert!(matches!(int_name_expr, RustExpr::Ident(name) if name == "count"));
        assert!(matches!(float_name_expr, RustExpr::Ident(name) if name == "ratio"));
        assert!(matches!(alias_int_name_expr, RustExpr::Ident(name) if name == "index"));
        assert!(matches!(alias_float_name_expr, RustExpr::Ident(name) if name == "weight"));
    }

    #[test]
    fn lowers_bool_and_enum_name_leaf_expr_variants() {
        let alias_bool_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "ready".to_string(),
            ty: Type::Alias("ReadyFlag".to_string(), Box::new(Type::Bool)),
        })
        .expect("alias-bool name lowered");
        let enum_ty = Type::Enum {
            name: "Mode".to_string(),
            variants: vec![("A".to_string(), Some(1)), ("B".to_string(), Some(2))],
        };
        let enum_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "mode".to_string(),
            ty: enum_ty.clone(),
        })
        .expect("enum name lowered");
        let alias_enum_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "mode_alias".to_string(),
            ty: Type::Alias("ModeAlias".to_string(), Box::new(enum_ty)),
        })
        .expect("alias-enum name lowered");

        assert!(matches!(alias_bool_name_expr, RustExpr::Ident(name) if name == "ready"));
        assert!(matches!(enum_name_expr, RustExpr::Ident(name) if name == "mode"));
        assert!(matches!(alias_enum_name_expr, RustExpr::Ident(name) if name == "mode_alias"));
    }

    #[test]
    fn lowers_string_name_leaf_expr_variants() {
        let string_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "label".to_string(),
            ty: Type::Str,
        })
        .expect("string name lowered");
        let alias_string_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "title".to_string(),
            ty: Type::Alias("Title".to_string(), Box::new(Type::Str)),
        })
        .expect("alias-string name lowered");

        assert!(matches!(string_name_expr, RustExpr::Ident(name) if name == "label"));
        assert!(matches!(
            alias_string_name_expr,
            RustExpr::Ident(name) if name == "title"
        ));
    }

    #[test]
    fn lowers_simple_compound_expr_variants() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(1)),
            op: "+".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Int,
        };
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(3)),
            ops: vec![">".to_string()],
            comparators: vec![HirExpr::IntLiteral(1)],
            ty: Type::Bool,
        };
        let cond = HirExpr::IfExpr {
            condition: Box::new(HirExpr::BoolLiteral(true)),
            then_expr: Box::new(HirExpr::IntLiteral(1)),
            else_expr: Box::new(HirExpr::IntLiteral(0)),
            ty: Type::Int,
        };

        assert!(matches!(try_lower_leaf_expr(&bin), Some(RustExpr::BinOp { .. })));
        assert!(matches!(try_lower_leaf_expr(&cmp), Some(RustExpr::BinOp { .. })));
        assert!(matches!(try_lower_leaf_expr(&cond), Some(RustExpr::If { .. })));
    }

    #[test]
    fn lowers_simple_float_division_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(6.0)),
            op: "/".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, .. }) if op == "/"
        ));
    }

    #[test]
    fn lowers_simple_numeric_binop_with_name_operands() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: Type::Int,
            }),
            op: "+".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("int-name binop lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "+"
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_simple_mixed_int_float_division_with_name_operands() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: Type::Int,
            }),
            op: "/".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Float,
            }),
            ty: Type::Float,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("mixed int/float-name division lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "/"
                    && matches!(
                        left.as_ref(),
                        RustExpr::Cast {
                            expr,
                            ty: RustType::F64
                        } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    )
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_alias_wrapped_numeric_binop_with_name_operands() {
        let alias_int = Type::Alias("Meters".to_string(), Box::new(Type::Int));
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: alias_int.clone(),
            }),
            op: "+".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: alias_int.clone(),
            }),
            ty: alias_int,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("alias int-name binop lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "+"
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_simple_alias_base_int_binop_with_name_operands() {
        let alias_int = Type::Alias("Meters".to_string(), Box::new(Type::Int));
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: alias_int,
            }),
            op: "+".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("alias/base int-name binop lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "+"
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_alias_wrapped_mixed_int_float_division_with_name_operands() {
        let alias_int = Type::Alias("Count".to_string(), Box::new(Type::Int));
        let alias_float = Type::Alias("Ratio".to_string(), Box::new(Type::Float));
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: alias_int,
            }),
            op: "/".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: alias_float.clone(),
            }),
            ty: alias_float,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("alias mixed int/float-name division lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "/"
                    && matches!(
                        left.as_ref(),
                        RustExpr::Cast {
                            expr,
                            ty: RustType::F64
                        } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    )
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_simple_alias_base_float_division_with_name_operands() {
        let alias_float = Type::Alias("Ratio".to_string(), Box::new(Type::Float));
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: alias_float,
            }),
            op: "/".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Float,
            }),
            ty: Type::Float,
        };

        let lowered =
            try_lower_leaf_expr(&bin).expect("alias/base float-name division lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn does_not_lower_simple_int_division_binop_with_non_float_result() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(6)),
            op: "/".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Int,
        };
        assert!(try_lower_leaf_expr(&bin).is_none());
    }

    #[test]
    fn lowers_simple_floor_division_int_binop_as_div() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "//".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Int,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, .. }) if op == "/"
        ));
    }

    #[test]
    fn lowers_simple_floor_division_float_binop_as_div() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "//".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, .. }) if op == "/"
        ));
    }

    #[test]
    fn does_not_lower_simple_floor_division_float_binop_with_non_float_result() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "//".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Int,
        };
        assert!(try_lower_leaf_expr(&bin).is_none());
    }

    #[test]
    fn lowers_simple_mixed_int_float_floor_division_binop_as_div_with_casts() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "//".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_float_int_floor_division_binop_as_div_with_casts() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "//".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_int_float_division_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "/".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_float_int_division_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "/".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_int_float_addition_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "+".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "+"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_float_int_modulo_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "%".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "%"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_int_true_division_binop_with_float_casts() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "/".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_multi_operand_boolop_variants() {
        let and_expr = HirExpr::BoolOp {
            op: "and".to_string(),
            values: vec![
                HirExpr::BoolLiteral(true),
                HirExpr::BoolLiteral(false),
                HirExpr::BoolLiteral(true),
            ],
            ty: Type::Bool,
        };
        let or_expr = HirExpr::BoolOp {
            op: "or".to_string(),
            values: vec![
                HirExpr::BoolLiteral(true),
                HirExpr::BoolLiteral(false),
                HirExpr::BoolLiteral(true),
            ],
            ty: Type::Bool,
        };

        assert!(matches!(
            try_lower_leaf_expr(&and_expr),
            Some(RustExpr::BinOp { op, .. }) if op == "&&"
        ));
        assert!(matches!(
            try_lower_leaf_expr(&or_expr),
            Some(RustExpr::BinOp { op, .. }) if op == "||"
        ));
    }

    #[test]
    fn lowers_unary_not_with_bool_name_operand() {
        let unary = HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ok".to_string(),
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary not bool-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::UnaryOp {
                op: ref operator,
                operand: ref inner,
            } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

    #[test]
    fn lowers_unary_not_with_option_name_operand() {
        let unary = HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary not option-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_unary_not_with_alias_option_name_operand() {
        let unary = HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Alias(
                    "MaybeInt".to_string(),
                    Box::new(Type::Union(vec![Type::Int, Type::None])),
                ),
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary not alias-option-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_unary_not_with_alias_bool_name_operand() {
        let alias_bool = Type::Alias("Decision".to_string(), Box::new(Type::Bool));
        let unary = HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ok".to_string(),
                ty: alias_bool,
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary not alias-bool-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::UnaryOp {
                op: ref operator,
                operand: ref inner,
            } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

    #[test]
    fn lowers_unary_bitwise_invert_with_int_operand() {
        let unary = HirExpr::UnaryOp {
            op: "~".to_string(),
            operand: Box::new(HirExpr::IntLiteral(7)),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary invert int lowered");
        assert!(matches!(
            lowered,
            RustExpr::UnaryOp {
                op: ref operator,
                operand: ref inner,
            } if operator == "!" && matches!(inner.as_ref(), RustExpr::Cast { ty: RustType::I64, .. })
        ));
    }

    #[test]
    fn lowers_unary_bitwise_invert_with_alias_int_name_operand() {
        let alias_int = Type::Alias("Bits".to_string(), Box::new(Type::Int));
        let unary = HirExpr::UnaryOp {
            op: "~".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "mask".to_string(),
                ty: alias_int,
            }),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary invert alias-int-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::UnaryOp {
                op: ref operator,
                operand: ref inner,
            } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "mask")
        ));
    }

    #[test]
    fn does_not_lower_unary_bitwise_invert_with_non_int_operand() {
        let unary = HirExpr::UnaryOp {
            op: "~".to_string(),
            operand: Box::new(HirExpr::BoolLiteral(true)),
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&unary).is_none());
    }

    #[test]
    fn lowers_option_is_none_compare_with_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("option is-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_none_compare_with_alias_option_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Alias(
                    "MaybeInt".to_string(),
                    Box::new(Type::Union(vec![Type::Int, Type::None])),
                ),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("alias option is-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_not_none_compare_with_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("option is-not-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_not_none_compare_with_alias_option_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Alias(
                    "MaybeInt".to_string(),
                    Box::new(Type::Union(vec![Type::Int, Type::None])),
                ),
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("alias option is-not-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_none_compare_with_reversed_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("reversed option is-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_not_none_compare_with_reversed_alias_option_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Alias(
                    "MaybeInt".to_string(),
                    Box::new(Type::Union(vec![Type::Int, Type::None])),
                ),
            }],
            ty: Type::Bool,
        };

        let lowered =
            try_lower_leaf_expr(&cmp).expect("reversed alias option is-not-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_is_compare_as_eq() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::IntLiteral(1)],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("is compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "=="
        ));
    }

    #[test]
    fn lowers_simple_is_not_compare_as_ne() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::IntLiteral(2)],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("is-not compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "!="
        ));
    }

    #[test]
    fn lowers_bool_compare_with_literal_bool_name_operands() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: Type::LiteralBool(true),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Bool,
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("bool/literal-bool compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "=="
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn does_not_lower_mismatched_bool_int_compare() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::BoolLiteral(true)),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::IntLiteral(1)],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_string_literal_compare() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::StringLiteral("alpha".to_string())),
            ops: vec!["<".to_string()],
            comparators: vec![HirExpr::StringLiteral("beta".to_string())],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("string compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "<"
        ));
    }

    #[test]
    fn does_not_lower_mismatched_string_int_compare() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::StringLiteral("x".to_string())),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::IntLiteral(1)],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_enum_variant_equality_compare() {
        let enum_ty = Type::Enum {
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
        };
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "RED".to_string(),
                ty: enum_ty.clone(),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "BLUE".to_string(),
                ty: enum_ty,
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("enum equality compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "=="
        ));
    }

    #[test]
    fn does_not_lower_enum_variant_ordering_compare() {
        let enum_ty = Type::Enum {
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
        };
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "RED".to_string(),
                ty: enum_ty.clone(),
            }),
            ops: vec!["<".to_string()],
            comparators: vec![HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "BLUE".to_string(),
                ty: enum_ty,
            }],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_alias_wrapped_enum_variant_equality_compare() {
        let alias_enum_ty = Type::Alias(
            "ColorAlias".to_string(),
            Box::new(Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
            }),
        );
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "RED".to_string(),
                ty: alias_enum_ty.clone(),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "BLUE".to_string(),
                ty: alias_enum_ty,
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("alias enum equality compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "=="
        ));
    }

    #[test]
    fn does_not_lower_alias_wrapped_enum_variant_ordering_compare() {
        let alias_enum_ty = Type::Alias(
            "ColorAlias".to_string(),
            Box::new(Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
            }),
        );
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "RED".to_string(),
                ty: alias_enum_ty.clone(),
            }),
            ops: vec!["<".to_string()],
            comparators: vec![HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "BLUE".to_string(),
                ty: alias_enum_ty,
            }],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_alias_wrapped_scalar_compare() {
        let alias_int = Type::Alias("Meters".to_string(), Box::new(Type::Int));
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: alias_int.clone(),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::Name {
                name: "y".to_string(),
                ty: alias_int,
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("alias scalar compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "=="
        ));
    }

    #[test]
    fn does_not_lower_mismatched_alias_wrapped_scalar_compare() {
        let int_alias = Type::Alias("Meters".to_string(), Box::new(Type::Int));
        let bool_alias = Type::Alias("Flag".to_string(), Box::new(Type::Bool));
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: int_alias,
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::Name {
                name: "ok".to_string(),
                ty: bool_alias,
            }],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_simple_chained_compare_variants() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["<".to_string(), "<".to_string()],
            comparators: vec![HirExpr::IntLiteral(2), HirExpr::IntLiteral(3)],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("chained compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp {
                op: ref top_op,
                left: ref top_left,
                right: ref top_right,
            } if top_op == "&&"
                && matches!(top_left.as_ref(), RustExpr::BinOp { op, .. } if op == "<")
                && matches!(top_right.as_ref(), RustExpr::BinOp { op, .. } if op == "<")
        ));
    }

    #[test]
    fn does_not_lower_option_is_none_compare_with_non_leaf_left() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_range_literal_with_step() {
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(1)),
            end: Box::new(HirExpr::IntLiteral(10)),
            step: Some(Box::new(HirExpr::IntLiteral(2))),
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with step lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver,
                method,
                args,
            } if method == "step_by"
                && matches!(receiver.as_ref(), RustExpr::Range { .. })
                && matches!(
                    args.first(),
                    Some(RustExpr::Cast { ty: RustType::Named(name), .. }) if name == "usize"
                )
        ));
    }

    #[test]
    fn lowers_none_identity_compare_with_none_typed_left() {
        let is_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };
        let is_not_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered_is = try_lower_leaf_expr(&is_cmp).expect("none identity is lowered");
        let lowered_is_not =
            try_lower_leaf_expr(&is_not_cmp).expect("none identity is-not lowered");

        assert!(matches!(
            lowered_is,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(
            lowered_is_not,
            RustExpr::Literal(RustLiteral::Bool(false))
        ));
    }

    #[test]
    fn lowers_none_identity_compare_with_alias_none_typed_left() {
        let alias_none = Type::Alias("Nothing".to_string(), Box::new(Type::None));
        let is_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none.clone(),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };
        let is_not_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none,
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered_is = try_lower_leaf_expr(&is_cmp).expect("alias-none identity is lowered");
        let lowered_is_not =
            try_lower_leaf_expr(&is_not_cmp).expect("alias-none identity is-not lowered");

        assert!(matches!(
            lowered_is,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(
            lowered_is_not,
            RustExpr::Literal(RustLiteral::Bool(false))
        ));
    }

    #[test]
    fn lowers_none_identity_compare_with_none_typed_right() {
        let is_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            }],
            ty: Type::Bool,
        };
        let is_not_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            }],
            ty: Type::Bool,
        };

        let lowered_is = try_lower_leaf_expr(&is_cmp).expect("none identity reversed is lowered");
        let lowered_is_not =
            try_lower_leaf_expr(&is_not_cmp).expect("none identity reversed is-not lowered");

        assert!(matches!(
            lowered_is,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(
            lowered_is_not,
            RustExpr::Literal(RustLiteral::Bool(false))
        ));
    }

    #[test]
    fn lowers_none_identity_compare_with_alias_none_typed_right() {
        let alias_none = Type::Alias("Nothing".to_string(), Box::new(Type::None));
        let is_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none.clone(),
            }],
            ty: Type::Bool,
        };
        let is_not_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none,
            }],
            ty: Type::Bool,
        };

        let lowered_is =
            try_lower_leaf_expr(&is_cmp).expect("alias-none identity reversed is lowered");
        let lowered_is_not =
            try_lower_leaf_expr(&is_not_cmp).expect("alias-none identity reversed is-not lowered");

        assert!(matches!(
            lowered_is,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(
            lowered_is_not,
            RustExpr::Literal(RustLiteral::Bool(false))
        ));
    }

    #[test]
    fn lowers_range_literal_with_name_bounds() {
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::Name {
                name: "start".to_string(),
                ty: Type::Int,
            }),
            end: Box::new(HirExpr::Name {
                name: "end".to_string(),
                ty: Type::Int,
            }),
            step: None,
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with name bounds lowered");
        assert!(matches!(
            lowered,
            RustExpr::Range { start, end }
                if matches!(start.as_ref(), RustExpr::Ident(name) if name == "start")
                    && matches!(end.as_ref(), RustExpr::Ident(name) if name == "end")
        ));
    }

    #[test]
    fn lowers_range_literal_with_name_step() {
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(1)),
            end: Box::new(HirExpr::IntLiteral(10)),
            step: Some(Box::new(HirExpr::Name {
                name: "step".to_string(),
                ty: Type::Int,
            })),
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with name step lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall { method, args, .. }
                if method == "step_by"
                    && matches!(
                        args.first(),
                        Some(RustExpr::Cast { expr, ty: RustType::Named(name) })
                            if matches!(expr.as_ref(), RustExpr::Ident(step_name) if step_name == "step")
                                && name == "usize"
                    )
        ));
    }

    #[test]
    fn lowers_range_literal_with_alias_name_bounds() {
        let alias_int = Type::Alias("Index".to_string(), Box::new(Type::Int));
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::Name {
                name: "start".to_string(),
                ty: alias_int.clone(),
            }),
            end: Box::new(HirExpr::Name {
                name: "end".to_string(),
                ty: alias_int,
            }),
            step: None,
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with alias-name bounds lowered");
        assert!(matches!(
            lowered,
            RustExpr::Range { start, end }
                if matches!(start.as_ref(), RustExpr::Ident(name) if name == "start")
                    && matches!(end.as_ref(), RustExpr::Ident(name) if name == "end")
        ));
    }

    #[test]
    fn lowers_range_literal_with_alias_name_step() {
        let alias_int = Type::Alias("Step".to_string(), Box::new(Type::Int));
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(1)),
            end: Box::new(HirExpr::IntLiteral(10)),
            step: Some(Box::new(HirExpr::Name {
                name: "step".to_string(),
                ty: alias_int,
            })),
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with alias-name step lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall { method, args, .. }
                if method == "step_by"
                    && matches!(
                        args.first(),
                        Some(RustExpr::Cast { expr, ty: RustType::Named(name) })
                            if matches!(expr.as_ref(), RustExpr::Ident(step_name) if step_name == "step")
                                && name == "usize"
                    )
        ));
    }

    #[test]
    fn does_not_lower_range_literal_with_non_int_name_operand() {
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::Name {
                name: "start".to_string(),
                ty: Type::Bool,
            }),
            end: Box::new(HirExpr::IntLiteral(10)),
            step: None,
            ty: Type::Range,
        };

        assert!(try_lower_leaf_expr(&range).is_none());
    }
}
