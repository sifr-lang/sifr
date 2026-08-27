use crate::errors::{LspError, LspResult};
use crate::python_input_fingerprint::package_input_fingerprint;
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::FileId;
use sifr_analysis::{DiskSourceProvider, SourceProvider};
use sifr_compiler_services::{
    PackagePythonRuntime, PythonInteropPlan, PythonInteropPlanDiagnostic, PythonTargetInspection,
    PythonTargetProbeStatus,
};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, DiagnosticSpan, RenderedDiagnostic};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PythonDeclarationInsight {
    pub(crate) file: FileId,
    pub(crate) name: String,
    pub(crate) target: String,
    pub(crate) kind: String,
    pub(crate) status: &'static str,
    pub(crate) policy_help: &'static str,
}

#[derive(Clone, Debug)]
struct ScopedDiagnostic {
    file: Option<FileId>,
    diagnostic: RenderedDiagnostic,
}

#[derive(Clone, Debug)]
struct PackageSnapshot {
    insights: Vec<PythonDeclarationInsight>,
    diagnostics: Vec<ScopedDiagnostic>,
}

#[derive(Clone, Debug)]
pub(crate) struct PythonDeclarationSnapshot {
    pub(crate) insights: Vec<PythonDeclarationInsight>,
    pub(crate) diagnostics: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    graph_revision: u64,
    source_revision: u64,
    package_diagnostic_owner: Option<String>,
    snapshot: PackageSnapshot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PythonDeclarationCacheStats {
    pub(crate) hits: u64,
    pub(crate) misses: u64,
    pub(crate) external_fingerprint_runs: u64,
    pub(crate) snapshot_builds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EnvironmentCacheKey {
    package_root: PathBuf,
    external_fingerprint: String,
    required_import_roots: Vec<String>,
}

#[derive(Clone, Debug)]
struct EnvironmentSnapshot {
    runtime: Option<PackagePythonRuntime>,
    diagnostics: Vec<RenderedDiagnostic>,
}

#[derive(Default)]
pub(crate) struct PythonDeclarationCache {
    entries: BTreeMap<PathBuf, CacheEntry>,
    environments: BTreeMap<EnvironmentCacheKey, EnvironmentSnapshot>,
    target_inspections: BTreeMap<(PathBuf, String, String), Result<PythonTargetInspection, String>>,
    stats: PythonDeclarationCacheStats,
    #[cfg(test)]
    probe_runs: usize,
    #[cfg(test)]
    environment_probe_runs: usize,
}

impl PythonDeclarationCache {
    pub(crate) fn invalidate_source(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn invalidate_external(&mut self) {
        self.entries.clear();
        self.environments.clear();
        self.target_inspections.clear();
    }

    #[cfg(test)]
    pub(crate) const fn probe_runs(&self) -> usize {
        self.probe_runs
    }

    #[cfg(test)]
    pub(crate) const fn environment_probe_runs(&self) -> usize {
        self.environment_probe_runs
    }

    pub(crate) const fn stats(&self) -> PythonDeclarationCacheStats {
        self.stats
    }
}

impl PackageSnapshot {
    fn for_document(
        &self,
        file: FileId,
        include_package_diagnostics: bool,
    ) -> PythonDeclarationSnapshot {
        PythonDeclarationSnapshot {
            insights: self.insights.clone(),
            diagnostics: self
                .diagnostics
                .iter()
                .filter(|diagnostic| {
                    diagnostic.file == Some(file)
                        || diagnostic.file.is_none() && include_package_diagnostics
                })
                .map(|diagnostic| diagnostic.diagnostic.clone())
                .collect(),
        }
    }
}

impl PythonDeclarationSnapshot {
    pub(crate) fn insight(&self, file: FileId, name: &str) -> Option<&PythonDeclarationInsight> {
        self.insights
            .iter()
            .find(|insight| insight.file == file && insight.name == name)
    }
}

impl Session {
    pub(crate) fn python_declaration_cache_stats(&self) -> PythonDeclarationCacheStats {
        self.python_declarations.stats()
    }

    pub(crate) fn python_declaration_snapshot(
        &mut self,
        uri: &str,
    ) -> LspResult<PythonDeclarationSnapshot> {
        self.check_active_request_cancelled()?;
        let document_path = self.store().document(uri)?.path().to_path_buf();
        let mut provider = DiskSourceProvider::new();
        let package_root = package_root_for(&document_path, &mut provider);
        let cache_key = package_root
            .clone()
            .unwrap_or_else(|| document_path.clone());
        let (graph_revision, source_revision, current_file) =
            self.with_document_analysis(uri, |snapshot, _host, file, _source| {
                Ok((
                    snapshot.revision().graph.as_u64(),
                    snapshot.revision().source.as_u64(),
                    file,
                ))
            })?;
        if let Some(entry) = self.python_declarations.entries.get(&cache_key) {
            if entry.graph_revision == graph_revision && entry.source_revision == source_revision {
                self.python_declarations.stats.hits += 1;
                let include_package_diagnostics = entry
                    .package_diagnostic_owner
                    .as_deref()
                    .is_none_or(|owner| owner == uri);
                return Ok(entry
                    .snapshot
                    .for_document(current_file, include_package_diagnostics));
            }
        }
        self.python_declarations.stats.misses += 1;
        let external_fingerprint = package_root.as_deref().map_or_else(String::new, |root| {
            self.python_declarations.stats.external_fingerprint_runs += 1;
            package_input_fingerprint(root, &mut provider)
        });
        let (analysis_plan, compiler_has_errors) =
            self.with_document_analysis(uri, |snapshot, host, _file, _source| {
                let plan = snapshot
                    .python_interop_plan(host)
                    .map_err(|error| LspError::internal(error.message))?
                    .into_value();
                let compiler_has_errors = snapshot
                    .workspace_diagnostics(host)
                    .map_err(|error| LspError::internal(error.message))?
                    .into_value()
                    .into_iter()
                    .flat_map(|file| file.diagnostics)
                    .any(|diagnostic| diagnostic.severity == sifr_diagnostics::Severity::Error);
                Ok((plan, compiler_has_errors))
            })?;
        self.python_declarations.stats.snapshot_builds += 1;
        let package_diagnostic_owner = package_root
            .as_deref()
            .and_then(|root| self.python_package_diagnostic_owner(root, &mut provider));

        let mut plan = analysis_plan.plan;
        let mut diagnostics = Vec::new();
        if let Some(root) = package_root.as_deref() {
            let environment =
                self.package_python_environment(root, &external_fingerprint, &plan, &mut provider);
            diagnostics.extend(environment.diagnostics.into_iter().map(|diagnostic| {
                ScopedDiagnostic {
                    file: None,
                    diagnostic,
                }
            }));
            if let Some(runtime) = environment.runtime {
                if !compiler_has_errors && !plan.declarations.is_empty() {
                    diagnostics.extend(scoped_diagnostics(
                        sifr_compiler_services::validate_protocol_certifications_for_plan(
                            &plan, &runtime,
                        ),
                        &analysis_plan.module_files,
                    ));
                    diagnostics.extend(self.probe_python_targets(
                        root,
                        &external_fingerprint,
                        &mut plan,
                        runtime.interpreter(),
                        &analysis_plan.module_files,
                    )?);
                } else {
                    mark_embedded_bridge_targets(&mut plan);
                }
            } else {
                mark_embedded_bridge_targets(&mut plan);
            }
        } else {
            mark_embedded_bridge_targets(&mut plan);
        }
        self.check_active_request_cancelled()?;
        let snapshot = PackageSnapshot {
            insights: declaration_insights(&plan, &analysis_plan.module_files),
            diagnostics,
        };
        self.python_declarations.entries.insert(
            cache_key,
            CacheEntry {
                graph_revision,
                source_revision,
                package_diagnostic_owner: package_diagnostic_owner.clone(),
                snapshot: snapshot.clone(),
            },
        );
        let include_package_diagnostics = package_diagnostic_owner
            .as_deref()
            .is_none_or(|owner| owner == uri);
        Ok(snapshot.for_document(current_file, include_package_diagnostics))
    }

    fn package_python_environment(
        &mut self,
        package_root: &Path,
        external_fingerprint: &str,
        plan: &PythonInteropPlan,
        provider: &mut impl SourceProvider,
    ) -> EnvironmentSnapshot {
        let mut required_import_roots = plan.required_import_roots.clone();
        required_import_roots.sort();
        required_import_roots.dedup();
        let key = EnvironmentCacheKey {
            package_root: package_root.to_path_buf(),
            external_fingerprint: external_fingerprint.to_string(),
            required_import_roots: required_import_roots.clone(),
        };
        if let Some(snapshot) = self.python_declarations.environments.get(&key) {
            return snapshot.clone();
        }
        let snapshot =
            resolve_package_python_environment(package_root, &required_import_roots, provider);
        #[cfg(test)]
        if snapshot.runtime.is_some() {
            self.python_declarations.environment_probe_runs += 1;
        }
        self.python_declarations
            .environments
            .insert(key, snapshot.clone());
        snapshot
    }

    fn probe_python_targets(
        &mut self,
        package_root: &Path,
        external_fingerprint: &str,
        plan: &mut PythonInteropPlan,
        interpreter: &Path,
        module_files: &BTreeMap<String, FileId>,
    ) -> LspResult<Vec<ScopedDiagnostic>> {
        mark_embedded_bridge_targets(plan);
        let mut seen_targets = std::collections::BTreeSet::new();
        let targets = plan
            .target_probes
            .iter()
            .filter(|probe| !probe.target_path.starts_with("__sifr_bridge__."))
            .filter(|probe| seen_targets.insert(probe.target_path.clone()))
            .map(|probe| probe.target_path.clone())
            .collect::<Vec<_>>();
        let mut diagnostics = Vec::new();
        for target in targets {
            self.check_active_request_cancelled()?;
            let key = (
                package_root.to_path_buf(),
                external_fingerprint.to_string(),
                target.clone(),
            );
            let inspection =
                if let Some(inspection) = self.python_declarations.target_inspections.get(&key) {
                    inspection.clone()
                } else {
                    let inspection =
                        sifr_compiler_services::inspect_python_target(interpreter, &target);
                    #[cfg(test)]
                    {
                        self.python_declarations.probe_runs += 1;
                    }
                    self.python_declarations
                        .target_inspections
                        .insert(key, inspection.clone());
                    inspection
                };
            diagnostics.extend(scoped_diagnostics(
                sifr_compiler_services::apply_python_target_inspection(
                    plan,
                    &target,
                    inspection.as_ref().map_err(String::as_str),
                ),
                module_files,
            ));
        }
        Ok(diagnostics)
    }

    fn python_package_diagnostic_owner(
        &self,
        package_root: &Path,
        provider: &mut impl SourceProvider,
    ) -> Option<String> {
        self.document_uris()
            .into_iter()
            .filter(|candidate| self.can_analyze_document(candidate))
            .filter_map(|candidate| {
                let path = self.store().document(&candidate).ok()?.path();
                (package_root_for(path, provider).as_deref() == Some(package_root))
                    .then_some(candidate)
            })
            .min()
    }
}

fn resolve_package_python_environment(
    package_root: &Path,
    required_import_roots: &[String],
    provider: &mut impl SourceProvider,
) -> EnvironmentSnapshot {
    if !provider.is_file(&package_root.join("Cargo.toml")) {
        return EnvironmentSnapshot {
            runtime: None,
            diagnostics: Vec::new(),
        };
    }
    match resolve_package_python_environment_inner(package_root, required_import_roots, provider) {
        Ok(snapshot) => snapshot,
        Err(diagnostics) => EnvironmentSnapshot {
            runtime: None,
            diagnostics,
        },
    }
}

fn resolve_package_python_environment_inner(
    package_root: &Path,
    required_import_roots: &[String],
    provider: &mut impl SourceProvider,
) -> Result<EnvironmentSnapshot, Vec<RenderedDiagnostic>> {
    let session = sifr_package::PackageSession::discover(
        sifr_package::PackageSessionOptions {
            current_dir: package_root.to_path_buf(),
            lock_mode: sifr_package::CargoLockMode::Frozen,
        },
        provider,
    )
    .map_err(|error| vec![sifr_compiler_services::render_package_diagnostic(error)])?;
    let snapshot = match sifr_package::load_package_graph_snapshot(
        &session.workspace_root,
        sifr_package::CargoLockMode::Frozen,
        provider,
    ) {
        Ok(snapshot) => snapshot,
        Err(failure) if missing_lockfile_frozen_failure(&failure) => {
            return Ok(EnvironmentSnapshot {
                runtime: None,
                diagnostics: Vec::new(),
            });
        }
        Err(failure) => return Err(render_package_diagnostics(failure.into_diagnostics())),
    };
    let package_id = session.package_id(&snapshot.graph).ok_or_else(|| {
        vec![diagnostic_with_code(
            DiagnosticCode::PACKAGE_METADATA_PARSE,
            "current Sifr package is missing from the Cargo package graph".to_string(),
            "repair Cargo package metadata before requesting Python editor status".to_string(),
        )]
    })?;
    let mut requirements = required_import_roots
        .iter()
        .map(|root| sifr_package::PythonRequirementContribution {
            root: root.clone(),
            package_id: package_id.clone(),
            kind: sifr_package::PythonRequirementKind::Declaration,
            source: "language-server compiler plan".to_string(),
        })
        .collect::<Vec<_>>();
    let bridge = sifr_package::resolve_python_bridge_graph(&snapshot.graph, &package_id)
        .map_err(render_package_diagnostics)?;
    requirements.extend(bridge.requirements);
    let allow_deferral = session
        .runnable_app_paths()
        .map_err(|error| vec![sifr_compiler_services::render_package_diagnostic(error)])?
        .is_empty();
    let resolution = sifr_package::resolve_python_environment_for_check(
        &snapshot.graph,
        &package_id,
        &requirements,
        allow_deferral,
    )
    .map_err(render_package_diagnostics)?;
    let sifr_package::PythonEnvironmentResolution::Resolved(resolved) = resolution else {
        return Ok(EnvironmentSnapshot {
            runtime: None,
            diagnostics: Vec::new(),
        });
    };
    let request = sifr_package::PythonEnvironmentProbeRequest::from(&resolved);
    let probe = sifr_package::probe_python_environment(&request)
        .map_err(|error| vec![sifr_compiler_services::render_package_diagnostic(error)])?;
    let digest = sifr_package::digest_python_environment_probe(&request, &probe)
        .map_err(|error| {
            vec![diagnostic_with_code(
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                format!("could not serialize Python environment identity: {error}"),
                "retry the language-server request".to_string(),
            )]
        })?
        .hex;
    let mut runtime = PackagePythonRuntime::from_probe(
        &request,
        &probe,
        digest.clone(),
        resolved.required_imports,
        resolved.trusted_imports,
        resolved.trusted_native_imports,
    )
    .map_err(|error| {
        vec![diagnostic_with_code(
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            format!("could not serialize Python authoring environment identity: {error}"),
            "retry the language-server request".to_string(),
        )]
    })?;
    let mut diagnostics = Vec::new();
    let binding_path = package_root.join(sifr_package::PYTHON_BINDINGS_FILE);
    if binding_path.is_file() {
        match sifr_package::load_python_bindings(
            package_root,
            runtime.authoring_environment_digest(),
        ) {
            Ok(artifact) => match serde_json::to_string(&artifact.bindings) {
                Ok(identity) => runtime.set_binding_identity(identity),
                Err(error) => diagnostics.push(diagnostic_with_code(
                    DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
                    format!("could not fingerprint Python bindings: {error}"),
                    "rerun `sifr python bind --check`".to_string(),
                )),
            },
            Err(reason) => diagnostics.push(diagnostic_with_code(
                DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
                format!("invalid Python binding artifact: {reason}"),
                "rerun `sifr python bind --check`".to_string(),
            )),
        }
    }
    let certification_path = package_root.join(sifr_package::PYTHON_CERTIFICATIONS_FILE);
    if certification_path.is_file() {
        match sifr_package::load_python_certifications(package_root, &digest) {
            Ok(artifact) => {
                match sifr_compiler_services::validate_certification_distributions(
                    &runtime, &artifact,
                ) {
                    Ok(()) => {
                        runtime.set_arrow_certifications(artifact.arrow);
                        runtime.set_dlpack_certifications(artifact.dlpack);
                    }
                    Err(reason) => diagnostics.push(diagnostic_with_code(
                        DiagnosticCode::PYZC_INVALID_DECLARATION,
                        format!("invalid Python certification artifact: {reason}"),
                        "rerun `sifr python certify --check`".to_string(),
                    )),
                }
            }
            Err(reason) => diagnostics.push(diagnostic_with_code(
                DiagnosticCode::PYZC_INVALID_DECLARATION,
                format!("invalid Python certification artifact: {reason}"),
                "rerun `sifr python certify --check`".to_string(),
            )),
        }
    }
    Ok(EnvironmentSnapshot {
        runtime: diagnostics.is_empty().then_some(runtime),
        diagnostics,
    })
}

fn missing_lockfile_frozen_failure(failure: &sifr_package::PackageGraphLoadFailure) -> bool {
    if failure.plan.current_dir.join("Cargo.lock").exists() {
        return false;
    }
    let sifr_package::PackageGraphLoadFailureKind::Command { output, .. } = &failure.kind else {
        return false;
    };
    let output = output.to_ascii_lowercase();
    output.contains("lock file")
        && (output.contains("needs to be updated")
            || output.contains("cannot create")
            || output.contains("could not be updated"))
}

fn render_package_diagnostics(
    diagnostics: Vec<sifr_package::PackageDiagnostic>,
) -> Vec<RenderedDiagnostic> {
    diagnostics
        .into_iter()
        .map(sifr_compiler_services::render_package_diagnostic)
        .collect()
}

fn scoped_diagnostics(
    diagnostics: Vec<PythonInteropPlanDiagnostic>,
    module_files: &BTreeMap<String, FileId>,
) -> Vec<ScopedDiagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            let file = diagnostic
                .module_name
                .as_ref()
                .and_then(|module| module_files.get(module))
                .copied();
            ScopedDiagnostic {
                file,
                diagnostic: with_span(diagnostic.diagnostic, diagnostic.span),
            }
        })
        .collect()
}

