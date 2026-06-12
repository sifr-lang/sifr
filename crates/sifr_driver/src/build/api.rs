use crate::build::{
    build_cached_package_project_binary, build_cached_project_binary,
    build_cached_single_file_binary, build_rooted_entrypoint_binary, check_single_file_entrypoint,
    emit_project_entrypoint, resolve_package_project_entrypoint_plan,
    resolve_project_entrypoint_plan, CachedBinaryArtifact, PackageEntrypoint, RootedEntrypoint,
};
use crate::diagnostics::{CompileResult, RenderedDiagnostic};
use sifr_lowering::LoweringOptions;
use std::path::{Path, PathBuf};

pub fn build_project(
    main_file: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    build_rooted_entrypoint_binary(&RootedEntrypoint::Project { main_file }, output_dir)
}

pub fn check_project(main_file: &Path) -> Vec<RenderedDiagnostic> {
    match resolve_project_entrypoint_plan(main_file) {
        Ok(project_plan) => project_plan.frontend_diagnostics(),
        Err(errors) => errors,
    }
}

pub fn check_package_project(entrypoint: &PackageEntrypoint) -> Vec<RenderedDiagnostic> {
    match resolve_package_project_entrypoint_plan(entrypoint) {
        Ok(project_plan) => project_plan.frontend_diagnostics(),
        Err(errors) => errors,
    }
}

pub fn check_single_file(source: &str, entrypoint_file: &Path) -> Vec<RenderedDiagnostic> {
    check_single_file_entrypoint(source, entrypoint_file)
}

pub fn emit_project(main_file: &Path) -> CompileResult {
    emit_project_entrypoint(main_file)
}

pub fn build(source: &str, output_dir: &Path) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    build_rooted_entrypoint_binary(
        &RootedEntrypoint::SingleFile {
            source,
            display_path: "main",
            lowering_options: LoweringOptions::default(),
        },
        output_dir,
    )
}

pub fn build_cached_project(
    main_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_project_binary(main_file)
}

pub fn build_cached_package_project(
    entrypoint: &PackageEntrypoint,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_package_project_binary(entrypoint)
}

pub fn build_cached_single_file(
    source: &str,
    entrypoint_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_single_file_binary(source, entrypoint_file)
}
