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
mod safe_optional;
#[cfg(test)]
mod type_capability_identity_tests;
mod types;
pub mod union;
#[cfg(test)]
mod union_operation_tests;

pub use check::{
    type_check_binary_op, type_check_bool_op, type_check_comparison, type_check_unary_op,
};
pub use infer::infer_literal_type;
pub use types::{
    class_rust_name, is_crate_root_rust_nominal_identity, is_global_rust_nominal_identity,
    source_class_rust_name, stdlib_class_rust_name, FixedIntType, FunctionType,
    IterationCapability, IterationMetadata, OwnershipKind, ParamConvention, ParamMutability,
    ParamOwnership, PythonArrowKind, ReceiverConvention, Type, COMPILER_RUST_PATH_ROOTS,
    CRATE_ROOT_RUST_NOMINAL_IDENTITIES, GLOBAL_RUST_NOMINAL_IDENTITIES,
};
pub mod narrow;
pub use literal::{widen as widen_literal, LiteralValue};
pub use narrow::{narrow_type, NarrowingCondition};
pub use safe_optional::safe_optional_result;

/// Returns whether a declaration belongs to a public stdlib module's compiled
/// export surface. Underscore-prefixed declarations are private except for the
/// three CPython-compatible max-heap operations whose leading underscore is
/// their architecture-approved public name.
pub fn should_export_stdlib_declaration(module_name: &str, declaration_name: &str) -> bool {
    !declaration_name.starts_with('_')
        || matches!(
            (module_name, declaration_name),
            (
                "sifr.heapq",
                "_heapify_max" | "_heappop_max" | "_heapreplace_max"
            )
        )
}
pub use union::{
    intersect_with_union, make_union, remove_none_from_union, subtract_from_union, union_contains,
    union_contains_none,
};
