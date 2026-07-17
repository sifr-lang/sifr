//! Core type definitions for the Sifr type system.

mod definitions;
pub use definitions::*;
mod display_impl;
mod python_interop;
mod rust_trait_capabilities;
mod source_names;
pub use source_names::{
    class_rust_name, source_class_rust_name, stdlib_class_rust_name, COMPILER_RUST_PATH_ROOTS,
    GLOBAL_RUST_NOMINAL_NAMES,
};
mod type_queries;
mod type_rendering;
mod union_identity;
