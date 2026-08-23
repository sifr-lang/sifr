use super::test_support::{request, valid_probe};
use super::{PythonImportProbe, probe_python_environment, validate_python_environment_probe};
use sifr_diagnostics::DiagnosticCode;
use std::path::PathBuf;

#[test]
fn probe_rejects_missing_interpreter_with_pyenv_0004() {
    let request = request();
    let diagnostic = probe_python_environment(&request).expect_err("missing interpreter must fail");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_PROBE_FAILED);
}

#[test]
fn probe_rejects_non_cpython_json_with_pyenv_0005() {
    let request = request();
    let mut probe = valid_probe();
    probe.implementation_name = "PyPy".to_string();

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("non-CPython probe must fail");

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PYENV_UNSUPPORTED_INTERPRETER
    );
}

#[test]
fn probe_rejects_prefix_outside_venv_with_pyenv_0006() {
    let request = request();
    let mut probe = valid_probe();
    probe.sys_prefix = "/tmp/other".to_string();

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("prefix outside venv must fail");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_VENV_PREFIX_MISMATCH);
}

#[test]
fn probe_rejects_system_prefix_matching_base_prefix_with_pyenv_0006() {
    let request = request();
    let mut probe = valid_probe();
    probe.sys_base_prefix = probe.sys_prefix.clone();

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("system interpreter must fail venv isolation");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_VENV_PREFIX_MISMATCH);
}

#[test]
fn probe_rejects_missing_site_packages_with_pyenv_0007() {
    let request = request();
    let mut probe = valid_probe();
    probe.site_packages = Vec::new();

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("missing site-packages must fail");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_SITE_PACKAGES_MISSING);
}

#[test]
fn probe_rejects_site_packages_outside_venv_with_pyenv_0007() {
    let request = request();
    let mut probe = valid_probe();
    probe.site_packages = vec!["/usr/local/lib/python3.13/site-packages".to_string()];

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("system site-packages must fail venv isolation");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_SITE_PACKAGES_MISSING);
}

#[test]
fn probe_rejects_missing_declared_import_with_pyenv_0008() {
    let request = request();
    let mut probe = valid_probe();
    probe.imports = vec![PythonImportProbe {
        root: "numpy".to_string(),
        ok: false,
        origin: None,
        distributions: Vec::new(),
        error: Some("module spec not found".to_string()),
    }];

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("missing declared import must fail");

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PYENV_DECLARED_IMPORT_MISSING
    );
}

#[test]
fn probe_rejects_native_import_failure_with_pyenv_0009() {
    let request = request();
    let mut probe = valid_probe();
    probe.native_imports = vec![PythonImportProbe {
        root: "numpy".to_string(),
        ok: false,
        origin: None,
        distributions: Vec::new(),
        error: Some("ImportError: broken extension".to_string()),
    }];

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("native import failure must fail");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_NATIVE_IMPORT_FAILED);
}

#[test]
fn probe_rejects_free_threaded_cpython_with_pyenv_0010() {
    let request = request();
    let mut probe = valid_probe();
    probe.free_threaded = true;

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("free-threaded CPython must fail");

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PYENV_FREE_THREADED_UNSUPPORTED
    );
}

#[test]
fn probe_rejects_missing_lock_digest_with_pyenv_0011() {
    let mut request = request();
    request.lock = Some(PathBuf::from("/tmp/venv/uv.lock"));
    let probe = valid_probe();

    let diagnostic = validate_python_environment_probe(&request, probe)
        .expect_err("missing lock digest must fail");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_LOCK_OR_PROJECT_STALE);
}

#[test]
fn valid_probe_preserves_canonical_environment_json() {
    let request = request();
    let probe = validate_python_environment_probe(&request, valid_probe())
        .expect("valid probe should pass");

    assert_eq!(probe.implementation_name, "CPython");
    assert_eq!(probe.pointer_width, 64);
}
