use super::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Comprehension, Expr};

fn reject_unsupported_expression_form(ctx: &mut LowerCtx, message: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        message.to_string(),
        range,
    );
}

fn first_await_range_in_expr(expr: &Expr) -> Option<TextRange> {
    match expr {
        Expr::Await(await_expr) => Some(await_expr.range()),
        Expr::Call(call) => first_await_range_in_expr(call.func.as_ref()).or_else(|| {
            call.arguments
                .args
                .iter()
                .find_map(first_await_range_in_expr)
                .or_else(|| {
                    call.arguments
                        .keywords
                        .iter()
                        .find_map(|keyword| first_await_range_in_expr(&keyword.value))
                })
        }),
        Expr::Attribute(attr) => first_await_range_in_expr(attr.value.as_ref()),
        Expr::Subscript(sub) => first_await_range_in_expr(sub.value.as_ref())
            .or_else(|| first_await_range_in_expr(sub.slice.as_ref())),
        Expr::BinOp(bin) => first_await_range_in_expr(bin.left.as_ref())
            .or_else(|| first_await_range_in_expr(bin.right.as_ref())),
        Expr::BoolOp(bool_op) => bool_op.values.iter().find_map(first_await_range_in_expr),
        Expr::UnaryOp(unary) => first_await_range_in_expr(unary.operand.as_ref()),
        Expr::Compare(compare) => first_await_range_in_expr(compare.left.as_ref()).or_else(|| {
            compare
                .comparators
                .iter()
                .find_map(first_await_range_in_expr)
        }),
        Expr::If(if_expr) => first_await_range_in_expr(if_expr.test.as_ref())
            .or_else(|| first_await_range_in_expr(if_expr.body.as_ref()))
            .or_else(|| first_await_range_in_expr(if_expr.orelse.as_ref())),
        Expr::List(list) => list.elts.iter().find_map(first_await_range_in_expr),
        Expr::Tuple(tuple) => tuple.elts.iter().find_map(first_await_range_in_expr),
        Expr::Set(set) => set.elts.iter().find_map(first_await_range_in_expr),
        Expr::Dict(dict) => dict.items.iter().find_map(|item| {
            item.key
                .as_ref()
                .and_then(first_await_range_in_expr)
                .or_else(|| first_await_range_in_expr(&item.value))
        }),
        _ => None,
    }
}

pub(in crate::lower) fn reject_deferred_async_comprehension_shape(
    ctx: &mut LowerCtx,
    comprehension_kind: &str,
    generators: &[Comprehension],
    fallback_range: TextRange,
) -> bool {
    if !generators.iter().any(|generator| generator.is_async) {
        return false;
    }

    if generators.len() != 1 {
        reject_unsupported_expression_form(
            ctx,
            "nested async comprehensions are deferred in v1; use a single async for clause",
            generators
                .iter()
                .find(|generator| generator.is_async)
                .map_or(fallback_range, Ranged::range),
        );
        return true;
    }

    if let Some(await_range) = generators[0].ifs.iter().find_map(first_await_range_in_expr) {
        reject_unsupported_expression_form(
            ctx,
            "await inside async comprehension filters is deferred in v1; compute the awaited value before the comprehension",
            await_range,
        );
        return true;
    }

    reject_unsupported_expression_form(
        ctx,
        &format!(
            "async {comprehension_kind} comprehensions require async-comprehension lowering and are not supported yet"
        ),
        fallback_range,
    );
    true
}

pub(in crate::lower) fn reject_unsupported_basic_async_comprehension_shape(
    ctx: &mut LowerCtx,
    generators: &[Comprehension],
    fallback_range: TextRange,
) -> bool {
    if !generators.iter().any(|generator| generator.is_async) {
        return false;
    }

    if generators.len() != 1 {
        reject_unsupported_expression_form(
            ctx,
            "nested async comprehensions are deferred in v1; use a single async for clause",
            generators
                .iter()
                .find(|generator| generator.is_async)
                .map_or(fallback_range, Ranged::range),
        );
        return true;
    }

    if let Some(await_range) = generators[0].ifs.iter().find_map(first_await_range_in_expr) {
        reject_unsupported_expression_form(
            ctx,
            "await inside async comprehension filters is deferred in v1; compute the awaited value before the comprehension",
            await_range,
        );
        return true;
    }

    false
}

pub(in crate::lower) fn reject_async_generator_expression(ctx: &mut LowerCtx, range: TextRange) {
    reject_unsupported_expression_form(
        ctx,
        "async generator expressions are deferred in v1; use an async def with yield or an eager async comprehension",
        range,
    );
}
