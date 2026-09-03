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
mod receiver_mutation;
mod safe_optional;
#[cfg(test)]
mod structural_record_tests;
mod substitution;
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
    COMPILER_RUST_PATH_ROOTS, CRATE_ROOT_RUST_NOMINAL_IDENTITIES, FixedIntType, FunctionType,
    GLOBAL_RUST_NOMINAL_IDENTITIES, IterationCapability, IterationMetadata, OwnershipKind,
    ParamConvention, ParamMutability, ParamOwnership, PythonArrowKind, ReceiverConvention,
    StructuralRecordField, StructuralRecordType, Type, class_rust_name,
    is_crate_root_rust_nominal_identity, is_global_rust_nominal_identity, source_class_rust_name,
    stdlib_class_rust_name,
};
pub mod narrow;
pub use literal::{LiteralValue, widen as widen_literal};
pub use narrow::{NarrowingCondition, narrow_type};
pub use receiver_mutation::{
    ReceiverFactDomain, ReceiverFactInvalidation, ReceiverMutationEffect, ReceiverMutationSummary,
    receiver_mutation_summary,
};
pub use safe_optional::safe_optional_result;
pub use substitution::{
    UnionStructureClassScope, substitute_type_vars, substitute_type_vars_with_class_scopes,
    substitution_preserves_union_structure,
    substitution_preserves_union_structure_with_class_scopes,
};

pub const IO_ERROR_KIND_CASES: [(&str, &str); 6] = [
    ("FileNotFoundError", "FileNotFound"),
    ("PermissionError", "PermissionDenied"),
    ("FileExistsError", "FileExists"),
    ("IsADirectoryError", "IsADirectory"),
    ("NotADirectoryError", "NotADirectory"),
    ("DirectoryNotEmptyError", "DirectoryNotEmpty"),
];

pub fn io_error_kind(error_type: &str) -> Option<&'static str> {
    IO_ERROR_KIND_CASES
        .iter()
        .find_map(|(name, kind)| (*name == error_type).then_some(*kind))
}

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
