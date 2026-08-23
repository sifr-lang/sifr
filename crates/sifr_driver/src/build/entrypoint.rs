use super::cargo_resolution::CargoResolutionPolicy;
use super::entrypoint_artifact::CachedBinaryArtifact;
use super::entrypoint_stages::{import_closure_label, measure_stage, module_analysis_label};
use super::materialize::{
    cached_binary_path, materialize_binary_project_with_report,
    materialize_cached_binary_project_with_report,
};
use super::project_codegen::{
    GeneratedBinaryProject, apply_package_runtime_metadata, codegen_single_file_frontend,
    generated_project_binary_project, generated_single_file_binary_project,
};
use super::python_bridges::apply_package_python_bridge_metadata;
use super::python_bridges::package_bridge_lowering_options;
use super::python_runtime::PackagePythonRuntime;
use super::report::{BuildCompilationMode, BuildReport, BuildReportInput, BuildStageReport};
use super::rust_interop::{PackageRustInteropContext, RustInteropModuleSource};
use super::rust_interop_probe_policy::DirectProbePolicy;
use super::single_file_interop_cache::{CompiledSingleFileMetadata, resolve_single_file_metadata};
use super::sysroot_interop::attach_stdlib_rust_interop;
use crate::diagnostics::{CompileResult, RenderedDiagnostic};
use crate::frontend::{FrontendCompiled, parse_source};
use crate::project::{
    DiscoveryDiagnosticStyle, ModuleResolver, ProjectLowering, collect_project_hir_source_modules,
    collect_project_hir_source_modules_with_options,
    compile_single_frontend_module_with_source_and_options, emit_project_frontend_diagnostics,
    parse_import_closure_source_modules, parse_package_import_closure_source_project,
};
use crate::stdlib::{StdlibCompiled, compile_stdlib};
use crate::workspace::find_workspace_root;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::{FrontendDiagnosticStyle, FrontendSourceContext, SourceProvider};
use sifr_lowering::LoweringOptions;
use sifr_package::{PackageSourceMap, SifrPackageGraph, SifrPackageId};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootedEntrypointShape {
    SingleFile,
    Project,
}

pub(crate) enum RootedEntrypoint<'a> {
    SingleFile {
        source: &'a str,
        display_path: &'a str,
        lowering_options: LoweringOptions,
    },
    Project {
        main_file: &'a Path,
        provider: &'a mut dyn SourceProvider,
    },
    PackageProject {
        entrypoint: &'a PackageEntrypoint,
        provider: &'a mut dyn SourceProvider,
    },
}

impl RootedEntrypoint<'_> {
    fn build_mode(&self) -> BuildCompilationMode {
        match self {
            Self::SingleFile { .. } => BuildCompilationMode::SingleFile,
            Self::Project { .. } => BuildCompilationMode::Project,
            Self::PackageProject { .. } => BuildCompilationMode::PackageProject,
        }
    }

    fn display_path(&self) -> PathBuf {
        match self {
            Self::SingleFile { display_path, .. } => PathBuf::from(display_path),
            Self::Project { main_file, .. } => (*main_file).to_path_buf(),
            Self::PackageProject { entrypoint, .. } => entrypoint.main_file.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PackageEntrypoint {
    pub main_file: PathBuf,
    pub package_id: SifrPackageId,
    pub graph: SifrPackageGraph,
    pub source_map: PackageSourceMap,
    pub python_runtime: Option<PackagePythonRuntime>,
    pub lock_mode: sifr_package::CargoLockMode,
}

pub(crate) struct RootedEntrypointPlan {
    shape: RootedEntrypointShape,
    stdlib: StdlibCompiled,
    project_lowering: ProjectLowering,
    python_runtime: Option<PackagePythonRuntime>,
    python_bridges: Option<sifr_package::ResolvedPythonBridgeGraph>,
    rust_interop_context: Option<PackageRustInteropContext>,
    cargo_resolution: CargoResolutionPolicy,
}

pub(crate) fn compile_single_file_frontend(
    source: &str,
) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
    RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::SingleFile {
        source,
        display_path: "main",
        lowering_options: LoweringOptions::default(),
    })?
    .into_single_file_frontend()
}

