//! Core type definitions for the Sifr type system.

mod definitions;
pub use definitions::*;
mod display_impl;
mod python_interop;
mod rust_trait_capabilities;
mod source_names;
pub use source_names::{source_class_rust_name, COMPILER_RUST_PATH_ROOTS};
mod type_queries;
mod type_rendering;
mod union_identity;
