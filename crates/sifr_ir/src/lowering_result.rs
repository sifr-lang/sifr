use crate::{
    ConstSpecializationRequest, FlowGraph, HirExpr, HirModule, JsonIntegerBoundaryRequest,
    LoweringWarningDiagnostic, RevealTypeDiagnostic, StaticSpecializationOutput,
    TypedDeclarationMetadata,
};
use num_bigint::BigInt;
use std::collections::HashMap;

/// Result of lowering, including the HIR module and any diagnostics.
pub struct LoweringResult {
    pub module: HirModule,
    pub flow_graph: FlowGraph,
    /// Class declaration defaults keyed by class name and declaration-order field index.
    ///
    /// This is distinct from constructor defaults: it records whether a declared field is
    /// required even when the class defines an explicit `__init__`.
    pub class_field_defaults: HashMap<String, Vec<(usize, HirExpr)>>,
    /// Typed, const-evaluable metadata attached to declarations in this module.
    pub declaration_metadata: Vec<TypedDeclarationMetadata>,
    pub specialization_requests: Vec<ConstSpecializationRequest>,
    /// Validated, deterministic outputs produced by package specializers.
    pub specialization_outputs: Vec<StaticSpecializationOutput>,
    pub json_integer_boundary_requests: Vec<JsonIntegerBoundaryRequest>,
    pub function_defaults: HashMap<String, Vec<(usize, HirExpr)>>,
    pub function_varargs: HashMap<String, usize>,
    pub function_python_call_shapes: HashMap<String, Vec<crate::PythonParameterKind>>,
    pub function_workloads: HashMap<String, String>,
    pub constant_integer_values: HashMap<String, BigInt>,
    /// `reveal_type()` diagnostics (informational, printed to stderr)
    pub reveal_types: Vec<RevealTypeDiagnostic>,
    /// Compiler warnings (non-fatal diagnostics)
    pub warnings: Vec<LoweringWarningDiagnostic>,
}
