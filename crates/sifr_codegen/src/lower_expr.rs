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
        HirExpr::Name {
            name,
            ty: Type::Bool | Type::LiteralBool(_),
        } => {
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
                "~" if matches!(operand.ty(), Type::Int | Type::LiteralInt(_)) => {
                    Some(RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(try_lower_leaf_expr(operand)?),
                    })
                }
                "not" if matches!(operand.ty(), Type::Bool | Type::LiteralBool(_)) => {
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
                "not" if crate::helpers::is_option_type(operand.ty()) => {
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
        } if !ops.is_empty() && ops.len() == comparators.len() => {
            if ops.len() == 1 {
                let right = comparators.first()?;
                if let Some(lowered) = try_lower_option_none_compare_expr(left, &ops[0], right) {
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
                    left: Box::new(try_lower_leaf_expr(lhs_expr)?),
                    op: normalized_op.to_string(),
                    right: Box::new(try_lower_leaf_expr(rhs_expr)?),
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

fn normalize_compare_op(op: &str) -> &str {
    match op {
        "is" => "==",
        "is not" => "!=",
        _ => op,
    }
}

fn is_safe_simple_compare(op: &str, left_ty: &Type, right_ty: &Type) -> bool {
    if !matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=") {
        return false;
    }
    let comparable = matches!(left_ty, Type::Int | Type::Float | Type::LiteralInt(_))
        || matches!(left_ty, Type::Bool | Type::LiteralBool(_));
    left_ty == right_ty && comparable
}

fn is_safe_simple_binop(op: &str, left_ty: &Type, right_ty: &Type, result_ty: &Type) -> bool {
    if !matches!(op, "+" | "-" | "*" | "%") {
        return false;
    }
    left_ty == right_ty && left_ty == result_ty && is_numeric_simple(left_ty)
}

fn try_lower_option_none_compare_expr(left: &HirExpr, op: &str, right: &HirExpr) -> Option<RustExpr> {
    if !matches!(right, HirExpr::NoneLiteral) {
        return None;
    }
    let HirExpr::Name { name, ty } = left else {
        return None;
    };
    if !crate::helpers::is_option_type(ty) {
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
}
