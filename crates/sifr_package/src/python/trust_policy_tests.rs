use super::resolve_python_environment;
use super::test_support::{graph, package, package_id};
use crate::manifest::sifr::{PythonConfig, TrustPolicy};
use sifr_diagnostics::DiagnosticCode;
use std::path::PathBuf;

#[test]
fn python_trust_rejects_wildcard_roots() {
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
            "lib",
            PythonConfig {
                allow_imports: vec!["*".to_string()],
                ..PythonConfig::default()
            },
            TrustPolicy {
                python: vec!["*".to_string()],
                ..TrustPolicy::default()
            },
        ),
    ]);

    let root = package_id("app");
    let diagnostics =
        resolve_python_environment(&graph, &root).expect_err("dependency wildcards must fail");

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::PYTRUST_WILDCARD_REJECTED));
}

#[test]
fn root_python_trust_allows_wildcard_roots() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            venv: Some(PathBuf::from(".venv")),
            allow_imports: vec!["*".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy {
            python: vec!["*".to_string()],
            python_native: vec!["*".to_string()],
            ..TrustPolicy::default()
        },
    )]);

    let root = package_id("app");
    let resolved = resolve_python_environment(&graph, &root)
        .expect("root wildcard trust should pass")
        .expect("environment should be selected");

    assert!(resolved.declared_imports.is_empty());
    assert_eq!(resolved.allowed_imports, ["*".to_string()]);
    assert_eq!(resolved.trusted_imports, ["*".to_string()]);
    assert_eq!(resolved.trusted_native_imports, ["*".to_string()]);
}

#[test]
fn python_trust_requires_allowed_roots_to_be_trusted() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            venv: Some(PathBuf::from(".venv")),
            allow_imports: vec!["numpy".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy::default(),
    )]);

    let root = package_id("app");
    let diagnostics =
        resolve_python_environment(&graph, &root).expect_err("untrusted import must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYTRUST_UNTRUSTED_IMPORT
    );
}

#[test]
fn python_trust_requires_native_roots_to_be_allowed() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            venv: Some(PathBuf::from(".venv")),
            ..PythonConfig::default()
        },
        TrustPolicy {
            python_native: vec!["numpy".to_string()],
            ..TrustPolicy::default()
        },
    )]);

    let root = package_id("app");
    let diagnostics =
        resolve_python_environment(&graph, &root).expect_err("native root without allow must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYTRUST_UNTRUSTED_NATIVE_IMPORT
    );
}
