use crate::hir_nodes::{HirExpr, HirIteratorOp};
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;

use super::builtin_calls::{callable_builtin_element_type, reject_zip_keywords_if_present};
use super::expression_diagnostics;
use super::expressions::{callable_signature, lower_expr, lower_lambda_with_context};
use super::LowerCtx;

fn first_keyword_range(call: &ExprCall) -> TextRange {
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

pub(in crate::lower) fn lower_zip_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if reject_zip_keywords_if_present(call, ctx) {
        return None;
    }
    let mut args = Vec::with_capacity(call.arguments.args.len());
    let mut elem_types = Vec::with_capacity(call.arguments.args.len());
    for (index, arg_expr) in call.arguments.args.iter().enumerate() {
        let arg = lower_expr(arg_expr, ctx)?;
        let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "zip() argument {} must be an iterable with a statically-known element type, got '{}'",
                    index + 1,
                    arg.ty().display_name()
                ),
                arg_expr.range(),
            );
            return None;
        };
        if super::statement_diagnostics::reject_affine_iterator_builtin(
            ctx,
            "zip",
            &elem_ty,
            arg_expr.range(),
        ) {
            return None;
        }
        elem_types.push(elem_ty);
        args.push(arg);
    }
    let result_ty = Type::Iterator(Box::new(Type::Tuple(elem_types)));
    Some(HirExpr::IteratorCall {
        op: HirIteratorOp::Zip,
        args,
        mutable_arg_places: Vec::new(),
        ty: result_ty,
    })
}

pub(in crate::lower) fn lower_any_all_call(
    call: &ExprCall,
    builtin_name: &str,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        let range = if call.arguments.keywords.is_empty() {
            call_arity_range(call)
        } else {
            first_keyword_range(call)
        };
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            format!("{builtin_name}() takes exactly 1 argument"),
            range,
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    if let Some(element_type) = callable_builtin_element_type(arg.ty()) {
        if super::statement_diagnostics::reject_affine_iterator_builtin(
            ctx,
            builtin_name,
            &element_type,
            call.arguments.args[0].range(),
        ) {
            return None;
        }
    }
    Some(HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: builtin_name.to_string(),
        args: vec![arg],
        ty: Type::Bool,
    })
}

pub(in crate::lower) fn lower_map_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "map() does not accept keyword arguments".to_string(),
            first_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() < 2 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "map() takes a callable followed by at least one iterable".to_string(),
            call_arity_range(call),
        );
        return None;
    }
    let mut iter_args = Vec::with_capacity(call.arguments.args.len() - 1);
    let mut context_types = Vec::with_capacity(call.arguments.args.len() - 1);
    for arg_expr in call.arguments.args.iter().skip(1) {
        let iter_arg = lower_expr(arg_expr, ctx)?;
        let Some(elem_ty) = callable_builtin_element_type(iter_arg.ty()) else {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "map() iterable arguments must have statically-known element types, got '{}'",
                    iter_arg.ty().display_name()
                ),
                arg_expr.range(),
            );
            return None;
        };
        if super::statement_diagnostics::reject_affine_iterator_builtin(
            ctx,
            "map",
            &elem_ty,
            arg_expr.range(),
        ) {
            return None;
        }
        context_types.push(elem_ty);
        iter_args.push(iter_arg);
    }
    let func_arg = lower_lambda_with_context(&call.arguments.args[0], &context_types, ctx)?;
    let Some((param_types, _conventions, result_elem_ty)) = callable_signature(&func_arg) else {
        expression_diagnostics::call_not_callable_or_arity(
            ctx,
            "map() first argument must be callable".to_string(),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if param_types.len() != context_types.len() {
        let expected_count = param_types.len();
        let actual_count = context_types.len();
        let range = if actual_count > expected_count {
            call.arguments.args[expected_count + 1].range()
        } else {
            call.func.range()
        };
        expression_diagnostics::call_not_callable_or_arity(
            ctx,
            format!(
                "map() callable expects {expected_count} argument(s), got {actual_count} iterable(s)"
            ),
            range,
        );
        return None;
    }
    if super::statement_diagnostics::reject_affine_iterator_builtin(
        ctx,
        "map",
        &result_elem_ty,
        call.arguments.args[0].range(),
    ) {
        return None;
    }
    let result_ty = Type::Iterator(Box::new(result_elem_ty));
    Some(HirExpr::IteratorCall {
        op: HirIteratorOp::Map,
        args: std::iter::once(func_arg).chain(iter_args).collect(),
        mutable_arg_places: Vec::new(),
        ty: result_ty,
    })
}

pub(in crate::lower) fn lower_filter_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "filter() does not accept keyword arguments".to_string(),
            first_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() != 2 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "filter() takes exactly 2 arguments (function, iterable)".to_string(),
            call_arity_range(call),
        );
        return None;
    }
    let iter_arg = lower_expr(&call.arguments.args[1], ctx)?;
    let Some(elem_ty) = callable_builtin_element_type(iter_arg.ty()) else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "filter() argument must be an iterable with a statically-known element type, got '{}'",
                iter_arg.ty().display_name()
            ),
            call.arguments.args[1].range(),
        );
        return None;
    };
    if super::statement_diagnostics::reject_affine_iterator_builtin(
        ctx,
        "filter",
        &elem_ty,
        call.arguments.args[1].range(),
    ) {
        return None;
    }
    let func_arg =
        lower_lambda_with_context(&call.arguments.args[0], std::slice::from_ref(&elem_ty), ctx)?;
    let Some((param_types, _conventions, return_ty)) = callable_signature(&func_arg) else {
        expression_diagnostics::call_not_callable_or_arity(
            ctx,
            "filter() first argument must be callable".to_string(),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if param_types.len() != 1 {
        expression_diagnostics::call_not_callable_or_arity(
            ctx,
            format!(
                "filter() callable expects {} argument(s), got 1 iterable(s)",
                param_types.len()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    if !return_ty.is_assignable_to(&Type::Bool) && !Type::Bool.is_assignable_to(&return_ty) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "filter() callable must return 'bool', got '{}'",
                return_ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    Some(HirExpr::IteratorCall {
        op: HirIteratorOp::Filter,
        args: vec![func_arg, iter_arg],
        mutable_arg_places: Vec::new(),
        ty: Type::Iterator(Box::new(elem_ty)),
    })
}
