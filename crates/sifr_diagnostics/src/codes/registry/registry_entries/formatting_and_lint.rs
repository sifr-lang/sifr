//! Formatter and lint diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-FMT-0001",
        "FMT",
        "Source formatting drift detected by sifr fmt --check.",
        Severity::Error,
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
        "unknown Sifr policy rule id '{rule}'",
        "sifr_lint",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0002",
        "LINT",
        "Suppression did not suppress any diagnostic.",
        Severity::Warning,
        "unused Sifr suppression for policy rule '{rule}'",
        "sifr_lint",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0003",
        "LINT",
        "Suppression must list explicit Sifr policy rule ids.",
        Severity::Warning,
        "sifr suppression must list explicit policy rule ids",
        "sifr_lint",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0004",
        "LINT",
        "Line ends with trailing horizontal whitespace.",
        Severity::Warning,
        "line has trailing whitespace",
        "sifr_lint",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0005",
        "LINT",
        "Comment contains a tracked TODO or FIXME marker.",
        Severity::Warning,
        "comment contains tracked task marker '{marker}'",
        "sifr_lint::rules::todo_comment",
        [arg!("rule"), arg!("marker")],
        ["rule", "marker"]
    ),
    active_entry!(
        "SIFR-LINT-0006",
        "LINT",
        "Call passes a boolean literal positionally.",
        Severity::Warning,
        "boolean literal passed as a positional argument",
        "sifr_lint::rules::boolean_positional_argument",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0007",
        "LINT",
        "Function has more parameters than the policy limit.",
        Severity::Warning,
        "function '{function}' has {count} parameters, exceeding the policy limit of {limit}",
        "sifr_lint::rules::large_parameter_list",
        [arg!("rule"), arg!("function"), arg!("count"), arg!("limit")],
        ["rule", "function", "count", "limit"]
    ),
    active_entry!(
        "SIFR-LINT-0008",
        "LINT",
        "Import duplicates a module/name pair already imported in the same source file.",
        Severity::Warning,
        "duplicate import of '{import}'",
        "sifr_lint::rules::duplicate_import",
        [arg!("rule"), arg!("import")],
        ["rule", "import"]
    ),
];
