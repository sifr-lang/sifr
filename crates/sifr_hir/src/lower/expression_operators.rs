use super::arithmetic_warnings::check_int_overflow_risk;
use super::empty_collection_refinement::refine_empty_set_binding_expr;
use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::numeric_sentinels::{
    lower_sentinel_expr_for_name_domain, maybe_resolve_numeric_sentinel_name_from_type,
    retag_numeric_sentinel_name_expr,
};
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_python_ast::{CmpOp, ExprBinOp, ExprCompare, ExprUnaryOp, Operator, UnaryOp};
use sifr_type_system::{type_check_binary_op, type_check_comparison, type_check_unary_op, Type};

/// Map a binary operator to its corresponding dunder method name.
fn op_to_dunder(op: &str) -> Option<&'static str> {
    match op {
        "+" => Some("__add__"),
        "-" => Some("__sub__"),
        "*" => Some("__mul__"),
        "/" => Some("__truediv__"),
        "//" => Some("__floordiv__"),
        "%" => Some("__mod__"),
        "**" => Some("__pow__"),
        _ => None,
    }
}

pub(super) fn lower_binop(binop: &ExprBinOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&binop.left, ctx)?;
    let right = lower_expr(&binop.right, ctx)?;

    if [&left, &right]
        .iter()
        .any(|expr| matches!(expr, HirExpr::Name { name, .. } if ctx.is_poisoned_binding(name)))
    {
        return None;
    }

    let op_str = match binop.op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mult => "*",
        Operator::Div => "/",
        Operator::FloorDiv => "//",
        Operator::Mod => "%",
        Operator::Pow => "**",
        Operator::BitAnd => "&",
        Operator::BitOr => "|",
        Operator::BitXor => "^",
        Operator::LShift => "<<",
        Operator::RShift => ">>",
        Operator::MatMult => {
            expression_diagnostics::matrix_multiplication(ctx, binop.range());
            return None;
        }
    };

    match type_check_binary_op(left.ty(), op_str, right.ty()) {
        Ok(result_ty) => {
            if result_ty == Type::Int {
                check_int_overflow_risk(op_str, &left, &right, ctx, binop.range());
            }
            Some(HirExpr::BinOp {
                left: Box::new(left),
                op: op_str.to_string(),
                right: Box::new(right),
                ty: result_ty,
            })
        }
        Err((code, message)) => {
            if let Type::Class { methods, .. } = left.ty() {
                if let Some(dunder) = op_to_dunder(op_str) {
                    if let Some((_, ft)) = methods.iter().find(|(name, _)| name == dunder) {
                        let result_ty = *ft.return_type.clone();
                        return Some(HirExpr::BinOp {
                            left: Box::new(left),
                            op: op_str.to_string(),
                            right: Box::new(right),
                            ty: result_ty,
                        });
                    }
                }
            }
            ctx.error_with_code_at(code, message, binop.range());
            None
        }
    }
}

pub(super) fn lower_unaryop(unary: &ExprUnaryOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let operand = lower_expr(&unary.operand, ctx)?;
    if matches!(&operand, HirExpr::Name { name, .. } if ctx.is_poisoned_binding(name)) {
        return None;
    }

    let op_str = match unary.op {
        UnaryOp::USub => "-",
        UnaryOp::UAdd => "+",
        UnaryOp::Not => "not",
        UnaryOp::Invert => "~",
    };

    match type_check_unary_op(op_str, operand.ty()) {
        Ok(result_ty) => Some(HirExpr::UnaryOp {
            op: op_str.to_string(),
            operand: Box::new(operand),
            ty: result_ty,
        }),
        Err((code, message)) => {
            ctx.error_with_code_at(code, message, unary.range());
            None
        }
    }
}