fn with_span(
    mut diagnostic: RenderedDiagnostic,
    span: ruff_text_size::TextRange,
) -> RenderedDiagnostic {
    diagnostic.spans = vec![DiagnosticSpan {
        file: None,
        byte_start: span.start().to_u32(),
        byte_end: span.end().to_u32(),
        line: None,
        column: None,
        end_line: None,
        end_column: None,
        is_primary: true,
        label: Some("Python declaration".to_string()),
        lines: Vec::new(),
    }];
    diagnostic
}

fn declaration_insights(
    plan: &PythonInteropPlan,
    module_files: &BTreeMap<String, FileId>,
) -> Vec<PythonDeclarationInsight> {
    let statuses = plan
        .target_probes
        .iter()
        .map(|probe| (probe.target_path.as_str(), status_name(probe.status)))
        .collect::<HashMap<_, _>>();
    plan.declarations
        .iter()
        .filter_map(|declaration| {
            let module = declaration.module_name.as_ref()?;
            let file = *module_files.get(module)?;
            let target = declaration.declaration.target.as_ref()?.dotted();
            let kind = format!("{:?}", declaration.declaration.kind);
            Some(PythonDeclarationInsight {
                file,
                name: declaration.function_name.clone(),
                status: statuses.get(target.as_str()).copied().unwrap_or("deferred"),
                policy_help: policy_help(&kind),
                target,
                kind,
            })
        })
        .collect()
}

