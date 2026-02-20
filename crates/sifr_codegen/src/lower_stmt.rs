//! Statement lowering scaffolds for the IR migration.

use crate::{try_lower_leaf_expr, CodegenError, RustStmt};
use sifr_hir::{HirExpr, HirStmt};

pub fn lower_stmt_raw(raw: &str) -> Result<Vec<RustStmt>, CodegenError> {
    Ok(vec![RustStmt::RawCode(raw.to_string())])
}

/// Lowers an expression statement when the expression is a leaf
/// supported by `try_lower_leaf_expr`.
pub fn try_lower_expr_stmt(expr: &HirExpr) -> Option<Vec<RustStmt>> {
    try_lower_leaf_expr(expr).map(|lowered_expr| vec![RustStmt::Expr(lowered_expr)])
}

/// Lowers statement variants that are context-light and safe to convert
/// without touching complex emitter state.
pub fn try_lower_simple_stmt(stmt: &HirStmt, in_loop_with_else: bool) -> Option<Vec<RustStmt>> {
    match stmt {
        HirStmt::Expr { expr } => try_lower_expr_stmt(expr),
        HirStmt::Pass => Some(vec![]),
        HirStmt::Continue => Some(vec![RustStmt::Continue]),
        HirStmt::Break => {
            if in_loop_with_else {
                Some(vec![RustStmt::RawCode("_broke = true;".to_string()), RustStmt::Break])
            } else {
                Some(vec![RustStmt::Break])
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_raw_stmt_placeholder() {
        let stmts = lower_stmt_raw("let x = 1;").expect("placeholder lower should succeed");
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], RustStmt::RawCode(_)));
    }

    #[test]
    fn lowers_leaf_expression_statement() {
        let stmts = try_lower_expr_stmt(&HirExpr::IntLiteral(1)).expect("leaf stmt lowered");
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], RustStmt::Expr(_)));
    }

    #[test]
    fn lowers_pass_and_continue_and_break() {
        let pass = try_lower_simple_stmt(&HirStmt::Pass, false).expect("pass lowered");
        assert!(pass.is_empty());

        let cont = try_lower_simple_stmt(&HirStmt::Continue, false).expect("continue lowered");
        assert!(matches!(cont[0], RustStmt::Continue));

        let brk = try_lower_simple_stmt(&HirStmt::Break, true).expect("break lowered");
        assert_eq!(brk.len(), 2);
        assert!(matches!(brk[0], RustStmt::RawCode(_)));
        assert!(matches!(brk[1], RustStmt::Break));
    }
}
