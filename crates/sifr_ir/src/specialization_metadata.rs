//! Typed compiler records for package-neutral compile-time specialization.

use crate::{HirExpr, MethodKind};
use ruff_text_size::TextRange;
use sifr_type_system::{ParamConvention, ReceiverConvention, Type};

/// Opaque compiler-issued source-origin identity used only for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceOriginId {
    namespace: [u8; 32],
    index: u32,
}

impl SourceOriginId {
    #[must_use]
    pub const fn new(namespace: [u8; 32], index: u32) -> Self {
        Self { namespace, index }
    }

    #[must_use]
    pub fn belongs_to(self, namespace: [u8; 32]) -> bool {
        self.namespace == namespace
    }
}
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeclarationMetadataTargetKind {
    Type,
    Field,
    EnumVariant,
    Function,
    Method,
    Parameter,
}

/// General compile-time metadata. Package semantics are intentionally opaque to the compiler.
#[derive(Debug, Clone)]
pub struct TypedDeclarationMetadata {
    /// Stable local owner path (`Class`, `Class.method`, or a module function name).
    pub owner: String,
    pub target_kind: DeclarationMetadataTargetKind,
    /// Field, enum variant, or parameter name for targeted metadata.
    pub target_name: Option<String>,
    /// Package-qualified metadata key.
    pub key: String,
    pub value_type: Type,
    pub value: HirExpr,
    pub range: TextRange,
}

/// Return the canonical structural-identity spelling for a closed HIR value.
///
/// Lowering and code generation use this one predicate so a type cannot satisfy
/// the `Structural` bound unless code generation can preserve all of its
/// defaults and declaration metadata in the wire identity.
pub fn canonical_structural_identity_value(value: &HirExpr) -> Option<String> {
    match value {
        HirExpr::IntLiteral(value) => Some(format!("int:{value}")),
        HirExpr::LargeIntLiteral(value) => Some(format!("int:{value}")),
        HirExpr::FloatLiteral(value) => Some(format!("float:{:016x}", value.to_bits())),
        HirExpr::StringLiteral(value) => Some(format!("str:{}:{value}", value.len())),
        HirExpr::BoolLiteral(value) => Some(format!("bool:{value}")),
        HirExpr::NoneLiteral => Some("none".to_string()),
        HirExpr::ListLiteral { elements, .. } => canonical_sequence("list", elements),
        HirExpr::TupleLiteral { elements, .. } => canonical_sequence("tuple", elements),
        HirExpr::SetLiteral { elements, .. } => {
            let mut elements = elements
                .iter()
                .map(canonical_structural_identity_value)
                .collect::<Option<Vec<_>>>()?;
            elements.sort();
            Some(format!("set[{}]", elements.join(",")))
        }
        HirExpr::DictLiteral { keys, values, .. } if keys.len() == values.len() => {
            let mut entries = keys
                .iter()
                .zip(values)
                .map(|(key, value)| {
                    Some(format!(
                        "{}={}",
                        canonical_structural_identity_value(key)?,
                        canonical_structural_identity_value(value)?
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            entries.sort();
            Some(format!("dict[{}]", entries.join(",")))
        }
        _ => None,
    }
}

fn canonical_sequence(tag: &str, values: &[HirExpr]) -> Option<String> {
    let values = values
        .iter()
        .map(canonical_structural_identity_value)
        .collect::<Option<Vec<_>>>()?;
    Some(format!("{tag}[{}]", values.join(",")))
}

/// Return whether enum discriminants have one exact, non-overflowing structural encoding.
pub fn structural_identity_enum_variants_supported(variants: &[(String, Option<i64>)]) -> bool {
    if variants.is_empty() {
        return false;
    }
    let mut next = Some(1_i64);
    let mut seen = HashSet::new();
    for (_, declared) in variants {
        let Some(value) = declared.or(next) else {
            return false;
        };
        if !seen.insert(value) {
            return false;
        }
        next = value.checked_add(1);
    }
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstSpecializationRequest {
    pub owner: String,
    pub package_module: String,
    pub function: String,
    pub range: TextRange,
}

/// The declaration location accepted by one package descriptor function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeclarationDescriptorKind {
    Field,
    Class,
    Method,
    Type,
}

/// Canonical identity of a package-owned class-adapter provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassAdapterProviderDeclaration {
    pub module: String,
    pub function: String,
    pub descriptor_module: String,
    pub descriptor_symbol: String,
    pub descriptor_type: Type,
    pub range: TextRange,
}

/// A field-less compile-time marker that selects one canonical adapter provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassAdapterMarkerDeclaration {
    pub module: String,
    pub symbol: String,
    pub provider_module: String,
    pub provider_function: String,
    pub descriptor_type: Type,
    pub range: TextRange,
}

/// Canonical identity of one erased package-owned attached-API namespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AttachedApiSetIdentity {
    pub module: String,
    pub symbol: String,
}

/// A field-less compile-time namespace that owns a fixed attached-API set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachedApiSetDeclaration {
    pub identity: AttachedApiSetIdentity,
    pub range: TextRange,
}

/// Receiver form selected by one attached package function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AttachedApiReceiver {
    Type,
    Immutable,
    Mutable,
    Owned,
}

/// Checked package function exported as one member of an attached-API set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachedApiDeclaration {
    pub module: String,
    pub function: String,
    pub set: AttachedApiSetIdentity,
    pub public_name: String,
    pub receiver: AttachedApiReceiver,
    pub owner_type_param: String,
    pub type_params: Vec<String>,
    pub type_param_bounds: std::collections::BTreeMap<String, Vec<String>>,
    pub function_type: sifr_type_system::FunctionType,
    pub range: TextRange,
}

