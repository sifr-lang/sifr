use crate::hir_nodes::HirExpr;
use sifr_python_ast::{Expr, Number, Operator, UnaryOp};
use sifr_type_system::Type;

use super::integer_literals::canonical_large_int_literal_text;

/// Lower a simple expression without requiring a full `LowerCtx`.
/// Used for collecting default parameter values in the first pass.
pub(in crate::lower) fn lower_expr_simple(expr: &Expr) -> Option<HirExpr> {
    lower_expr_simple_inner(expr, false)
}

pub(in crate::lower) fn lower_integer_const_expr_simple(expr: &Expr) -> Option<HirExpr> {
    lower_expr_simple_inner(expr, true)
}

fn lower_expr_simple_inner(expr: &Expr, allow_integer_binop: bool) -> Option<HirExpr> {
    match expr {
        Expr::NumberLiteral(num) => match &num.value {
            Number::Int(i) => {
                if let Some(value) = i.as_i64() {
                    Some(HirExpr::IntLiteral(value))
                } else {
                    Some(HirExpr::LargeIntLiteral(canonical_large_int_literal_text(
                        i,
                    )))
                }
            }
            Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
            Number::Complex { .. } => None,
        },
        Expr::StringLiteral(s) => Some(HirExpr::StringLiteral(s.value.to_str().to_string())),
        Expr::BytesLiteral(bytes) => {
            let mut elements = Vec::new();
            for part in &bytes.value {
                for value in part.as_slice() {
                    elements.push(HirExpr::IntLiteral(i64::from(*value)));
                }
            }
            Some(HirExpr::ListLiteral {
                elements,
                ty: Type::Bytes,
            })
        }
        Expr::BooleanLiteral(b) => Some(HirExpr::BoolLiteral(b.value)),
        Expr::NoneLiteral(_) => Some(HirExpr::NoneLiteral),
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::UAdd) => {
            lower_expr_simple_inner(&unary.operand, allow_integer_binop)
        }
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::USub) => {
            lower_expr_simple_inner(&unary.operand, allow_integer_binop)
                .and_then(negate_simple_expr)
        }
        Expr::BinOp(binop) => {
            if !allow_integer_binop {
                return None;
            }
            let left = lower_expr_simple_inner(&binop.left, true)?;
            let right = lower_expr_simple_inner(&binop.right, true)?;
            if !matches!(left.ty(), Type::Int) || !matches!(right.ty(), Type::Int) {
                return None;
            }
            Some(HirExpr::BinOp {
                left: Box::new(left),
                op: integer_binop_source(binop.op)?.to_string(),
                right: Box::new(right),
                ty: Type::Int,
            })
        }
        Expr::List(list) => {
            let mut elements = Vec::new();
            let mut elem_ty: Option<Type> = None;
            for elt in &list.elts {
                let lowered = lower_expr_simple_inner(elt, allow_integer_binop)?;
                let lowered_ty = lowered.ty().clone();
                if let Some(ref expected) = elem_ty {
                    if !lowered_ty.is_assignable_to(expected) {
                        return None;
                    }
                } else {
                    elem_ty = Some(lowered_ty);
                }
                elements.push(lowered);
            }
            Some(HirExpr::ListLiteral {
                elements,
                ty: Type::List(Box::new(elem_ty.unwrap_or(Type::Any))),
            })
        }
        Expr::Set(set) => {
            let mut elements = Vec::new();
            let mut elem_ty: Option<Type> = None;
            for elt in &set.elts {
                let lowered = lower_expr_simple_inner(elt, allow_integer_binop)?;
                let lowered_ty = lowered.ty().clone();
                if let Some(ref expected) = elem_ty {
                    if !lowered_ty.is_assignable_to(expected) {
                        return None;
                    }
                } else {
                    elem_ty = Some(lowered_ty);
                }
                elements.push(lowered);
            }
            Some(HirExpr::SetLiteral {
                elements,
                ty: Type::Set(Box::new(elem_ty.unwrap_or(Type::Any))),
            })
        }
        Expr::Dict(dict) => {
            let mut keys = Vec::new();
            let mut values = Vec::new();
            let mut key_ty: Option<Type> = None;
            let mut val_ty: Option<Type> = None;

            for item in &dict.items {
                let key_expr = item.key.as_ref()?;
                let lowered_key = lower_expr_simple_inner(key_expr, allow_integer_binop)?;
                let lowered_val = lower_expr_simple_inner(&item.value, allow_integer_binop)?;
                let lowered_key_ty = lowered_key.ty().clone();
                let lowered_val_ty = lowered_val.ty().clone();

                if let Some(ref expected) = key_ty {
                    if !lowered_key_ty.is_assignable_to(expected) {
                        return None;
                    }
                } else {
                    key_ty = Some(lowered_key_ty);
                }

                if let Some(ref expected) = val_ty {
                    if !lowered_val_ty.is_assignable_to(expected) {
                        return None;
                    }
                } else {
                    val_ty = Some(lowered_val_ty);
                }

                keys.push(lowered_key);
                values.push(lowered_val);
            }

            Some(HirExpr::DictLiteral {
                keys,
                values,
                ty: Type::Dict(
                    Box::new(key_ty.unwrap_or(Type::Any)),
                    Box::new(val_ty.unwrap_or(Type::Any)),
                ),
            })
        }
        Expr::Tuple(tuple) => {
            let mut elements = Vec::new();
            let mut element_types = Vec::new();
            for elt in &tuple.elts {
                let lowered = lower_expr_simple_inner(elt, allow_integer_binop)?;
                element_types.push(lowered.ty().clone());
                elements.push(lowered);
            }
            Some(HirExpr::TupleLiteral {
                elements,
                ty: Type::Tuple(element_types),
            })
        }
        _ => None,
    }
}

pub(in crate::lower) fn negate_simple_expr(expr: HirExpr) -> Option<HirExpr> {
    match expr {
        HirExpr::IntLiteral(value) => Some(HirExpr::IntLiteral(-value)),
        HirExpr::LargeIntLiteral(value) => Some(HirExpr::UnaryOp {
            op: "-".to_string(),
            operand: Box::new(HirExpr::LargeIntLiteral(value)),
            ty: Type::Int,
        }),
        HirExpr::FloatLiteral(value) => Some(HirExpr::FloatLiteral(-value)),
        _ => None,
    }
}

pub(in crate::lower) fn integer_binop_source(op: Operator) -> Option<&'static str> {
    match op {
        Operator::Add => Some("+"),
        Operator::Sub => Some("-"),
        Operator::Mult => Some("*"),
        Operator::FloorDiv => Some("//"),
        Operator::Mod => Some("%"),
        Operator::Pow => Some("**"),
        Operator::LShift => Some("<<"),
        Operator::RShift => Some(">>"),
        _ => None,
    }
}