pub(crate) fn compile_single_file_entrypoint_with_metadata(
    source: &str,
) -> Result<CompiledSingleFileMetadata, Vec<RenderedDiagnostic>> {
    compile_single_file_entrypoint_with_metadata_and_options(source, LoweringOptions::default())
}

pub(crate) fn compile_single_file_entrypoint_with_metadata_and_options(
    source: &str,
    lowering_options: LoweringOptions,
) -> Result<CompiledSingleFileMetadata, Vec<RenderedDiagnostic>> {
    let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::SingleFile {
        source,
        display_path: "main",
        lowering_options,
    })?;
    plan.emit_frontend_diagnostics();
    let rust_interop_context = plan.rust_interop_context.clone();
    let stdlib_interop = plan.stdlib.interop.clone();
    let codegen_result = plan.into_single_file_codegen_result()?;
    resolve_single_file_metadata(codegen_result, rust_interop_context, &stdlib_interop)
}

pub(crate) fn check_single_file_entrypoint(
    source: &str,
    entrypoint_file: &Path,
) -> Vec<RenderedDiagnostic> {
    let display_path = entrypoint_file.to_string_lossy();
    match RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::SingleFile {
        source,
        display_path: &display_path,
        lowering_options: LoweringOptions::default(),
    }) {
        Ok(plan) => plan.frontend_diagnostics(),
        Err(errors) => errors,
    }
}

pub(crate) fn resolve_project_entrypoint_plan(
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<RootedEntrypointPlan, Vec<RenderedDiagnostic>> {
    RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
        main_file,
        provider,
    })
}

pub(crate) fn resolve_package_project_entrypoint_plan(
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Result<RootedEntrypointPlan, Vec<RenderedDiagnostic>> {
    RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::PackageProject {
        entrypoint,
        provider,
    })
}

