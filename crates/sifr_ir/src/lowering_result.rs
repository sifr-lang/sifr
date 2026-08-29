use crate::{
    AppliedAdapterMetadata, AttachedApiDeclaration, AttachedApiSetDeclaration,
    ClassAdapterMarkerDeclaration, ClassAdapterProviderDeclaration, ClassAdapterSelection,
    ConstSpecializationRequest, DeclarationDescriptorFunction, FlowGraph, HirExpr, HirModule,
    JsonIntegerBoundaryRequest, LoweringWarningDiagnostic, RevealTypeDiagnostic,
    StaticSpecializationOutput, TypedDeclarationDescriptor, TypedDeclarationMetadata,
};
use num_bigint::BigInt;
use sifr_type_system::Type;
use std::collections::HashMap;

/// A nested-function inference site where the type checker rejected an
/// operation and the inference-only heuristic supplied a provisional type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NestedInferenceDivergence {
    pub function: Option<String>,
    pub operator: String,
    pub left_type: String,
    pub right_type: String,
    pub fallback_type: String,
    pub checker_error: String,
}

/// Result of lowering, including the HIR module and any diagnostics.
#[derive(Clone)]
pub struct LoweringResult {
    pub module: HirModule,
    pub flow_graph: FlowGraph,
    /// Deterministic evidence for inference/checker disagreements encountered
    /// while reaching nested-function fixed points.
    pub nested_inference_divergences: Vec<NestedInferenceDivergence>,
    /// Class declaration defaults keyed by class name and declaration-order field index.
    ///
    /// This is distinct from constructor defaults: it records whether a declared field is
    /// required even when the class defines an explicit `__init__`.
    pub class_field_defaults: HashMap<String, Vec<(usize, HirExpr)>>,
    /// Typed, const-evaluable metadata attached to declarations in this module.
    pub declaration_metadata: Vec<TypedDeclarationMetadata>,
    /// Package-owned provider declarations exported by this module.
    pub class_adapter_providers: Vec<ClassAdapterProviderDeclaration>,
    /// Field-less erased marker declarations exported by this module.
    pub class_adapter_markers: Vec<ClassAdapterMarkerDeclaration>,
    /// Erased attached-API namespaces exported by this module.
    pub attached_api_sets: Vec<AttachedApiSetDeclaration>,
    /// Checked package functions exported through attached-API namespaces.
    pub attached_apis: Vec<AttachedApiDeclaration>,
    /// Canonical providers selected for adapted classes in this module.
    pub class_adapter_selections: Vec<ClassAdapterSelection>,
    /// Package-owned descriptor function declarations exported by this module.
    pub descriptor_functions: Vec<DeclarationDescriptorFunction>,
    /// Evaluated descriptor uses attached to declarations in this module.
    pub declaration_descriptors: Vec<TypedDeclarationDescriptor>,
    /// Validated typed metadata returned by early class adapters.
    pub applied_adapter_metadata: Vec<AppliedAdapterMetadata>,
    /// Resolved non-generic module type aliases available to package declarations.
    pub type_aliases: HashMap<String, Type>,
    /// Resolved generic module type aliases and their declared type parameters.
    pub generic_type_aliases: HashMap<String, (Vec<String>, Type)>,
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
