//! Sifr HIR (High-level Intermediate Representation)
//!
//! Lowers the untyped Python AST into a typed IR with:
//! - Name resolution (every name reference resolved to a definition)
//! - Type checking (every expression carries its resolved type)
//! - Ownership tracking (move vs copy semantics)

mod hir_nodes;
mod lower;
mod scope;

pub use hir_nodes::*;
pub use lower::{lower_module, LoweringError};
pub use scope::Scope;
