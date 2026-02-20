//! Expression lowering scaffolds for the IR migration.

use crate::{CodegenError, RustExpr};

pub fn lower_expr_raw(raw: &str) -> Result<RustExpr, CodegenError> {
    Ok(RustExpr::RawCode(raw.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_raw_expr_placeholder() {
        let expr = lower_expr_raw("a + b").expect("placeholder lower should succeed");
        assert!(matches!(expr, RustExpr::RawCode(_)));
    }
}
