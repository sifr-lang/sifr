use crate::RustEmitter;
use sifr_hir::HirStmt;

impl RustEmitter {
    pub(super) fn emit_stmt_string_backend(&mut self, stmt: &HirStmt) {
        panic!(
            "emit_stmt_string_backend is unreachable in production structured codegen path: {stmt:?}"
        );
    }
}
