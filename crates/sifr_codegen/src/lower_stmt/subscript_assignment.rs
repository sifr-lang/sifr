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
#[path = "../../tests/unit/lower_stmt/subscript_assignment/tests.rs"]
mod tests;
