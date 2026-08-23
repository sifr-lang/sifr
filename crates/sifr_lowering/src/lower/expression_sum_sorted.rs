use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprCall};
use sifr_type_system::Type;

use super::LowerCtx;
use super::builtin_calls::{callable_builtin_element_type, callable_builtin_list_output_type};
use super::expression_diagnostics;
use super::expressions::{
    callable_signature, consume_owned_value, lower_expr, lower_lambda_with_context,
};
use super::type_bounds::supports_total_order_in_context;

fn first_call_keyword_range(call: &ExprCall) -> TextRange {
    call.arguments
        .keywords
        .first()
        .map_or_else(|| call.func.range(), |keyword| keyword.range)
}

fn call_arity_range(call: &ExprCall) -> TextRange {
    call.arguments
        .args
        .last()
        .map_or_else(|| call.func.range(), Ranged::range)
}

fn sorted_preserves_iterable_source(expr: &HirExpr) -> bool {
    if matches!(expr.ty().resolve_alias(), Type::Iterator(_))
        || expr.ty().resolve_alias().ownership() == sifr_type_system::OwnershipKind::Copy
    {
        return false;
    }
    match expr {
        HirExpr::Name { .. } | HirExpr::FieldAccess { .. } | HirExpr::Index { .. } => true,
        HirExpr::IfExpr {
            then_expr,
            else_expr,
            ..
        } => {
            sorted_preserves_iterable_source(then_expr)
                && sorted_preserves_iterable_source(else_expr)
        }
        _ => false,
    }
}

fn sorted_materialization_requires_clone(expr: &HirExpr) -> bool {
    if matches!(expr.ty().resolve_alias(), Type::Iterator(_))
        || expr.ty().resolve_alias().ownership() == sifr_type_system::OwnershipKind::Copy
    {
        return false;
    }
    match expr {
        HirExpr::Name { .. } | HirExpr::FieldAccess { .. } | HirExpr::Index { .. } => true,
        HirExpr::IfExpr {
            then_expr,
            else_expr,
            ..
        } => {
            sorted_materialization_requires_clone(then_expr)
                || sorted_materialization_requires_clone(else_expr)
        }
        _ => false,
    }
}

pub(in crate::lower) fn lower_sum_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "sum() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() != 1 {
        let actual_count = call.arguments.args.len();
        let range = call_arity_range(call);
        ctx.error_with_code_at(
            DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
            format!("sum() takes exactly 1 argument(s), got {actual_count}"),
            range,
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "sum() argument must be an iterable with a statically-known element type, got '{}'",
                arg.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if super::statement_diagnostics::reject_affine_iterator_builtin(
        ctx,
        "sum",
        &elem_ty,
        call.arguments.args[0].range(),
    ) {
        return None;
    }
    let result_ty = match elem_ty.resolve_alias() {
        Type::FixedInt(fixed) if fixed.supports_current_int_builtin_widening() => Type::Int,
        _ => elem_ty,
    };
    Some(HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "sum".to_string(),
        args: vec![arg],
        ty: result_ty,
    })
}

