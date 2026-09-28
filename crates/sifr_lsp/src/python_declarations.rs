use crate::errors::{LspError, LspResult};
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::FileId;
use sifr_analysis::{DiskSourceProvider, SourceProvider};
use sifr_compiler_services::python::{
    PythonEditorEnvironment, PythonInteropPlan, PythonInteropPlanDiagnostic,
    PythonTargetInspection, apply_python_target_inspection, inspect_python_target_if_active,
    mark_embedded_bridge_targets, policy_help, resolve_editor_environment, status_name,
    validate_protocol_certifications_for_plan,
};
use sifr_diagnostics::{DiagnosticSpan, RenderedDiagnostic};
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
    external_fingerprint: u64,
    snapshot: PackageSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EnvironmentCacheKey {
    package_root: PathBuf,
    external_fingerprint: u64,
    required_import_roots: Vec<String>,
}

type EnvironmentSnapshot = PythonEditorEnvironment;

#[derive(Default)]
pub(crate) struct PythonDeclarationCache {
    entries: BTreeMap<PathBuf, CacheEntry>,
    environments: BTreeMap<EnvironmentCacheKey, EnvironmentSnapshot>,
    target_inspections: BTreeMap<(PathBuf, u64, String), Result<PythonTargetInspection, String>>,
    #[cfg(test)]
    snapshot_builds: usize,
    #[cfg(test)]
    analysis_plan_builds: usize,
    #[cfg(test)]
    before_verification_change: Option<(PathBuf, Option<Vec<u8>>)>,
    #[cfg(test)]
    probe_runs: usize,
    #[cfg(test)]
    environment_probe_runs: usize,
}

impl PythonDeclarationCache {
    pub(crate) fn invalidate_source_for_path(&mut self, path: &Path) {
        let mut provider = DiskSourceProvider::new();
        let root = package_root_for(path, &mut provider).unwrap_or_else(|| path.to_path_buf());
        self.entries.remove(&root);
    }

    pub(crate) fn retain_open_projects(
        &mut self,
        documents: &crate::document_store::DocumentStore,
    ) {
        let mut provider = DiskSourceProvider::new();
        let roots = documents
            .documents()
            .map(|document| {
                package_root_for(document.path(), &mut provider)
                    .unwrap_or_else(|| document.path().to_path_buf())
            })
            .collect::<std::collections::BTreeSet<_>>();
        self.entries.retain(|root, _| roots.contains(root));
        self.environments
            .retain(|key, _| roots.contains(&key.package_root));
        self.target_inspections
            .retain(|(root, _, _), _| roots.contains(root));
    }

    pub(crate) fn invalidate_external_root(&mut self, root: &Path) {
        self.entries.remove(root);
        self.environments.retain(|key, _| key.package_root != root);
        self.target_inspections
            .retain(|(owner, _, _), _| owner != root);
    }

    pub(crate) fn invalidate_external(&mut self) {
        self.entries.clear();
        self.environments.clear();
        self.target_inspections.clear();
    }

    #[cfg(test)]
    pub(crate) const fn snapshot_builds(&self) -> usize {
        self.snapshot_builds
    }

    #[cfg(test)]
    pub(crate) const fn analysis_plan_builds(&self) -> usize {
        self.analysis_plan_builds
    }

    #[cfg(test)]
    pub(crate) fn inject_external_change_before_verification(
        &mut self,
        path: PathBuf,
        contents: Option<Vec<u8>>,
    ) {
        self.before_verification_change = Some((path, contents));
    }

    #[cfg(test)]
    pub(crate) fn has_entry(&self, root: &Path) -> bool {
        self.entries.contains_key(root)
    }

    #[cfg(test)]
    pub(crate) fn has_environment(&self, root: &Path) -> bool {
        self.environments.keys().any(|key| key.package_root == root)
    }

    #[cfg(test)]
    pub(crate) const fn probe_runs(&self) -> usize {
        self.probe_runs
    }

