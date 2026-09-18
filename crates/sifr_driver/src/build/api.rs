use super::report::{BuildReport, MaterializedRustProjectReport, PythonInteropCheckReport};
use super::rust_interop_probe_policy::DirectProbePolicy;
use crate::build::{
    CachedBinaryArtifact, PackageEntrypoint, RootedEntrypoint, build_cached_package_project_binary,
    build_cached_project_binary, build_cached_single_file_binary,
    build_rooted_entrypoint_binary_with_report, check_single_file_entrypoint,
    emit_project_entrypoint, materialize_rooted_entrypoint_rust_project,
    resolve_package_project_entrypoint_plan, resolve_project_entrypoint_plan,
};
use crate::diagnostics::{CompileResult, RenderedDiagnostic};
use sifr_frontend::SourceProvider;
use sifr_lowering::LoweringOptions;
use std::path::{Path, PathBuf};

pub fn build_project(
    compiler: &crate::CompilerContext,
    main_file: &Path,
    output_dir: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    build_project_report(compiler, main_file, output_dir, provider)
        .map(|report| report.binary_path().to_path_buf())
}

pub fn build_project_report(
    compiler: &crate::CompilerContext,
    main_file: &Path,
    output_dir: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<BuildReport, Vec<RenderedDiagnostic>> {
    build_rooted_entrypoint_binary_with_report(
        compiler,
        RootedEntrypoint::Project {
            main_file,
            provider,
        },
        output_dir,
    )
}

pub fn build_package_project_report(
    compiler: &crate::CompilerContext,
    entrypoint: &PackageEntrypoint,
    output_dir: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<BuildReport, Vec<RenderedDiagnostic>> {
    build_rooted_entrypoint_binary_with_report(
        compiler,
        RootedEntrypoint::PackageProject {
            entrypoint,
            provider,
        },
        output_dir,
    )
}

#[doc(hidden)]
pub fn materialize_package_project(
    compiler: &crate::CompilerContext,
    entrypoint: &PackageEntrypoint,
    output_dir: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<MaterializedRustProjectReport, Vec<RenderedDiagnostic>> {
    materialize_rooted_entrypoint_rust_project(
        compiler,
        RootedEntrypoint::PackageProject {
            entrypoint,
            provider,
        },
        output_dir,
    )
}

#[doc(hidden)]
pub fn materialize_project(
    compiler: &crate::CompilerContext,
    main_file: &Path,
    output_dir: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<MaterializedRustProjectReport, Vec<RenderedDiagnostic>> {
    materialize_rooted_entrypoint_rust_project(
        compiler,
        RootedEntrypoint::Project {
            main_file,
            provider,
        },
        output_dir,
    )
}

#[doc(hidden)]
pub fn materialize_single_file(
    compiler: &crate::CompilerContext,
    source: &str,
    entrypoint_file: &Path,
    output_dir: &Path,
) -> Result<MaterializedRustProjectReport, Vec<RenderedDiagnostic>> {
    let display_path = entrypoint_file.to_string_lossy();
    materialize_rooted_entrypoint_rust_project(
        compiler,
        RootedEntrypoint::SingleFile {
            source,
            display_path: &display_path,
            lowering_options: LoweringOptions::default(),
        },
        output_dir,
    )
}

pub fn check_project(
    compiler: &crate::CompilerContext,
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> Vec<RenderedDiagnostic> {
    match resolve_project_entrypoint_plan(compiler, main_file, provider) {
        Ok(project_plan) => project_plan.frontend_diagnostics(),
        Err(errors) => errors,
    }
}

pub fn check_package_project(
    compiler: &crate::CompilerContext,
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Vec<RenderedDiagnostic> {
    match check_package_python_interop(compiler, entrypoint, provider) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}

pub fn check_package_python_interop(
    compiler: &crate::CompilerContext,
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Result<PythonInteropCheckReport, Vec<RenderedDiagnostic>> {
    check_package_python_interop_completion(compiler, entrypoint, provider)
        .map(|(report, _)| report)
}

/// Completed package checking carries the actual generated interop demand. An
/// empty package manifest alone cannot prove that imported stdlib calls need no
/// live native probe, so persistence must use this owner-produced attestation.
pub fn check_package_project_completion(
    compiler: &crate::CompilerContext,
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> crate::project_cache::CheckComputation {
    match check_package_python_interop_completion(compiler, entrypoint, provider) {
        Ok((_, reusable)) => crate::project_cache::CheckComputation {
            diagnostics: Vec::new(),
            reusable,
        },
        Err(diagnostics) => diagnostics.into(),
    }
}

fn check_package_python_interop_completion(
    compiler: &crate::CompilerContext,
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Result<(PythonInteropCheckReport, bool), Vec<RenderedDiagnostic>> {
    let project_plan = resolve_package_project_entrypoint_plan(compiler, entrypoint, provider)?;
    let diagnostics = project_plan.frontend_diagnostics();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let generated = project_plan
        .into_generated_binary_project_with_probe_policy(true, DirectProbePolicy::ExecuteAll)?;
    let reusable = generated.interop == sifr_codegen::InteropBuildPlan::default()
        && generated.python_runtime.is_none();
    Ok((
        super::python_check::python_interop_check_report(&generated),
        reusable,
    ))
}

pub fn check_single_file(
    compiler: &crate::CompilerContext,
    source: &str,
    entrypoint_file: &Path,
) -> Vec<RenderedDiagnostic> {
    check_single_file_entrypoint(compiler, source, entrypoint_file)
}

pub fn emit_project(
    compiler: &crate::CompilerContext,
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> CompileResult {
    emit_project_entrypoint(compiler, main_file, provider)
}

pub fn build(
    compiler: &crate::CompilerContext,
    source: &str,
    output_dir: &Path,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    build_single_file_report(compiler, source, Path::new("main"), output_dir)
        .map(|report| report.binary_path().to_path_buf())
}

pub fn build_single_file_report(
    compiler: &crate::CompilerContext,
    source: &str,
    entrypoint_file: &Path,
    output_dir: &Path,
) -> Result<BuildReport, Vec<RenderedDiagnostic>> {
    let display_path = entrypoint_file.to_string_lossy();
    build_rooted_entrypoint_binary_with_report(
        compiler,
        RootedEntrypoint::SingleFile {
            source,
            display_path: &display_path,
            lowering_options: LoweringOptions::default(),
        },
        output_dir,
    )
}

pub fn build_cached_project(
    compiler: &crate::CompilerContext,
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_project_binary(compiler, main_file, provider)
}

pub fn build_cached_package_project(
    compiler: &crate::CompilerContext,
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_package_project_binary(compiler, entrypoint, provider)
}

pub fn build_cached_single_file(
    compiler: &crate::CompilerContext,
    source: &str,
    entrypoint_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_single_file_binary(compiler, source, entrypoint_file)
}
