use super::{LowerCtx, LoweringWarningDiagnostic};
use ruff_text_size::TextRange;

impl LowerCtx {
    pub(in crate::lower) fn warn_arithmetic_overflow_risk(
        &mut self,
        operation: &'static str,
        range: TextRange,
    ) {
        self.warnings
            .push(LoweringWarningDiagnostic::ArithmeticOverflowRisk {
                operation: operation.to_string(),
                primary_range: Some(range),
            });
    }

    pub(in crate::lower) fn warn_unreachable_statement(&mut self, range: TextRange) {
        self.warnings
            .push(LoweringWarningDiagnostic::UnreachableStatement {
                primary_range: Some(range),
            });
    }
}
