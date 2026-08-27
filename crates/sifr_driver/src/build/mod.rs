mod api;
mod cargo_invocation_trace;
mod cargo_manifest;
mod cargo_resolution;
mod entrypoint;
mod entrypoint_artifact;
mod entrypoint_resolution;
mod entrypoint_single_file;
mod entrypoint_stages;
mod generated_cargo_project;
mod materialize;
mod project_codegen;
mod python_bridges;
mod python_certification;
mod python_check;
mod python_interop;
#[cfg(test)]
mod python_interop_shared_target_tests;
mod python_runtime;
mod report;
mod rust_interop;
#[cfg(test)]
mod rust_interop_advanced_data_contract_tests;
#[cfg(test)]
mod rust_interop_async_contract_tests;
mod rust_interop_bridge_audit;
mod rust_interop_bridge_sources;
#[cfg(test)]
mod rust_interop_callback_contract_tests;
mod rust_interop_callback_probe;
mod rust_interop_cargo_inputs;
#[cfg(test)]
mod rust_interop_contract_tests;
mod rust_interop_contracts;
mod rust_interop_diagnostics;
mod rust_interop_digest;
#[cfg(test)]
mod rust_interop_evidence_contract_tests;
#[cfg(test)]
mod rust_interop_opaque_contract_tests;
#[cfg(test)]
mod rust_interop_panic_contract_tests;
mod rust_interop_panic_probe;
mod rust_interop_probe;
mod rust_interop_probe_cache;
mod rust_interop_probe_diagnostics;
mod rust_interop_probe_features;
mod rust_interop_probe_manifest;
mod rust_interop_probe_nonce;
mod rust_interop_probe_paths;
mod rust_interop_probe_policy;
mod rust_interop_resolution;
mod rust_interop_sqlx_modules;
mod rust_interop_sqlx_offline;
#[cfg(test)]
mod rust_interop_test_support;
#[cfg(test)]
mod rust_interop_tests;
mod rust_interop_trust;
#[cfg(test)]
mod rust_interop_zero_copy_contract_tests;
mod single_file_interop_cache;
mod sysroot_interop;
#[cfg(test)]
mod sysroot_interop_tests;
mod workspace;

pub use api::{
    build, build_cached_package_project, build_cached_project, build_cached_single_file,
    build_package_project_report, build_project, build_project_report, build_single_file_report,
    check_package_project, check_package_python_interop, check_project, check_single_file,
    emit_project,
};
#[doc(hidden)]
pub use cargo_invocation_trace::{CargoInvocation, capture_cargo_invocations};
pub use cargo_manifest::{
    generate_dependency_cargo_toml, sysroot_cargo_config_args,
    try_generate_standalone_dependency_plan,
};
pub use entrypoint::PackageEntrypoint;
pub use entrypoint_artifact::CachedBinaryArtifact;
pub use python_certification::{
    validate_binding_distributions, validate_certification_distributions,
    validate_protocol_certifications_for_plan,
};
pub use python_interop::{
    PythonInteropPlanDiagnostic, PythonTargetInspection, PythonTargetParameter,
    apply_python_target_inspection, inspect_python_target, probe_python_interop_plan,
};
pub use python_runtime::PackagePythonRuntime;
pub use report::{
    BuildCompilationMode, BuildReport, BuildReportInput, BuildStageReport, BuildSysrootReport,
    PythonDeclarationCheck, PythonEnvironmentCheck, PythonInteropCheckReport, PythonTargetCheck,
    PythonTargetCheckStatus,
};

pub(crate) use cargo_manifest::{
    generate_dependency_cargo_toml_with_interop, try_generate_sysroot_dependency_plan,
};
pub(crate) use cargo_resolution::{CargoResolutionPolicy, cargo_resolution_cache_key_fragment};
pub(crate) use entrypoint::{
    RootedEntrypoint, build_cached_package_project_binary, build_cached_project_binary,
    build_cached_single_file_binary, build_rooted_entrypoint_binary_with_report,
    check_single_file_entrypoint, compile_single_file_entrypoint_with_metadata,
    compile_single_file_frontend, emit_project_entrypoint, resolve_package_project_entrypoint_plan,
    resolve_project_entrypoint_plan,
};
pub(crate) use generated_cargo_project::{
    GeneratedCargoCommand, GeneratedCargoProject, materialize_generated_cargo_project,
    run_generated_cargo_command,
};
pub(crate) use workspace::{
    ArtifactCacheReport, CachedArtifactEntry, PreparedArtifactCache, prepare_cached_artifact,
};

pub(crate) use workspace::create_invocation_workspace;
