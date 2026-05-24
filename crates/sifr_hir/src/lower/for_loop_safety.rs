use crate::hir_nodes::{HirExpr, HirStmt};
use sifr_type_system::Type;

use super::mutating_methods::is_collection_mutating_method;

pub(in crate::lower) fn is_collection_backed_iter_source(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::List(_)
            | Type::Dict(_, _)
            | Type::Set(_)
            | Type::Str
            | Type::Range
            | Type::Tuple(_)
            | Type::Iterable(_)
    )
}

pub(in crate::lower) fn loop_body_mutates_iter_source(
    stmts: &[HirStmt],
    source_name: &str,
) -> bool {
    stmts
        .iter()
        .any(|stmt| stmt_mutates_iter_source(stmt, source_name))
}

fn stmt_mutates_iter_source(stmt: &HirStmt, source_name: &str) -> bool {
    match stmt {
        HirStmt::Assign { name, .. } | HirStmt::AugAssign { name, .. } => name == source_name,
        HirStmt::FieldAssign { object, .. } | HirStmt::NestedFieldAssign { object, .. } => {
            object == source_name
        }
        HirStmt::SubscriptAssign { object, .. }
        | HirStmt::NestedSubscriptAssign { object, .. }
        | HirStmt::SubscriptAugAssign { object, .. } => object == source_name,
        HirStmt::Delete { object, .. } => {
            matches!(object, HirExpr::Name { name, .. } if name == source_name)
        }
        HirStmt::Expr { expr } => expr_mutates_iter_source(expr, source_name),
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            loop_body_mutates_iter_source(then_body, source_name)
                || elif_clauses
                    .iter()
                    .any(|(_, body)| loop_body_mutates_iter_source(body, source_name))
                || else_body
                    .as_ref()
                    .is_some_and(|body| loop_body_mutates_iter_source(body, source_name))
        }
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        }
        | HirStmt::AsyncFor {
            body, else_body, ..
        } => {
            loop_body_mutates_iter_source(body, source_name)
                || else_body
                    .as_ref()
                    .is_some_and(|body| loop_body_mutates_iter_source(body, source_name))
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            loop_body_mutates_iter_source(body, source_name)
                || handlers
                    .iter()
                    .any(|handler| loop_body_mutates_iter_source(&handler.body, source_name))
        }
        HirStmt::TryFinally { body, finalbody } => {
            loop_body_mutates_iter_source(body, source_name)
                || loop_body_mutates_iter_source(finalbody, source_name)
        }
        HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
            loop_body_mutates_iter_source(body, source_name)
        }
        HirStmt::Match { arms, .. } => arms
            .iter()
            .any(|arm| loop_body_mutates_iter_source(&arm.body, source_name)),
        _ => false,
    }
}

fn expr_mutates_iter_source(expr: &HirExpr, source_name: &str) -> bool {
    match expr {
        HirExpr::MethodCall { object, method, .. } => {
            if let HirExpr::Name { name, ty } = object.as_ref() {
                if name == source_name && is_collection_mutating_method(ty, method) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}
