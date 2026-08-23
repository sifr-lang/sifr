use super::test_support::{cargo_id, graph, package, package_id};
use super::{
    PythonEnvironmentResolution, resolve_python_environment, resolve_python_environment_for_check,
};
use crate::manifest::sifr::{PythonConfig, SifrManifest, TrustPolicy};
use sifr_diagnostics::DiagnosticCode;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
fn removed_python_allow_imports_key_is_rejected() {
    let source = r#"
[package]
name = "app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[python]
allow-imports = ["numpy"]
"#;
    let diagnostic = SifrManifest::parse(&cargo_id("app"), &PathBuf::from("sifr.toml"), source)
        .expect_err("removed authority must not be ignored");

    assert_eq!(diagnostic.code, DiagnosticCode::PYENV_INVALID_CONFIG);
    assert!(diagnostic.message.contains("allow-imports"));
}

#[test]
fn python_environment_uses_uv_project_defaults_without_repeated_paths() {
    let project_root = temp_root("default-discovery");
    fs::create_dir_all(&project_root).expect("create uv project");
    fs::write(project_root.join("pyproject.toml"), "[project]\n").expect("write project marker");
    fs::write(project_root.join("uv.lock"), "version = 1\n").expect("write lock marker");
    let mut app = package(
        "app",
        PythonConfig {
            requires_imports: vec!["numpy".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy {
            python: vec!["numpy".to_string()],
            ..TrustPolicy::default()
        },
    );
    app.package_root.clone_from(&project_root);
    let graph = graph(vec![app]);

    let root = package_id("app");
    let resolved = resolve_python_environment(&graph, &root)
        .expect("default discovery should resolve")
        .expect("Python is required");

    assert_eq!(resolved.venv_root, project_root.join(".venv"));
    assert_eq!(
        resolved.pyproject,
        Some(project_root.join("pyproject.toml"))
    );
    assert_eq!(resolved.lock, Some(project_root.join("uv.lock")));
    assert_eq!(resolved.interpreter, project_root.join(".venv/bin/python"));
    fs::remove_dir_all(project_root).expect("remove uv project");
}

#[test]
fn missing_uv_environment_selection_reports_pyenv_0003() {
    let mut app = package(
        "app",
        PythonConfig {
            requires_imports: vec!["numpy".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy {
            python: vec!["numpy".to_string()],
            ..TrustPolicy::default()
        },
    );
    app.package_root = temp_root("missing-discovery");
    let graph = graph(vec![app]);

    let diagnostics = resolve_python_environment(&graph, &package_id("app"))
        .expect_err("missing uv project must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, DiagnosticCode::PYENV_MISSING_SELECTION);
}

#[test]
fn read_only_library_resolution_defers_only_missing_final_application_authority() {
    let mut library = package(
        "lib",
        PythonConfig {
            requires_imports: vec!["numpy".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy::default(),
    );
    library.package_root = temp_root("deferred-library");
    let graph = graph(vec![library]);

    let resolution = resolve_python_environment_for_check(&graph, &package_id("lib"), &[], true)
        .expect("missing final-application authority should defer");
    let PythonEnvironmentResolution::DeferredToFinalApplication(deferred) = resolution else {
        panic!("library resolution should be deferred");
    };
    assert_eq!(deferred.required_imports, ["numpy"]);
    assert_eq!(deferred.missing_trusted_imports, ["numpy"]);
    assert!(deferred.environment_selection_missing);

    let diagnostics = resolve_python_environment_for_check(&graph, &package_id("lib"), &[], false)
        .expect_err("strict resolution must preserve trust failure");
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED
    );
}

#[test]
fn read_only_library_resolution_uses_discovered_environment_authority() {
    let project_root = temp_root("check-default-discovery");
    fs::create_dir_all(&project_root).expect("create uv project");
    fs::write(project_root.join("pyproject.toml"), "[project]\n").expect("write project marker");
    fs::write(project_root.join("uv.lock"), "version = 1\n").expect("write lock marker");
    let mut library = package(
        "lib",
        PythonConfig {
            requires_imports: vec!["numpy".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy {
            python: vec!["numpy".to_string()],
            ..TrustPolicy::default()
        },
    );
    library.package_root.clone_from(&project_root);
    let graph = graph(vec![library]);

    let resolution = resolve_python_environment_for_check(&graph, &package_id("lib"), &[], true)
        .expect("discovered environment should resolve");
    let PythonEnvironmentResolution::Resolved(resolved) = resolution else {
        panic!("discovered library environment should not defer");
    };
    assert_eq!(resolved.venv_root, project_root.join(".venv"));
    assert_eq!(resolved.trusted_imports, ["numpy"]);
    fs::remove_dir_all(project_root).expect("remove uv project");
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
                requires_imports: vec!["numpy".to_string(), "pandas".to_string()],
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
        resolved.required_imports,
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
            venv: Some(PathBuf::from(".python-env")),
            interpreter: Some(PathBuf::from(".python-env/custom-python")),
            pyproject: Some(PathBuf::from("python/pyproject.toml")),
            lock: Some(PathBuf::from("python/uv.lock")),
            ..PythonConfig::default()
        },
        TrustPolicy::default(),
    )]);
    let root = package_id("app");

    let resolved = resolve_python_environment(&graph, &root)
        .expect("same root environment should pass")
        .expect("environment should be selected");

    assert_eq!(resolved.selected_by, root);
    assert_eq!(resolved.venv_root, PathBuf::from("/ws/app/.python-env"));
    assert_eq!(
        resolved.interpreter,
        PathBuf::from("/ws/app/.python-env/custom-python")
    );
    assert_eq!(
        resolved.pyproject,
        Some(PathBuf::from("/ws/app/python/pyproject.toml"))
    );
    assert_eq!(resolved.lock, Some(PathBuf::from("/ws/app/python/uv.lock")));
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
