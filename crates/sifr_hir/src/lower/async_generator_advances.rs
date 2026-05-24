use super::async_for::async_iterator_parts;
use super::call_argument_ranges::{call_arity_range, first_call_keyword_range};
use super::expression_diagnostics;
use super::expressions::lower_expr;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;
use std::collections::HashMap;

#[derive(Default)]
pub(in crate::lower) struct AsyncGeneratorAdvanceTracker {
    pending_generators: HashMap<String, TextRange>,
    pending_bindings: HashMap<String, String>,
}

pub(in crate::lower) fn lower_anext_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        expression_diagnostics::call_unexpected_keyword(
            ctx,
            "anext() does not accept keyword arguments".to_string(),
            first_call_keyword_range(call),
        );
        return None;
    }
    if call.arguments.args.len() != 1 {
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            format!(
                "anext() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ),
            call_arity_range(call),
        );
        return None;
    }
    let iterator = lower_expr(&call.arguments.args[0], ctx)?;
    let Some((item_ty, err_ty)) = async_iterator_parts(iterator.ty()) else {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "anext() argument must be an async iterator, got '{}'",
                iterator.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if matches!(iterator.ty().resolve_alias(), Type::AsyncGenerator(_, _)) {
        if let HirExpr::Name { name, .. } = &iterator {
            begin_async_generator_advance(ctx, name, call.arguments.args[0].range());
        }
    }
    Some(HirExpr::Call {
        func: "anext".to_string(),
        args: vec![iterator],
        ty: Type::Awaitable(Box::new(Type::Result(
            Box::new(Type::Union(vec![item_ty, Type::None])),
            Box::new(err_ty),
        ))),
    })
}

pub(in crate::lower) fn record_async_generator_advance_binding(
    ctx: &mut LowerCtx,
    binding_name: &str,
    value: &HirExpr,
) {
    clear_async_generator_advance_binding(ctx, binding_name);
    if let Some(generator_name) = async_generator_anext_source(value) {
        ctx.async_generator_advances
            .pending_bindings
            .insert(binding_name.to_string(), generator_name);
    }
}

pub(in crate::lower) fn finish_async_generator_advance_for_expr(
    ctx: &mut LowerCtx,
    value: &HirExpr,
) {
    if let Some(generator_name) = async_generator_anext_source(value) {
        ctx.async_generator_advances
            .pending_generators
            .remove(&generator_name);
        return;
    }
    if let HirExpr::Name { name, .. } = value {
        clear_async_generator_advance_binding(ctx, name);
    }
}

fn begin_async_generator_advance(ctx: &mut LowerCtx, generator_name: &str, range: TextRange) {
    if ctx
        .async_generator_advances
        .pending_generators
        .contains_key(generator_name)
    {
        ctx.error_with_code_at(
            DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
            format!(
                "async generator `{generator_name}` already has a pending anext() advance; await it before starting another advance"
            ),
            range,
        );
        return;
    }
    ctx.async_generator_advances
        .pending_generators
        .insert(generator_name.to_string(), range);
}

fn clear_async_generator_advance_binding(ctx: &mut LowerCtx, binding_name: &str) {
    if let Some(generator_name) = ctx
        .async_generator_advances
        .pending_bindings
        .remove(binding_name)
    {
        ctx.async_generator_advances
            .pending_generators
            .remove(&generator_name);
    }
}

fn async_generator_anext_source(value: &HirExpr) -> Option<String> {
    let HirExpr::Call { func, args, .. } = value else {
        return None;
    };
    if func != "anext" || args.len() != 1 {
        return None;
    }
    let HirExpr::Name { name, ty } = &args[0] else {
        return None;
    };
    if matches!(ty.resolve_alias(), Type::AsyncGenerator(_, _)) {
        Some(name.clone())
    } else {
        None
    }
}