pub(in crate::lower) fn lower_sorted_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() > 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "sorted() takes at most 1 positional argument".to_string(),
            call_arity_range(call),
        );
        return None;
    }
    let mut iterable_keyword = None;
    let mut key_keyword = None;
    let mut reverse_keyword = None;
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "sorted() does not support unpacked keyword arguments".to_string(),
                keyword.range,
            );
            return None;
        };
        match name.as_str() {
            "iterable" => {
                if iterable_keyword.is_some() {
                    expression_diagnostics::call_duplicate_argument(
                        ctx,
                        "sorted() got multiple values for keyword argument 'iterable'".to_string(),
                        keyword.range,
                    );
                    return None;
                }
                iterable_keyword = Some(keyword);
            }
            "key" => {
                if key_keyword.is_some() {
                    expression_diagnostics::call_duplicate_argument(
                        ctx,
                        "sorted() got multiple values for keyword argument 'key'".to_string(),
                        keyword.range,
                    );
                    return None;
                }
                key_keyword = Some(keyword);
            }
            "reverse" => {
                if reverse_keyword.is_some() {
                    expression_diagnostics::call_duplicate_argument(
                        ctx,
                        "sorted() got multiple values for keyword argument 'reverse'".to_string(),
                        keyword.range,
                    );
                    return None;
                }
                reverse_keyword = Some(keyword);
            }
            other => {
                ctx.error_with_code_at(
                    DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
                    format!("sorted() got an unexpected keyword argument '{other}'"),
                    name.range(),
                );
                return None;
            }
        }
    }
    let (iterable, iterable_range) = match (call.arguments.args.first(), iterable_keyword) {
        (Some(_), Some(keyword)) => {
            expression_diagnostics::call_duplicate_argument(
                ctx,
                "sorted() got multiple values for argument 'iterable'".to_string(),
                keyword.range,
            );
            return None;
        }
        (Some(arg), None) => (lower_expr(arg, ctx)?, arg.range()),
        (None, Some(keyword)) => (lower_expr(&keyword.value, ctx)?, keyword.value.range()),
        (None, None) => {
            ctx.error_with_code_at(
                DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT,
                "sorted() missing required argument 'iterable'".to_string(),
                call.func.range(),
            );
            return None;
        }
    };
    let Some(elem_ty) = callable_builtin_element_type(iterable.ty()) else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "sorted() argument must be an iterable with a statically-known element type, got '{}'",
                iterable.ty().display_name()
            ),
            iterable_range,
        );
        return None;
    };
    if elem_ty.contains_affine_resource()
        && super::statement_diagnostics::reject_affine_iterator_builtin(
            ctx,
            "sorted",
            &elem_ty,
            iterable_range,
        )
    {
        return None;
    }
    let preserves_source = sorted_preserves_iterable_source(&iterable);
    if sorted_materialization_requires_clone(&iterable) && !elem_ty.supports_derived_clone() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "sorted() preserves this iterable and must clone its '{}' elements into the result, but that element type is not Clone-capable",
                elem_ty.display_name()
            ),
            iterable_range,
        );
        return None;
    }
    let mut key_arg = None;
    let mut ordering_ty = elem_ty.clone();
    let mut reverse_arg = HirExpr::BoolLiteral(false);
    if let Some(keyword) = key_keyword {
        let lowered = if matches!(keyword.value, Expr::NoneLiteral(_)) {
            lower_expr(&keyword.value, ctx)?
        } else {
            lower_lambda_with_context(&keyword.value, std::slice::from_ref(&elem_ty), ctx)?
        };
        if !matches!(lowered, HirExpr::NoneLiteral) {
            let Some((param_types, conventions, return_ty)) = callable_signature(&lowered) else {
                expression_diagnostics::call_not_callable_or_arity(
                    ctx,
                    "sorted() keyword argument 'key' must be callable".to_string(),
                    keyword.value.range(),
                );
                return None;
            };
            if param_types.len() != 1 {
                expression_diagnostics::call_not_callable_or_arity(
                    ctx,
                    "sorted() key callable must accept exactly 1 argument".to_string(),
                    keyword.value.range(),
                );
                return None;
            }
            let convention = conventions[0];
            if convention.is_mut_borrow() {
                expression_diagnostics::type_mismatch(
                    ctx,
                    "sorted() key callable cannot require a mutable borrow because sorting compares shared element references"
                        .to_string(),
                    keyword.value.range(),
                );
                return None;
            }
            if param_types[0].resolve_alias() != elem_ty.resolve_alias() {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "sorted() key callable parameter must accept '{}', got '{}'",
                        elem_ty.display_name(),
                        param_types[0].display_name()
                    ),
                    keyword.value.range(),
                );
                return None;
            }
            if convention.is_owned() && !elem_ty.supports_derived_clone() {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "sorted() key callable takes its '{}' element by ownership, but generated comparison requires a Clone-capable element",
                        elem_ty.display_name()
                    ),
                    keyword.value.range(),
                );
                return None;
            }
            ordering_ty = return_ty;
        }
        key_arg = Some(lowered);
    }
    if !matches!(ordering_ty.resolve_alias(), Type::Float)
        && !supports_total_order_in_context(&ordering_ty, ctx)
    {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "sorted() requires an element or key type with generated Rust total Ord support, unavailable for '{}'",
                ordering_ty.display_name()
            ),
            key_keyword.map_or(iterable_range, |keyword| keyword.value.range()),
        );
        return None;
    }
    if let Some(keyword) = reverse_keyword {
        let lowered = lower_expr(&keyword.value, ctx)?;
        if lowered.ty() != &Type::Bool {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "sorted() keyword argument 'reverse' must be 'bool', got '{}'",
                    lowered.ty().display_name()
                ),
                keyword.value.range(),
            );
            return None;
        }
        reverse_arg = lowered;
    }
    let list_ty = callable_builtin_list_output_type(iterable.ty())?;
    if !preserves_source {
        consume_owned_value(&iterable, iterable_range, ctx);
    }
    let mut args = vec![iterable];
    if let Some(key_arg) = key_arg {
        args.push(key_arg);
        args.push(reverse_arg);
    } else if !matches!(reverse_arg, HirExpr::BoolLiteral(false)) {
        args.push(HirExpr::NoneLiteral);
        args.push(reverse_arg);
    }
    Some(HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "sorted".to_string(),
        args,
        ty: list_ty,
    })
}
