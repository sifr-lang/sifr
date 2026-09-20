//! Driver-owned project storage behind the frontend's completed result contract.
#[cfg(test)]
mod dx14_tests;
#[cfg(test)]
mod history_tests;
mod interface_reuse;
mod package_context;
#[cfg(test)]
mod package_reuse_tests;
mod storage;
#[cfg(test)]
mod tests;
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{
    SourceProvider,
    persistence::{CapturingSourceProvider, CompletedCheck, SemanticInputs, identity},
};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

/// The computation owner explicitly attests that no live external operation
/// would be skipped by reusing this completed source result.
pub struct CheckComputation {
    pub diagnostics: Vec<RenderedDiagnostic>,
    pub reusable: bool,
}
impl From<Vec<RenderedDiagnostic>> for CheckComputation {
    fn from(diagnostics: Vec<RenderedDiagnostic>) -> Self {
        Self {
            diagnostics,
            reusable: true,
        }
    }
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ProjectCacheReport {
    pub status: String,
    pub restored_checks: usize,
    pub computed_checks: usize,
    pub captured_sources: usize,
    pub validation_us: u128,
    pub serialization_us: u128,
    pub payload_bytes: usize,
    pub retained_records: usize,
    pub candidate_checks: usize,
    pub interface_proofs: usize,
    pub observation_count: usize,
    pub modules: Vec<sifr_frontend::ModuleCheckDecision>,
}

/// Saved-source checks with a complete resolved context. Pure package graphs
/// are supplied only after ordinary package resolution. Dynamic component,
/// schema, Python and native contexts retain their live owner checks.
pub fn check_saved_sources(
    compiler: &crate::CompilerContext,
    file: &Path,
    provider: &mut dyn SourceProvider,
    enabled: bool,
    package: Option<&crate::PackageEntrypoint>,
    compute: impl FnOnce(&mut dyn SourceProvider) -> CheckComputation,
) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    if !enabled {
        return (
            compute(provider).diagnostics,
            ProjectCacheReport {
                status: "disabled".into(),
                computed_checks: 1,
                ..Default::default()
            },
        );
    }
    let package_context = package.map(package_context::identity);
    if matches!(package_context, Some(None)) {
        return (
            compute(provider).diagnostics,
            ProjectCacheReport {
                status: "external-context".into(),
                computed_checks: 1,
                ..Default::default()
            },
        );
    }
    // Resolve/pin required installed metadata even on a project-cache hit.
    let metadata = match compiler.metadata_provider() {
        Ok(metadata) => metadata,
        Err(errors) => {
            return (
                errors,
                ProjectCacheReport {
                    status: "metadata-unavailable".into(),
                    ..Default::default()
                },
            );
        }
    };
    let normalized_file = if let Some(package) = package {
        let absolute = std::path::absolute(file).ok();
        package
            .source_map
            .modules
            .values()
            .find(|module| std::path::absolute(&module.file_path).ok() == absolute)
            .map(|module| module.file_path.clone())
            .unwrap_or_else(|| file.to_path_buf())
    } else {
        file.to_path_buf()
    };
    let file = normalized_file.as_path();
    let workspace = package
        .and_then(|package| package.graph.packages.get(&package.package_id))
        .map(|package| package.package_root.as_path())
        .unwrap_or_else(|| {
            file.parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
        });
    let mut inputs = manifestless_inputs(compiler, &metadata.metadata.metadata_id, file);
    inputs.package_and_lock = package_context
        .flatten()
        .unwrap_or_else(|| "manifestless-owner-v1".into());
    let defs = match crate::stdlib_external_defs(compiler) {
        Ok(defs) => defs,
        Err(errors) => {
            return (
                errors,
                ProjectCacheReport {
                    status: "metadata-unavailable".into(),
                    ..Default::default()
                },
            );
        }
    };
    check(
        (&crate::cache_storage::root(), workspace),
        file,
        provider,
        inputs,
        &AtomicBool::new(false),
        Some((compiler.identity(), &defs, package)),
        compute,
    )
}
fn check(
    roots: (&Path, &Path),
    file: &Path,
    provider: &mut dyn SourceProvider,
    inputs: SemanticInputs,
    cancel: &AtomicBool,
    module_context: Option<(
        &sifr_identity::CompilerIdentity,
        &sifr_lowering::ExternalDefs,
        Option<&crate::PackageEntrypoint>,
    )>,
    compute: impl FnOnce(&mut dyn SourceProvider) -> CheckComputation,
) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    let (cache, workspace) = roots;
    let mut report = ProjectCacheReport::default();
    let validation = Instant::now();
    let store = inputs
        .identity()
        .ok()
        .and_then(|context| storage::Store::open(cache, workspace, &context).ok());
    let mut capture = CapturingSourceProvider::new(provider);
    if let Some(store) = &store {
        if let Some(generation) = store.latest() {
            report.retained_records = generation.manifest.records.len();
            let records: Vec<_> = generation.records().flatten().collect();
            for record in &records {
                if cancel.load(Ordering::Acquire) {
                    break;
                }
                report.candidate_checks += 1;
                if record.result.inputs.source.path == file
                    && record.validate(&inputs, &mut capture)
                {
                    if let Ok(diagnostics) = record.diagnostics() {
                        report.observation_count = capture.observations().len();
                        report.status = "restored".into();
                        report.modules =
                            record.result.resolution.ready().map_or_else(Vec::new, |r| {
                                r.sources
                                    .iter()
                                    .filter(|source| {
                                        source.path.extension().is_some_and(|ext| ext == "sifr")
                                    })
                                    .map(|source| sifr_frontend::ModuleCheckDecision {
                                        path: source.path.clone(),
                                        family: "diagnostics",
                                        action: "restored",
                                        reason: "exact-source-and-context",
                                    })
                                    .collect()
                            });
                        report.restored_checks = 1;
                        report.captured_sources = record
                            .result
                            .resolution
                            .ready()
                            .map_or(0, |resolution| resolution.sources.len());
                        report.payload_bytes =
                            serde_json::to_vec(&record).map_or(0, |bytes| bytes.len());
                        report.validation_us = validation.elapsed().as_micros();
                        return (diagnostics, report);
                    }
                }
            }
            for record in &records {
                if cancel.load(Ordering::Acquire) || report.interface_proofs == 1 {
                    break;
                }
                if let Some((compiler, defs, package)) = module_context {
                    if !interface_reuse::candidate(record, file, &inputs, &mut capture) {
                        continue;
                    }
                    report.interface_proofs += 1;
                    if let Some(modules) = interface_reuse::restore(
                        &record,
                        file,
                        &inputs,
                        &mut capture,
                        compiler,
                        defs.clone(),
                        package,
                    ) {
                        report.status = "interface-restored".into();
                        report.restored_checks = 1;
                        report.modules = modules;
                        report.captured_sources = capture.sources().len();
                        report.observation_count = capture.observations().len();
                        report.validation_us = validation.elapsed().as_micros();
                        let serialization = Instant::now();
                        match CompletedCheck::capture(file, inputs.clone(), &capture, &[]) {
                            Ok(updated)
                                if !cancel.load(Ordering::Acquire) && capture.unchanged() =>
                            {
                                report.payload_bytes =
                                    serde_json::to_vec(&updated).map_or(0, |bytes| bytes.len());
                                if store.publish(&updated, cancel).is_err() {
                                    report.status = "interface-restored-write-unavailable".into();
                                }
                            }
                            Ok(_) => {
                                report.status = if cancel.load(Ordering::Acquire) {
                                    "cancelled".into()
                                } else {
                                    "interface-restored-changed-inputs".into()
                                }
                            }
                            Err(_) => report.status = "interface-restored-uncacheable".into(),
                        }
                        report.serialization_us = serialization.elapsed().as_micros();
                        return (Vec::new(), report);
                    }
                }
            }
        }
    }
    report.validation_us = validation.elapsed().as_micros();
    // Validation's captured bytes are reused by the actual checker. Publishing
    // replays observations against disk; edits during capture cannot be mislabeled.
    let computation = compute(&mut capture);
    let diagnostics = computation.diagnostics;
    report.computed_checks = 1;
    report.modules = capture
        .sources()
        .iter()
        .filter(|source| source.path.extension().is_some_and(|ext| ext == "sifr"))
        .map(|source| sifr_frontend::ModuleCheckDecision {
            path: source.path.clone(),
            family: "diagnostics",
            action: "computed",
            reason: "no-proven-compatible-record",
        })
        .collect();
    report.captured_sources = capture.sources().len();
    report.observation_count = capture.observations().len();
    report.status = if store.is_some() {
        "miss"
    } else {
        "unavailable"
    }
    .into();
    if cancel.load(Ordering::Acquire) {
        report.status = "cancelled".into();
        return (diagnostics, report);
    }
    if !computation.reusable {
        report.status = "external-context".into();
        return (diagnostics, report);
    }
    let serialization = Instant::now();
    if let Some(store) = &store {
        match CompletedCheck::capture(file, inputs, &capture, &diagnostics) {
            Ok(record) if capture.unchanged() => {
                report.payload_bytes = serde_json::to_vec(&record).map_or(0, |bytes| bytes.len());
                if store.publish(&record, cancel).is_ok() {
                    report.status = "published".into();
                } else {
                    report.status = "write-unavailable".into();
                }
            }
            Ok(_) => report.status = "changed-inputs".into(),
            Err(_) => report.status = "uncacheable".into(),
        }
    }
    report.serialization_us = serialization.elapsed().as_micros();
    (diagnostics, report)
}

