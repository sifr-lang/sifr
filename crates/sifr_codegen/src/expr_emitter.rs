use crate::RustEmitter;
use sifr_hir::HirExpr;

impl RustEmitter {
    pub(super) fn emit_expr_string_backend(&mut self, expr: &HirExpr) {
        panic!(
            "emit_expr_string_backend is unreachable in production structured codegen path (indent={}): {expr:?}",
            self.indent
        );
    }
}
