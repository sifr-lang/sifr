//! Typed compiler records for package-neutral compile-time specialization.

use crate::HirExpr;
use ruff_text_size::TextRange;
use sifr_type_system::Type;

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

#[derive(Debug, Clone)]
pub struct ConstSpecializationRequest {
    pub owner: String,
    pub package_module: String,
    pub function: String,
    pub range: TextRange,
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
