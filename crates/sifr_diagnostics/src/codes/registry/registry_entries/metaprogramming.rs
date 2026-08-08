//! Deterministic package metaprogramming diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-META-0001",
        "META",
        "Package const specialization reported a fatal issue.",
        Severity::Error,
        "crates/sifr_frontend/src/const_specialization.rs",
        "package {package} specialization failed: {reason_code}",
        "sifr_frontend::const_specialization",
        [arg!("package"), arg!("reason_code")],
        ["package", "reason_code"]
    ),
    active_entry!(
        "SIFR-META-0002",
        "META",
        "Package const specialization reported a hard warning.",
        Severity::Warning,
        "crates/sifr_frontend/src/const_specialization.rs",
        "package {package} specialization warning: {reason_code}",
        "sifr_frontend::const_specialization",
        [arg!("package"), arg!("reason_code")],
        ["package", "reason_code"]
    ),
    active_entry!(
        "SIFR-META-0003",
        "META",
        "Package const-specialization issue declaration is malformed.",
        Severity::Error,
        "crates/sifr_frontend/src/const_specialization.rs",
        "package {package} declared malformed specialization issue {reason_code}: {declaration_problem}",
        "sifr_frontend::const_specialization",
        [
            arg!("package"),
            arg!("reason_code"),
            arg!("declaration_problem")
        ],
        ["package", "reason_code", "declaration_problem"]
    ),
];
