//! Expression lowering scaffolds for the IR migration.

use crate::{CodegenError, RustExpr, RustLiteral};
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
        HirExpr::IntLiteral(v) => Some(RustExpr::RawCode(format!("{v}_i64"))),
        HirExpr::FloatLiteral(v) => {
            let mut s = v.to_string();
            if !s.contains('.') {
                s.push_str(".0");
            }
            Some(RustExpr::RawCode(format!("{s}_f64")))
        }
        HirExpr::StringLiteral(s) => Some(RustExpr::Literal(RustLiteral::Str(s.clone()))),
        HirExpr::BoolLiteral(v) => Some(RustExpr::Literal(RustLiteral::Bool(*v))),
        HirExpr::NoneLiteral => Some(RustExpr::Literal(RustLiteral::None)),
        HirExpr::EnumVariant { enum_name, variant, .. } => {
            Some(RustExpr::Path(vec![enum_name.clone(), variant.clone()]))
        }
        HirExpr::UnaryOp { op, operand, .. } => {
            let lowered_operand = try_lower_leaf_expr(operand)?;
            match op.as_str() {
                "-" => Some(RustExpr::UnaryOp {
                    op: "-".to_string(),
                    operand: Box::new(lowered_operand),
                }),
                "+" => Some(lowered_operand),
                "not" if matches!(operand.ty(), Type::Bool | Type::LiteralBool(_)) => {
                    Some(RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(lowered_operand),
                    })
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
            Some(RustExpr::BinOp {
                left: Box::new(try_lower_leaf_expr(left)?),
                op: op.clone(),
                right: Box::new(try_lower_leaf_expr(right)?),
            })
        }
        HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } if ops.len() == 1 && comparators.len() == 1 => {
            let right = comparators.first()?;
            if !is_safe_simple_compare(&ops[0], left.ty(), right.ty()) {
                return None;
            }
            Some(RustExpr::BinOp {
                left: Box::new(try_lower_leaf_expr(left)?),
                op: ops[0].clone(),
                right: Box::new(try_lower_leaf_expr(right)?),
            })
        }
        HirExpr::BoolOp { op, values, .. } if values.len() == 2 => {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return None,
            };
            Some(RustExpr::BinOp {
                left: Box::new(try_lower_leaf_expr(values.first()?)?),
                op: lowered_op.to_string(),
                right: Box::new(try_lower_leaf_expr(values.get(1)?)?),
            })
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
        } if step.is_none() => Some(RustExpr::Range {
            start: Box::new(try_lower_leaf_expr(start)?),
            end: Box::new(try_lower_leaf_expr(end)?),
        }),
        _ => None,
    }
}

fn is_numeric_simple(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::Float | Type::LiteralInt(_))
}

fn is_comparable_simple(ty: &Type) -> bool {
    is_numeric_simple(ty) || matches!(ty, Type::Bool | Type::LiteralBool(_))
}

fn is_safe_simple_binop(op: &str, left_ty: &Type, right_ty: &Type, result_ty: &Type) -> bool {
    if !matches!(op, "+" | "-" | "*" | "%") {
        return false;
    }
    left_ty == right_ty && left_ty == result_ty && is_numeric_simple(left_ty)
}

fn is_safe_simple_compare(op: &str, left_ty: &Type, right_ty: &Type) -> bool {
    if !matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=") {
        return false;
    }
    left_ty == right_ty && is_comparable_simple(left_ty)
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

        assert!(matches!(int_expr, RustExpr::RawCode(_)));
        assert!(matches!(str_expr, RustExpr::Literal(RustLiteral::Str(_))));
        assert!(matches!(bool_expr, RustExpr::Literal(RustLiteral::Bool(true))));
        assert!(matches!(none_expr, RustExpr::Literal(RustLiteral::None)));
        assert!(matches!(enum_expr, RustExpr::Path(_)));
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
}
