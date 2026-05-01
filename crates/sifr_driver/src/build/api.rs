use crate::build::{
    build_cached_project_binary, build_cached_single_file_binary, build_rooted_entrypoint_binary,
    emit_project_entrypoint, resolve_project_entrypoint_plan, CachedBinaryArtifact,
    RootedEntrypoint,
};
use crate::diagnostics::{CompileResult, RenderedDiagnostic};
use std::path::{Path, PathBuf};

pub fn build_project(
    main_file: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    build_rooted_entrypoint_binary(&RootedEntrypoint::Project { main_file }, output_dir)
}

pub fn check_project(main_file: &Path) -> Vec<RenderedDiagnostic> {
    match resolve_project_entrypoint_plan(main_file) {
        Ok(project_plan) => {
            project_plan.emit_frontend_diagnostics();
            vec![]
        }
        Err(errors) => errors,
    }
}

pub fn emit_project(main_file: &Path) -> CompileResult {
    emit_project_entrypoint(main_file)
}

pub fn build(source: &str, output_dir: &Path) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    build_rooted_entrypoint_binary(&RootedEntrypoint::SingleFile { source }, output_dir)
}

pub fn build_cached_project(
    main_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_project_binary(main_file)
}

pub fn build_cached_single_file(
    source: &str,
    entrypoint_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_single_file_binary(source, entrypoint_file)
}
