use crate::hir_nodes::{HirExpr, HirIteratorOp};
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;

use super::LowerCtx;
use super::builtin_calls::{callable_builtin_element_type, lower_builtin_reverseable_arg};
use super::expression_diagnostics;
use super::expressions::lower_expr;

fn call_arity_range(call: &ExprCall) -> TextRange {
    call.arguments
        .args
        .last()
        .map_or_else(|| call.func.range(), Ranged::range)
}

pub(in crate::lower) fn lower_reversed_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let (arg, elem_ty) = lower_builtin_reverseable_arg(call, "reversed", ctx)?;
    if super::statement_diagnostics::reject_affine_iterator_builtin(
        ctx,
        "reversed",
        &elem_ty,
        call.arguments.args[0].range(),
    ) {
        return None;
    }
    Some(HirExpr::IteratorCall {
        op: HirIteratorOp::Reversed,
        args: vec![arg],
        mutable_arg_places: Vec::new(),
        ty: Type::Iterator(Box::new(elem_ty)),
    })
}

pub(in crate::lower) fn lower_enumerate_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "enumerate() takes 1 or 2 arguments".to_string(),
            call_arity_range(call),
        );
        return None;
    }

    let mut start_keyword = None;
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "enumerate() does not support unpacked keyword arguments".to_string(),
                keyword.range,
            );
            return None;
        };
        if name.as_str() != "start" {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                format!("enumerate() got an unexpected keyword argument '{name}'"),
                name.range(),
            );
            return None;
        }
        if call.arguments.args.len() == 2 || start_keyword.is_some() {
            expression_diagnostics::call_duplicate_argument(
                ctx,
                "enumerate() got multiple values for argument 'start'".to_string(),
                name.range(),
            );
            return None;
        }
        start_keyword = Some(keyword);
    }

    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "enumerate() argument must be an iterable with a statically-known element type, got '{}'",
                arg.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if super::statement_diagnostics::reject_affine_iterator_builtin(
        ctx,
        "enumerate",
        &elem_ty,
        call.arguments.args[0].range(),
    ) {
        return None;
    }

    let start = if call.arguments.args.len() == 2 {
        let start_expr = &call.arguments.args[1];
        let lowered = lower_expr(start_expr, ctx)?;
        if lowered.ty() != &Type::Int {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "enumerate() start argument must be 'int', got '{}'",
                    lowered.ty().display_name()
                ),
                start_expr.range(),
            );
            return None;
        }
        lowered
    } else if let Some(keyword) = start_keyword {
        let lowered = lower_expr(&keyword.value, ctx)?;
        if lowered.ty() != &Type::Int {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "enumerate() keyword argument 'start' must be 'int', got '{}'",
                    lowered.ty().display_name()
                ),
                keyword.value.range(),
            );
            return None;
        }
        lowered
    } else {
        HirExpr::IntLiteral(0)
    };

    let tuple_ty = Type::Tuple(vec![Type::Int, elem_ty]);
    let result_ty = Type::Iterator(Box::new(tuple_ty));
    let args = if matches!(start, HirExpr::IntLiteral(0)) {
        vec![arg]
    } else {
        vec![arg, start]
    };
    Some(HirExpr::IteratorCall {
        op: HirIteratorOp::Enumerate,
        args,
        mutable_arg_places: Vec::new(),
        ty: result_ty,
    })
}
