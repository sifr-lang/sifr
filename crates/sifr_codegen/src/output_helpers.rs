use crate::RustEmitter;

impl RustEmitter {
    pub(super) fn emit_rust_expr(&mut self, expr: &crate::RustExpr) {
        panic!(
            "direct expression string emission is forbidden in IR-first codegen; lower to RustExpr and attach to a RustStmt/RustItem instead: {expr:?}"
        );
    }

    pub(super) fn emit_rust_stmt_with_current_indent(&mut self, stmt: &crate::RustStmt) {
        let Some(captured) = self.stmt_capture_stack.last_mut() else {
            panic!(
                "direct statement string emission is forbidden in IR-first codegen; emit_rust_stmt_with_current_indent requires active IR capture"
            );
        };
        captured.push(stmt.clone());
    }
}
