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
mod private_re_exports;
mod project;
mod python_binding;
mod stdlib;
mod test_runner;
mod workspace;

pub use build::{
    apply_python_target_inspection, build, build_cached_package_project, build_cached_project,
    build_cached_single_file, build_package_project_report, build_project, build_project_report,
    build_single_file_report, capture_cargo_invocations, check_package_project,
    check_package_python_interop, check_project, check_single_file, emit_project,
    generate_dependency_cargo_toml, inspect_python_target, probe_python_interop_plan,
    sysroot_cargo_config_args, try_generate_standalone_dependency_plan,
    validate_binding_distributions, validate_certification_distributions,
    validate_protocol_certifications_for_plan, BuildCompilationMode, BuildReport, BuildReportInput,
    BuildStageReport, BuildSysrootReport, CachedBinaryArtifact, CargoInvocation, PackageEntrypoint,
    PackagePythonRuntime, PythonDeclarationCheck, PythonEnvironmentCheck, PythonInteropCheckReport,
    PythonInteropPlanDiagnostic, PythonTargetCheck, PythonTargetCheckStatus,
    PythonTargetInspection, PythonTargetParameter,
};
pub use diagnostics::{
    apply_diagnostic_recovery_limits, diagnostic_label_for_code, render_package_diagnostic,
    CompileResult, CompileResultFull, GeneratedSourceMapFile,
};
pub use frontend::{
    check, compile, compile_with_metadata, lower_source, parse_source, type_check_source,
};
pub use python_binding::{
    render_python_binding_scaffold, PythonBindingDeclaration, PythonBindingDeclarationKind,
    PythonBindingParameter, PythonBindingParameterKind, PythonBindingProbeError,
    PythonBindingProbeReport, PythonBindingProbeSource, PythonBindingProbeSymbol,
    PythonBindingScaffold,
};
pub use sifr_codegen::{
    interop_build_plan_for_named_modules, InteropBuildPlan, LoweringStats, PythonInteropPlan,
    PythonInteropPlanDeclaration, PythonTargetProbe, PythonTargetProbeStatus,
};
pub use stdlib::{
    external_defs as stdlib_external_defs, sysroot_probe as stdlib_tooling_sysroot_probe,
    sysroot_status as stdlib_tooling_sysroot_status, tooling_sources as stdlib_tooling_sources,
    ToolingSysrootDiagnostic, ToolingSysrootProbe, ToolingSysrootStatus,
};
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