/// Explicit project-only pressure cleanup. Does not inspect metadata or Cargo.
/// All semantic contexts under this exact canonical workspace remain separately
/// leased; dry-run and no-pressure calls never delete generations.
pub fn prune_project_cache(
    workspace: &Path,
    pressure: bool,
    dry_run: bool,
) -> std::io::Result<usize> {
    let cache = crate::cache_storage::root();
    let workspace = workspace.canonicalize()?;
    let workspace_id =
        identity("project-workspace-v1", &workspace).map_err(std::io::Error::other)?;
    let parent = cache.join("projects").join(workspace_id);
    if !parent.exists() {
        return Ok(0);
    }
    crate::cache_storage::directory(&parent)?;
    let mut removed = 0;
    for entry in std::fs::read_dir(&parent)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(context) = name.to_str() else {
            continue;
        };
        if context.len() != 64 || !context.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            continue;
        }
        crate::cache_storage::check_owned(&entry.path())?;
        match storage::Store::open(&cache, &workspace, context)?.prune(pressure, dry_run) {
            Ok(count) => removed += count,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error),
        }
    }
    Ok(removed)
}

fn manifestless_inputs(
    compiler: &crate::CompilerContext,
    metadata: &str,
    file: &Path,
) -> SemanticInputs {
    let cwd = std::env::current_dir().unwrap_or_default();

    SemanticInputs {
        compiler: compiler.identity().as_str().into(),
        metadata: metadata.into(),
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        workspace_and_source_policy: identity("saved-check-policy-v1", &(file.parent(), &cwd))
            .unwrap_or_default(),
        package_and_lock: "manifestless-owner-v1".into(),
        language_options: "ordinary-check-defaults-v1".into(),
        diagnostic_policy: "canonical-source-diagnostics-v1".into(),
        components: BTreeMap::new(),
        required_external: Default::default(),
        external: BTreeMap::new(),
    }
}

