use super::*;
use crate::{RustLiteral, RustStmt, render_stmts};

#[test]
fn normalized_index_uses_the_total_runtime_contract() {
    let stmt = RustStmt::Expr(build_normalized_list_index_i64_expr(
        RustExpr::Ident("values".to_string()),
        "raw",
    ));
    let rendered = render_stmts(&[stmt]);
    assert_eq!(rendered, "raw.normalize_index_or_len(values.len());\n");

    let literal = build_normalized_index_expr("raw", RustExpr::Literal(RustLiteral::Int(3)));
    assert!(
        matches!(literal, RustExpr::MethodCall { method, .. } if method == "normalize_index_or_len")
    );
}
