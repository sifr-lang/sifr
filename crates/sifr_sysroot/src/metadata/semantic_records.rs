//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::TypeParameterBounds;
use super::{
    AppliedAdapterMetadata, AttachedApiDeclaration, AttachedApiSetDeclaration, BTreeMap, BTreeSet,
    CallableIdentity, ClassAdapterMarkerDeclaration, ClassAdapterProviderDeclaration,
    ClassAdapterSelection, CompilerIntrinsicId, ConstSpecializationRequest,
    DeclarationDescriptorFunction, FunctionType, HirExpr, HirFunction, HirParam,
    JsonIntegerBoundaryRequest, MethodKind, PythonParameterKind, ReceiverConvention, Ref,
    StaticSpecializationOutput, Text, Type, TypedDeclarationDescriptor, TypedDeclarationMetadata,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticExports {
    pub functions: BTreeMap<Ref<Text>, Ref<FunctionType>>,
    pub compiler_intrinsics: BTreeMap<Ref<Text>, Ref<CompilerIntrinsicId>>,
    pub classes: BTreeMap<Ref<Text>, Ref<Type>>,
    pub generic_type_aliases: BTreeMap<Ref<Text>, GenericAliasExport>,
    pub class_instance_methods: BTreeMap<Ref<Text>, BTreeSet<Ref<Text>>>,
    pub rust_consuming_methods: BTreeMap<Ref<Text>, BTreeSet<Ref<Text>>>,
    pub rust_opaque_classes: BTreeSet<Ref<Text>>,
    pub rust_structural_classes: BTreeSet<Ref<Text>>,
    pub class_type_params: BTreeMap<Ref<Text>, Vec<Ref<Text>>>,
    pub structural_methods: Vec<(Ref<Text>, Vec<Ref<StructuralMethodExport>>)>,
    pub class_field_defaults: BTreeMap<Ref<Text>, Vec<(u32, Ref<HirExpr>)>>,
    pub declaration_metadata: Vec<Ref<TypedDeclarationMetadata>>,
    pub class_adapter_providers: BTreeMap<Ref<Text>, Ref<ClassAdapterProviderDeclaration>>,
    pub class_adapter_markers: BTreeMap<Ref<Text>, Ref<ClassAdapterMarkerDeclaration>>,
    pub attached_api_sets: BTreeMap<Ref<Text>, Ref<AttachedApiSetDeclaration>>,
    pub attached_apis: BTreeMap<Ref<Text>, Ref<AttachedApiDeclaration>>,
    pub class_adapter_selections: BTreeMap<Ref<Text>, Ref<ClassAdapterSelection>>,
    pub descriptor_functions: BTreeMap<Ref<Text>, Ref<DeclarationDescriptorFunction>>,
    pub declaration_descriptors: Vec<Ref<TypedDeclarationDescriptor>>,
    pub applied_adapter_metadata: Vec<Ref<AppliedAdapterMetadata>>,
    pub const_functions: BTreeMap<Ref<Text>, Ref<HirFunction>>,
    pub specialization_requests: Vec<Ref<ConstSpecializationRequest>>,
    pub specialization_outputs: Vec<Ref<StaticSpecializationOutput>>,
    pub json_integer_boundary_requests: Vec<Ref<JsonIntegerBoundaryRequest>>,
    pub constants: BTreeMap<Ref<Text>, Ref<Type>>,
    pub constant_integer_values: BTreeMap<Ref<Text>, Ref<Text>>,
    pub error_types: BTreeSet<Ref<Text>>,
    pub type_param_bounds: TypeParameterBounds,
    pub generic_functions: BTreeMap<Ref<Text>, Vec<Ref<Text>>>,
    pub function_varargs: BTreeMap<Ref<Text>, u32>,
    pub function_python_call_shapes: BTreeMap<Ref<Text>, Vec<Ref<PythonParameterKind>>>,
    pub rust_threadsafe_callback_targets: BTreeMap<Ref<Text>, Vec<u32>>,
    pub function_workloads: BTreeMap<Ref<Text>, Ref<Text>>,
    pub function_defaults: BTreeMap<Ref<Text>, Vec<(u32, Ref<HirExpr>)>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralMethodExport {
    pub handler_target: Option<Ref<CallableIdentity>>,
    pub name: Ref<Text>,
    pub params: Vec<Ref<HirParam>>,
    pub return_type: Ref<Type>,
    pub is_async: bool,
    pub method_kind: Ref<MethodKind>,
    pub receiver: Option<Ref<ReceiverConvention>>,
}

pub type GenericAliasExport = (Vec<Ref<Text>>, Ref<Type>);
