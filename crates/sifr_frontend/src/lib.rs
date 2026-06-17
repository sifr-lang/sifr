//! Canonical Sifr frontend query API.
//!
//! This crate owns project/session loading, source maps, module graph identity,
//! parse/lower/type-check diagnostics, process-local query caching, and
//! deterministic invalidation reports.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod cache_keys;
pub use cache_keys::*;
mod frontend_reuse;
pub use frontend_reuse::{FrontendCacheEntryIdentity, FrontendReuseStats};
mod graph_cache_and_queries;
pub use graph_cache_and_queries::*;
mod hir_views;
mod module_signatures;
mod query_diagnostic_rendering;
pub(crate) use query_diagnostic_rendering::{
    diagnostic_with_code, diagnostic_with_source_range, diagnostic_with_source_range_args_help,
};
mod query_diagnostics;
pub use query_diagnostics::*;
#[cfg(test)]
mod query_diagnostics_behavior_tests;
#[cfg(test)]
mod query_diagnostics_equivalence_tests;
mod source_provider;
pub use source_provider::*;
mod source_maps;
pub use source_maps::*;
mod workspace_session;
pub use workspace_session::*;
mod workspace_residency;
pub use workspace_residency::*;
mod workspace_trace;
pub use workspace_trace::*;
