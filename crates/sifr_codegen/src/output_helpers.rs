use crate::RustEmitter;
use std::fmt::Write as _;

impl RustEmitter {
    pub(super) fn write(&mut self, s: &str) {
        let _ = write!(self.output, "{s}");
    }

    pub(super) fn emit_rust_expr(&mut self, expr: &crate::RustExpr) {
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
        self.write_indent();
        let _ = write!(self.output, "{s}");
        let _ = self.output.write_char('\n');
    }

    pub(super) fn write_indent(&mut self) {
        for _ in 0..self.indent {
            let _ = write!(self.output, "    ");
        }
    }
}
