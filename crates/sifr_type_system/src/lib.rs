//! Sifr Type System
//!
//! Defines the type representations, type inference, type checking,
//! and subtyping rules for the Sifr language.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod check;
#[cfg(test)]
mod check_equality_capability_tests;
mod collection_capabilities;
pub mod infer;
pub mod literal;
#[cfg(test)]
mod type_capability_identity_tests;
mod types;
pub mod union;

pub use check::{
    type_check_binary_op, type_check_bool_op, type_check_comparison, type_check_unary_op,
};
pub use infer::infer_literal_type;
pub use types::{
    class_rust_name, is_global_rust_nominal_identity, source_class_rust_name,
    stdlib_class_rust_name, FixedIntType, FunctionType, IterationCapability, IterationMetadata,
    OwnershipKind, ParamConvention, ParamMutability, ParamOwnership, PythonArrowKind, Type,
    COMPILER_RUST_PATH_ROOTS, GLOBAL_RUST_NOMINAL_IDENTITIES,
};
pub mod narrow;
pub use literal::{widen as widen_literal, LiteralValue};
pub use narrow::{narrow_type, NarrowingCondition};
pub use union::{
    intersect_with_union, make_union, remove_none_from_union, subtract_from_union, union_contains,
    union_contains_none,
};