/// The canonical provider selected for one source class before finalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassAdapterSelection {
    pub owner: String,
    pub provider_module: String,
    pub provider_function: String,
    pub descriptor_type: Type,
    pub marker_identities: Vec<String>,
    pub data_parent: Option<String>,
    pub field_plans: Vec<AdapterFieldPlan>,
    pub handler_plans: Vec<AdapterHandlerPlan>,
    pub attached_api_set: Option<AttachedApiSetIdentity>,
    pub adapter_invocation_identity: [u8; 32],
    pub post_adapter_identity: [u8; 32],
    pub range: TextRange,
}

/// One package-selected user method carried from adaptation into specialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterHandlerPlan {
    pub callable: CallableIdentity,
    pub descriptor_type: Type,
    pub descriptor_value: StaticProgramValue,
    pub descriptor_origin: SourceOriginId,
    pub descriptor_range: TextRange,
    pub declaration_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterFieldPlan {
    pub identity: String,
    pub name: String,
    pub declared_type: Type,
    pub default: AdapterFieldDefault,
    pub validation_policy: Option<StaticProgramValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterFieldDefault {
    Required,
    Const(StaticProgramValue),
    Factory(CallableIdentity),
}

/// Canonical declaration exported by a package descriptor function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationDescriptorFunction {
    pub module: String,
    pub function: String,
    pub provider_module: String,
    pub provider_function: String,
    pub descriptor_type: Type,
    pub return_type: Type,
    pub kind: DeclarationDescriptorKind,
    pub range: TextRange,
}

/// Compiler-sealed identity for a statically checked callable const value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CallableIdentity {
    pub module: String,
    pub owner: Option<String>,
    pub symbol: String,
    pub generic_arguments: Vec<String>,
    pub signature: String,
}

/// One evaluated descriptor attached to an existing declaration target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedDeclarationDescriptor {
    pub owner: String,
    pub target_kind: DeclarationDescriptorKind,
    pub target_identity: String,
    /// Sealed checked method identity for method descriptors only.
    pub target_callable: Option<CallableIdentity>,
    pub provider_module: String,
    pub provider_function: String,
    pub value_type: Type,
    pub value: StaticProgramValue,
    pub range: TextRange,
}

/// Typed metadata produced by a validated class-adapter plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedAdapterMetadata {
    pub owner: String,
    pub target_kind: DeclarationMetadataTargetKind,
    pub target_name: Option<String>,
    pub key: String,
    pub value_type: Type,
    pub value: StaticProgramValue,
}

/// Closed package-neutral value retained from successful const specialization.
///
/// Integers use canonical decimal text so later compilation stages do not need
/// an arbitrary-precision integer representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticProgramValue {
    None,
    Bool(bool),
    Integer(String),
    FloatBits(u64),
    String(String),
    Bytes(Vec<u8>),
    Tuple(Vec<Self>),
    List(Vec<Self>),
    Record(Vec<(String, Self)>),
    CallableIdentity(CallableIdentity),
}

/// A package-owned, compile-time specialization result retained by the frontend.
///
/// The value uses the compiler's deterministic closed-value encoding so later
/// compilation stages can consume it without re-running package code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticSpecializationOutput {
    pub owner: String,
    pub package_module: String,
    pub function: String,
    pub canonical_value: String,
    pub value: StaticProgramValue,
    /// Complete deterministic identity for cache keys and emitted envelopes.
    pub program_identity: [u8; 32],
    pub structural_contract_version: u32,
    /// Package-selected, compiler-resolved methods in static invocation order.
    pub method_slots: Vec<StaticMethodSlot>,
    /// The one context contract derived from the selected methods. This is
    /// present exactly when `method_slots` is nonempty.
    pub method_slot_context: Option<StaticMethodSlotContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticMethodSlotContext {
    None,
    Shared(Type),
    Mutable(Type),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticMethodSlot {
    /// Exact module-qualified nominal owner identity.
    pub owner_identity: String,
    /// Concrete owner type used by generated monomorphic glue.
    pub owner_type: Type,
    /// Source spelling retained in the static program slot reference.
    pub name: String,
    /// HIR/Rust spelling (`new` for a source `__init__`).
    pub hir_name: String,
    pub method_kind: MethodKind,
    pub receiver: Option<ReceiverConvention>,
    pub params: Vec<StaticMethodParam>,
    pub return_type: Type,
    pub is_async: bool,
    pub input_type: Type,
    pub output_type: Type,
    pub context_type: Option<Type>,
    pub context_mutable: bool,
    /// Package method descriptor retained for handler-aware code generation.
    pub descriptor_type: Option<Type>,
    pub descriptor_value: Option<StaticProgramValue>,
    pub descriptor_origin: Option<SourceOriginId>,
    pub descriptor_range: Option<TextRange>,
    pub declaration_order: Option<usize>,
    pub is_fallible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticMethodParam {
    pub name: String,
    pub ty: Type,
    pub keyword_only: bool,
    pub convention: ParamConvention,
}

#[derive(Debug, Clone)]
pub struct JsonIntegerBoundaryRequest {
    pub owner: String,
    pub field: String,
    pub profile: Option<String>,
    pub representation: String,
    pub static_minimum: Option<num_bigint::BigInt>,
    pub static_maximum: Option<num_bigint::BigInt>,
    pub range: TextRange,
}
