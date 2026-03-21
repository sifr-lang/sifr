//! Sifr Compiler Driver
//!
//! Orchestrates the full compilation pipeline:
//! parse -> type-check/HIR -> codegen -> build
//!
//! Stdlib `.sifr` files are embedded in the compiler binary via `include_str!`.
//! They are compiled before user code (two-phase compilation).

mod build;
mod diagnostics;
mod export_policy;
mod frontend;
mod project;
mod stdlib;
mod test_runner;

pub use build::{
    build, build_cached_project, build_cached_single_file, build_project, check_project,
    CachedBinaryArtifact,
};
pub use diagnostics::{
    apply_diagnostic_recovery_limits, compile_errors_to_diagnostics, CompileError, CompilePhase,
    CompileResult, CompileResultFull, CompilerDiagnostic, DiagnosticChild, DiagnosticSpan,
    DiagnosticSuggestion, RelatedSpan, Severity, SuggestionKind,
};
pub use frontend::{
    check, compile, compile_with_metadata, lower_source, parse_source, type_check_source,
};
pub use sifr_codegen::LoweringStats;
pub use test_runner::run_tests;

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
    DiscoveryDiagnosticStyle,
};
#[cfg(test)]
pub(crate) use stdlib::compile_stdlib;
#[cfg(test)]
pub(crate) use test_runner::{build_test_runner_project, execute_test_runner_project};
#[cfg(test)]
pub(crate) use test_runner::{compose_test_runner_lib, generate_test_runner_cargo_toml};

#[cfg(test)]
mod tests;
