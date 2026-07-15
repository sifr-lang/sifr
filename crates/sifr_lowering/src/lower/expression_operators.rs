use super::arithmetic_warnings::check_int_overflow_risk;
use super::empty_collection_refinement::{
    refine_empty_dict_index_comparison_expr, refine_empty_dict_membership_expr,
    refine_empty_set_binding_expr,
};
use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::integer_failure_diagnostics::exact_int_division_requires_handling;
use super::numeric_sentinels::{
    lower_sentinel_expr_for_name_domain, maybe_resolve_numeric_sentinel_name_from_type,
    retag_numeric_sentinel_name_expr,
};
use super::type_bounds::type_satisfies_bound;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use num_bigint::BigInt;
use ruff_text_size::Ranged;
use sifr_python_ast::{CmpOp, ExprBinOp, ExprCompare, ExprUnaryOp, Operator, UnaryOp};
use sifr_type_system::{type_check_binary_op, type_check_comparison, type_check_unary_op, Type};

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
    ty.supports_hash_key()
        || matches!(ty.resolve_alias(), Type::TypeVar(_))
            && type_satisfies_bound(ty, "Hashable", ctx)
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

pub(in crate::lower) fn lower_binop(binop: &ExprBinOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
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

    if exact_int_division_requires_handling(&left, op_str, &right, ctx, binop.range()) {
        return None;
    }
    if generic_addition_requires_addable_bound(&left, op_str, &right, ctx, binop.range()) {
        return None;
    }
    if let Some(result_ty) = exact_int_floor_result_type(&left, op_str, &right, ctx) {
        return Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        });
    }
    if let Some(result_ty) = exact_int_true_division_result_type(&left, op_str, &right, ctx) {
        return Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        });
    }

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

fn exact_int_floor_result_type(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if ctx.is_stdlib_lowering() {
        return None;
    }
    if !matches!(op, "//" | "%") {
        return None;
    }
    if !is_exact_int_like(left.ty()) || !is_exact_int_like(right.ty()) {
        return None;
    }
    if is_proven_nonzero_integer_expr(right, ctx) {
        return None;
    }
    Some(Type::Result(
        Box::new(Type::Int),
        Box::new(division_error_type(ctx)),
    ))
}

fn division_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("DivisionError")
        .cloned()
        .unwrap_or(Type::Class {
            name: "DivisionError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        })
}

fn exact_int_true_division_result_type(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if ctx.is_stdlib_lowering() || op != "/" {
        return None;
    }
    if !is_exact_int_like(left.ty()) || !is_exact_int_like(right.ty()) {
        return None;
    }
    let left_value = proven_exact_integer_value(left, ctx)?;
    let right_value = proven_exact_integer_value(right, ctx)?;
    if right_value == BigInt::from(0) {
        return None;
    }
    (is_exactly_representable_as_float(&left_value)
        && is_exactly_representable_as_float(&right_value))
    .then_some(Type::Float)
}

fn is_exact_int_like(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::LiteralInt(_))
}

fn proven_exact_integer_value(expr: &HirExpr, ctx: &LowerCtx) -> Option<BigInt> {
    match expr {
        HirExpr::IntLiteral(value) => Some(BigInt::from(*value)),
        HirExpr::LargeIntLiteral(value) => value.parse::<BigInt>().ok(),
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            proven_exact_integer_value(operand, ctx).map(|value| -value)
        }
        HirExpr::Name { name, .. } => ctx
            .scope
            .const_integer_value(name)
            .or_else(|| ctx.const_integer_values.get(name))
            .cloned(),
        _ => None,
    }
}

fn is_exactly_representable_as_float(value: &BigInt) -> bool {
    let max_exact = BigInt::from(9_007_199_254_740_992_i64);
    let min_exact = -max_exact.clone();
    value >= &min_exact && value <= &max_exact
}

fn is_proven_nonzero_integer_expr(expr: &HirExpr, ctx: &LowerCtx) -> bool {
    match expr {
        HirExpr::IntLiteral(value) => *value != 0,
        HirExpr::LargeIntLiteral(value) => value != "0",
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            is_proven_nonzero_integer_expr(operand, ctx)
        }
        HirExpr::Name { name, .. } => ctx.is_proven_nonzero_integer_binding(name),
        _ => false,
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
                            "membership is unavailable for values containing affine Python buffers because it requires reusable structural equality"
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
                            "membership is unavailable for values containing affine Python buffers because it requires reusable structural equality"
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
