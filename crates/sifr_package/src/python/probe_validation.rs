use super::{PythonEnvironmentProbe, PythonEnvironmentProbeRequest};
use crate::diag::PackageDiagnostic;
use sifr_diagnostics::DiagnosticCode;
use std::path::{Path, PathBuf};

pub fn validate_python_environment_probe(
    request: &PythonEnvironmentProbeRequest,
    probe: PythonEnvironmentProbe,
) -> Result<PythonEnvironmentProbe, PackageDiagnostic> {
    if probe.implementation_name != "CPython" {
        return Err(probe_error(
            DiagnosticCode::PYENV_UNSUPPORTED_INTERPRETER,
            request,
            format!(
                "unsupported Python interpreter '{}'",
                probe.implementation_name
            ),
            "select a CPython interpreter created by uv",
        ));
    }
    if probe.free_threaded {
        return Err(probe_error(
            DiagnosticCode::PYENV_FREE_THREADED_UNSUPPORTED,
            request,
            "free-threaded CPython is not supported for embedded Python interop",
            "select a regular GIL-enabled CPython build",
        ));
    }
    if !path_is_within(Path::new(&probe.sys_prefix), &request.venv_root)
        || same_path(
            Path::new(&probe.sys_prefix),
            Path::new(&probe.sys_base_prefix),
        )
    {
        return Err(probe_error(
            DiagnosticCode::PYENV_VENV_PREFIX_MISMATCH,
            request,
            format!(
                "Python sys.prefix '{}' is outside selected venv '{}'",
                probe.sys_prefix,
                request.venv_root.display()
            ),
            "make [python].venv and [python].interpreter point to the same uv environment",
        ));
    }
    if !probe
        .site_packages
        .iter()
        .any(|path| path_is_within(Path::new(path), &request.venv_root))
    {
        return Err(probe_error(
            DiagnosticCode::PYENV_SITE_PACKAGES_MISSING,
            request,
            "selected Python environment has no site-packages path inside the selected venv",
            "run `uv sync` for the configured project before running Sifr",
        ));
    }
    if let Some(missing) = probe.imports.iter().find(|import| !import.ok) {
        return Err(probe_error(
            DiagnosticCode::PYENV_DECLARED_IMPORT_MISSING,
            request,
            format!("declared Python import root '{}' is missing", missing.root),
            "run `uv sync` for the configured project before running Sifr",
        ));
    }
    if let Some(failed) = probe.native_imports.iter().find(|import| !import.ok) {
        return Err(probe_error(
            DiagnosticCode::PYENV_NATIVE_IMPORT_FAILED,
            request,
            format!(
                "trusted native Python import root '{}' failed to load: {}",
                failed.root,
                failed.error.as_deref().unwrap_or("unknown import failure")
            ),
            "fix the trusted native package installation before running Sifr",
        ));
    }
    if request.pyproject.is_some() && probe.pyproject_digest.is_none() {
        return Err(stale_metadata_error(
            request,
            "configured pyproject.toml was not readable during probe",
        ));
    }
    if request.lock.is_some() && probe.uv_lock_digest.is_none() {
        return Err(stale_metadata_error(
            request,
            "configured uv.lock was not readable during probe",
        ));
    }
    Ok(probe)
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

fn same_path(left: &Path, right: &Path) -> bool {
    canonical_or_normalized(left) == canonical_or_normalized(right)
}

fn path_is_within(path: &Path, parent: &Path) -> bool {
    canonical_or_normalized(path).starts_with(canonical_or_normalized(parent))
}

fn canonical_or_normalized(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| normalize_path(path))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
