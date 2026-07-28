use super::{HirStmt, RustEmitter, RustStmt};

impl RustEmitter {
    pub(crate) fn try_lower_nested_function_stmt_for_block(
        &mut self,
        stmt: &HirStmt,
    ) -> Option<Vec<RustStmt>> {
        let HirStmt::NestedFunction { func, .. } = stmt else {
            return None;
        };

        let captures = self.collect_recursive_nested_fn_captures(func);
        if captures.is_empty() {
            self.nested_fn_captures.remove(&func.name);
        } else {
            self.nested_fn_captures.insert(func.name.clone(), captures);
        }

        let mut handled = false;
        let lowered = self.capture_structured_stmts(|inner| {
            handled = inner.try_lower_structured_nested_function_stmt(stmt);
        });
        handled.then_some(lowered)
    }
}
