use crate::{RustEmitter, RustStmt};
use sifr_hir::HirStmt;

impl RustEmitter {
    /// Emit a generator initialization statement (always mutable for closure capture)
    pub(super) fn emit_generator_init_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let {
                name, ty, value, ..
            } => {
                self.write_indent();
                self.write("let mut ");
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                match self.try_emit_structured_expr(value) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        assert!(
                            self.try_emit_expr_legacy_bridge(value),
                            "structured generator-init expression emission missing for production path: {value:?}"
                        );
                    }
                }
                self.write(";\n");
            }
            _ => {
                match self.try_emit_structured_stmt(stmt) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        assert!(
                            self.try_emit_stmt_legacy_bridge(stmt),
                            "structured generator-init statement emission missing for production path: {stmt:?}"
                        );
                    }
                }
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

    pub(super) fn current_loop_has_else(&self) -> bool {
        self.loop_else_stack.last().copied().unwrap_or(false)
    }
}
