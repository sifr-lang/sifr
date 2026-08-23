//! Core type definitions for the Sifr type system.

mod definitions;
pub use definitions::*;
mod display_impl;
mod error_contracts;
mod python_interop;
mod rust_trait_capabilities;
mod source_names;
pub use source_names::{
    COMPILER_RUST_PATH_ROOTS, CRATE_ROOT_RUST_NOMINAL_IDENTITIES, GLOBAL_RUST_NOMINAL_IDENTITIES,
    class_rust_name, is_crate_root_rust_nominal_identity, is_global_rust_nominal_identity,
    source_class_rust_name, stdlib_class_rust_name,
};
mod type_queries;
mod type_rendering;
mod union_identity;
