//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::{
    BTreeMap, FunctionType, HirExpr, MethodKind, ParamConvention, ReceiverConvention, Ref,
    SourceLocation, Text, TextRange, Type,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceOriginId {
    pub location: Ref<SourceLocation>,
    pub local_ordinal: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DeclarationMetadataTargetKind {
    Type,
    Field,
    EnumVariant,
    Function,
    Method,
    Parameter,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedDeclarationMetadata {
    pub owner: Ref<Text>,
    pub target_kind: Ref<DeclarationMetadataTargetKind>,
    pub target_name: Option<Ref<Text>>,
    pub key: Ref<Text>,
    pub value_type: Ref<Type>,
    pub value: Ref<HirExpr>,
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstSpecializationRequest {
    pub owner: Ref<Text>,
    pub package_module: Ref<Text>,
    pub function: Ref<Text>,
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DeclarationDescriptorKind {
    Field,
    Class,
    Method,
    Type,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassAdapterProviderDeclaration {
    pub module: Ref<Text>,
    pub function: Ref<Text>,
    pub descriptor_module: Ref<Text>,
    pub descriptor_symbol: Ref<Text>,
    pub descriptor_type: Ref<Type>,
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassAdapterMarkerDeclaration {
    pub module: Ref<Text>,
    pub symbol: Ref<Text>,
    pub provider_module: Ref<Text>,
    pub provider_function: Ref<Text>,
    pub descriptor_type: Ref<Type>,
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachedApiSetIdentity {
    pub module: Ref<Text>,
    pub symbol: Ref<Text>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachedApiSetDeclaration {
    pub identity: Ref<AttachedApiSetIdentity>,
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AttachedApiReceiver {
    Type,
    Immutable,
    Mutable,
    Owned,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachedApiDeclaration {
    pub module: Ref<Text>,
    pub function: Ref<Text>,
    pub set: Ref<AttachedApiSetIdentity>,
    pub public_name: Ref<Text>,
    pub receiver: Ref<AttachedApiReceiver>,
    pub owner_type_param: Ref<Text>,
    pub type_params: Vec<Ref<Text>>,
    pub type_param_bounds: BTreeMap<Ref<Text>, Vec<Ref<Text>>>,
    pub function_type: Ref<FunctionType>,
    pub defaults: Vec<(u32, Ref<HirExpr>)>,
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassAdapterSelection {
    pub owner: Ref<Text>,
    pub provider_module: Ref<Text>,
    pub provider_function: Ref<Text>,
    pub descriptor_type: Ref<Type>,
    pub marker_identities: Vec<Ref<Text>>,
    pub data_parent: Option<Ref<Text>>,
    pub field_plans: Vec<Ref<AdapterFieldPlan>>,
    pub handler_plans: Vec<Ref<AdapterHandlerPlan>>,
    pub attached_api_set: Option<Ref<AttachedApiSetIdentity>>,
    pub adapter_invocation_identity: [u8; 32],
    pub post_adapter_identity: [u8; 32],
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterHandlerPlan {
    pub callable: Ref<CallableIdentity>,
    pub descriptor_type: Ref<Type>,
    pub descriptor_value: Ref<StaticProgramValue>,
    pub descriptor_origin: Ref<SourceOriginId>,
    pub descriptor_range: Ref<TextRange>,
    pub declaration_order: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterFieldPlan {
    pub identity: Ref<Text>,
    pub name: Ref<Text>,
    pub declared_type: Ref<Type>,
    pub default: Ref<AdapterFieldDefault>,
    pub validation_policy: Option<Ref<StaticProgramValue>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AdapterFieldDefault {
    Required,
    Const(Ref<StaticProgramValue>),
    Factory(Ref<CallableIdentity>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationDescriptorFunction {
    pub module: Ref<Text>,
    pub function: Ref<Text>,
    pub provider_module: Ref<Text>,
    pub provider_function: Ref<Text>,
    pub descriptor_type: Ref<Type>,
    pub return_type: Ref<Type>,
    pub kind: Ref<DeclarationDescriptorKind>,
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallableIdentity {
    pub module: Ref<Text>,
    pub owner: Option<Ref<Text>>,
    pub symbol: Ref<Text>,
    pub generic_arguments: Vec<Ref<Text>>,
    pub signature: Ref<Text>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedDeclarationDescriptor {
    pub owner: Ref<Text>,
    pub target_kind: Ref<DeclarationDescriptorKind>,
    pub target_identity: Ref<Text>,
    pub target_callable: Option<Ref<CallableIdentity>>,
    pub provider_module: Ref<Text>,
    pub provider_function: Ref<Text>,
    pub value_type: Ref<Type>,
    pub value: Ref<StaticProgramValue>,
    pub range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedAdapterMetadata {
    pub owner: Ref<Text>,
    pub target_kind: Ref<DeclarationMetadataTargetKind>,
    pub target_name: Option<Ref<Text>>,
    pub key: Ref<Text>,
    pub value_type: Ref<Type>,
    pub value: Ref<StaticProgramValue>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StaticProgramValue {
    None,
    Bool(bool),
    Integer(Ref<Text>),
    FloatBits(u64),
    String(Ref<Text>),
    Bytes(Vec<u8>),
    Tuple(Vec<Ref<StaticProgramValue>>),
    List(Vec<Ref<StaticProgramValue>>),
    Record(Vec<(Ref<Text>, Ref<StaticProgramValue>)>),
    CallableIdentity(Ref<CallableIdentity>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticSpecializationOutput {
    pub owner: Ref<Text>,
    pub package_module: Ref<Text>,
    pub function: Ref<Text>,
    pub canonical_value: Ref<Text>,
    pub value: Ref<StaticProgramValue>,
    pub program_identity: [u8; 32],
    pub structural_contract_version: u32,
    pub method_slots: Vec<Ref<StaticMethodSlot>>,
    pub method_slot_context: Option<Ref<StaticMethodSlotContext>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StaticMethodSlotContext {
    None,
    Shared(Ref<Type>),
    Mutable(Ref<Type>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StaticMethodSlotInputRole {
    Value,
    Receiver,
    ReceiverAndValue,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticMethodSlot {
    pub owner_identity: Ref<Text>,
    pub owner_type: Ref<Type>,
    pub name: Ref<Text>,
    pub hir_name: Ref<Text>,
    pub method_kind: Ref<MethodKind>,
    pub receiver: Option<Ref<ReceiverConvention>>,
    pub params: Vec<Ref<StaticMethodParam>>,
    pub return_type: Ref<Type>,
    pub is_async: bool,
    pub input_role: Ref<StaticMethodSlotInputRole>,
    pub input_type: Ref<Type>,
    pub output_type: Ref<Type>,
    pub context_type: Option<Ref<Type>>,
    pub context_mutable: bool,
    pub descriptor_type: Option<Ref<Type>>,
    pub descriptor_value: Option<Ref<StaticProgramValue>>,
    pub descriptor_origin: Option<Ref<SourceOriginId>>,
    pub descriptor_range: Option<Ref<TextRange>>,
    pub declaration_order: Option<u32>,
    pub is_fallible: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticMethodParam {
    pub name: Ref<Text>,
    pub ty: Ref<Type>,
    pub keyword_only: bool,
    pub convention: Ref<ParamConvention>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonIntegerBoundaryRequest {
    pub owner: Ref<Text>,
    pub field: Ref<Text>,
    pub profile: Option<Ref<Text>>,
    pub representation: Ref<Text>,
    pub static_minimum: Option<Ref<Text>>,
    pub static_maximum: Option<Ref<Text>>,
    pub range: Ref<TextRange>,
}
