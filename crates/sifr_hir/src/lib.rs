//! Sifr HIR (High-level Intermediate Representation)
//!
//! Lowers the untyped Python AST into a typed IR with:
//! - Name resolution (every name reference resolved to a definition)
//! - Type checking (every expression carries its resolved type)
//! - Ownership tracking (move vs copy semantics)

pub mod cfg;
mod hir_nodes;
mod lower;
mod scope;
pub mod stdlib;

pub use hir_nodes::*;
pub use lower::{
    lower_module, lower_module_stdlib, lower_module_stdlib_with_externals,
    lower_module_with_externals, ExternalDefs, LoweringError, LoweringResult,
};
pub use scope::{NarrowingSnapshot, Scope};
