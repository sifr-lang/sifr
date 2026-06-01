//! Native Sifr Language Server Protocol adapter.
//!
//! This crate owns JSON-RPC/LSP transport, document/session state, protocol
//! conversions, and command dispatch. Semantic answers come from
//! `sifr_analysis`.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
#![allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]

mod analysis_workspace;
mod capabilities;
mod commands;
mod conversion;
mod diagnostics;
mod document_store;
mod errors;
mod notifications;
mod request_queue;
mod requests;
mod scheduler;
mod server;
mod session;
mod settings;

pub use server::run_stdio;
