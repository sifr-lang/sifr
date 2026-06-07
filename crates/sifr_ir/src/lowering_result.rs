use crate::{FlowGraph, HirExpr, HirModule, LoweringWarningDiagnostic, RevealTypeDiagnostic};
use num_bigint::BigInt;
use std::collections::HashMap;

/// Result of lowering, including the HIR module and any diagnostics.
pub struct LoweringResult {
    pub module: HirModule,
    pub flow_graph: FlowGraph,
    pub function_defaults: HashMap<String, Vec<(usize, HirExpr)>>,
    pub function_varargs: HashMap<String, usize>,
    pub function_workloads: HashMap<String, String>,
    pub constant_integer_values: HashMap<String, BigInt>,
    /// `reveal_type()` diagnostics (informational, printed to stderr)
    pub reveal_types: Vec<RevealTypeDiagnostic>,
    /// Compiler warnings (non-fatal diagnostics)
    pub warnings: Vec<LoweringWarningDiagnostic>,
}
