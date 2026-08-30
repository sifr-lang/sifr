//! Native Sifr Language Server Protocol adapter.
//!
//! This crate owns JSON-RPC/LSP transport, document/session state, protocol
//! conversions, and command dispatch. Semantic answers come from
//! `sifr_analysis`.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
#![allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]

mod analysis_workspace;
mod cancellation;
mod capabilities;
mod commands;
mod conversion;
mod diagnostic_jobs;
mod diagnostics;
mod document_events;
mod document_store;
mod errors;
mod notifications;
mod progress;
mod python_declarations;
mod request_queue;
mod requests;
mod scheduler;
mod server;
mod session;
mod settings;
mod sql_editor_contract;
mod watchdog;

pub use server::{run_stdio, run_stdio_with_options};
pub use watchdog::LspServerOptions;
