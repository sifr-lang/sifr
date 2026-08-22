use crate::{is_simple_stmt_candidate, HirStmt, RustEmitter, RustExpr, RustLiteral, RustStmt};

impl RustEmitter {
    pub(crate) fn emit_stmt(&mut self, stmt: &HirStmt) {
        self.emit_stmt_with_following(stmt, None);
    }

    pub(crate) fn emit_stmt_with_following(
        &mut self,
        stmt: &HirStmt,
        following_stmts: Option<&[HirStmt]>,
    ) {
        self.lowering_stats.stmt_total += 1;
        if is_simple_stmt_candidate(stmt) {
            self.lowering_stats.stmt_candidate_total += 1;
        }
        match self.try_lower_structured_stmt_with_following(stmt, following_stmts) {
            Ok(true) => {}
            Ok(false) => {
                self.lowering_stats.stmt_lowering_errors += 1;
                self.push_captured_stmt(&RustStmt::Expr(RustExpr::MacroCall {
                    name: "compile_error".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Str(format!(
                        "structured statement emission missing for production path: {stmt:?}"
                    )))],
                }));
            }
            Err(err) => {
                self.lowering_stats.stmt_lowering_errors += 1;
                self.push_captured_stmt(&RustStmt::Expr(RustExpr::MacroCall {
                    name: "compile_error".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Str(format!(
                        "structured statement lowering failed for production path: {stmt:?}; error={err}"
                    )))],
                }));
            }
        }
    }
}
