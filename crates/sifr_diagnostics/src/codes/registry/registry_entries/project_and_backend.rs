//! Workspace, build, and internal compiler diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-IO-0801",
        "IO",
        "Text-mode open requires an explicit encoding.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/text_i18n_open_without_encoding.sifr",
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
        "crates/sifr/tests/e2e/fail/text_i18n_open_dynamic_mode.sifr",
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
        "crates/sifr/tests/e2e/fail/text_i18n_dynamic_errors_handler.sifr",
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
            "crates/sifr/tests/verification/project/workspace_malformed_manifest",
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
            "crates/sifr_driver/src/tests/discovery_and_workspace.rs",
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
            "crates/sifr_driver/src/tests/discovery_and_workspace.rs",
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
            "crates/sifr_driver/src/tests/discovery_and_workspace.rs",
            "invalid source root entry {entry}",
            "sifr_driver::workspace",
            [arg!("entry")],
            ["entry"]
        ),
    active_entry!(
            "SIFR-WORKSPACE-0101",
            "WORKSPACE",
            "Legacy workspace import target could not be resolved; source imports use SIFR-IMPORT-0002.",
            Severity::Error,
            "crates/sifr/tests/verification/project/workspace_unresolved_import",
            "could not resolve import {module}",
            "sifr_driver::project::discovery",
            [arg!("module"), json_arg!("searched_paths")],
            ["module", "searched_paths"]
        ),
    active_entry!(
            "SIFR-WORKSPACE-0102",
            "WORKSPACE",
            "Legacy workspace import target is ambiguous; source imports use SIFR-IMPORT-0005.",
            Severity::Error,
            "crates/sifr/tests/verification/project/workspace_ambiguous_import",
            "module {module} is ambiguous in workspace {workspace}",
            "sifr_driver::project::discovery",
            [
                arg!("module"),
                arg!("workspace"),
                json_arg!("candidate_paths")
            ],
            ["module", "workspace", "candidate_paths"]
        ),
    active_entry!(
            "SIFR-WORKSPACE-0103",
            "WORKSPACE",
            "Legacy workspace namespace package collision; source imports use SIFR-IMPORT-0006.",
            Severity::Error,
            "crates/sifr_driver/src/tests/discovery_and_workspace.rs",
            "module {module} collides with namespace path {path}",
            "sifr_driver::project::discovery",
            [arg!("module"), arg!("path")],
            ["module", "path"]
        ),
    active_entry!(
            "SIFR-WORKSPACE-0104",
            "WORKSPACE",
            "Legacy workspace import graph cycle; source imports use SIFR-IMPORT-0007.",
            Severity::Error,
            "crates/sifr_driver/src/tests/project_graph.rs",
            "workspace import cycle detected: {cycle}",
            "sifr_driver::project::compile_order",
            [arg!("cycle")],
            ["cycle"]
        ),
    active_entry!(
            "SIFR-BUILD-0002",
            "BUILD",
            "Build file materialization failed.",
            Severity::Error,
            "crates/sifr_driver/src/tests/project_build_check.rs",
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
            "crates/sifr_driver/src/tests/project_build_check.rs",
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
            "crates/sifr_driver/src/tests/project_build_check.rs",
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
            "crates/sifr_driver/src/tests/project_build_check.rs",
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
            "crates/sifr_driver/src/tests/project_build_check.rs",
            "expected build artifact {path} was not produced",
            "sifr_driver::build::workspace",
            [arg!("path")],
            ["path"]
        ),
    active_entry!(
            "SIFR-BUILD-0901",
            "BUILD",
            "Standalone install receipt is missing or outside the self-update contract.",
            Severity::Error,
            "crates/sifr/src/self_update_receipt.rs",
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
            "crates/sifr_driver/src/tests/panic_boundary.rs::planned_internal_0001",
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
            "crates/sifr_driver/src/tests/diagnostics.rs::test_apply_diagnostic_recovery_limits_summarizes_similar_diagnostics",
            "{omitted_count} additional {omitted_kind} omitted by recovery cap ({cap_kind})",
            "sifr_driver::diagnostics",
            [arg!("omitted_count"), arg!("omitted_kind"), arg!("cap_kind")],
            ["cap_kind"]
        ),
];
