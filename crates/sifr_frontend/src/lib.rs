//! Canonical Sifr frontend query API.
//!
//! This crate owns project/session loading, source maps, module graph identity,
//! parse/lower/type-check diagnostics, process-local query caching, and
//! deterministic invalidation reports.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod cache_keys;
mod callable_exports;
mod class_declarations;
pub use class_declarations::SourceOriginId;
mod callable_identities;
mod const_specialization;
mod descriptor_exports;
mod typed_descriptors;
pub use const_specialization::*;
mod const_canonical;
mod const_evaluator;
pub use const_evaluator::*;
mod structural_shape;
pub use structural_shape::*;
mod class_method_exports;
mod slot_table;
#[cfg(test)]
mod slot_table_tests;
mod specialization_runner;
mod specialization_support;
pub use cache_keys::*;
mod analysis_views;
pub use analysis_views::*;
mod editor_semantics;
pub use editor_semantics::*;
mod export_type_localization;
mod frontend_reuse;
pub use frontend_reuse::{FrontendCacheEntryIdentity, FrontendReuseStats};
mod graph_cache_and_queries;
pub use graph_cache_and_queries::*;
mod hir_views;
mod module_export_storage;
mod module_signatures;
mod package_issues;
mod query_diagnostic_rendering;
pub(crate) use query_diagnostic_rendering::{
    diagnostic_with_code, diagnostic_with_source_range, diagnostic_with_source_range_args_help,
    diagnostic_with_source_ranges_args_help,
};
mod warning_diagnostics;
pub use warning_diagnostics::{reveal_type_diagnostics, warning_diagnostics};
mod query_diagnostics;
pub use query_diagnostics::*;
#[cfg(test)]
mod query_diagnostics_behavior_tests;
#[cfg(test)]
mod query_diagnostics_equivalence_tests;
mod source_provider;
#[cfg(test)]
mod structural_shape_import_tests;
pub use source_provider::*;
mod source_maps;
pub use source_maps::*;
mod workspace_session;
pub use workspace_session::*;
mod workspace_residency;
pub use workspace_residency::*;
mod workspace_trace;
pub use workspace_trace::*;
