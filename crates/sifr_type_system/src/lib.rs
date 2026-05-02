//! Sifr Type System
//!
//! Defines the type representations, type inference, type checking,
//! and subtyping rules for the Sifr language.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod check;
pub mod infer;
pub mod literal;
mod types;
pub mod union;

pub use check::{
    type_check_binary_op, type_check_bool_op, type_check_comparison, type_check_unary_op,
};
pub use infer::infer_literal_type;
pub use types::{
    FunctionType, IterationCapability, IterationMetadata, OwnershipKind, ParamConvention,
    ParamMutability, ParamOwnership, Type,
};
pub mod narrow;
pub use literal::{widen as widen_literal, LiteralValue};
pub use narrow::{narrow_type, NarrowingCondition};
use sifr_diagnostics::DiagnosticCode;
pub use union::{
    intersect_with_union, make_union, remove_none_from_union, subtract_from_union, union_contains,
    union_contains_none,
};

/// A canonical diagnostic produced during type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCheckDiagnostic {
    pub code: DiagnosticCode,
    pub message: String,
}

impl std::fmt::Display for TypeCheckDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeCheckDiagnostic {}
