//! Sifr HIR (High-level Intermediate Representation)
//!
//! Lowers the untyped Python AST into a typed IR with:
//! - Name resolution (every name reference resolved to a definition)
//! - Type checking (every expression carries its resolved type)
//! - Ownership tracking (move vs copy semantics)

mod hir_nodes;
mod lower;
mod scope;
pub mod cfg;
pub mod stdlib;

pub use hir_nodes::*;
pub use lower::{lower_module, lower_module_with_externals, ExternalDefs, LoweringError, LoweringResult};
pub use scope::{Scope, NarrowingSnapshot};
