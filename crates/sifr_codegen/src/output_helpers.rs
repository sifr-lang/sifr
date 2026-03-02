use crate::RustEmitter;
use std::fmt::Write as _;

impl RustEmitter {
    fn panic_if_stmt_capture_active_for_string_emission(&self, context: &str) {
        if !self.stmt_capture_stack.is_empty() {
            panic!(
                "string emission reached strict IR capture path ({context}); legacy emission is forbidden during structured lowering"
            );
        }
    }

    pub(super) fn write(&mut self, s: &str) {
        self.panic_if_stmt_capture_active_for_string_emission("write");
        let _ = write!(self.output, "{s}");
    }

    pub(super) fn emit_rust_expr(&mut self, expr: &crate::RustExpr) {
        self.panic_if_stmt_capture_active_for_string_emission("emit_rust_expr");
        let _ = write!(self.output, "{}", crate::render_expr(expr));
    }

    pub(super) fn emit_rust_stmt_with_current_indent(&mut self, stmt: &crate::RustStmt) {
        if let Some(captured) = self.stmt_capture_stack.last_mut() {
            captured.push(stmt.clone());
            return;
        }
        let rendered = crate::render_stmts(std::slice::from_ref(stmt));
        if self.indent == 0 {
            let _ = write!(self.output, "{rendered}");
            return;
        }
        for line in rendered.lines() {
            self.write_indent();
            let _ = write!(self.output, "{line}");
            let _ = self.output.write_char('\n');
        }
    }

    pub(super) fn emit_line(&mut self, s: &str) {
        self.panic_if_stmt_capture_active_for_string_emission("emit_line");
        self.write_indent();
        let _ = write!(self.output, "{s}");
        let _ = self.output.write_char('\n');
    }

    pub(super) fn write_indent(&mut self) {
        self.panic_if_stmt_capture_active_for_string_emission("write_indent");
        for _ in 0..self.indent {
            let _ = write!(self.output, "    ");
        }
    }
}