/// Restore saved diagnostic facts into an already captured editor generation.
/// This function never publishes editor state. Both disk observations and the
/// frontend's captured source bytes must agree with the CLI record.
pub fn restore_editor_checks(
    compiler: &crate::CompilerContext,
    session: &mut sifr_frontend::WorkspaceSession,
) -> Vec<sifr_frontend::ModuleCheckDecision> {
    if !compiler.project_incremental() {
        return Vec::new();
    }
    let Some(frontend) = session.context() else {
        return Vec::new();
    };
    let graph = frontend.module_graph();
    let Some(entry) = graph
        .modules
        .iter()
        .find(|module| module.id == graph.entrypoint)
    else {
        return Vec::new();
    };
    let file = entry.canonical_path.as_path();
    let Some(workspace) = file.parent() else {
        return Vec::new();
    };
    let Ok(metadata) = compiler.metadata_provider() else {
        return Vec::new();
    };
    let inputs = manifestless_inputs(compiler, &metadata.metadata.metadata_id, file);
    let Ok(context) = inputs.identity() else {
        return Vec::new();
    };
    let Ok(store) = storage::Store::open(&crate::cache_storage::root(), workspace, &context) else {
        return Vec::new();
    };
    let Some(generation) = store.latest() else {
        return Vec::new();
    };
    for record in generation.records().flatten() {
        if record.result.inputs.source.path == file {
            if let Some(decisions) = session.restore_saved_checks(&record, &inputs) {
                return decisions;
            }
        }
    }
    Vec::new()
}
