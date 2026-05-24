use crate::hir_nodes::HirStmt;
use sifr_python_ast::{Expr, Number};

pub(in crate::lower) fn then_body_always_exits(stmts: &[HirStmt]) -> bool {
    crate::cfg::flow_facts(stmts).always_exits()
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
