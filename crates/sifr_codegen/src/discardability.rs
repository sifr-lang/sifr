//! Shared, conservative policy for deleting an evaluated Rust expression.
//!
//! Both structured-IR optimization and post-render syntax cleanup use this
//! module. An expression is discardable only when evaluating and immediately
//! discarding it cannot call user code, panic, or observe `Drop`. Untyped
//! expressions also exclude allocation; typed cleanup can prove standard
//! allocation-only value construction with inert element/drop behavior.

use crate::{RustExpr, RustLiteral};

/// A stored immutable source field needs no owned copy when its value is unused.
/// The receiver must still be evaluated unless it is a local-name read.
pub(crate) fn hir_unused_string_projection_receiver(
    expression: &crate::HirExpr,
) -> Option<&crate::HirExpr> {
    let crate::HirExpr::FieldAccess {
        object,
        field,
        ty: crate::Type::Str,
    } = expression
    else {
        return None;
    };
    let crate::Type::Class { fields, .. } = object.ty().resolve_alias() else {
        return None;
    };
    fields
        .iter()
        .any(|(name, ty)| name == field && *ty == crate::Type::Str)
        .then_some(object)
}

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
        | RustExpr::SourceMethodCall { .. }
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

pub(crate) use sifr_generated_rust::{
    syntax_expression_is_discardable, syntax_expression_is_discardable_with_standard_proof,
};
