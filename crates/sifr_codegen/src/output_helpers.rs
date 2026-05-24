use crate::RustEmitter;

impl RustEmitter {
    pub(crate) fn push_captured_stmt(&mut self, stmt: &crate::RustStmt) {
        let Some(captured) = self.stmt_capture_stack.last_mut() else {
            panic!(
                "direct statement string emission is forbidden in IR-first codegen; push_captured_stmt requires active IR capture"
            );
        };
        captured.push(stmt.clone());
    }
}
