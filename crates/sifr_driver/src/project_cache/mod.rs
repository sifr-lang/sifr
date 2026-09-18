//! Driver-owned project storage behind the frontend's completed result contract.
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

#[derive(Debug, Default, serde::Serialize)]
pub struct ProjectCacheReport {
    pub status: String,
    pub restored_checks: usize,
    pub computed_checks: usize,
    pub captured_sources: usize,
    pub validation_us: u128,
    pub serialization_us: u128,
    pub retained_bytes: usize,
}

/// Saved-source CLI checks only. Package commands retain their live trust,
/// component, schema, Python and native checks; no unresolved context is cached.
pub fn check_saved_sources(
    compiler: &crate::CompilerContext,
    file: &Path,
    provider: &mut dyn SourceProvider,
    enabled: bool,
    compute: impl FnOnce(&mut dyn SourceProvider) -> Vec<RenderedDiagnostic>,
) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    if !enabled {
        return (
            compute(provider),
            ProjectCacheReport {
                status: "disabled".into(),
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
    let cwd = std::env::current_dir().unwrap_or_default();
    let inputs = SemanticInputs {
        compiler: compiler.identity().as_str().into(),
        metadata: metadata.metadata.metadata_id.clone(),
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        workspace_and_source_policy: identity("saved-check-policy-v1", &(file.parent(), &cwd))
            .unwrap_or_default(),
        package_and_lock: "manifestless-owner-v1".into(),
        language_options: "ordinary-check-defaults-v1".into(),
        diagnostic_policy: "canonical-source-diagnostics-v1".into(),
        components: BTreeMap::new(),
        required_external: Default::default(),
        external: BTreeMap::new(),
    };
    check(
        &crate::cache_storage::root(),
        file,
        provider,
        inputs,
        &AtomicBool::new(false),
        compute,
    )
}
fn check(
    cache: &Path,
    file: &Path,
    provider: &mut dyn SourceProvider,
    inputs: SemanticInputs,
    cancel: &AtomicBool,
    compute: impl FnOnce(&mut dyn SourceProvider) -> Vec<RenderedDiagnostic>,
) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    let mut report = ProjectCacheReport::default();
    let validation = Instant::now();
    let root = file
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let store = inputs
        .identity()
        .ok()
        .and_then(|context| storage::Store::open(cache, root, &context).ok());
    let mut capture = CapturingSourceProvider::new(provider);
    if let Some(store) = &store {
        if let Ok(Some(generation)) = store.latest() {
            for record in generation.records().flatten() {
                if cancel.load(Ordering::Acquire) {
                    break;
                }
                if record.result.inputs.source.path == file
                    && record.validate(&inputs, &mut capture)
                {
                    if let Ok(diagnostics) = record.diagnostics() {
                        report.status = "restored".into();
                        report.restored_checks = 1;
                        report.captured_sources = record
                            .result
                            .resolution
                            .ready()
                            .map_or(0, |resolution| resolution.sources.len());
                        report.retained_bytes =
                            serde_json::to_vec(&record).map_or(0, |bytes| bytes.len());
                        report.validation_us = validation.elapsed().as_micros();
                        return (diagnostics, report);
                    }
                }
            }
        }
    }
    report.validation_us = validation.elapsed().as_micros();
    // Validation's captured bytes are reused by the actual checker. Publishing
    // replays observations against disk; edits during capture cannot be mislabeled.
    let diagnostics = compute(&mut capture);
    report.computed_checks = 1;
    report.captured_sources = capture.sources().len();
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
    let serialization = Instant::now();
    if let Some(store) = &store {
        match CompletedCheck::capture(file, inputs, &capture, &diagnostics) {
            Ok(record) if capture.unchanged() => {
                report.retained_bytes = serde_json::to_vec(&record).map_or(0, |bytes| bytes.len());
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
