//! Rust interop diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-RUST-CONFIG-0001",
        "RUST-CONFIG",
        "Rust interop decorator syntax is malformed.",
        Severity::Error,
        "malformed Rust interop decorator: {reason}",
        "sifr_lowering::lower::rust_interop",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-RESOLVE-0001",
        "RUST-RESOLVE",
        "Rust interop target root cannot be resolved.",
        Severity::Error,
        "unresolved Rust target root {root}",
        "sifr_driver::build::rust_interop",
        [arg!("root"), arg!("target")],
        ["root", "target"]
    ),
    active_entry!(
        "SIFR-RUST-TRUST-0001",
        "RUST-TRUST",
        "Rust interop trust declaration is missing.",
        Severity::Error,
        "missing Rust interop trust declaration for {target}",
        "sifr_driver::build::rust_interop",
        [arg!("target"), arg!("required_trust"), arg!("evidence")],
        ["target", "required_trust", "evidence"]
    ),
    active_entry!(
        "SIFR-RUST-TYPE-0001",
        "RUST-TYPE",
        "Rust bridge probe failed type-contract validation.",
        Severity::Error,
        "Rust bridge probe failed for {target}",
        "sifr_driver::build::rust_interop",
        [arg!("target")],
        ["target"]
    ),
    active_entry!(
        "SIFR-RUST-HANDLE-0001",
        "RUST-HANDLE",
        "Rust opaque handle contract is invalid.",
        Severity::Error,
        "opaque Rust handle {target} requires {method} cleanup method",
        "sifr_driver::build::rust_interop",
        [arg!("target"), arg!("method")],
        ["target", "method"]
    ),
    active_entry!(
        "SIFR-RUST-ASYNC-0001",
        "RUST-ASYNC",
        "Rust async interop contract is invalid.",
        Severity::Error,
        "invalid Rust async contract: {reason}",
        "sifr_driver::build::rust_interop",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-ZC-0001",
        "RUST-ZC",
        "Rust zero-copy or borrowed-view contract is invalid.",
        Severity::Error,
        "invalid Rust zero-copy/view contract: {reason}",
        "sifr_driver::build::rust_interop",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-CB-0001",
        "RUST-CB",
        "Rust callback lifetime, threading, or policy contract is invalid.",
        Severity::Error,
        "invalid Rust callback contract for {target}: {reason}",
        "sifr_driver::build::rust_interop",
        [arg!("target"), arg!("reason")],
        ["target", "reason"]
    ),
    active_entry!(
        "SIFR-RUST-SLOT-0001",
        "RUST-SLOT",
        "The reserved method-slot list is malformed.",
        Severity::Error,
        "invalid reserved method-slot list: {reason}",
        "sifr_frontend::specialization_runner",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-SLOT-0002",
        "RUST-SLOT",
        "A selected method-slot target is unavailable or unsupported.",
        Severity::Error,
        "invalid method-slot target: {reason}",
        "sifr_frontend::specialization_runner",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-SLOT-0003",
        "RUST-SLOT",
        "A selected method has an invalid method-slot signature.",
        Severity::Error,
        "invalid method-slot signature: {reason}",
        "sifr_frontend::specialization_runner",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-SLOT-0004",
        "RUST-SLOT",
        "A method-slot bridge bound is incomplete or misplaced.",
        Severity::Error,
        "invalid method-slot bridge bound: {reason}",
        "sifr_lowering::lower::rust_interop_structural",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-SLOT-0005",
        "RUST-SLOT",
        "A method-slot context type or borrow mode is invalid.",
        Severity::Error,
        "invalid method-slot context: {reason}",
        "sifr_frontend::specialization_runner",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-SLOT-0006",
        "RUST-SLOT",
        "A method-slot handler is used outside its affine call scope.",
        Severity::Error,
        "invalid method-slot handler contract: {reason}",
        "sifr_driver::build::rust_interop",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-PANIC-0001",
        "RUST-PANIC",
        "Rust panic boundary contract is invalid.",
        Severity::Error,
        "invalid Rust panic boundary contract: {reason}",
        "sifr_driver::build::rust_interop",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-CARGO-0001",
        "RUST-CARGO",
        "Rust interop Cargo metadata is unavailable or inconsistent.",
        Severity::Error,
        "Rust interop declarations require a Sifr package Cargo context",
        "sifr_driver::build::rust_interop",
        [],
        []
    ),
];
