//! Shared compiler environment services below driver and editor orchestration.
//!
//! This crate owns standard-library bootstrap, tooling sysroot views, generated
//! Rust previews, and Python authoring-environment validation. It does not own
//! Cargo execution, build workspaces, CLI behavior, or LSP protocol handling.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod diagnostics;
mod export_policy;
mod preview;
mod private_re_exports;
mod python_certification;
mod python_interop;
mod python_runtime;
mod stdlib;

pub use diagnostics::{GeneratedSourceMapFile, render_package_diagnostic};
pub use preview::{CompilerPreview, compile_source_preview};
pub use python_certification::{
    validate_binding_distributions, validate_certification_distributions,
    validate_protocol_certifications_for_plan,
};
pub use python_interop::{
    PythonInteropPlanDiagnostic, PythonTargetInspection, PythonTargetParameter,
    apply_python_target_inspection, inspect_python_target, mark_embedded_bridge_targets,
    probe_python_interop_plan,
};
pub use python_runtime::{
    EmbeddedPythonBridgeSource, PackagePythonRuntime, inject_python_runtime_bootstrap,
    render_python_runtime_prelude,
};
pub use sifr_codegen::{
    InteropBuildPlan, PythonInteropPlan, PythonInteropPlanDeclaration, PythonTargetProbe,
    PythonTargetProbeStatus, interop_build_plan_for_named_modules,
};
pub use stdlib::{
    StdlibCompiled, StdlibRustInterop, StdlibRustInteropModuleSource, ToolingSysrootDiagnostic,
    ToolingSysrootProbe, ToolingSysrootStatus, compile_stdlib, external_defs, sysroot_probe,
    sysroot_status, tooling_sources,
};
