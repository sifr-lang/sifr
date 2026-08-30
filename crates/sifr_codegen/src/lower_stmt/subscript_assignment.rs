use super::RustExpr;

pub(crate) fn build_normalized_list_index_i64_expr(
    receiver: RustExpr,
    raw_index_name: &str,
) -> RustExpr {
    build_normalized_index_expr(
        raw_index_name,
        RustExpr::MethodCall {
            receiver: Box::new(receiver),
            method: "len".to_string(),
            args: Vec::new(),
        },
    )
}

pub(crate) fn build_normalized_index_expr(raw_index_name: &str, len: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(raw_index_name.to_string())),
        method: "normalize_index_or_len".to_string(),
        args: vec![len],
    }
}

#[cfg(test)]
mod tests {
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
}
