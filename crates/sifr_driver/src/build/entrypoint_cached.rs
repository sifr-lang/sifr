//! Cached native entrypoint execution and ownership.
use super::*;

pub(crate) fn build_cached_project_binary(
    compiler: &crate::CompilerContext,
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_rooted_entrypoint_binary(
        compiler,
        RootedEntrypoint::Project {
            main_file,
            provider,
        },
        main_file.parent().unwrap_or(Path::new(".")),
        "run",
    )
}

pub(crate) fn build_cached_package_project_binary(
    compiler: &crate::CompilerContext,
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
    leases: super::super::native_child_leases::NativeChildLeases,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_rooted_entrypoint_binary(
        compiler,
        RootedEntrypoint::PackageProject {
            entrypoint,
            provider,
            leases,
        },
        entrypoint.main_file.parent().unwrap_or(Path::new(".")),
        "run",
    )
}

pub(crate) fn build_cached_single_file_binary(
    compiler: &crate::CompilerContext,
    source: &str,
    entrypoint_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    let display_path = entrypoint_file.to_string_lossy();
    build_cached_rooted_entrypoint_binary(
        compiler,
        RootedEntrypoint::SingleFile {
            source,
            display_path: &display_path,
            lowering_options: LoweringOptions::default(),
        },
        entrypoint_file.parent().unwrap_or(Path::new(".")),
        "run",
    )
}

fn build_cached_rooted_entrypoint_binary(
    compiler: &crate::CompilerContext,
    entrypoint: RootedEntrypoint<'_>,
    cache_scope: &Path,
    cache_namespace: &str,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    let total_start = Instant::now();
    let mut stages = Vec::new();
    let (plan, mode, entrypoint_path) = RootedEntrypointPlan::from_entrypoint_with_stages(
        compiler,
        entrypoint,
        &mut stages,
        true,
        None,
    )?;
    let frontend_diagnostics = plan.frontend_diagnostics();
    let cargo_resolution = plan.cargo_resolution.clone();
    let generated_project = measure_stage(&mut stages, "Generating Rust project", || {
        plan.into_generated_binary_project()
    })?;
    let requested_vendor_mode =
        super::super::entrypoint_resolution::requested_vendor_mode_for_build(mode);
    let (cache_entry, native_report, sysroot) = materialize_cached_binary_project_with_report(
        cache_namespace,
        cache_scope,
        "sifr_output",
        generated_project,
        requested_vendor_mode,
        &cargo_resolution,
    )?;
    #[cfg(test)]
    let generated_project_root = native_report
        .as_ref()
        .map(|report| report.generated_project_root.clone());
    let cargo_artifact_profile = native_report
        .as_ref()
        .map(|report| report.cargo_artifact_profile.clone());
    if let Some(native_report) = native_report {
        stages.push(BuildStageReport::new(
            "Materializing Cargo project",
            native_report.materialize_elapsed,
        ));
        stages.push(BuildStageReport::new(
            compiler.application_profile().build_stage(),
            native_report.cargo_elapsed,
        ));
    }
    let binary_path = cached_binary_path(cache_entry.workspace_root(), "sifr_output");
    let build_report = BuildReport::new(BuildReportInput {
        cargo_artifact_profile,
        application_profile: compiler.application_profile(),
        compiler_identity: compiler.identity().as_str().to_owned(),
        native_toolchain_identity: cargo_resolution
            .native_toolchain
            .as_ref()
            .ok()
            .map(|tools| tools.identity().to_owned()),
        entrypoint_path,
        mode,
        sysroot,
        binary_path: binary_path.clone(),
        total_elapsed: total_start.elapsed(),
        stages,
        frontend_diagnostics,
        cache_hit: cache_entry.report().cache_hit(),
        query_signature_artifact_path: None,
    });
    Ok(CachedBinaryArtifact {
        #[cfg(test)]
        generated_project_root,
        _cache_lease: cache_entry,
        input_leases: cargo_resolution.child_leases,
        binary_path,
        build_report,
    })
}
