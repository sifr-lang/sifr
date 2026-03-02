use crate::RustEmitter;

impl RustEmitter {
    pub(super) fn emit_rust_stmt_with_current_indent(&mut self, stmt: &crate::RustStmt) {
        let Some(captured) = self.stmt_capture_stack.last_mut() else {
            panic!(
                "direct statement string emission is forbidden in IR-first codegen; emit_rust_stmt_with_current_indent requires active IR capture"
            );
        };
        captured.push(stmt.clone());
    }
}
