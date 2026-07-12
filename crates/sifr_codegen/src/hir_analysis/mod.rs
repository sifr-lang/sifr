//! Canonical HIR analysis ownership for `sifr_codegen`.
//!
//! Workflow for adding or extending analysis:
//! 1. Extend `traversal` when `HirExpr`/`HirStmt`/`HirPattern` variants change.
//! 2. Expose reusable semantics in `queries`.
//! 3. Consume query APIs from emitters/lowering; do not add local recursive descent.

pub(crate) mod queries;
pub(crate) mod traversal;
