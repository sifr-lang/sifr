//! Sifr intermediate representation data contracts.
//!
//! This crate owns immutable HIR, CFG, flow-graph, and lowering result data.
//! Lowering construction algorithms remain in the producer crate.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

pub mod cfg;
pub mod diagnostic_types;
pub mod flow_graph;
pub mod hir_nodes;
pub mod lowering_outcome;
pub mod lowering_result;

pub use cfg::*;
pub use diagnostic_types::*;
pub use flow_graph::*;
pub use hir_nodes::*;
pub use lowering_outcome::LoweringOutcome;
pub use lowering_result::LoweringResult;
