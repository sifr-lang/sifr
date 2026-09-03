//! Shared, conservative policy for deleting an evaluated Rust expression.
//!
//! Both structured-IR optimization and post-render syntax cleanup use this
//! module. An expression is discardable only when evaluating and immediately
//! discarding it cannot call user code, panic, allocate, or observe `Drop`.

use crate::{RustExpr, RustLiteral};

pub(crate) fn rust_ir_expression_is_discardable(expression: &RustExpr) -> bool {
    match expression {
        RustExpr::Literal(
            RustLiteral::Int(_)
            | RustLiteral::Float(_)
            | RustLiteral::Bool(_)
            | RustLiteral::StaticStr(_)
            | RustLiteral::Char(_)
            | RustLiteral::Unit
            | RustLiteral::None,
        ) => true,
        RustExpr::Paren(inner) => rust_ir_expression_is_discardable(inner),
        RustExpr::Tuple(elements) | RustExpr::Array(elements) => {
            elements.iter().all(rust_ir_expression_is_discardable)
        }
        RustExpr::Literal(RustLiteral::Str(_))
        | RustExpr::Verbatim(_)
        | RustExpr::Ident(_)
        | RustExpr::Path(_)
        | RustExpr::MethodCall { .. }
        | RustExpr::FnCall { .. }
        | RustExpr::MacroCall { .. }
        | RustExpr::FormatMacro { .. }
        | RustExpr::BinOp { .. }
        | RustExpr::UnaryOp { .. }
        | RustExpr::Field { .. }
        | RustExpr::Index { .. }
        | RustExpr::Slice { .. }
        | RustExpr::Ref { .. }
        | RustExpr::Deref(_)
        | RustExpr::Clone(_)
        | RustExpr::Cast { .. }
        | RustExpr::Block { .. }
        | RustExpr::If { .. }
        | RustExpr::Match { .. }
        | RustExpr::Closure { .. }
        | RustExpr::ClosureBlock { .. }
        | RustExpr::AsyncBlock { .. }
        | RustExpr::StructInit { .. }
        | RustExpr::Vec(_)
        | RustExpr::TimeoutAwait { .. }
        | RustExpr::Try(_)
        | RustExpr::Await(_)
        | RustExpr::Range { .. } => false,
    }
}

pub(crate) fn syntax_expression_is_discardable(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Lit(_) => true,
        syn::Expr::Paren(paren) => syntax_expression_is_discardable(&paren.expr),
        syn::Expr::Tuple(tuple) => tuple.elems.iter().all(syntax_expression_is_discardable),
        syn::Expr::Array(array) => array.elems.iter().all(syntax_expression_is_discardable),
        _ => false,
    }
}
