//! Generated Rust syntax, ownership, visibility and support dependency transformations.
//! Language-level type identities and catalogs are supplied by codegen.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
mod generated_rust_canonicalizer;
pub use generated_rust_canonicalizer::*;
mod discardability;
pub mod generated_visibility;
pub mod stdlib_filter;
pub mod task_local_support;
pub use discardability::*;
mod error_refs;
pub use error_refs::collect_source_builtin_error_classes;

#[cfg(test)]
#[path = "../tests/unit/generated_rust_canonicalizer_ownership_tests.rs"]
mod generated_rust_canonicalizer_ownership_tests;
