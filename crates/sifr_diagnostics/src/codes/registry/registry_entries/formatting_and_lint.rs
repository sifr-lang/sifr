//! Formatter and lint diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-FMT-0001",
        "FMT",
        "Source formatting drift detected by sifr fmt --check.",
        Severity::Error,
        "crates/sifr_format/src/lib.rs::check_reports_formatting_drift",
        "source is not formatted with sifr fmt",
        "sifr_format",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-LINT-0001",
        "LINT",
        "Suppression references an unknown policy rule id.",
        Severity::Warning,
        "crates/sifr_lint/src/lib.rs::unknown_and_unused_suppressions_are_reported",
        "unknown Sifr policy rule id '{rule}'",
        "sifr_lint::suppressions",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0002",
        "LINT",
        "Suppression did not suppress any diagnostic.",
        Severity::Warning,
        "crates/sifr_lint/src/lib.rs::unknown_and_unused_suppressions_are_reported",
        "unused Sifr suppression for policy rule '{rule}'",
        "sifr_lint::suppressions",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0003",
        "LINT",
        "Suppression must list explicit Sifr policy rule ids.",
        Severity::Warning,
        "crates/sifr_lint/src/lib.rs::blanket_suppression_is_reported",
        "sifr suppression must list explicit policy rule ids",
        "sifr_lint::suppressions",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0004",
        "LINT",
        "Line ends with trailing horizontal whitespace.",
        Severity::Warning,
        "crates/sifr_lint/src/lib.rs::suppression_only_suppresses_matching_policy_rule",
        "line has trailing whitespace",
        "sifr_lint::rules::trailing_whitespace",
        [arg!("rule")],
        ["rule"]
    ),
];
