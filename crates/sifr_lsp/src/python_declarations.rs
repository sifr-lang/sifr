use crate::errors::{LspError, LspResult};
use crate::session::Session;
use serde_json::Value;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_driver::{PythonInteropPlan, PythonTargetProbeStatus};
use std::collections::{BTreeMap, HashMap};
use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PythonDeclarationInsight {
    pub(crate) name: String,
    pub(crate) target: String,
    pub(crate) kind: String,
    pub(crate) status: &'static str,
    pub(crate) policy_help: &'static str,
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
    snapshot: PythonDeclarationSnapshot,
}

#[derive(Default)]
pub(crate) struct PythonDeclarationCache {
    entries: BTreeMap<PathBuf, CacheEntry>,
    #[cfg(test)]
    probe_runs: usize,
}

impl PythonDeclarationCache {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    #[cfg(test)]
    pub(crate) const fn probe_runs(&self) -> usize {
        self.probe_runs
    }
}

impl PythonDeclarationSnapshot {
    pub(crate) fn insight(&self, label: &str) -> Option<&PythonDeclarationInsight> {
        self.insights.iter().find(|insight| {
            insight.name == label
                || insight
                    .name
                    .rsplit('.')
                    .next()
                    .is_some_and(|name| name == label)
        })
    }
}

impl Session {
    pub(crate) fn python_declaration_snapshot(
        &mut self,
        uri: &str,
    ) -> LspResult<PythonDeclarationSnapshot> {
        self.check_active_request_cancelled()?;
        let document_path = self.store().document(uri)?.path().to_path_buf();
        let package_root = package_root_for(&document_path);
        let external_fingerprint = package_root.as_deref().map_or(0, package_input_fingerprint);
        let (graph_revision, source_revision, mut plan, compiler_has_errors) = self
            .with_document_analysis(uri, |snapshot, host, _file, _source| {
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
                Ok((
                    snapshot.revision().graph.as_u64(),
                    snapshot.revision().source.as_u64(),
                    plan,
                    compiler_has_errors,
                ))
            })?;
        let cache_key = package_root
            .clone()
            .unwrap_or_else(|| document_path.clone());
        if let Some(entry) = self.python_declarations.entries.get(&cache_key) {
            if entry.graph_revision == graph_revision
                && entry.source_revision == source_revision
                && entry.external_fingerprint == external_fingerprint
            {
                return Ok(entry.snapshot.clone());
            }
        }

        let mut diagnostics = package_root
            .as_deref()
            .map_or_else(Vec::new, validate_authoring_artifacts);
        if !compiler_has_errors && !plan.declarations.is_empty() {
            if let Some(interpreter) = package_root
                .as_deref()
                .and_then(python_environment_selection)
                .map(|selection| selection.interpreter)
            {
                diagnostics.extend(sifr_driver::probe_python_interop_plan(
                    &mut plan,
                    &interpreter,
                ));
                #[cfg(test)]
                {
                    self.python_declarations.probe_runs += 1;
                }
            } else {
                mark_embedded_bridge_targets(&mut plan);
                if plan
                    .target_probes
                    .iter()
                    .any(|probe| !probe.target_path.starts_with("__sifr_bridge__."))
                {
                    diagnostics.push(diagnostic_with_code(
                        DiagnosticCode::PYENV_MISSING_SELECTION,
                        "Python declarations require a selected package Python environment"
                            .to_string(),
                        "configure [python] or add a uv pyproject.toml, uv.lock, and .venv"
                            .to_string(),
                    ));
                }
            }
        }
        self.check_active_request_cancelled()?;
        let snapshot = PythonDeclarationSnapshot {
            insights: declaration_insights(&plan),
            diagnostics,
        };
        self.python_declarations.entries.insert(
            cache_key,
            CacheEntry {
                graph_revision,
                source_revision,
                external_fingerprint,
                snapshot: snapshot.clone(),
            },
        );
        Ok(snapshot)
    }
}

