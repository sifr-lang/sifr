use crate::hir_nodes::HirStmt;
use sifr_python_ast::{Expr, Number};

pub(in crate::lower) fn then_body_always_exits(stmts: &[HirStmt]) -> bool {
    crate::cfg::flow_facts(stmts).is_ok_and(|facts| facts.always_exits())
}

pub(in crate::lower) fn body_always_leaves_current_path(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(stmt_always_leaves_current_path)
}

fn stmt_always_leaves_current_path(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Return { .. } | HirStmt::Break | HirStmt::Continue | HirStmt::Raise { .. } => true,
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body: Some(else_body),
            ..
        } => {
            body_always_leaves_current_path(then_body)
                && elif_clauses
                    .iter()
                    .all(|(_, body)| body_always_leaves_current_path(body))
                && body_always_leaves_current_path(else_body)
        }
        _ => false,
    }
}

pub(in crate::lower) fn expr_to_literal_value(
    expr: &Expr,
) -> Option<sifr_type_system::LiteralValue> {
    match expr {
        Expr::StringLiteral(s) => Some(sifr_type_system::LiteralValue::Str(
            s.value.to_str().to_string(),
        )),
        Expr::NumberLiteral(num) => match &num.value {
            Number::Int(i) => i.as_i64().map(sifr_type_system::LiteralValue::Int),
            _ => None,
        },
        Expr::BooleanLiteral(b) => Some(sifr_type_system::LiteralValue::Bool(b.value)),
        _ => None,
    }
}
