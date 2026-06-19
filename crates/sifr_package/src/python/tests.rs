use super::test_support::{cargo_id, graph, package, package_id, request, valid_probe};
use super::{
    probe_python_environment, resolve_python_environment, validate_python_environment_probe,
    PythonImportProbe,
};
use crate::manifest::sifr::{PythonConfig, SifrManifest, TrustPolicy};
use sifr_diagnostics::DiagnosticCode;
use std::path::PathBuf;

#[test]
fn invalid_python_manifest_config_reports_pyenv_0001() {
    let source = r#"
[package]
name = "app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[python]
venv = "/absolute/.venv"
"#;
    let diagnostic = SifrManifest::parse(&cargo_id("app"), &PathBuf::from("sifr.toml"), source)
        .expect_err("absolute venv path must fail");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_INVALID_CONFIG);
}

#[test]
fn mistyped_python_table_reports_pyenv_0001() {
    let source = r#"
python = "venv"

[package]
name = "app"
edition = "2026"
sifr-version = ">=0.3,<0.4"
"#;
    let diagnostic = SifrManifest::parse(&cargo_id("app"), &PathBuf::from("sifr.toml"), source)
        .expect_err("mistyped python table must fail");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_INVALID_CONFIG);
}

#[test]
fn missing_python_environment_selection_reports_pyenv_0003() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            requires_imports: vec!["numpy".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy::default(),
    )]);

    let root = package_id("app");
    let diagnostics =
        resolve_python_environment(&graph, &root).expect_err("missing venv must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, DiagnosticCode::PYENV_MISSING_SELECTION);
}

#[test]
fn multiple_python_environment_selections_report_pyenv_0002() {
    let graph = graph(vec![
        package(
            "app",
            PythonConfig {
                venv: Some(PathBuf::from(".venv")),
                ..PythonConfig::default()
            },
            TrustPolicy::default(),
        ),
        package(
            "worker",
            PythonConfig {
                venv: Some(PathBuf::from(".other-venv")),
                ..PythonConfig::default()
            },
            TrustPolicy::default(),
        ),
    ]);

    let root = package_id("app");
    let diagnostics = resolve_python_environment(&graph, &root).expect_err("two venvs must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYENV_MULTIPLE_SELECTIONS
    );
}

#[test]
fn python_environment_resolution_deduplicates_declared_roots() {
    let graph = graph(vec![
        package(
            "app",
            PythonConfig {
                venv: Some(PathBuf::from(".venv")),
                allow_imports: vec!["numpy".to_string(), "pandas".to_string()],
                ..PythonConfig::default()
            },
            TrustPolicy {
                python: vec![
                    "numpy".to_string(),
                    "pandas".to_string(),
                    "pyarrow".to_string(),
                ],
                python_native: vec!["numpy".to_string()],
                ..TrustPolicy::default()
            },
        ),
        package(
            "lib",
            PythonConfig {
                requires_imports: vec!["pandas".to_string(), "pyarrow".to_string()],
                ..PythonConfig::default()
            },
            TrustPolicy::default(),
        ),
    ]);

    let root = package_id("app");
    let resolved = resolve_python_environment(&graph, &root)
        .expect("resolution should pass")
        .expect("environment should be selected");

    assert_eq!(
        resolved.declared_imports,
        ["numpy", "pandas", "pyarrow"].map(str::to_string)
    );
    assert_eq!(
        resolved.allowed_imports,
        ["numpy", "pandas", "pyarrow"].map(str::to_string)
    );
    assert_eq!(
        resolved.trusted_imports,
        ["numpy", "pandas", "pyarrow"].map(str::to_string)
    );
    assert_eq!(resolved.native_imports, ["numpy".to_string()]);
    assert!(resolved.interpreter.ends_with(".venv/bin/python"));
}

#[test]
fn same_root_python_environment_selection_resolves_once() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            venv: Some(PathBuf::from(".venv")),
            interpreter: Some(PathBuf::from(".venv/bin/python")),
            ..PythonConfig::default()
        },
        TrustPolicy::default(),
    )]);
    let root = package_id("app");

    let resolved = resolve_python_environment(&graph, &root)
        .expect("same root environment should pass")
        .expect("environment should be selected");

    assert_eq!(resolved.selected_by, root);
    assert_eq!(resolved.venv_root, PathBuf::from("/ws/app/.venv"));
}

#[test]
fn dependency_python_environment_selection_reports_pyenv_0001() {
    let graph = graph(vec![
        package("app", PythonConfig::default(), TrustPolicy::default()),
        package(
            "lib",
            PythonConfig {
                venv: Some(PathBuf::from(".venv")),
                ..PythonConfig::default()
            },
            TrustPolicy::default(),
        ),
    ]);
    let root = package_id("app");

    let diagnostics =
        resolve_python_environment(&graph, &root).expect_err("dependency venv must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, DiagnosticCode::PYENV_INVALID_CONFIG);
}

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