pub(super) fn lower_compare(cmp: &ExprCompare, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut left = lower_expr(&cmp.left, ctx)?;

    if cmp.ops.len() == 1 {
        match &cmp.ops[0] {
            CmpOp::In => {
                let mut collection = lower_expr(&cmp.comparators[0], ctx)?;
                collection = refine_empty_set_binding_expr(collection, left.ty().clone(), ctx);
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        expression_diagnostics::unsupported_operator(
                            ctx,
                            "in",
                            &format!(
                                "element type '{}' and collection element type '{}'",
                                left.ty().display_name(),
                                elem_ty.display_name()
                            ),
                            cmp.comparators[0].range(),
                        );
                    }
                } else {
                    expression_diagnostics::unsupported_operator(
                        ctx,
                        "in",
                        &collection_ty.display_name(),
                        cmp.comparators[0].range(),
                    );
                }
                return Some(HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                });
            }
            CmpOp::NotIn => {
                let mut collection = lower_expr(&cmp.comparators[0], ctx)?;
                collection = refine_empty_set_binding_expr(collection, left.ty().clone(), ctx);
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        expression_diagnostics::unsupported_operator(
                            ctx,
                            "not in",
                            &format!(
                                "element type '{}' and collection element type '{}'",
                                left.ty().display_name(),
                                elem_ty.display_name()
                            ),
                            cmp.comparators[0].range(),
                        );
                    }
                } else {
                    expression_diagnostics::unsupported_operator(
                        ctx,
                        "not in",
                        &collection_ty.display_name(),
                        cmp.comparators[0].range(),
                    );
                }
                let contains = HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                };
                return Some(HirExpr::UnaryOp {
                    op: "not".to_string(),
                    operand: Box::new(contains),
                    ty: Type::Bool,
                });
            }
            _ => {}
        }
    }

    let mut ops = Vec::new();
    let mut comparators = Vec::new();

    for (op, comparator) in cmp.ops.iter().zip(cmp.comparators.iter()) {
        let op_str = match op {
            CmpOp::Eq => "==",
            CmpOp::NotEq => "!=",
            CmpOp::Lt => "<",
            CmpOp::Gt => ">",
            CmpOp::LtE => "<=",
            CmpOp::GtE => ">=",
            CmpOp::Is => "is",
            CmpOp::IsNot => "is not",
            _ => {
                expression_diagnostics::unsupported_operator(
                    ctx,
                    "comparison",
                    "unsupported comparison operator",
                    comparator.range(),
                );
                return None;
            }
        };

        let mut right = if let Some(retagged_right) =
            lower_sentinel_expr_for_name_domain(comparator, &left, ctx)
        {
            retagged_right
        } else {
            lower_expr(comparator, ctx)?
        };
        maybe_resolve_numeric_sentinel_name_from_type(&left, right.ty(), ctx);
        maybe_resolve_numeric_sentinel_name_from_type(&right, left.ty(), ctx);
        left = retag_numeric_sentinel_name_expr(left, ctx);
        if let Some(retagged_right) = lower_sentinel_expr_for_name_domain(comparator, &left, ctx) {
            right = retagged_right;
        } else {
            right = retag_numeric_sentinel_name_expr(right, ctx);
        }

        if op_str != "is" && op_str != "is not" {
            if let Err((code, message)) = type_check_comparison(left.ty(), op_str, right.ty()) {
                let has_overload = match left.ty() {
                    Type::Class { methods, .. } => {
                        let dunder = match op_str {
                            "==" | "!=" => "__eq__",
                            "<" | ">" | "<=" | ">=" => "__lt__",
                            _ => "",
                        };
                        !dunder.is_empty() && methods.iter().any(|(name, _)| name == dunder)
                    }
                    _ => false,
                };
                if !has_overload {
                    ctx.error_with_code_at(code, message, comparator.range());
                    return None;
                }
            }
        }

        ops.push(op_str.to_string());
        comparators.push(right);
    }

    Some(HirExpr::Compare {
        left: Box::new(left),
        ops,
        comparators,
        ty: Type::Bool,
    })
}
