use super::LowerCtx;
use super::contextual_list_literal_specialization::specialize_empty_list_literal;
use super::empty_collection_refinement::{
    refine_empty_dict_index_comparison_expr, refine_empty_dict_membership_expr,
    refine_empty_set_binding_expr,
};
use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::integer_float_semantics::mixed_float_integer_arithmetic_result_type;
use super::numeric_sentinels::{
    lower_sentinel_expr_for_name_domain, maybe_resolve_numeric_sentinel_name_from_type,
    retag_numeric_sentinel_name_expr,
};
use super::type_bounds::{supports_hash_key_in_context, supports_structural_equality_in_context};
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_python_ast::{CmpOp, ExprBinOp, ExprCompare, ExprUnaryOp, Operator, UnaryOp};
use sifr_type_system::{Type, type_check_binary_op, type_check_comparison, type_check_unary_op};

mod integer_semantics;
pub(in crate::lower) use integer_semantics::{
    bounded_integer_arithmetic_result_type, builtin_error_type,
    exact_integer_expr_is_proven_float_representable, is_exact_or_fixed_int_like,
    proven_exact_integer_literal, proven_exact_integer_value,
    statically_safe_bounded_integer_augassign,
};
use integer_semantics::{
    checked_decimal_arithmetic_result_type, exact_int_floor_result_type,
    exact_int_true_division_result_type,
};

fn is_none_identity_comparison(left: &Type, right: &Type) -> bool {
    matches!(left.resolve_alias(), Type::None) || matches!(right.resolve_alias(), Type::None)
}

fn is_provisional_empty_dict_membership(
    collection: &HirExpr,
    element_ty: &Type,
    candidate_ty: &Type,
) -> bool {
    matches!(collection, HirExpr::Name { .. })
        && candidate_ty.supports_hash_key()
        && matches!(
            collection.ty().resolve_alias(),
            Type::Dict(key, _)
                if matches!(key.resolve_alias(), Type::Any | Type::Unknown)
                    && matches!(element_ty.resolve_alias(), Type::Any | Type::Unknown)
        )
}

fn membership_requires_hash_key(collection_ty: &Type) -> bool {
    matches!(
        collection_ty.resolve_alias(),
        Type::Set(_) | Type::Dict(_, _)
    )
}

fn supports_membership_hash_key(ty: &Type, ctx: &LowerCtx) -> bool {
    supports_hash_key_in_context(ty, ctx)
}

fn membership_capability_available(
    collection_ty: &Type,
    element_ty: &Type,
    candidate_ty: &Type,
    ctx: &LowerCtx,
) -> bool {
    if membership_requires_hash_key(collection_ty) {
        supports_membership_hash_key(element_ty, ctx)
            && supports_membership_hash_key(candidate_ty, ctx)
    } else {
        element_ty.supports_structural_equality() && candidate_ty.supports_structural_equality()
    }
}

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

fn validate_class_operator_specialization(
    ty: &Type,
    dunder: &str,
    range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Type::Class { name, methods, .. } = ty.resolve_alias() else {
        return true;
    };
    if !methods.iter().any(|(method, _)| method == dunder) {
        return true;
    }
    let class_name = name.clone();
    let concrete = ty.clone();
    super::generic_method_requirements::validate_generic_method_specialization(
        &class_name,
        &concrete,
        dunder,
        range,
        ctx,
    )
}

fn current_generic_method_can_defer_negation(ty: &Type, ctx: &LowerCtx) -> bool {
    let Type::TypeVar(param) = ty.resolve_alias() else {
        return false;
    };
    let Some(class) = ctx.current_class.as_ref() else {
        return false;
    };
    ctx.current_method.is_some()
        && ctx
            .class_declared_type_params
            .get(class)
            .is_some_and(|params| params.contains(param))
}

