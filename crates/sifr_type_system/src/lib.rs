//! Sifr Type System
//!
//! Defines the type representations, type inference, type checking,
//! and subtyping rules for the Sifr language.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod check;
mod collection_capabilities;
pub mod infer;
pub mod literal;
mod types;
pub mod union;

pub use check::{
    type_check_binary_op, type_check_bool_op, type_check_comparison, type_check_unary_op,
};
pub use infer::infer_literal_type;
pub use types::{
    FixedIntType, FunctionType, IterationCapability, IterationMetadata, OwnershipKind,
    ParamConvention, ParamMutability, ParamOwnership, Type,
};
pub mod narrow;
pub use literal::{widen as widen_literal, LiteralValue};
pub use narrow::{narrow_type, NarrowingCondition};
pub use union::{
    intersect_with_union, make_union, remove_none_from_union, subtract_from_union, union_contains,
    union_contains_none,
};
