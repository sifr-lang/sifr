use crate::{CodegenLoweringMode, RustEmitter, RustStmt};
use sifr_hir::HirStmt;

impl RustEmitter {
    /// Emit a generator initialization statement (always mutable for closure capture)
    pub(super) fn emit_generator_init_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let { name, ty, value, .. } => {
                self.write_indent();
                self.write("let mut ");
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            _ => {
                self.emit_stmt(stmt);
            }
        }
    }

    pub(super) fn emit_lowered_stmts(&mut self, lowered_stmts: &[RustStmt]) {
        for lowered_stmt in lowered_stmts {
            match lowered_stmt {
                RustStmt::Expr(lowered_expr) => {
                    self.write_indent();
                    self.write(&crate::render_expr(lowered_expr));
                    self.write(";\n");
                }
                RustStmt::RawCode(code) => {
                    self.write_indent();
                    self.write(code);
                    self.write("\n");
                }
                RustStmt::Break => {
                    self.writeln("break;");
                }
                RustStmt::Continue => {
                    self.writeln("continue;");
                }
                _ => {
                    self.write_indent();
                    let rendered = crate::render_stmts(std::slice::from_ref(lowered_stmt));
                    self.write(rendered.trim_end());
                    self.write("\n");
                }
            }
        }
    }

    pub(super) fn try_capture_legacy_stmt_as_raw(&mut self, stmt: &HirStmt) -> Option<Vec<RustStmt>> {
        if !matches!(stmt, HirStmt::TryExcept { .. } | HirStmt::NestedFunction { .. }) {
            return None;
        }

        let saved_output = std::mem::take(&mut self.output);
        let saved_indent = self.indent;
        let saved_mode = self.lowering_mode;

        self.output = String::new();
        self.indent = 0;
        self.lowering_mode = CodegenLoweringMode::LegacyOnly;
        self.emit_stmt(stmt);

        let captured = std::mem::take(&mut self.output);
        self.output = saved_output;
        self.indent = saved_indent;
        self.lowering_mode = saved_mode;

        Some(vec![RustStmt::RawCode(
            captured.trim_end_matches('\n').to_string(),
        )])
    }

    pub(super) fn current_loop_has_else(&self) -> bool {
        self.loop_else_stack.last().copied().unwrap_or(false)
    }
}