pub(in crate::lower) fn lower_binop(binop: &ExprBinOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&binop.left, ctx)?;
    let right = lower_expr(&binop.right, ctx)?;

    if [&left, &right]
        .iter()
        .any(|expr| super::expressions::is_poisoned_binding_expr(expr, ctx))
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

    if let Some(requirements) =
        super::generic_method_requirements::requirement_names_for_binary_operator(op_str)
    {
        super::generic_method_requirements::record_current_method_requirements(
            ctx,
            &[left.ty(), right.ty()],
            &requirements,
        );
    }

    if generic_addition_requires_addable_bound(&left, op_str, &right, ctx, binop.range()) {
        return None;
    }
    if let Some(result_ty) = checked_decimal_arithmetic_result_type(&left, op_str, &right, ctx) {
        return Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        });
    }
    if let Some(result_ty) = exact_int_floor_result_type(&left, op_str, &right, ctx) {
        return Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        });
    }
    if let Some(result_ty) = bounded_integer_arithmetic_result_type(&left, op_str, &right, ctx) {
        return Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        });
    }
    if let Some(result_ty) = exact_int_true_division_result_type(&left, op_str, &right, ctx) {
        let (left, right) = if matches!(result_ty, Type::Float) {
            (
                proven_exact_integer_literal(&left, ctx)?,
                proven_exact_integer_literal(&right, ctx)?,
            )
        } else {
            (left, right)
        };
        return Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        });
    }
    if let Some(result_ty) = mixed_float_integer_arithmetic_result_type(&left, op_str, &right, ctx)
    {
        let (left, right) = if matches!(result_ty, Type::Float) {
            (
                if matches!(left.ty().resolve_alias(), Type::Int | Type::LiteralInt(_)) {
                    proven_exact_integer_literal(&left, ctx).unwrap_or(left)
                } else {
                    left
                },
                if matches!(right.ty().resolve_alias(), Type::Int | Type::LiteralInt(_)) {
                    proven_exact_integer_literal(&right, ctx).unwrap_or(right)
                } else {
                    right
                },
            )
        } else {
            (left, right)
        };
        return Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        });
    }

    match type_check_binary_op(left.ty(), op_str, right.ty()) {
        Ok(result_ty) => Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        }),
        Err((code, message)) => {
            if let Type::Class { methods, .. } = left.ty() {
                if let Some(dunder) = op_to_dunder(op_str) {
                    if let Some((_, ft)) = methods.iter().find(|(name, _)| name == dunder) {
                        let result_ty = *ft.return_type.clone();
                        if !validate_class_operator_specialization(
                            left.ty(),
                            dunder,
                            binop.range(),
                            ctx,
                        ) {
                            return None;
                        }
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

fn generic_addition_requires_addable_bound(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &mut LowerCtx,
    range: ruff_text_size::TextRange,
) -> bool {
    if op != "+" {
        return false;
    }
    let Some(type_var) = addition_type_var(left.ty(), right.ty()) else {
        return false;
    };
    if current_owner_has_typevar_bound(ctx, type_var, "Addable") {
        return false;
    }
    ctx.error_with_code_at(
        sifr_diagnostics::DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
        format!(
            "generic addition on type parameter '{type_var}' requires an Addable bound with output assignable to {type_var}"
        ),
        range,
    );
    true
}

fn addition_type_var<'a>(left: &'a Type, right: &'a Type) -> Option<&'a str> {
    match (left.resolve_alias(), right.resolve_alias()) {
        (Type::TypeVar(left), Type::TypeVar(right)) if left == right => Some(left.as_str()),
        _ => None,
    }
}

fn current_owner_has_typevar_bound(ctx: &LowerCtx, type_var: &str, bound: &str) -> bool {
    let Some(owner) = ctx.current_owner.as_ref() else {
        return false;
    };
    ctx.type_param_bounds
        .get(owner)
        .and_then(|bounds| bounds.get(type_var))
        .is_some_and(|bounds| bounds.iter().any(|candidate| candidate == bound))
}

pub(in crate::lower) fn lower_unaryop(unary: &ExprUnaryOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let operand = lower_expr(&unary.operand, ctx)?;
    if super::expressions::is_poisoned_binding_expr(&operand, ctx) {
        return None;
    }

    let op_str = match unary.op {
        UnaryOp::USub => "-",
        UnaryOp::UAdd => "+",
        UnaryOp::Not => "not",
        UnaryOp::Invert => "~",
    };

    if op_str == "-" {
        super::generic_method_requirements::record_current_method_requirements(
            ctx,
            &[operand.ty()],
            &["Clone", "Neg"],
        );
        if current_generic_method_can_defer_negation(operand.ty(), ctx) {
            let ty = operand.ty().clone();
            return Some(HirExpr::UnaryOp {
                op: op_str.to_string(),
                operand: Box::new(operand),
                ty,
            });
        }
    }

    match type_check_unary_op(op_str, operand.ty()) {
        Ok(result_ty) => Some(HirExpr::UnaryOp {
            op: op_str.to_string(),
            operand: Box::new(operand),
            ty: result_ty,
        }),
        Err((code, message)) => {
            if op_str == "-" {
                if let Type::Class { methods, .. } = operand.ty() {
                    if let Some((_, function)) =
                        methods.iter().find(|(method, _)| method == "__neg__")
                    {
                        let result_ty = function.return_type.as_ref().clone();
                        if !validate_class_operator_specialization(
                            operand.ty(),
                            "__neg__",
                            unary.range(),
                            ctx,
                        ) {
                            return None;
                        }
                        return Some(HirExpr::UnaryOp {
                            op: op_str.to_string(),
                            operand: Box::new(operand),
                            ty: result_ty,
                        });
                    }
                }
            }
            ctx.error_with_code_at(code, message, unary.range());
            None
        }
    }
}

pub(in crate::lower) fn lower_compare(cmp: &ExprCompare, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut left = lower_expr(&cmp.left, ctx)?;

    if cmp.ops.len() == 1 {
        match &cmp.ops[0] {
            CmpOp::In => {
                let mut collection = lower_expr(&cmp.comparators[0], ctx)?;
                collection = refine_empty_set_binding_expr(collection, left.ty().clone(), ctx);
                collection = refine_empty_dict_membership_expr(collection, left.ty().clone(), ctx);
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if elem_ty.contains_affine_resource() || left.ty().contains_affine_resource() {
                        ctx.error_with_code_at(
                            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
                            "membership is unavailable for values containing affine Python resources because it requires reusable structural equality"
                                .to_string(),
                            cmp.range(),
                        );
                        return None;
                    }
                    if !membership_capability_available(&collection_ty, &elem_ty, left.ty(), ctx)
                        && !is_provisional_empty_dict_membership(&collection, &elem_ty, left.ty())
                    {
                        let capability = if membership_requires_hash_key(&collection_ty) {
                            "hash/equality"
                        } else {
                            "structural equality"
                        };
                        ctx.error_with_code_at(
                            sifr_diagnostics::DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                            format!(
                                "membership requires {capability}, which is unavailable for '{}' and '{}'",
                                left.ty().display_name(),
                                elem_ty.display_name()
                            ),
                            cmp.range(),
                        );
                        return None;
                    }
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
                collection = refine_empty_dict_membership_expr(collection, left.ty().clone(), ctx);
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if elem_ty.contains_affine_resource() || left.ty().contains_affine_resource() {
                        ctx.error_with_code_at(
                            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
                            "membership is unavailable for values containing affine Python resources because it requires reusable structural equality"
                                .to_string(),
                            cmp.range(),
                        );
                        return None;
                    }
                    if !membership_capability_available(&collection_ty, &elem_ty, left.ty(), ctx)
                        && !is_provisional_empty_dict_membership(&collection, &elem_ty, left.ty())
                    {
                        let capability = if membership_requires_hash_key(&collection_ty) {
                            "hash/equality"
                        } else {
                            "structural equality"
                        };
                        ctx.error_with_code_at(
                            sifr_diagnostics::DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                            format!(
                                "membership requires {capability}, which is unavailable for '{}' and '{}'",
                                left.ty().display_name(),
                                elem_ty.display_name()
                            ),
                            cmp.range(),
                        );
                        return None;
                    }
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
        let right_ty = right.ty().clone();
        left = refine_empty_dict_index_comparison_expr(left, &right_ty, ctx);
        let left_ty = left.ty().clone();
        right = refine_empty_dict_index_comparison_expr(right, &left_ty, ctx);
        if matches!(op_str, "==" | "!=") {
            right = specialize_empty_list_literal(right, left.ty());
            left = specialize_empty_list_literal(left, right.ty());
        }

        if let Some(borrowed) = super::python_interop::python_context_borrow_reference(&left, ctx)
            .or_else(|| super::python_interop::python_context_borrow_reference(&right, ctx))
        {
            ctx.error_with_code_at(
                sifr_diagnostics::DiagnosticCode::PYCTX_INVALID_DECLARATION,
                format!(
                    "invalid Python context declaration: entered binding '{borrowed}' is a context-scoped borrow and cannot participate in a captured comparison"
                ),
                comparator.range(),
            );
            return None;
        }

        if op_str == "is" || op_str == "is not" {
            if !is_none_identity_comparison(left.ty(), right.ty()) {
                ctx.error_with_code_at(
                    sifr_diagnostics::DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                    format!(
                        "'{op_str}' is only available for identity checks against None; use structural equality for '{}' and '{}'",
                        left.ty().display_name(),
                        right.ty().display_name()
                    ),
                    comparator.range(),
                );
                return None;
            }
        } else {
            let dunder = match op_str {
                "==" | "!=" => "__eq__",
                "<" | ">" | "<=" | ">=" => "__lt__",
                _ => "",
            };
            if !dunder.is_empty()
                && !validate_class_operator_specialization(
                    left.ty(),
                    dunder,
                    comparator.range(),
                    ctx,
                )
            {
                return None;
            }
            let mut requirements =
                super::generic_method_requirements::requirement_names_for_comparison(&[
                    op_str.to_string()
                ]);
            if matches!(ctx.current_method.as_deref(), Some("__eq__" | "__lt__")) {
                requirements.remove("Clone");
            }
            super::generic_method_requirements::record_current_method_requirements(
                ctx,
                &[left.ty(), right.ty()],
                &requirements.into_iter().collect::<Vec<_>>(),
            );
            if matches!(op_str, "==" | "!=")
                && left.ty().supports_structural_equality()
                && right.ty().supports_structural_equality()
                && (!supports_structural_equality_in_context(left.ty(), ctx)
                    || !supports_structural_equality_in_context(right.ty(), ctx))
            {
                ctx.error_with_code_at(
                    sifr_diagnostics::DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "structural equality requires generated Rust equality/hash traits, which are unavailable for '{}' and '{}'",
                        left.ty().display_name(),
                        right.ty().display_name()
                    ),
                    comparator.range(),
                );
                return None;
            }
            if let Err((code, message)) = type_check_comparison(left.ty(), op_str, right.ty()) {
                let has_overload = match left.ty() {
                    Type::Class { methods, .. } => {
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
