use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use crate::python::requirements::{
    canonical_python_requirements, CanonicalPythonRequirements, PythonRequirementContribution,
};
use crate::python::selection::{
    absolutize, non_root_environment_configuration, root_environment_selection, selects_environment,
};
use crate::python::trust_policy::{
    native_probe_imports, root_python_native_trust, root_python_trust, validate_python_trust_policy,
};
use serde::{Deserialize, Serialize};
use sifr_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPythonEnvironment {
    pub selected_by: SifrPackageId,
    pub venv_root: PathBuf,
    pub interpreter: PathBuf,
    pub pyproject: Option<PathBuf>,
    pub lock: Option<PathBuf>,
    pub requirements: CanonicalPythonRequirements,
    pub required_imports: Vec<String>,
    pub declared_imports: Vec<String>,
    pub native_imports: Vec<String>,
    pub trusted_imports: Vec<String>,
    pub trusted_native_imports: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PythonEnvironmentProbeRequest {
    pub venv_root: PathBuf,
    pub interpreter: PathBuf,
    pub pyproject: Option<PathBuf>,
    pub lock: Option<PathBuf>,
    pub required_imports: Vec<String>,
    pub declared_imports: Vec<String>,
    pub native_imports: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PythonEnvironmentProbe {
    pub implementation_name: String,
    pub implementation_version: String,
    pub cpython_version_tuple: Vec<u64>,
    pub executable: String,
    pub sys_prefix: String,
    pub sys_base_prefix: String,
    pub site_packages: Vec<String>,
    pub sys_path: Vec<String>,
    pub soabi: Option<String>,
    pub extension_suffixes: Vec<String>,
    pub pointer_width: u64,
    pub platform: String,
    pub machine: String,
    pub libpython: Option<String>,
    pub free_threaded: bool,
    pub imports: Vec<PythonImportProbe>,
    pub native_imports: Vec<PythonImportProbe>,
    pub pyproject_digest: Option<String>,
    pub uv_lock_digest: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PythonImportProbe {
    pub root: String,
    pub ok: bool,
    pub origin: Option<String>,
    pub error: Option<String>,
}

pub fn resolve_python_environment(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
) -> Result<Option<ResolvedPythonEnvironment>, Vec<PackageDiagnostic>> {
    resolve_python_environment_with_requirements(graph, root_package_id, &[])
}

pub fn resolve_python_environment_with_requirements(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
    derived: &[PythonRequirementContribution],
) -> Result<Option<ResolvedPythonEnvironment>, Vec<PackageDiagnostic>> {
    let requirements = canonical_python_requirements(graph, derived);
    let required_imports = requirements.import_roots();
    let declared_imports = required_imports
        .iter()
        .filter(|root| root.as_str() != "*")
        .cloned()
        .collect::<Vec<_>>();
    let trusted_imports = root_python_trust(graph, root_package_id);
    let trusted_native_imports = root_python_native_trust(graph, root_package_id);
    let explicit_venvs = graph
        .packages
        .values()
        .filter_map(|package| {
            package
                .manifest
                .python
                .venv
                .as_ref()
                .map(|path| absolutize(&package.package_root, path))
        })
        .collect::<BTreeSet<_>>();
    if explicit_venvs.len() > 1 {
        return Err(vec![PackageDiagnostic::python_environment_graph(
            DiagnosticCode::PYENV_MULTIPLE_SELECTIONS,
            format!(
                "multiple Python environments are selected: {}",
                explicit_venvs
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            None,
            "select one root-owned Python environment for the final process",
        )]);
    }
    let non_root_selections = non_root_environment_configuration(graph, root_package_id);
    if !non_root_selections.is_empty() {
        return Err(non_root_selections);
    }
    let root_package = graph.packages.get(root_package_id).ok_or_else(|| {
        vec![PackageDiagnostic::cargo_metadata_parse(
            "root Python package is missing from the package graph",
        )]
    })?;
    let config = &root_package.manifest.python;
    let requires_python = !required_imports.is_empty()
        || !trusted_imports.is_empty()
        || !trusted_native_imports.is_empty()
        || selects_environment(config);
    if !requires_python {
        return Ok(None);
    }
    validate_python_trust_policy(
        graph,
        root_package_id,
        &requirements,
        &trusted_imports,
        &trusted_native_imports,
    )?;
    let selected = root_environment_selection(root_package_id, root_package)?;
    let native_imports = native_probe_imports(&required_imports, &trusted_native_imports);
    Ok(Some(ResolvedPythonEnvironment {
        selected_by: selected.package_id,
        venv_root: selected.venv_root,
        interpreter: selected.interpreter,
        pyproject: selected.pyproject,
        lock: selected.lock,
        requirements,
        required_imports,
        declared_imports: declared_imports.clone(),
        native_imports,
        trusted_imports,
        trusted_native_imports,
    }))
}

pub fn probe_python_environment(
    request: &PythonEnvironmentProbeRequest,
) -> Result<PythonEnvironmentProbe, PackageDiagnostic> {
    crate::cargo::python_probe::validate_python_interpreter_exists(request)?;
    validate_configured_digest_inputs(request)?;
    validate_uv_lock_consistency(request)?;
    let stdout = crate::cargo::python_probe::run_python_probe_command(
        request,
        json_array(request, &request.declared_imports)?,
        json_array(request, &request.native_imports)?,
    )?;

    let probe = serde_json::from_slice::<PythonEnvironmentProbe>(&stdout).map_err(|error| {
        probe_error(
            DiagnosticCode::PYENV_PROBE_FAILED,
            request,
            format!("selected Python interpreter returned invalid probe JSON: {error}"),
            "report this as a Sifr probe bug if the interpreter is valid CPython",
        )
    })?;
    super::probe_validation::validate_python_environment_probe(request, probe)
}

impl From<&ResolvedPythonEnvironment> for PythonEnvironmentProbeRequest {
    fn from(resolved: &ResolvedPythonEnvironment) -> Self {
        Self {
            venv_root: resolved.venv_root.clone(),
            interpreter: resolved.interpreter.clone(),
            pyproject: resolved.pyproject.clone(),
            lock: resolved.lock.clone(),
            required_imports: resolved.required_imports.clone(),
            declared_imports: resolved.declared_imports.clone(),
            native_imports: resolved.native_imports.clone(),
        }
    }
}

fn validate_configured_digest_inputs(
    request: &PythonEnvironmentProbeRequest,
) -> Result<(), PackageDiagnostic> {
    for path in request.pyproject.iter().chain(request.lock.iter()) {
        if !path.is_file() {
            return Err(stale_metadata_error(
                request,
                format!(
                    "configured Python metadata file '{}' is missing",
                    path.display()
                ),
            ));
        }
    }
    Ok(())
}

fn validate_uv_lock_consistency(
    request: &PythonEnvironmentProbeRequest,
) -> Result<(), PackageDiagnostic> {
    let (Some(pyproject), Some(lock)) = (&request.pyproject, &request.lock) else {
        return Err(stale_metadata_error(
            request,
            "both pyproject.toml and uv.lock are required for a uv-managed Python environment",
        ));
    };
    let Some(project_root) = pyproject.parent() else {
        return Err(stale_metadata_error(
            request,
            "configured pyproject.toml has no project directory",
        ));
    };
    if pyproject != &project_root.join("pyproject.toml") {
        return Err(stale_metadata_error(
            request,
            "configured Python project must be a uv-compatible pyproject.toml",
        ));
    }
    if lock != &project_root.join("uv.lock") {
        return Err(stale_metadata_error(
            request,
            "configured uv.lock must be the uv project lock beside pyproject.toml",
        ));
    }
    crate::cargo::python_probe::run_uv_lock_check(request, project_root)
}

fn stale_metadata_error(
    request: &PythonEnvironmentProbeRequest,
    reason: impl Into<String>,
) -> PackageDiagnostic {
    probe_error(
        DiagnosticCode::PYENV_LOCK_OR_PROJECT_STALE,
        request,
        format!("Python environment metadata is stale: {}", reason.into()),
        "run `uv sync` for the configured project; Sifr will not run uv automatically",
    )
}

fn probe_error(
    code: DiagnosticCode,
    request: &PythonEnvironmentProbeRequest,
    message: impl Into<String>,
    help: impl Into<String>,
) -> PackageDiagnostic {
    PackageDiagnostic::python_environment_probe(
        code,
        &request.interpreter,
        &request.venv_root,
        message,
        help,
    )
}

fn json_array(
    request: &PythonEnvironmentProbeRequest,
    values: &[String],
) -> Result<String, PackageDiagnostic> {
    serde_json::to_string(values).map_err(|error| {
        probe_error(
            DiagnosticCode::PYENV_PROBE_FAILED,
            request,
            format!("could not serialize Python probe import roots: {error}"),
            "report this as a Sifr probe bug; import roots should be serializable strings",
        )
    })
}

#[cfg(test)]
mod environment_tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn uv_lock_check_accepts_current_lock_and_rejects_stale_project() {
        let root = temp_root("uv-lock-check");
        fs::create_dir_all(&root).expect("create project");
        let pyproject = root.join("pyproject.toml");
        let lock = root.join("uv.lock");
        fs::write(&pyproject, minimal_pyproject(">=3.13")).expect("write pyproject");
        let Some(lock_status) = crate::cargo::python_probe::generate_uv_lock_for_test(&root) else {
            fs::remove_dir_all(&root).expect("remove skipped uv fixture");
            return;
        };
        assert!(lock_status.success(), "fixture lock should generate");
        let request = PythonEnvironmentProbeRequest {
            venv_root: root.join(".venv"),
            interpreter: root.join(".venv/bin/python"),
            pyproject: Some(pyproject.clone()),
            lock: Some(lock),
            required_imports: Vec::new(),
            declared_imports: Vec::new(),
            native_imports: Vec::new(),
        };
        validate_uv_lock_consistency(&request).expect("current uv lock should pass");

        fs::write(&pyproject, minimal_pyproject(">=3.12")).expect("mutate pyproject");
        let error = validate_uv_lock_consistency(&request).expect_err("stale lock must fail");
        assert_eq!(error.code, DiagnosticCode::PYENV_LOCK_OR_PROJECT_STALE);
        fs::remove_dir_all(&root).expect("remove uv fixture");
    }

    fn minimal_pyproject(requires_python: &str) -> String {
        format!(
            "[project]\nname = \"sifr-uv-fixture\"\nversion = \"0.1.0\"\nrequires-python = \"{requires_python}\"\ndependencies = []\n"
        )
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sifr-python-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
