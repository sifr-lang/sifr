use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::{Expr, Stmt, StmtFunctionDef};

use super::LowerCtx;

pub(in crate::lower) fn function_uses_asyncio_run_entrypoint(
    func: &StmtFunctionDef,
    ctx: &LowerCtx,
) -> bool {
    !func.is_async
        && func.name.as_str() == "main"
        && first_asyncio_run_range_in_stmts(&func.body, ctx).is_some()
}

fn first_asyncio_run_range_in_stmts(stmts: &[Stmt], ctx: &LowerCtx) -> Option<TextRange> {
    stmts
        .iter()
        .find_map(|stmt| first_asyncio_run_range_in_stmt(stmt, ctx))
}

fn first_asyncio_run_range_in_stmt(stmt: &Stmt, ctx: &LowerCtx) -> Option<TextRange> {
    match stmt {
        Stmt::Expr(expr_stmt) => first_asyncio_run_range_in_expr(expr_stmt.value.as_ref(), ctx),
        Stmt::Return(ret) => ret
            .value
            .as_ref()
            .and_then(|expr| first_asyncio_run_range_in_expr(expr.as_ref(), ctx)),
        Stmt::AnnAssign(ann) => ann
            .value
            .as_ref()
            .and_then(|expr| first_asyncio_run_range_in_expr(expr.as_ref(), ctx)),
        Stmt::Assign(assign) => first_asyncio_run_range_in_expr(assign.value.as_ref(), ctx),
        Stmt::AugAssign(aug) => first_asyncio_run_range_in_expr(aug.value.as_ref(), ctx),
        Stmt::If(if_stmt) => first_asyncio_run_range_in_expr(if_stmt.test.as_ref(), ctx)
            .or_else(|| first_asyncio_run_range_in_stmts(&if_stmt.body, ctx))
            .or_else(|| {
                if_stmt.elif_else_clauses.iter().find_map(|clause| {
                    clause
                        .test
                        .as_ref()
                        .and_then(|expr| first_asyncio_run_range_in_expr(expr, ctx))
                        .or_else(|| first_asyncio_run_range_in_stmts(&clause.body, ctx))
                })
            }),
        Stmt::While(while_stmt) => first_asyncio_run_range_in_expr(while_stmt.test.as_ref(), ctx)
            .or_else(|| first_asyncio_run_range_in_stmts(&while_stmt.body, ctx))
            .or_else(|| first_asyncio_run_range_in_stmts(&while_stmt.orelse, ctx)),
        Stmt::For(for_stmt) => first_asyncio_run_range_in_expr(for_stmt.iter.as_ref(), ctx)
            .or_else(|| first_asyncio_run_range_in_stmts(&for_stmt.body, ctx))
            .or_else(|| first_asyncio_run_range_in_stmts(&for_stmt.orelse, ctx)),
        Stmt::With(with_stmt) => with_stmt
            .items
            .iter()
            .find_map(|item| first_asyncio_run_range_in_expr(&item.context_expr, ctx))
            .or_else(|| first_asyncio_run_range_in_stmts(&with_stmt.body, ctx)),
        Stmt::Try(try_stmt) => first_asyncio_run_range_in_stmts(&try_stmt.body, ctx)
            .or_else(|| {
                try_stmt.handlers.iter().find_map(|handler| {
                    let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                    first_asyncio_run_range_in_stmts(&handler.body, ctx)
                })
            })
            .or_else(|| first_asyncio_run_range_in_stmts(&try_stmt.orelse, ctx))
            .or_else(|| first_asyncio_run_range_in_stmts(&try_stmt.finalbody, ctx)),
        _ => None,
    }
}

fn first_asyncio_run_range_in_expr(expr: &Expr, ctx: &LowerCtx) -> Option<TextRange> {
    match expr {
        Expr::Call(call) => {
            if let Expr::Name(name) = call.func.as_ref() {
                if ctx
                    .asyncio_compat_imports
                    .get(name.id.as_str())
                    .is_some_and(|member| member == "run")
                {
                    return Some(call.range());
                }
            }
            first_asyncio_run_range_in_expr(call.func.as_ref(), ctx).or_else(|| {
                call.arguments
                    .args
                    .iter()
                    .find_map(|arg| first_asyncio_run_range_in_expr(arg, ctx))
                    .or_else(|| {
                        call.arguments.keywords.iter().find_map(|keyword| {
                            first_asyncio_run_range_in_expr(&keyword.value, ctx)
                        })
                    })
            })
        }
        Expr::Attribute(attr) => first_asyncio_run_range_in_expr(attr.value.as_ref(), ctx),
        Expr::Subscript(sub) => first_asyncio_run_range_in_expr(sub.value.as_ref(), ctx)
            .or_else(|| first_asyncio_run_range_in_expr(sub.slice.as_ref(), ctx)),
        Expr::BinOp(bin) => first_asyncio_run_range_in_expr(bin.left.as_ref(), ctx)
            .or_else(|| first_asyncio_run_range_in_expr(bin.right.as_ref(), ctx)),
        Expr::BoolOp(bool_op) => bool_op
            .values
            .iter()
            .find_map(|value| first_asyncio_run_range_in_expr(value, ctx)),
        Expr::UnaryOp(unary) => first_asyncio_run_range_in_expr(unary.operand.as_ref(), ctx),
        Expr::Compare(compare) => first_asyncio_run_range_in_expr(compare.left.as_ref(), ctx)
            .or_else(|| {
                compare
                    .comparators
                    .iter()
                    .find_map(|expr| first_asyncio_run_range_in_expr(expr, ctx))
            }),
        Expr::If(if_expr) => first_asyncio_run_range_in_expr(if_expr.test.as_ref(), ctx)
            .or_else(|| first_asyncio_run_range_in_expr(if_expr.body.as_ref(), ctx))
            .or_else(|| first_asyncio_run_range_in_expr(if_expr.orelse.as_ref(), ctx)),
        Expr::List(list) => list
            .elts
            .iter()
            .find_map(|expr| first_asyncio_run_range_in_expr(expr, ctx)),
        Expr::Tuple(tuple) => tuple
            .elts
            .iter()
            .find_map(|expr| first_asyncio_run_range_in_expr(expr, ctx)),
        Expr::Set(set) => set
            .elts
            .iter()
            .find_map(|expr| first_asyncio_run_range_in_expr(expr, ctx)),
        Expr::Dict(dict) => dict.items.iter().find_map(|item| {
            item.key
                .as_ref()
                .and_then(|expr| first_asyncio_run_range_in_expr(expr, ctx))
                .or_else(|| first_asyncio_run_range_in_expr(&item.value, ctx))
        }),
        _ => None,
    }
}
