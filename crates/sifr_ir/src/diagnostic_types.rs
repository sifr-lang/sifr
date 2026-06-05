use ruff_text_size::TextRange;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use std::collections::BTreeMap;

/// Structured diagnostics produced during HIR lowering.
#[derive(Debug, Clone)]
pub struct HirDiagnostic {
    pub code: Option<DiagnosticCode>,
    pub message: String,
    pub args: BTreeMap<String, DiagnosticArg>,
    pub help: Option<String>,
    pub primary_range: Option<TextRange>,
    pub line: Option<u32>,
    pub col: Option<u32>,
}

impl std::fmt::Display for HirDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(line), Some(col)) = (self.line, self.col) {
            write!(f, "{}:{}: {}", line, col, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevealTypeDiagnostic {
    pub revealed_type: String,
    pub primary_range: Option<TextRange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoweringWarningDiagnostic {
    ArithmeticOverflowRisk {
        operation: String,
        primary_range: Option<TextRange>,
    },
    UnreachableStatement {
        primary_range: Option<TextRange>,
    },
    BigIntTransitionAlias {
        primary_range: Option<TextRange>,
    },
}
