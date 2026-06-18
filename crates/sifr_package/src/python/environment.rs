use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use serde::{Deserialize, Serialize};
use sifr_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPythonEnvironment {
    pub selected_by: SifrPackageId,
    pub venv_root: PathBuf,
    pub interpreter: PathBuf,
    pub pyproject: Option<PathBuf>,
    pub lock: Option<PathBuf>,
    pub declared_imports: Vec<String>,
    pub native_imports: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PythonEnvironmentProbeRequest {
    pub venv_root: PathBuf,
    pub interpreter: PathBuf,
    pub pyproject: Option<PathBuf>,
    pub lock: Option<PathBuf>,
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
    let selections = python_environment_selections(graph);
    let declared_imports = declared_python_imports(graph);
    let native_imports = native_python_imports(graph);
    let requires_python = graph.packages.values().any(|package| {
        !package.manifest.python.requires_imports.is_empty()
            || !package.manifest.python.allow_imports.is_empty()
            || !package.manifest.trust.python.is_empty()
            || !package.manifest.trust.python_native.is_empty()
    });

    if selections.is_empty() {
        if requires_python || !declared_imports.is_empty() || !native_imports.is_empty() {
            return Err(vec![PackageDiagnostic::python_environment_graph(
                DiagnosticCode::PYENV_MISSING_SELECTION,
                "Python imports are required but no root [python].venv is selected",
                None,
                "select one uv-created virtual environment in the root application [python] table; Sifr will not run uv automatically",
            )]);
        }
        return Ok(None);
    }

    let distinct_venvs = selections
        .iter()
        .map(|selection| selection.venv_root.clone())
        .collect::<BTreeSet<_>>();
    if distinct_venvs.len() > 1 {
        let venvs = distinct_venvs
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(vec![PackageDiagnostic::python_environment_graph(
            DiagnosticCode::PYENV_MULTIPLE_SELECTIONS,
            format!("multiple Python environments are selected: {venvs}"),
            None,
            "select exactly one Python virtual environment for the final process",
        )]);
    }

    let non_root_selections = selections
        .iter()
        .filter(|selection| &selection.package_id != root_package_id)
        .map(|selection| {
            PackageDiagnostic::python_environment_config(
                &selection.cargo_package_id,
                &selection.manifest_path,
                "python.venv",
                "only the root application package may select a Python virtual environment",
            )
        })
        .collect::<Vec<_>>();
    if !non_root_selections.is_empty() {
        return Err(non_root_selections);
    }

    let selected = selections.into_iter().next().ok_or_else(|| {
        vec![PackageDiagnostic::cargo_metadata_parse(
            "missing Python selection",
        )]
    })?;
    Ok(Some(ResolvedPythonEnvironment {
        selected_by: selected.package_id,
        venv_root: selected.venv_root,
        interpreter: selected.interpreter,
        pyproject: selected.pyproject,
        lock: selected.lock,
        declared_imports,
        native_imports,
    }))
}

pub fn probe_python_environment(
    request: &PythonEnvironmentProbeRequest,
) -> Result<PythonEnvironmentProbe, PackageDiagnostic> {
    validate_configured_digest_inputs(request)?;
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
            declared_imports: resolved.declared_imports.clone(),
            native_imports: resolved.native_imports.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PythonEnvironmentSelection {
    package_id: SifrPackageId,
    cargo_package_id: crate::cargo::metadata::CargoPackageId,
    manifest_path: PathBuf,
    venv_root: PathBuf,
    interpreter: PathBuf,
    pyproject: Option<PathBuf>,
    lock: Option<PathBuf>,
}

fn python_environment_selections(graph: &SifrPackageGraph) -> Vec<PythonEnvironmentSelection> {
    graph
        .packages
        .values()
        .filter_map(|package| {
            let config = &package.manifest.python;
            let venv = config.venv.as_ref()?;
            let venv_root = absolutize(&package.package_root, venv);
            Some(PythonEnvironmentSelection {
                package_id: package.package_id.clone(),
                cargo_package_id: package.cargo_package_id.clone(),
                manifest_path: package.sifr_manifest.clone(),
                interpreter: config.interpreter.as_ref().map_or_else(
                    || default_interpreter(&venv_root),
                    |path| absolutize(&package.package_root, path),
                ),
                pyproject: config
                    .pyproject
                    .as_ref()
                    .map(|path| absolutize(&package.package_root, path)),
                lock: config
                    .lock
                    .as_ref()
                    .map(|path| absolutize(&package.package_root, path)),
                venv_root,
            })
        })
        .collect()
}

fn declared_python_imports(graph: &SifrPackageGraph) -> Vec<String> {
    graph
        .packages
        .values()
        .flat_map(|package| {
            package
                .manifest
                .python
                .allow_imports
                .iter()
                .chain(package.manifest.python.requires_imports.iter())
        })
        .filter(|root| root.as_str() != "*")
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn native_python_imports(graph: &SifrPackageGraph) -> Vec<String> {
    graph
        .packages
        .values()
        .flat_map(|package| package.manifest.trust.python_native.iter())
        .filter(|root| root.as_str() != "*")
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn default_interpreter(venv_root: &Path) -> PathBuf {
    if cfg!(windows) {
        venv_root.join("Scripts").join("python.exe")
    } else {
        venv_root.join("bin").join("python")
    }
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
