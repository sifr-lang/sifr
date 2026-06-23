//! Sifr Compiler Driver
//!
//! Orchestrates the full compilation pipeline:
//! parse -> type-check/HIR -> codegen -> build
//!
//! Stdlib `.sifr` files are loaded from the resolved Sifr sysroot.
//! They are compiled before user code (two-phase compilation).
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod build;
mod diagnostics;
mod export_policy;
mod frontend;
mod project;
mod stdlib;
mod test_runner;
mod workspace;

pub use build::{
    build, build_cached_package_project, build_cached_project, build_cached_single_file,
    build_package_project_report, build_project, build_project_report, build_single_file_report,
    check_package_project, check_project, check_single_file, emit_project, BuildCompilationMode,
    BuildReport, BuildStageReport, CachedBinaryArtifact, PackageEntrypoint, PackagePythonRuntime,
};
pub use diagnostics::{
    apply_diagnostic_recovery_limits, diagnostic_label_for_code, diagnostic_label_for_code_str,
    render_package_diagnostic, CompileResult, CompileResultFull,
};
pub use frontend::{
    check, compile, compile_with_metadata, compile_with_metadata_allowing_http_transport_harness,
    lower_source, parse_source, type_check_source,
};
pub use sifr_codegen::LoweringStats;
pub use stdlib::external_defs as stdlib_external_defs;
pub use test_runner::run_tests;
pub use workspace::{find_workspace_root, SifrWorkspaceConfig, WorkspaceRoot};

#[cfg(test)]
pub(crate) use build::create_invocation_workspace;
#[cfg(test)]
pub(crate) use diagnostics::run_codegen_with_boundary;
#[cfg(test)]
pub(crate) use frontend::FrontendDiagnosticStyle;
#[cfg(test)]
pub(crate) use project::{
    assemble_project_main_rs, collect_project_hir_modules, compile_frontend_modules,
    compute_module_compile_order, discover_test_root_modules, parse_import_closure_modules,
    DiscoveryDiagnosticStyle, ModuleResolver,
};
#[cfg(test)]
pub(crate) use stdlib::compile_stdlib;
#[cfg(test)]
pub(crate) use test_runner::{build_test_runner_project, execute_test_runner_project};
#[cfg(test)]
pub(crate) use test_runner::{compose_test_runner_lib, generate_test_runner_cargo_toml};

#[cfg(test)]
mod tests;
