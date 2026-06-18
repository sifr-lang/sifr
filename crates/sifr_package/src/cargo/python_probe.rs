use crate::diag::PackageDiagnostic;
use crate::python::PythonEnvironmentProbeRequest;
use sifr_diagnostics::DiagnosticCode;
use std::process::Command;

pub(crate) fn run_python_probe_command(
    request: &PythonEnvironmentProbeRequest,
    declared_imports_json: String,
    native_imports_json: String,
) -> Result<Vec<u8>, PackageDiagnostic> {
    if !request.interpreter.is_file() {
        return Err(probe_error(
            DiagnosticCode::PYENV_PROBE_FAILED,
            request,
            format!(
                "selected Python interpreter '{}' does not exist",
                request.interpreter.display()
            ),
            "create or sync the configured virtual environment before running Sifr",
        ));
    }

    let output = Command::new(&request.interpreter)
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

const PROBE_SCRIPT: &str = r#"
import hashlib
import importlib
import importlib.machinery
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

def import_probe(root, do_import):
    try:
        if do_import:
            module = importlib.import_module(root)
            origin = getattr(module, "__file__", None)
            return {"root": root, "ok": True, "origin": real(origin) if origin else None, "error": None}
        spec = importlib.util.find_spec(root)
        if spec is None:
            return {"root": root, "ok": False, "origin": None, "error": "module spec not found"}
        return {"root": root, "ok": True, "origin": real(spec.origin) if spec.origin else None, "error": None}
    except BaseException as exc:
        return {"root": root, "ok": False, "origin": None, "error": f"{type(exc).__name__}: {exc}"}

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
