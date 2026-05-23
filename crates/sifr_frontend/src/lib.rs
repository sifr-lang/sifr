//! Canonical Sifr frontend query API.
//!
//! This crate owns project/session loading, source maps, module graph identity,
//! parse/lower/type-check diagnostics, process-local query caching, and
//! deterministic invalidation reports.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

include!("lib/graph_cache_and_queries.rs");
include!("lib/query_diagnostics.rs");