const fn status_name(status: PythonTargetProbeStatus) -> &'static str {
    match status {
        PythonTargetProbeStatus::Planned => "deferred",
        PythonTargetProbeStatus::Verified => "verified",
        PythonTargetProbeStatus::RuntimeChecked => "runtime-checked",
    }
}

fn policy_help(kind: &str) -> &'static str {
    match kind {
        "Coroutine" => "Runs on the application-owned Python loop with typed cancellation.",
        "Opaque" => {
            "Preserves sealed Python identity and the declaration's consuming cleanup policy."
        }
        "ContextEnter" | "ContextExit" | "ContextAsyncEnter" | "ContextAsyncExit" => {
            "Context cleanup is consuming and follows the declared suppression/error precedence."
        }
        "Callback" => {
            "Callback lifetime, dispatch, concurrency, and owner policy are compiler checked."
        }
        "Buffer" => "Buffer access is borrow-scoped with checked layout and exact release.",
        "Arrow" => "Arrow transfer is affine, certified, no-copy, and exact-release.",
        "Dlpack" | "DlpackStream" => {
            "DLPack transfer is one-shot, certified, no-copy, and exact-deleter."
        }
        _ => "Arguments and results use the compiler's closed typed Python conversion grammar.",
    }
}

fn mark_embedded_bridge_targets(plan: &mut PythonInteropPlan) {
    for probe in &mut plan.target_probes {
        if probe.target_path.starts_with("__sifr_bridge__.") {
            probe.status = PythonTargetProbeStatus::RuntimeChecked;
        }
    }
}

