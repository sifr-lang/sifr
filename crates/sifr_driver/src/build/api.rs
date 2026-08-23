use super::report::{BuildReport, PythonInteropCheckReport};
use super::rust_interop_probe_policy::DirectProbePolicy;
use crate::build::{
    CachedBinaryArtifact, PackageEntrypoint, RootedEntrypoint, build_cached_package_project_binary,
    build_cached_project_binary, build_cached_single_file_binary,
    build_rooted_entrypoint_binary_with_report, check_single_file_entrypoint,
    emit_project_entrypoint, resolve_package_project_entrypoint_plan,
    resolve_project_entrypoint_plan,
};
use crate::diagnostics::{CompileResult, RenderedDiagnostic};
use sifr_frontend::SourceProvider;
use sifr_lowering::LoweringOptions;
use std::path::{Path, PathBuf};

pub fn build_project(
    main_file: &Path,
    output_dir: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    build_project_report(main_file, output_dir, provider)
        .map(|report| report.binary_path().to_path_buf())
}

pub fn build_project_report(
    main_file: &Path,
    output_dir: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<BuildReport, Vec<RenderedDiagnostic>> {
    build_rooted_entrypoint_binary_with_report(
        RootedEntrypoint::Project {
            main_file,
            provider,
        },
        output_dir,
    )
}

pub fn build_package_project_report(
    entrypoint: &PackageEntrypoint,
    output_dir: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<BuildReport, Vec<RenderedDiagnostic>> {
    build_rooted_entrypoint_binary_with_report(
        RootedEntrypoint::PackageProject {
            entrypoint,
            provider,
        },
        output_dir,
    )
}

pub fn check_project(
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> Vec<RenderedDiagnostic> {
    match resolve_project_entrypoint_plan(main_file, provider) {
        Ok(project_plan) => project_plan.frontend_diagnostics(),
        Err(errors) => errors,
    }
}

pub fn check_package_project(
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Vec<RenderedDiagnostic> {
    match check_package_python_interop(entrypoint, provider) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}

pub fn check_package_python_interop(
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Result<PythonInteropCheckReport, Vec<RenderedDiagnostic>> {
    let project_plan = resolve_package_project_entrypoint_plan(entrypoint, provider)?;
    let diagnostics = project_plan.frontend_diagnostics();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let generated = project_plan
        .into_generated_binary_project_with_probe_policy(true, DirectProbePolicy::ExecuteAll)?;
    Ok(super::python_check::python_interop_check_report(&generated))
}

pub fn check_single_file(source: &str, entrypoint_file: &Path) -> Vec<RenderedDiagnostic> {
    check_single_file_entrypoint(source, entrypoint_file)
}

pub fn emit_project(main_file: &Path, provider: &mut dyn SourceProvider) -> CompileResult {
    emit_project_entrypoint(main_file, provider)
}

pub fn build(source: &str, output_dir: &Path) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    build_single_file_report(source, Path::new("main"), output_dir)
        .map(|report| report.binary_path().to_path_buf())
}

pub fn build_single_file_report(
    source: &str,
    entrypoint_file: &Path,
    output_dir: &Path,
) -> Result<BuildReport, Vec<RenderedDiagnostic>> {
    let display_path = entrypoint_file.to_string_lossy();
    build_rooted_entrypoint_binary_with_report(
        RootedEntrypoint::SingleFile {
            source,
            display_path: &display_path,
            lowering_options: LoweringOptions::default(),
        },
        output_dir,
    )
}

pub fn build_cached_project(
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_project_binary(main_file, provider)
}

pub fn build_cached_package_project(
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_package_project_binary(entrypoint, provider)
}

pub fn build_cached_single_file(
    source: &str,
    entrypoint_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_single_file_binary(source, entrypoint_file)
}
