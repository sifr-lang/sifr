use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;

use super::LowerCtx;
use super::expression_diagnostics;
use super::expressions::lower_expr;

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

pub(in crate::lower) fn lower_abs_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "abs() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            format!(
                "abs() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ),
            call_arity_range(call),
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let ty = arg.ty().clone();
    let fixed_width_abs_widens = matches!(
        ty.resolve_alias(),
        Type::FixedInt(fixed) if fixed.supports_current_int_builtin_widening()
    );
    if !ty.is_numeric() && !fixed_width_abs_widens {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "abs() argument must be numeric, got '{}'",
                ty.display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    let ty = if fixed_width_abs_widens {
        Type::Int
    } else {
        ty
    };
    Some(HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "abs".to_string(),
        args: vec![arg],
        ty,
    })
}
