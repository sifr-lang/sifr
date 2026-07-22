use crate::diag::PackageDiagnostic;
use crate::python::PythonEnvironmentProbeRequest;
use sifr_diagnostics::DiagnosticCode;
use std::path::Path;
use std::process::Command;

pub(crate) fn validate_python_interpreter_exists(
    request: &PythonEnvironmentProbeRequest,
) -> Result<(), PackageDiagnostic> {
    if request.interpreter.is_file() {
        Ok(())
    } else {
        Err(probe_error(
            DiagnosticCode::PYENV_PROBE_FAILED,
            request,
            format!(
                "selected Python interpreter '{}' does not exist",
                request.interpreter.display()
            ),
            "create or sync the uv environment before running Sifr",
        ))
    }
}

pub(crate) fn run_uv_lock_check(
    request: &PythonEnvironmentProbeRequest,
    project_root: &Path,
) -> Result<(), PackageDiagnostic> {
    let output = Command::new("uv")
        .args(["lock", "--check", "--offline", "--project"])
        .arg(project_root)
        .output()
        .map_err(|error| uv_lock_check_spawn_error(request, &error))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(stale_metadata_error(
            request,
            format!(
                "uv reports the project and lock are inconsistent: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ))
    }
}

#[cfg(test)]
pub(crate) fn generate_uv_lock_for_test(project_root: &Path) -> Option<std::process::ExitStatus> {
    Command::new("uv")
        .args(["lock", "--offline", "--project"])
        .arg(project_root)
        .status()
        .ok()
}

pub(crate) fn run_python_probe_command(
    request: &PythonEnvironmentProbeRequest,
    declared_imports_json: String,
    native_imports_json: String,
) -> Result<Vec<u8>, PackageDiagnostic> {
    validate_python_interpreter_exists(request)?;

    let output = Command::new(&request.interpreter)
        .arg("-B")
        .arg("-I")
        .arg("-c")
        .arg(PROBE_SCRIPT)
        .arg(declared_imports_json)
        .arg(native_imports_json)
        .arg(optional_path_arg(request.pyproject.as_deref()))
        .arg(optional_path_arg(request.lock.as_deref()))
        .output()
        .map_err(|error| {
            probe_error(
                DiagnosticCode::PYENV_PROBE_FAILED,
                request,
                format!("could not execute selected Python interpreter: {error}"),
                "verify the configured [python].interpreter path",
            )
        })?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(probe_error(
            DiagnosticCode::PYENV_PROBE_FAILED,
            request,
            format!(
                "selected Python interpreter exited with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            "inspect the selected interpreter and run `uv sync` if the environment is stale",
        ))
    }
}

fn optional_path_arg(path: Option<&std::path::Path>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_default()
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

fn uv_lock_check_spawn_error(
    request: &PythonEnvironmentProbeRequest,
    error: &std::io::Error,
) -> PackageDiagnostic {
    if error.kind() == std::io::ErrorKind::NotFound {
        probe_error(
            DiagnosticCode::PYENV_PROBE_FAILED,
            request,
            "could not execute uv because it is not installed or not available on PATH",
            "install uv and ensure the `uv` executable is available on PATH",
        )
    } else {
        stale_metadata_error(
            request,
            format!("could not execute `uv lock --check --offline`: {error}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_uv_executable_is_a_probe_failure_not_stale_metadata() {
        let request = PythonEnvironmentProbeRequest {
            venv_root: "/tmp/venv".into(),
            interpreter: "/tmp/venv/bin/python".into(),
            pyproject: None,
            lock: None,
            required_imports: Vec::new(),
            declared_imports: Vec::new(),
            native_imports: Vec::new(),
        };
        let diagnostic = uv_lock_check_spawn_error(
            &request,
            &std::io::Error::new(std::io::ErrorKind::NotFound, "uv missing"),
        );

        assert_eq!(diagnostic.code, DiagnosticCode::PYENV_PROBE_FAILED);
        assert!(diagnostic.message.contains("not installed"));
    }
}

const PROBE_SCRIPT: &str = r#"
import hashlib
import importlib
import importlib.machinery
import importlib.metadata
import importlib.util
import json
import platform
import site
import struct
import sys
import sysconfig
from pathlib import Path

declared_roots = json.loads(sys.argv[1])
native_roots = json.loads(sys.argv[2])
pyproject = sys.argv[3] or None
lock = sys.argv[4] or None

def digest(path):
    if not path:
        return None
    try:
        return hashlib.sha256(Path(path).read_bytes()).hexdigest()
    except OSError:
        return None

def real(path):
    try:
        return str(Path(path).resolve())
    except OSError:
        return str(path)

def distributions(root):
    names = importlib.metadata.packages_distributions().get(root, [])
    return [
        {"name": name, "version": importlib.metadata.version(name)}
        for name in sorted(set(names), key=str.casefold)
    ]

def import_probe(root, do_import):
    try:
        if do_import:
            module = importlib.import_module(root)
            origin = getattr(module, "__file__", None)
            return {
                "root": root,
                "ok": True,
                "origin": real(origin) if origin else None,
                "distributions": distributions(root),
                "error": None,
            }
        spec = importlib.util.find_spec(root)
        if spec is None:
            return {"root": root, "ok": False, "origin": None, "distributions": [], "error": "module spec not found"}
        return {
            "root": root,
            "ok": True,
            "origin": real(spec.origin) if spec.origin else None,
            "distributions": distributions(root),
            "error": None,
        }
    except BaseException as exc:
        return {"root": root, "ok": False, "origin": None, "distributions": [], "error": f"{type(exc).__name__}: {exc}"}

libdir = sysconfig.get_config_var("LIBDIR")
ldlibrary = sysconfig.get_config_var("LDLIBRARY")
libpython = str(Path(libdir) / ldlibrary) if libdir and ldlibrary else None

payload = {
    "implementation_name": platform.python_implementation(),
    "implementation_version": platform.python_version(),
    "cpython_version_tuple": list(sys.version_info[:3]),
    "executable": real(sys.executable),
    "sys_prefix": real(sys.prefix),
    "sys_base_prefix": real(sys.base_prefix),
    "site_packages": [real(path) for path in site.getsitepackages()],
    "sys_path": [real(path) for path in sys.path],
    "soabi": sysconfig.get_config_var("SOABI"),
    "extension_suffixes": list(importlib.machinery.EXTENSION_SUFFIXES),
    "pointer_width": struct.calcsize("P") * 8,
    "platform": platform.platform(),
    "machine": platform.machine(),
    "libpython": real(libpython) if libpython else None,
    "free_threaded": bool(sysconfig.get_config_var("Py_GIL_DISABLED")),
    "imports": [import_probe(root, True) for root in declared_roots],
    "native_imports": [import_probe(root, True) for root in native_roots],
    "pyproject_digest": digest(pyproject),
    "uv_lock_digest": digest(lock),
}
print(json.dumps(payload, sort_keys=True, separators=(",", ":")))
"#;
