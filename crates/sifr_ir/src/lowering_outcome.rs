use crate::LoweringResult;
use sifr_diagnostics::SifrDiagnostic;

pub struct LoweringOutcome {
    pub result: LoweringResult,
    pub diagnostics: Vec<SifrDiagnostic>,
}
