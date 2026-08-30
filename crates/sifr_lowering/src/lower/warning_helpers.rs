use super::{LowerCtx, LoweringWarningDiagnostic};
use ruff_text_size::TextRange;

impl LowerCtx {
    pub(in crate::lower) fn warn_unreachable_statement(&mut self, range: TextRange) {
        self.warnings
            .push(LoweringWarningDiagnostic::UnreachableStatement {
                primary_range: Some(range),
            });
    }
}
