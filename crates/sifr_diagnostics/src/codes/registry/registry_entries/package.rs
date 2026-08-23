//! Cargo-backed Sifr package diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-PACKAGE-0001",
        "PACKAGE",
        "Missing or invalid Cargo Sifr discovery metadata.",
        Severity::Error,
        "crates/sifr_package/src/manifest/metadata.rs::tests",
        "invalid [package.metadata.sifr]: {reason}",
        "sifr_package::manifest::metadata",
        [arg!("reason"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0002",
        "PACKAGE",
        "Missing or invalid sifr.toml package manifest.",
        Severity::Error,
        "crates/sifr_package/src/manifest/sifr.rs::tests",
        "invalid sifr.toml: {reason}",
        "sifr_package::manifest::sifr",
        [
            arg!("reason"),
            json_arg!("cargo_package_id"),
            json_arg!("manifest_path")
        ],
        ["cargo_package_id", "manifest_path", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0003",
        "PACKAGE",
        "Unsupported Sifr compiler metadata appears in Cargo metadata.",
        Severity::Error,
        "crates/sifr_package/src/manifest/metadata.rs::tests",
        "unsupported Sifr compiler metadata in Cargo metadata: {key}",
        "sifr_package::manifest::metadata",
        [arg!("key"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "key"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0103",
        "PACKAGE",
        "Cargo metadata parsing or normalization failed.",
        Severity::Error,
        "crates/sifr_package/src/cargo/metadata.rs::tests",
        "could not parse cargo metadata: {reason}",
        "sifr_package::cargo::metadata",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0101",
        "PACKAGE",
        "Cargo command invocation failed; Sifr reports the redacted Cargo excerpt and safe Sifr-owned recovery context.",
        Severity::Error,
        "crates/sifr_package/src/cargo_backend_integration_tests.rs::cargo_failure_mapping_redacts_private_credentials",
        "cargo {action} failed",
        "sifr_package::cargo::errors",
        [arg!("action"), arg!("reason")],
        ["action", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0102",
        "PACKAGE",
        "A selected Cargo package is Rust-only.",
        Severity::Error,
        "crates/sifr_package/src/package_workspace_query_tests.rs::explicit_rust_only_selection_reports_0102",
        "selected Rust-only package '{package_name}'",
        "sifr_package::graph::workspace",
        [arg!("package_name"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "package_name"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0104",
        "PACKAGE",
        "Package source is unavailable in offline or frozen mode.",
        Severity::Error,
        "crates/sifr_package/src/cargo_backend_integration_tests.rs::offline_mode_reports_missing_sifr_source_package",
        "package source unavailable in {lock_mode} mode",
        "sifr_package::cargo::lock_modes",
        [
            arg!("lock_mode"),
            json_arg!("cargo_package_id"),
            json_arg!("package_path")
        ],
        ["cargo_package_id", "package_path", "lock_mode"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0106",
        "PACKAGE",
        "Rust-only package depends directly on a Sifr source package.",
        Severity::Error,
        "crates/sifr_package/src/package_workspace_query_tests.rs::rust_only_member_depending_on_sifr_reports_0106",
        "Rust-only package depends on Sifr package",
        "sifr_package::graph::workspace",
        [
            json_arg!("from_cargo_package_id"),
            json_arg!("to_cargo_package_id")
        ],
        ["from_cargo_package_id", "to_cargo_package_id"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0201",
        "PACKAGE",
        "Direct package import root resolves to multiple package instances.",
        Severity::Error,
        "crates/sifr_package/src/package_dependency_scope_tests.rs::duplicate_direct_import_root_in_one_scope_reports_0201",
        "ambiguous package import root '{import_root}'",
        "sifr_package::graph::scopes",
        [
            arg!("import_root"),
            json_arg!("cargo_package_id"),
            json_arg!("candidates")
        ],
        ["cargo_package_id", "import_root", "candidates"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0202",
        "PACKAGE",
        "Package imports a module outside its direct dependency scope.",
        Severity::Error,
        "crates/sifr_package/src/package_source_map_tests.rs::transitive_dependency_import_reports_0202",
        "undeclared direct package import '{import_path}'",
        "sifr_package::imports::source_map",
        [
            arg!("import_path"),
            json_arg!("cargo_package_id"),
            json_arg!("package_id")
        ],
        ["cargo_package_id", "package_id", "import_path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0203",
        "PACKAGE",
        "Package imports a private module from another package.",
        Severity::Error,
        "crates/sifr_package/src/package_source_map_tests.rs::private_dependency_module_reports_0203",
        "private package module access '{import_path}'",
        "sifr_package::imports::source_map",
        [
            arg!("import_path"),
            json_arg!("cargo_package_id"),
            json_arg!("package_id"),
            json_arg!("target_package_id")
        ],
        [
            "cargo_package_id",
            "package_id",
            "target_package_id",
            "import_path"
        ]
    ),
    active_entry!(
        "SIFR-PACKAGE-0204",
        "PACKAGE",
        "Type identity crosses resolved package instances.",
        Severity::Error,
        "crates/sifr_package/src/package_dependency_scope_tests.rs::type_identity_mismatch_reports_0204_for_distinct_package_instances",
        "package type identity mismatch: expected {expected}, got {actual}",
        "sifr_package::graph::type_identity",
        [
            arg!("expected"),
            arg!("actual"),
            json_arg!("cargo_package_id")
        ],
        ["cargo_package_id", "expected", "actual"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0301",
        "PACKAGE",
        "Backend Rust crate is not allowed by the Sifr trust policy.",
        Severity::Error,
        "crates/sifr_package/src/cargo_backend_integration_tests.rs::backend_trust_reports_untrusted_direct_backend_crate",
        "untrusted backend crate '{backend_name}'",
        "sifr_package::cargo::trust",
        [
            arg!("backend_name"),
            json_arg!("cargo_package_id"),
            json_arg!("package_id")
        ],
        ["cargo_package_id", "package_id", "backend_name"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0305",
        "PACKAGE",
        "Trust policy names a backend crate that is not a direct dependency.",
        Severity::Error,
        "crates/sifr_package/src/cargo_backend_integration_tests.rs::backend_trust_rejects_stale_non_direct_trust_entry",
        "trusted backend crate '{backend_name}' is not direct",
        "sifr_package::cargo::trust",
        [
            arg!("backend_name"),
            json_arg!("cargo_package_id"),
            json_arg!("package_id")
        ],
        ["cargo_package_id", "package_id", "backend_name"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0401",
        "PACKAGE",
        "Cargo package archive is missing required Sifr source.",
        Severity::Error,
        "crates/sifr_package/src/package_publish_archive_tests.rs::archive_missing_sifr_source_reports_0401",
        "package '{package_id}' archive contains no .sifr source files",
        "sifr_package::cargo::package",
        [json_arg!("cargo_package_id"), arg!("package_id")],
        ["cargo_package_id", "package_id"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0402",
        "PACKAGE",
        "Package publish or archive validation failed.",
        Severity::Error,
        "crates/sifr_package/src/package_publish_archive_tests.rs::publish_validation_failed_reports_0402",
        "package publish validation failed: {reason}",
        "sifr_package::cargo::package",
        [arg!("reason"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0403",
        "PACKAGE",
        "Cargo include/exclude rules omit required Sifr files.",
        Severity::Error,
        "crates/sifr_package/src/package_publish_archive_tests.rs::archive_missing_required_entry_reports_0403",
        "Cargo package include/exclude rules omit required Sifr file '{path}'",
        "sifr_package::cargo::package",
        [arg!("path"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0404",
        "PACKAGE",
        "Cargo package archive contains an unsafe path.",
        Severity::Error,
        "crates/sifr_package/src/package_publish_archive_tests.rs::archive_traversal_reports_0404",
        "Cargo package archive entry escapes the package root: {path}",
        "sifr_package::cargo::package",
        [arg!("path"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0501",
        "PACKAGE",
        "Pure Sifr Rust marker contains implementation.",
        Severity::Error,
        "crates/sifr_package/src/source/layout.rs::tests",
        "pure Sifr package marker contains Rust implementation: {reason}",
        "sifr_package::source::layout",
        [
            arg!("reason"),
            json_arg!("cargo_package_id"),
            json_arg!("marker_path")
        ],
        ["cargo_package_id", "marker_path", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0601",
        "PACKAGE",
        "Package selector is ambiguous or invalid.",
        Severity::Error,
        "crates/sifr_package/src/package_workspace_query_tests.rs::ambiguous_filter_reports_0601",
        "package selector '{selector}' is ambiguous or invalid",
        "sifr_package::graph::filters",
        [arg!("selector"), json_arg!("candidates")],
        ["selector", "candidates"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0602",
        "PACKAGE",
        "Workspace selection contains duplicate import roots.",
        Severity::Error,
        "crates/sifr_package/src/package_workspace_query_tests.rs::workspace_duplicate_import_roots_report_0602",
        "duplicate workspace import root '{import_root}'",
        "sifr_package::graph::workspace",
        [arg!("import_root"), json_arg!("packages")],
        ["import_root", "packages"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0603",
        "PACKAGE",
        "Changed file could not be mapped to a package.",
        Severity::Error,
        "crates/sifr_package/src/package_workspace_query_tests.rs::changed_file_mapping_reports_0603",
        "changed path '{path}' does not map to one Sifr package",
        "sifr_package::graph::changed",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0604",
        "PACKAGE",
        "Outdated query cannot inspect this Cargo source.",
        Severity::Error,
        "crates/sifr_package/src/package_workspace_query_tests.rs::outdated_unknown_source_reports_0604",
        "outdated query unsupported for source '{source}'",
        "sifr_package::ops::read",
        [arg!("source"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "source"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0605",
        "PACKAGE",
        "Runnable package target or script selection is missing or ambiguous.",
        Severity::Error,
        "crates/sifr_package/src/package_session_tests.rs::package_session_reports_script_target_ambiguity",
        "ambiguous package run target: {selector}",
        "sifr_package::ops::session",
        [arg!("selector"), json_arg!("candidates")],
        ["selector", "candidates"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0606",
        "PACKAGE",
        "Discovered app target name is invalid.",
        Severity::Error,
        "crates/sifr_package/src/package_session_tests.rs::package_session_rejects_invalid_nested_target_name",
        "invalid package app target name: {target}",
        "sifr_package::ops::session",
        [arg!("target")],
        ["target"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0607",
        "PACKAGE",
        "Selected workspace members use the same Sifr package name.",
        Severity::Error,
        "crates/sifr_package/src/package_workspace_query_tests.rs::workspace_duplicate_sifr_names_report_0607",
        "duplicate Sifr package name in workspace: {package}",
        "sifr_package::graph::workspace",
        [arg!("package"), json_arg!("members")],
        ["package", "members"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0703",
        "PACKAGE",
        "Sifr-managed Cargo projection manifest pointer drift.",
        Severity::Error,
        "crates/sifr_package/src/package_projection_tests.rs::repair_check_reports_missing_manifest_pointer_0703",
        "Cargo projection manifest pointer drift",
        "sifr_package::projection",
        [
            json_arg!("cargo_package_id"),
            json_arg!("path"),
            arg!("reason")
        ],
        ["cargo_package_id", "path", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0704",
        "PACKAGE",
        "Sifr-managed Cargo projection include rules omit required package files.",
        Severity::Error,
        "crates/sifr_package/src/package_projection_tests.rs::repair_check_reports_missing_required_include_0704",
        "Cargo projection include rules omit required entry '{required}'",
        "sifr_package::projection",
        [
            json_arg!("cargo_package_id"),
            json_arg!("path"),
            arg!("required")
        ],
        ["cargo_package_id", "path", "required"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0709",
        "PACKAGE",
        "Pure package marker is missing from Sifr-managed projection.",
        Severity::Error,
        "crates/sifr_package/src/package_projection_tests.rs::repair_regenerates_missing_pure_marker",
        "pure Sifr package marker is missing",
        "sifr_package::projection",
        [json_arg!("cargo_package_id"), json_arg!("marker_path")],
        ["cargo_package_id", "marker_path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0710",
        "PACKAGE",
        "Explicit Sifr file target is outside the package source root.",
        Severity::Error,
        "crates/sifr_package/src/package_session_tests.rs::package_session_rejects_explicit_file_outside_source_root",
        "explicit file is outside package source root",
        "sifr_package::ops::session",
        [arg!("file"), arg!("source_root")],
        ["file", "source_root"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0713",
        "PACKAGE",
        "Public API symbol is exported more than once.",
        Severity::Error,
        "crates/sifr_package/src/package_public_api_tests.rs::duplicate_init_public_symbol_reports_0713",
        "duplicate public API symbol '{symbol}'",
        "sifr_package::imports::namespace_api",
        [
            arg!("symbol"),
            json_arg!("cargo_package_id"),
            json_arg!("manifest_path")
        ],
        ["cargo_package_id", "manifest_path", "symbol"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0714",
        "PACKAGE",
        "Package script expansion attempted to invoke another script.",
        Severity::Error,
        "crates/sifr_package/src/package_session_tests.rs::package_session_rejects_nested_script_expansion",
        "package script recursion is not allowed: {script}",
        "sifr_package::ops::session",
        [arg!("script")],
        ["script"]
    ),
];