    #[cfg(test)]
    pub(crate) const fn environment_probe_runs(&self) -> usize {
        self.environment_probe_runs
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
    pub(crate) fn python_declaration_snapshot(
        &mut self,
        uri: &str,
    ) -> LspResult<PythonDeclarationSnapshot> {
        self.check_active_request_cancelled()?;
        let document_path = self.store().document(uri)?.path().to_path_buf();
        let mut provider = DiskSourceProvider::new();
        let package_root = package_root_for(&document_path, &mut provider);
        let (graph_revision, source_revision, current_file) =
            self.with_document_analysis(uri, |snapshot, _host, file, _source| {
                Ok((
                    snapshot.revision().graph.as_u64(),
                    snapshot.revision().source.as_u64(),
                    file,
                ))
            })?;
        let cache_key = package_root
            .clone()
            .unwrap_or_else(|| document_path.clone());
        let external_fingerprint = self.external_input_generation_for_path(&document_path);
        let package_owner = package_root
            .as_deref()
            .is_none_or(|root| self.is_python_package_diagnostic_owner(uri, root, &mut provider));
        if let Some(entry) = self.python_declarations.entries.get(&cache_key) {
            if entry.graph_revision == graph_revision
                && entry.source_revision == source_revision
                && entry.external_fingerprint == external_fingerprint
            {
                let result = entry.snapshot.for_document(current_file, package_owner);
                self.check_active_request_cancelled()?;
                self.verify_python_request_input(&document_path, &cache_key, external_fingerprint)?;
                return Ok(result);
            }
        }

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

        #[cfg(test)]
        {
            self.python_declarations.analysis_plan_builds += 1;
        }
        let mut plan = analysis_plan.plan;
        let mut diagnostics = Vec::new();
        if let Some(root) = package_root.as_deref() {
            let environment =
                self.package_python_environment(root, external_fingerprint, &plan, &mut provider)?;
            diagnostics.extend(environment.diagnostics.into_iter().map(|diagnostic| {
                ScopedDiagnostic {
                    file: None,
                    diagnostic,
                }
            }));
            if let Some(runtime) = environment.runtime {
                if !compiler_has_errors && !plan.declarations.is_empty() {
                    diagnostics.extend(scoped_diagnostics(
                        validate_protocol_certifications_for_plan(&plan, &runtime),
                        &analysis_plan.module_files,
                    ));
                    diagnostics.extend(self.probe_python_targets(
                        root,
                        external_fingerprint,
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
        self.verify_python_request_input(&document_path, &cache_key, external_fingerprint)?;
        let snapshot = PackageSnapshot {
            insights: declaration_insights(&plan, &analysis_plan.module_files),
            diagnostics,
        };
        #[cfg(test)]
        {
            self.python_declarations.snapshot_builds += 1;
        }
        self.python_declarations.entries.insert(
            cache_key,
            CacheEntry {
                graph_revision,
                source_revision,
                external_fingerprint,
                snapshot: snapshot.clone(),
            },
        );
        Ok(snapshot.for_document(current_file, package_owner))
    }

    fn verify_python_request_input(
        &mut self,
        document_path: &Path,
        expected_root: &Path,
        fingerprint: u64,
    ) -> LspResult<()> {
        #[cfg(test)]
        if let Some((path, contents)) = self.python_declarations.before_verification_change.take() {
            match contents {
                Some(contents) => std::fs::write(path, contents),
                None => std::fs::remove_file(path),
            }
            .map_err(|error| LspError::internal(format!("test input mutation failed: {error}")))?;
        }
        let current_fingerprint = self.observe_external_inputs_for_path(document_path);
        let mut provider = DiskSourceProvider::new();
        let current_root = package_root_for(document_path, &mut provider)
            .unwrap_or_else(|| document_path.to_path_buf());
        if current_root != expected_root || current_fingerprint != fingerprint {
            return Err(LspError::content_modified(
                "Python declaration inputs changed during the request",
            ));
        }
        self.generations.publish(self.generation, |current| {
            if current {
                Ok(())
            } else {
                Err(LspError::content_modified(
                    "Python declaration request was superseded",
                ))
            }
        })
    }

    fn package_python_environment(
        &mut self,
        package_root: &Path,
        external_fingerprint: u64,
        plan: &PythonInteropPlan,
        provider: &mut impl SourceProvider,
    ) -> LspResult<EnvironmentSnapshot> {
        let mut required_import_roots = plan.required_import_roots.clone();
        required_import_roots.sort();
        required_import_roots.dedup();
        let key = EnvironmentCacheKey {
            package_root: package_root.to_path_buf(),
            external_fingerprint,
            required_import_roots: required_import_roots.clone(),
        };
        if let Some(snapshot) = self.python_declarations.environments.get(&key) {
            return Ok(snapshot.clone());
        }
        let snapshot =
            resolve_editor_environment(package_root, &required_import_roots, provider, || {
                self.check_active_request_cancelled().is_err()
            })
            .map_err(|_| LspError::request_cancelled("Python declaration request was cancelled"))?;
        #[cfg(test)]
        if snapshot.runtime.is_some() {
            self.python_declarations.environment_probe_runs += 1;
        }
        self.python_declarations
            .environments
            .insert(key, snapshot.clone());
        Ok(snapshot)
    }

    fn probe_python_targets(
        &mut self,
        package_root: &Path,
        external_fingerprint: u64,
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
                external_fingerprint,
                target.clone(),
            );
            let inspection =
                if let Some(inspection) = self.python_declarations.target_inspections.get(&key) {
                    inspection.clone()
                } else {
                    let inspection = inspect_python_target_if_active(interpreter, &target, || {
                        self.check_active_request_cancelled().is_err()
                    })
                    .map_err(|_| {
                        LspError::request_cancelled("Python declaration request was cancelled")
                    })?;
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
                apply_python_target_inspection(
                    plan,
                    &target,
                    inspection.as_ref().map_err(String::as_str),
                ),
                module_files,
            ));
        }
        Ok(diagnostics)
    }

    fn is_python_package_diagnostic_owner(
        &self,
        uri: &str,
        package_root: &Path,
        provider: &mut impl SourceProvider,
    ) -> bool {
        let owner = self
            .document_uris()
            .into_iter()
            .filter(|candidate| self.can_analyze_document(candidate))
            .filter_map(|candidate| {
                let path = self.store().document(&candidate).ok()?.path();
                (package_root_for(path, provider).as_deref() == Some(package_root))
                    .then_some(candidate)
            })
            .min();
        owner.as_deref() == Some(uri)
    }
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

pub(crate) fn package_root_for(path: &Path, provider: &mut impl SourceProvider) -> Option<PathBuf> {
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
    use super::policy_help;
    use crate::external_inputs::package_input_fingerprint;
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

#[cfg(test)]
#[path = "python_retention_tests.rs"]
mod retention_tests;
