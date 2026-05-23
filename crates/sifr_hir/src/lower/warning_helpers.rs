use super::{LowerCtx, LoweringWarningDiagnostic};
use ruff_text_size::TextRange;

impl LowerCtx {
    pub(super) fn warn_arithmetic_overflow_risk(
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

    pub(super) fn warn_unreachable_statement(&mut self, range: TextRange) {
        self.warnings
            .push(LoweringWarningDiagnostic::UnreachableStatement {
                primary_range: Some(range),
            });
    }

    pub(super) fn warn_bigint_transition_alias(&mut self, range: TextRange) {
        self.warnings
            .push(LoweringWarningDiagnostic::BigIntTransitionAlias {
                primary_range: Some(range),
            });
    }
}
