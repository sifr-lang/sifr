//! Generated Rust syntax, ownership, visibility and support dependency transformations.
//! Language-level type identities and catalogs are supplied by codegen.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
mod generated_rust_canonicalizer;
pub use generated_rust_canonicalizer::*;
mod discardability;
mod error_refs;
pub mod generated_visibility;
pub mod stdlib_filter;
mod task_local_support;
pub use error_refs::collect_source_builtin_error_classes;

#[cfg(test)]
#[path = "../tests/unit/generated_rust_canonicalizer_ownership_tests.rs"]
mod generated_rust_canonicalizer_ownership_tests;

#[cfg(test)]
#[path = "../tests/unit/generated_support_regression_tests.rs"]
mod generated_support_regression_tests;
