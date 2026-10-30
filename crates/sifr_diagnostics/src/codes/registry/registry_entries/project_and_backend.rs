//! Workspace, build, and internal compiler diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-IO-0801",
        "IO",
        "Text-mode open requires an explicit encoding.",
        Severity::Error,
        "text-mode open requires an explicit encoding; Sifr does not use locale-derived default encodings",
        "sifr_lowering::lower::expressions::call_shadowable_builtins",
        [],
        []
    ),
    active_entry!(
        "SIFR-IO-0802",
        "IO",
        "Open mode must be statically known.",
        Severity::Error,
        "open mode must be a string literal so Sifr can choose a binary or text handle type",
        "sifr_lowering::lower::expressions::call_shadowable_builtins",
        [],
        []
    ),
    active_entry!(
        "SIFR-ENCODING-0803",
        "ENCODING",
        "Encoding error handler must be statically known.",
        Severity::Error,
        "encoding error handlers must be statically known typed values",
        "sifr_lowering::lower::bytes_methods",
        [],
        []
    ),
    active_entry!(
        "SIFR-WORKSPACE-0001",
        "WORKSPACE",
        "Malformed workspace manifest.",
        Severity::Error,
        "could not parse workspace manifest at {path}: {reason}",
        "sifr_driver::workspace",
        [arg!("path"), arg!("reason")],
        ["path", "reason"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0002",
        "WORKSPACE",
        "Workspace source root escapes the workspace root.",
        Severity::Error,
        "source root {path} escapes the workspace root",
        "sifr_driver::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0003",
        "WORKSPACE",
        "Workspace source root is not a directory.",
        Severity::Error,
        "source root {path} is not a directory",
        "sifr_driver::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0004",
        "WORKSPACE",
        "Workspace source root entry has an invalid shape or path.",
        Severity::Error,
        "invalid source root entry {entry}",
        "sifr_driver::workspace",
        [arg!("entry")],
        ["entry"]
    ),
    active_entry!(
        "SIFR-BUILD-0002",
        "BUILD",
        "Build file materialization failed.",
        Severity::Error,
        "failed to materialize build file {path}",
        "sifr_driver::build::materialize",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-BUILD-0003",
        "BUILD",
        "Temporary build workspace creation failed.",
        Severity::Error,
        "failed to create temporary build workspace {path}",
        "sifr_driver::build::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-BUILD-0004",
        "BUILD",
        "Cargo manifest generation failed.",
        Severity::Error,
        "failed to generate Cargo manifest at {path}",
        "sifr_driver::build::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-BUILD-0005",
        "BUILD",
        "Rustc or Cargo execution failed.",
        Severity::Error,
        "{tool} failed with exit status {status}",
        "sifr_driver::build::workspace",
        [arg!("tool"), arg!("status")],
        ["tool", "status"]
    ),
    active_entry!(
        "SIFR-BUILD-0006",
        "BUILD",
        "Expected build artifact was not produced.",
        Severity::Error,
        "expected build artifact {path} was not produced",
        "sifr_driver::build::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-BUILD-0901",
        "BUILD",
        "Standalone install receipt is missing or outside the self-update install rules.",
        Severity::Error,
        "{message}",
        "sifr::self_update_receipt",
        [arg!("message")],
        ["message"]
    ),
    active_entry!(
        "SIFR-INTERNAL-0001",
        "INTERNAL",
        "Unclassified compiler panic after a panic boundary.",
        Severity::Error,
        "internal compiler error",
        "sifr_driver::diagnostics",
        [],
        []
    ),
    active_entry!(
        "SIFR-INTERNAL-0002",
        "INTERNAL",
        "Structured recovery-cap omission summary.",
        Severity::Note,
        "{omitted_count} additional {omitted_kind} omitted by recovery cap ({cap_kind})",
        "sifr_driver::diagnostics",
        [
            arg!("omitted_count"),
            arg!("omitted_kind"),
            arg!("cap_kind")
        ],
        ["cap_kind"]
    ),
];
