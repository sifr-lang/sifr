//! Rust interop diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-RUST-CONFIG-0001",
        "RUST-CONFIG",
        "Rust interop decorator syntax is malformed.",
        Severity::Error,
        "crates/sifr_lowering/src/lower/rust_interop_tests.rs::rust_interop_rejects_string_target",
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
        "crates/sifr_driver/src/build/rust_interop_tests.rs::package_rust_interop_rejects_unknown_target_root",
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
        "crates/sifr_driver/src/build/rust_interop_tests.rs::package_rust_interop_rejects_untrusted_build_script",
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
        "crates/sifr_driver/src/build/rust_interop_tests.rs::package_rust_interop_rejects_unrepresentable_probe_owner",
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
        "crates/sifr_driver/src/build/rust_interop_contract_tests.rs::package_rust_interop_opaque_close_policy_requires_close_method_contract",
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
        "crates/sifr_driver/src/build/rust_interop_async_contract_tests.rs::package_rust_interop_async_requires_send_future_by_default",
        "invalid Rust async contract: {reason}",
        "sifr_driver::build::rust_interop",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-RUST-PANIC-0001",
        "RUST-PANIC",
        "Rust panic boundary contract is invalid.",
        Severity::Error,
        "crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs::package_rust_interop_rejects_unknown_panic_policy",
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
        "crates/sifr_driver/src/build/rust_interop_tests.rs::package_rust_interop_requires_cargo_context",
        "Rust interop declarations require a Sifr package Cargo context",
        "sifr_driver::build::rust_interop",
        [],
        []
    ),
];