fn package_root_for(path: &Path, provider: &mut impl SourceProvider) -> Option<PathBuf> {
    let mut current = path.parent()?.to_path_buf();
    loop {
        if provider.is_file(&current.join("sifr.toml")) {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn diagnostic_with_code(code: DiagnosticCode, message: String, help: String) -> RenderedDiagnostic {
    let mut args = BTreeMap::new();
    args.insert(
        "message".to_string(),
        DiagnosticArg::String(message.clone()),
    );
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: "{message}".to_string(),
        args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: Some(help),
        suggestions: Vec::new(),
    }
}

pub(crate) fn enrich_completion_item(item: &mut Value, snapshot: &PythonDeclarationSnapshot) {
    let Some(label) = item.get("label").and_then(Value::as_str) else {
        return;
    };
    let Some(file) = item
        .pointer("/data/sifrFile")
        .and_then(Value::as_u64)
        .and_then(|file| u32::try_from(file).ok())
        .map(FileId::new)
    else {
        return;
    };
    let Some(insight) = snapshot.insight(file, label) else {
        return;
    };
    item["detail"] = Value::String(format!(
        "Python {} · {} · {}",
        insight.kind, insight.status, insight.target
    ));
    item["documentation"] = serde_json::json!({
        "kind": "markdown",
        "value": format!(
            "Python target `{}` is **{}**. {}",
            insight.target, insight.status, insight.policy_help
        )
    });
    item["data"]["pythonStatus"] = Value::String(insight.status.to_string());
    item["data"]["pythonTarget"] = Value::String(insight.target.clone());
}

pub(crate) fn enrich_hover(
    hover: &mut Value,
    snapshot: &PythonDeclarationSnapshot,
    file: FileId,
    symbol_name: &str,
) {
    let Some(insight) = snapshot.insight(file, symbol_name) else {
        return;
    };
    let Some(contents) = hover.pointer_mut("/contents/value") else {
        return;
    };
    let Some(rendered) = contents.as_str() else {
        return;
    };
    *contents = Value::String(format!(
        "{rendered}\n\nPython target: `{}`  \nStatus: **{}**  \n{}",
        insight.target, insight.status, insight.policy_help
    ));
}

#[cfg(test)]
mod tests {
    use super::{package_input_fingerprint, policy_help};
    use sifr_analysis::DiskSourceProvider;

    #[test]
    fn protocol_policy_help_covers_affine_and_callback_contracts() {
        assert!(policy_help("Arrow").contains("affine"));
        assert!(policy_help("Dlpack").contains("one-shot"));
        assert!(policy_help("Buffer").contains("borrow-scoped"));
        assert!(policy_help("Callback").contains("lifetime"));
        assert!(policy_help("ContextExit").contains("consuming"));
    }

    #[test]
    fn package_fingerprint_tracks_manifest_selected_metadata_paths() {
        let root = tempfile::tempdir().expect("temporary package");
        let metadata = root.path().join("python");
        std::fs::create_dir(&metadata).expect("metadata directory");
        std::fs::write(
            root.path().join("sifr.toml"),
            "[package]\nname = \"lsp-fingerprint\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[python]\ninterpreter = \"python-bin\"\npyproject = \"python/pyproject.toml\"\nlock = \"python/uv.lock\"\n",
        )
        .expect("manifest");
        std::fs::write(metadata.join("pyproject.toml"), "project-a").expect("pyproject");
        std::fs::write(metadata.join("uv.lock"), "lock-a").expect("selected lock");
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"lsp-fingerprint\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n\n[workspace]\n",
        )
        .expect("Cargo manifest");
        std::fs::write(
            root.path().join("Cargo.lock"),
            "# This file is automatically @generated by Cargo.\n# It is not intended for manual editing.\nversion = 4\n\n[[package]]\nname = \"lsp-fingerprint\"\nversion = \"0.1.0\"\n",
        )
        .expect("Cargo lock");

        let mut provider = DiskSourceProvider::new();
        let initial = package_input_fingerprint(root.path(), &mut provider);
        std::fs::write(root.path().join("uv.lock"), "unselected-lock-change")
            .expect("unselected lock");
        assert_eq!(
            initial,
            package_input_fingerprint(root.path(), &mut provider)
        );

        std::fs::write(metadata.join("uv.lock"), "lock-b").expect("selected lock drift");
        assert_ne!(
            initial,
            package_input_fingerprint(root.path(), &mut provider)
        );

        let before_app = package_input_fingerprint(root.path(), &mut provider);
        std::fs::create_dir(root.path().join("src")).expect("source directory");
        std::fs::write(root.path().join("src/main.sifr"), "def main():\n    pass\n")
            .expect("application entrypoint");
        assert_ne!(
            before_app,
            package_input_fingerprint(root.path(), &mut provider)
        );
    }
}
