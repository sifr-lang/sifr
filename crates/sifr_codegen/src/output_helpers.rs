use crate::RustEmitter;

impl RustEmitter {
    pub(super) fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub(super) fn emit_rust_expr(&mut self, expr: &crate::RustExpr) {
        self.output.push_str(&crate::render_expr(expr));
    }

    pub(super) fn emit_rust_stmt_with_current_indent(&mut self, stmt: &crate::RustStmt) {
        let rendered = crate::render_stmts(std::slice::from_ref(stmt));
        if self.indent == 0 {
            self.output.push_str(&rendered);
            return;
        }
        for line in rendered.lines() {
            self.write_indent();
            self.output.push_str(line);
            self.output.push('\n');
        }
    }

    pub(super) fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    pub(super) fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }
}
