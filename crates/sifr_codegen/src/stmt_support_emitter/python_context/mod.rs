mod async_context;
mod cancellation_scope;
mod outcome;
mod sync;
pub(crate) use sync::{rewrite_context_control_flow, rust_stmts_always_exit};

#[cfg(test)]
use crate::{HirStmt, RustEmitter, RustExpr, RustStmt, Type};
#[cfg(test)]
use sifr_ir::{HirWithItem, HirWithItemKind};
#[cfg(test)]
use sync::classify_cause_kind;

#[cfg(test)]
#[path = "../python_context_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../python_async_context_tests.rs"]
mod async_context_tests;
