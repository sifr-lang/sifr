use crate::build::{
    build_rooted_entrypoint_binary, resolve_project_entrypoint_plan, RootedEntrypoint,
};
use crate::diagnostics::CompileError;
use std::path::{Path, PathBuf};

pub fn build_project(main_file: &Path, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>> {
    build_rooted_entrypoint_binary(RootedEntrypoint::Project { main_file }, output_dir)
}

pub fn check_project(main_file: &Path) -> Vec<CompileError> {
    match resolve_project_entrypoint_plan(main_file) {
        Ok(project_plan) => {
            project_plan.emit_frontend_diagnostics();
            vec![]
        }
        Err(errors) => errors,
    }
}

pub fn build(source: &str, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>> {
    build_rooted_entrypoint_binary(RootedEntrypoint::SingleFile { source }, output_dir)
}