fn declaration_insights(plan: &PythonInteropPlan) -> Vec<PythonDeclarationInsight> {
    let statuses = plan
        .target_probes
        .iter()
        .map(|probe| (probe.target_path.as_str(), status_name(probe.status)))
        .collect::<HashMap<_, _>>();
    plan.declarations
        .iter()
        .filter_map(|declaration| {
            let target = declaration.declaration.target.as_ref()?.dotted();
            let kind = format!("{:?}", declaration.declaration.kind);
            Some(PythonDeclarationInsight {
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

fn package_root_for(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent()?.to_path_buf();
    loop {
        if current.join("sifr.toml").is_file() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn python_environment_selection(root: &Path) -> Option<sifr_package::PythonEnvironmentSelection> {
    let session = sifr_package::PackageSession::discover(sifr_package::PackageSessionOptions {
        current_dir: root.to_path_buf(),
        lock_mode: sifr_package::CargoLockMode::Frozen,
    })
    .ok()?;
    let manifest = session.manifest?;
    sifr_package::select_root_python_environment(root, &manifest.python)
}

fn package_input_fingerprint(root: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut paths = vec![
        root.join("sifr.toml"),
        root.join(sifr_package::PYTHON_BINDINGS_FILE),
        root.join(sifr_package::PYTHON_CERTIFICATIONS_FILE),
    ];
    let interpreter = python_environment_selection(root).map(|selection| {
        paths.extend(selection.pyproject);
        paths.extend(selection.lock);
        selection.interpreter
    });
    for path in paths {
        path.hash(&mut hasher);
        match std::fs::read(&path) {
            Ok(bytes) => bytes.hash(&mut hasher),
            Err(error) => error.kind().hash(&mut hasher),
        }
    }
    if let Some(interpreter) = interpreter {
        interpreter.hash(&mut hasher);
        match std::fs::metadata(&interpreter) {
            Ok(metadata) => {
                metadata.len().hash(&mut hasher);
                metadata.modified().ok().hash(&mut hasher);
            }
            Err(error) => error.kind().hash(&mut hasher),
        }
    }
    hasher.finish()
}

fn validate_authoring_artifacts(root: &Path) -> Vec<RenderedDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_binding_artifact(root, &mut diagnostics);
    validate_certification_artifact(root, &mut diagnostics);
    diagnostics
}

fn validate_binding_artifact(root: &Path, diagnostics: &mut Vec<RenderedDiagnostic>) {
    let path = root.join(sifr_package::PYTHON_BINDINGS_FILE);
    if !path.is_file() {
        return;
    }
    let result = std::fs::read(&path)
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<sifr_package::PythonBindingArtifact>(&bytes)
                .map_err(|error| error.to_string())
        })
        .and_then(|artifact| {
            sifr_package::load_python_bindings(root, &artifact.environment_digest).map(|_| ())
        });
    if let Err(reason) = result {
        diagnostics.push(diagnostic(format!(
            "invalid or drifted Python binding artifact: {reason}"
        )));
    }
}

fn validate_certification_artifact(root: &Path, diagnostics: &mut Vec<RenderedDiagnostic>) {
    let path = root.join(sifr_package::PYTHON_CERTIFICATIONS_FILE);
    if !path.is_file() {
        return;
    }
    let result = std::fs::read(&path)
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<sifr_package::PythonCertificationArtifact>(&bytes)
                .map_err(|error| error.to_string())
        })
        .and_then(|artifact| {
            sifr_package::load_python_certifications(root, &artifact.environment_digest).map(|_| ())
        });
    if let Err(reason) = result {
        diagnostics.push(diagnostic(format!(
            "invalid or drifted Python certification artifact: {reason}"
        )));
    }
}

fn diagnostic(message: String) -> RenderedDiagnostic {
    diagnostic_with_code(
        DiagnosticCode::PYENV_LOCK_OR_PROJECT_STALE,
        message,
        "rerun `sifr python bind --check` or `sifr python certify --check`".to_string(),
    )
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
    let Some(insight) = snapshot.insight(label) else {
        return;
    };
    let detail = format!(
        "Python {} · {} · {}",
        insight.kind, insight.status, insight.target
    );
    item["detail"] = Value::String(detail);
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

pub(crate) fn enrich_hover(hover: &mut Value, snapshot: &PythonDeclarationSnapshot) {
    let Some(contents) = hover.pointer_mut("/contents/value") else {
        return;
    };
    let Some(rendered) = contents.as_str() else {
        return;
    };
    let Some(insight) = snapshot.insights.iter().find(|insight| {
        rendered.contains(&insight.name)
            || insight
                .name
                .rsplit('.')
                .next()
                .is_some_and(|name| rendered.contains(name))
    }) else {
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

        let initial = package_input_fingerprint(root.path());
        std::fs::write(root.path().join("uv.lock"), "unselected-lock-change")
            .expect("unselected lock");
        assert_eq!(initial, package_input_fingerprint(root.path()));

        std::fs::write(metadata.join("uv.lock"), "lock-b").expect("selected lock drift");
        assert_ne!(initial, package_input_fingerprint(root.path()));
    }
}
