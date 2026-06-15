//! Sifr HIR (High-level Intermediate Representation)
//!
//! Lowers the untyped Python AST into a typed IR with:
//! - Name resolution (every name reference resolved to a definition)
//! - Type checking (every expression carries its resolved type)
//! - Ownership tracking (move vs copy semantics)
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

pub mod cfg;
pub mod flow_graph;
mod hir_nodes;
#[cfg(test)]
mod hir_snapshot_tests;
mod lower;
mod scope;

pub use hir_nodes::*;
pub use lower::{
    lower_module, lower_module_stdlib, lower_module_stdlib_with_externals,
    lower_module_with_externals, lower_module_with_externals_and_name,
    lower_module_with_externals_name_and_options, ExternalDefs, LoweringOptions,
};
pub use scope::{NarrowingSnapshot, Scope};
pub use sifr_ir::{
    HirDiagnostic, LoweringOutcome, LoweringResult, LoweringWarningDiagnostic, RevealTypeDiagnostic,
};
