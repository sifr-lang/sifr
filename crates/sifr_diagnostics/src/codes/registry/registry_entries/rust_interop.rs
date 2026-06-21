//! Rust interop diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[active_entry!(
    "SIFR-RUST-CONFIG-0001",
    "RUST-CONFIG",
    "Rust interop decorator syntax is malformed.",
    Severity::Error,
    "crates/sifr_lowering/src/lower/rust_interop_tests.rs::rust_interop_rejects_string_target",
    "malformed Rust interop decorator: {reason}",
    "sifr_lowering::lower::rust_interop",
    [arg!("reason")],
    ["reason"]
)];
