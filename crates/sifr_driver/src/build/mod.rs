mod api;
mod cargo_manifest;
mod entrypoint;
mod materialize;
mod project_codegen;
mod python_runtime;
mod report;
mod rust_interop;
mod rust_interop_digest;
mod rust_interop_probe;
#[cfg(test)]
mod rust_interop_tests;
mod workspace;

pub use api::{
    build, build_cached_package_project, build_cached_project, build_cached_single_file,
    build_package_project_report, build_project, build_project_report, build_single_file_report,
    check_package_project, check_project, check_single_file, emit_project,
};
pub use entrypoint::{CachedBinaryArtifact, PackageEntrypoint};
pub use python_runtime::PackagePythonRuntime;
pub use report::{BuildCompilationMode, BuildReport, BuildStageReport};

pub(crate) use cargo_manifest::generate_dependency_cargo_toml;
pub(crate) use entrypoint::{
    build_cached_package_project_binary, build_cached_project_binary,
    build_cached_single_file_binary, build_rooted_entrypoint_binary_with_report,
    check_single_file_entrypoint, compile_single_file_entrypoint_with_metadata,
    compile_single_file_entrypoint_with_metadata_and_options, compile_single_file_frontend,
    emit_project_entrypoint, resolve_package_project_entrypoint_plan,
    resolve_project_entrypoint_plan, RootedEntrypoint,
};
pub(crate) use workspace::{
    prepare_cached_artifact, ArtifactCacheReport, CachedArtifactEntry, PreparedArtifactCache,
};

#[cfg(test)]
pub(crate) use workspace::create_invocation_workspace;