pub(crate) fn emit_project_entrypoint(
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> CompileResult {
    let plan = match resolve_project_entrypoint_plan(main_file, provider) {
        Ok(plan) => plan,
        Err(errors) => return CompileResult::Errors { errors },
    };
    plan.emit_frontend_diagnostics();
    match plan.into_generated_binary_project_with_probe_policy(
        false,
        DirectProbePolicy::DeferTrustedSysroot,
    ) {
        Ok(generated_project) => CompileResult::Success {
            rust_source: generated_project.emit_source_listing(),
        },
        Err(errors) => CompileResult::Errors { errors },
    }
}

pub(crate) fn build_rooted_entrypoint_binary_with_report(
    entrypoint: RootedEntrypoint<'_>,
    output_dir: &Path,
) -> Result<BuildReport, Vec<RenderedDiagnostic>> {
    let total_start = Instant::now();
    let mut stages = Vec::new();
    let (plan, mode, entrypoint_path) =
        RootedEntrypointPlan::from_entrypoint_with_stages(entrypoint, &mut stages)?;
    let frontend_diagnostics = plan.frontend_diagnostics();
    let cargo_resolution = plan.cargo_resolution.clone();
    let generated_project = measure_stage(&mut stages, "Generating Rust project", || {
        plan.into_generated_binary_project()
    })?;
    let requested_vendor_mode = super::entrypoint_resolution::requested_vendor_mode_for_build(mode);
    let materialized = materialize_binary_project_with_report(
        output_dir,
        "sifr_output",
        generated_project,
        requested_vendor_mode,
        &cargo_resolution,
    )?;
    stages.push(BuildStageReport::new(
        "Materializing Cargo project",
        materialized.materialize_elapsed,
    ));
    stages.push(BuildStageReport::new(
        "Building release binary",
        materialized.cargo_elapsed,
    ));
    Ok(BuildReport::new(BuildReportInput {
        entrypoint_path,
        mode,
        sysroot: materialized.sysroot,
        binary_path: materialized.binary_path,
        total_elapsed: total_start.elapsed(),
        stages,
        frontend_diagnostics,
        cache_hit: false,
    }))
}

pub(crate) fn build_cached_project_binary(
    main_file: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_rooted_entrypoint_binary(
        RootedEntrypoint::Project {
            main_file,
            provider,
        },
        main_file.parent().unwrap_or(Path::new(".")),
        "run",
    )
}

pub(crate) fn build_cached_package_project_binary(
    entrypoint: &PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_rooted_entrypoint_binary(
        RootedEntrypoint::PackageProject {
            entrypoint,
            provider,
        },
        entrypoint.main_file.parent().unwrap_or(Path::new(".")),
        "run",
    )
}

pub(crate) fn build_cached_single_file_binary(
    source: &str,
    entrypoint_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    let display_path = entrypoint_file.to_string_lossy();
    build_cached_rooted_entrypoint_binary(
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
    entrypoint: RootedEntrypoint<'_>,
    cache_scope: &Path,
    cache_namespace: &str,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    let total_start = Instant::now();
    let mut stages = Vec::new();
    let (plan, mode, entrypoint_path) =
        RootedEntrypointPlan::from_entrypoint_with_stages(entrypoint, &mut stages)?;
    let frontend_diagnostics = plan.frontend_diagnostics();
    let cargo_resolution = plan.cargo_resolution.clone();
    let generated_project = measure_stage(&mut stages, "Generating Rust project", || {
        plan.into_generated_binary_project()
    })?;
    let requested_vendor_mode = super::entrypoint_resolution::requested_vendor_mode_for_build(mode);
    let (cache_entry, native_report, sysroot) = materialize_cached_binary_project_with_report(
        cache_namespace,
        cache_scope,
        "sifr_output",
        generated_project,
        requested_vendor_mode,
        &cargo_resolution,
    )?;
    if let Some(native_report) = native_report {
        stages.push(BuildStageReport::new(
            "Materializing Cargo project",
            native_report.materialize_elapsed,
        ));
        stages.push(BuildStageReport::new(
            "Building release binary",
            native_report.cargo_elapsed,
        ));
    }
    let binary_path = cached_binary_path(cache_entry.workspace_root(), "sifr_output");
    let build_report = BuildReport::new(BuildReportInput {
        entrypoint_path,
        mode,
        sysroot,
        binary_path: binary_path.clone(),
        total_elapsed: total_start.elapsed(),
        stages,
        frontend_diagnostics,
        cache_hit: cache_entry.report().cache_hit(),
    });
    Ok(CachedBinaryArtifact {
        binary_path,
        build_report,
    })
}

impl RootedEntrypointPlan {
    fn from_entrypoint(entrypoint: RootedEntrypoint<'_>) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut stages = Vec::new();
        Self::from_entrypoint_with_stages(entrypoint, &mut stages)
            .map(|(plan, _mode, _entrypoint_path)| plan)
    }

    fn from_entrypoint_with_stages(
        entrypoint: RootedEntrypoint<'_>,
        stages: &mut Vec<BuildStageReport>,
    ) -> Result<(Self, BuildCompilationMode, PathBuf), Vec<RenderedDiagnostic>> {
        let mode = entrypoint.build_mode();
        let entrypoint_path = entrypoint.display_path();
        let stdlib = measure_stage(stages, "Loading Sifr standard library", compile_stdlib)?;
        let package_entrypoint = match &entrypoint {
            RootedEntrypoint::PackageProject { entrypoint, .. } => Some(*entrypoint),
            RootedEntrypoint::SingleFile { .. } | RootedEntrypoint::Project { .. } => None,
        };
        let cargo_resolution = super::entrypoint_resolution::package_cargo_resolution_policy(
            package_entrypoint,
            &stdlib,
        );
        let resolved = match entrypoint {
            RootedEntrypoint::SingleFile {
                source,
                display_path,
                lowering_options,
            } => {
                let parsed_suite =
                    measure_stage(stages, "Parsing source (1 module)", || parse_source(source))?;
                let project_lowering = measure_stage(stages, "Analyzing 1 module", || {
                    compile_single_frontend_module_with_source_and_options(
                        "main",
                        &parsed_suite,
                        FrontendSourceContext {
                            display_path,
                            source,
                        },
                        stdlib.defs.clone(),
                        FrontendDiagnosticStyle::Bare,
                        lowering_options.clone(),
                    )
                })?;
                (
                    RootedEntrypointShape::SingleFile,
                    project_lowering,
                    None,
                    None,
                    None,
                )
            }
            RootedEntrypoint::Project {
                main_file,
                provider,
            } => {
                let project_dir = main_file.parent().unwrap_or(Path::new("."));
                let Some(main_module_name) = main_file
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                else {
                    return Err(vec![crate::diagnostics::diagnostic_with_code(
                        format!("invalid project entrypoint path '{}'", main_file.display()),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )]);
                };
                let root_modules = BTreeSet::from([main_module_name.clone()]);
                let resolver = match find_workspace_root(main_file, provider)? {
                    Some(workspace_root) => {
                        ModuleResolver::with_workspace(project_dir, workspace_root)
                    }
                    None => ModuleResolver::entry_parent(project_dir),
                };
                let parse_start = Instant::now();
                let mut parsed_modules = parse_import_closure_source_modules(
                    &resolver,
                    &root_modules,
                    DiscoveryDiagnosticStyle::ModuleName,
                    provider,
                )?;
                if main_module_name != "main" {
                    if let Some(entry_module) = parsed_modules.remove(&main_module_name) {
                        parsed_modules.insert("main".to_string(), entry_module);
                    }
                }
                let module_count = parsed_modules.len();
                stages.push(BuildStageReport::new(
                    import_closure_label(module_count),
                    parse_start.elapsed(),
                ));
                let project_lowering =
                    measure_stage(stages, module_analysis_label(module_count), || {
                        collect_project_hir_source_modules(&parsed_modules, stdlib.defs.clone())
                    })?;
                (
                    RootedEntrypointShape::Project,
                    project_lowering,
                    None,
                    None,
                    None,
                )
            }
            RootedEntrypoint::PackageProject {
                entrypoint,
                provider,
            } => {
                let python_bridges = sifr_package::resolve_python_bridge_graph(
                    &entrypoint.graph,
                    &entrypoint.package_id,
                )
                .map_err(|errors| {
                    errors
                        .into_iter()
                        .map(crate::diagnostics::render_package_diagnostic)
                        .collect::<Vec<_>>()
                })?;
                let parse_start = Instant::now();
                let mut package_project = parse_package_import_closure_source_project(
                    &entrypoint.graph,
                    &entrypoint.source_map,
                    &entrypoint.package_id,
                    &entrypoint.main_file,
                    DiscoveryDiagnosticStyle::ModuleName,
                    provider,
                )?;
                let entry_module_name = package_project.entry_module_name.clone();
                if entry_module_name != "main" {
                    if let Some(entry_module) =
                        package_project.parsed_modules.remove(&entry_module_name)
                    {
                        package_project
                            .parsed_modules
                            .insert("main".to_string(), entry_module);
                    }
                    if let Some(entry_package) =
                        package_project.module_packages.remove(&entry_module_name)
                    {
                        package_project
                            .module_packages
                            .insert("main".to_string(), entry_package);
                    }
                }
                let module_sources = package_project
                    .parsed_modules
                    .iter()
                    .map(|(module_name, parsed)| {
                        (
                            module_name.clone(),
                            RustInteropModuleSource::from_parsed(parsed),
                        )
                    })
                    .collect();
                let rust_interop_context = PackageRustInteropContext {
                    package_id: entrypoint.package_id.clone(),
                    graph: entrypoint.graph.clone(),
                    source_map: entrypoint.source_map.clone(),
                    module_packages: package_project.module_packages.clone(),
                    module_sources,
                    sysroot_runtime_crate: stdlib
                        .interop
                        .sysroot
                        .as_ref()
                        .map(|sysroot| sysroot.paths.runtime_crate.clone()),
                    sysroot_trust: None,
                };
                let module_count = package_project.parsed_modules.len();
                stages.push(BuildStageReport::new(
                    import_closure_label(module_count),
                    parse_start.elapsed(),
                ));
                let project_lowering =
                    measure_stage(stages, module_analysis_label(module_count), || {
                        let lowering_options = package_bridge_lowering_options(
                            entrypoint.python_runtime.as_ref(),
                            &package_project.module_packages,
                            &python_bridges,
                        );
                        collect_project_hir_source_modules_with_options(
                            &package_project.parsed_modules,
                            stdlib.defs.clone(),
                            &lowering_options,
                        )
                    })?;
                (
                    RootedEntrypointShape::Project,
                    project_lowering,
                    entrypoint.python_runtime.clone(),
                    Some(python_bridges),
                    Some(rust_interop_context),
                )
            }
        };
        let (shape, project_lowering, python_runtime, python_bridges, rust_interop_context) =
            resolved;

        Ok((
            Self {
                shape,
                stdlib,
                project_lowering,
                python_runtime,
                python_bridges,
                rust_interop_context,
                cargo_resolution,
            },
            mode,
            entrypoint_path,
        ))
    }

    pub(crate) fn emit_frontend_diagnostics(&self) {
        emit_project_frontend_diagnostics(&self.project_lowering);
    }

    pub(crate) fn frontend_diagnostics(&self) -> Vec<RenderedDiagnostic> {
        self.project_lowering
            .compile_order
            .iter()
            .filter_map(|module_name| {
                self.project_lowering
                    .module_diagnostics
                    .get(module_name.as_str())
            })
            .flat_map(|diagnostics| {
                diagnostics
                    .rendered_warnings
                    .clone()
                    .into_iter()
                    .chain(diagnostics.rendered_reveal_types.clone())
            })
            .collect()
    }

    fn into_single_file_frontend(self) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
        if self.shape != RootedEntrypointShape::SingleFile {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                "internal error: rooted project entrypoint cannot be converted into a single-file frontend result",
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        }

        super::entrypoint_single_file::into_frontend(self.stdlib, self.project_lowering)
    }

    fn into_single_file_codegen_result(
        self,
    ) -> Result<sifr_codegen::CodegenResult, Vec<RenderedDiagnostic>> {
        let frontend = self.into_single_file_frontend()?;
        codegen_single_file_frontend(&frontend)
    }

    fn into_generated_binary_project(
        self,
    ) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
        self.into_generated_binary_project_with_probe_policy(false, DirectProbePolicy::ExecuteAll)
    }

    pub(super) fn into_generated_binary_project_with_probe_policy(
        self,
        allow_deferred_python_probes: bool,
        direct_probe_policy: DirectProbePolicy,
    ) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
        let python_runtime = self.python_runtime.clone();
        let python_bridges = self.python_bridges.clone();
        let rust_interop_context = self.rust_interop_context.clone();
        let cargo_resolution = self.cargo_resolution.clone();
        let stdlib_interop = self.stdlib.interop.clone();
        let generated = match self.shape {
            RootedEntrypointShape::SingleFile => {
                let codegen_result = self.into_single_file_codegen_result()?;
                generated_single_file_binary_project(codegen_result)
            }
            RootedEntrypointShape::Project => {
                generated_project_binary_project(&self.stdlib.code, self.project_lowering)?
            }
        };
        let (generated, rust_interop_context) =
            attach_stdlib_rust_interop(generated, rust_interop_context, &stdlib_interop);
        let generated = super::rust_interop::apply_package_rust_interop_metadata_with_resolution(
            generated,
            rust_interop_context,
            &cargo_resolution,
            direct_probe_policy,
        )?;
        let generated = apply_package_python_bridge_metadata(generated, python_bridges.as_ref());
        if allow_deferred_python_probes {
            let generated = super::python_interop::apply_python_interop_metadata_for_check(
                generated,
                python_runtime.as_ref(),
            )?;
            Ok(
                super::project_codegen::attach_package_runtime_metadata_for_check(
                    generated,
                    python_runtime,
                ),
            )
        } else {
            let generated = super::python_interop::apply_python_interop_metadata(
                generated,
                python_runtime.as_ref(),
            )?;
            apply_package_runtime_metadata(generated, python_runtime)
        }
    }
}

#[cfg(test)]
#[path = "entrypoint_tests.rs"]
mod tests;
