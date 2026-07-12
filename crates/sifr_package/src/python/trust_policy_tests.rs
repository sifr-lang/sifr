use super::test_support::{graph, package, package_id};
use super::{
    resolve_python_environment, resolve_python_environment_with_requirements,
    PythonRequirementContribution, PythonRequirementKind,
};
use crate::manifest::sifr::{PythonConfig, TrustPolicy};
use sifr_diagnostics::DiagnosticCode;

#[test]
fn dependency_python_requirement_wildcard_is_rejected() {
    let graph = graph(vec![
        package(
            "app",
            PythonConfig::default(),
            TrustPolicy {
                python: vec!["*".to_string()],
                ..TrustPolicy::default()
            },
        ),
        package(
            "lib",
            PythonConfig {
                requires_imports: vec!["*".to_string()],
                ..PythonConfig::default()
            },
            TrustPolicy::default(),
        ),
    ]);

    let diagnostics = resolve_python_environment(&graph, &package_id("app"))
        .expect_err("dependency wildcard must fail");

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::PYTRUST_WILDCARD_REJECTED));
}

#[test]
fn root_python_trust_and_requirement_allow_wildcard_local_control() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            venv: Some(".venv".into()),
            requires_imports: vec!["*".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy {
            python: vec!["*".to_string()],
            python_native: vec!["*".to_string()],
            ..TrustPolicy::default()
        },
    )]);

    let resolved = resolve_python_environment(&graph, &package_id("app"))
        .expect("root wildcard trust should pass")
        .expect("environment should be discovered");

    assert_eq!(resolved.required_imports, ["*".to_string()]);
    assert!(resolved.declared_imports.is_empty());
    assert_eq!(resolved.trusted_imports, ["*".to_string()]);
    assert_eq!(resolved.trusted_native_imports, ["*".to_string()]);
}

#[test]
fn required_python_root_must_be_authorized_by_root() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            requires_imports: vec!["numpy".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy::default(),
    )]);

    let diagnostics = resolve_python_environment(&graph, &package_id("app"))
        .expect_err("unauthorized requirement must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED
    );
    assert!(
        diagnostics[0]
            .help
            .as_deref()
            .is_some_and(|help| help.contains("/ws/app/sifr.toml:[python].requires-imports")),
        "unauthorized-root diagnostics must expose requirement provenance"
    );
}

#[test]
fn derived_requirement_provenance_reaches_unauthorized_root_diagnostic() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            venv: Some(".venv".into()),
            ..PythonConfig::default()
        },
        TrustPolicy::default(),
    )]);
    let derived = [PythonRequirementContribution {
        root: "numpy".to_string(),
        package_id: package_id("app"),
        kind: PythonRequirementKind::Declaration,
        source: "src/main.sifr:4:1".to_string(),
    }];

    let diagnostics =
        resolve_python_environment_with_requirements(&graph, &package_id("app"), &derived)
            .expect_err("derived unauthorized requirement must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED
    );
    assert!(
        diagnostics[0]
            .help
            .as_deref()
            .is_some_and(|help| help.contains("src/main.sifr:4:1")),
        "derived provenance must reach the unauthorized-root diagnostic"
    );
}

#[test]
fn wildcard_requirement_probes_explicit_native_trust_roots() {
    let graph = graph(vec![package(
        "app",
        PythonConfig {
            venv: Some(".venv".into()),
            requires_imports: vec!["*".to_string()],
            ..PythonConfig::default()
        },
        TrustPolicy {
            python: vec!["*".to_string()],
            python_native: vec!["numpy".to_string()],
            ..TrustPolicy::default()
        },
    )]);

    let resolved = resolve_python_environment(&graph, &package_id("app"))
        .expect("root wildcard trust should pass")
        .expect("environment should be discovered");

    assert_eq!(resolved.native_imports, ["numpy".to_string()]);
}

#[test]
fn dependency_cannot_authorize_its_python_requirement() {
    let graph = graph(vec![
        package("app", PythonConfig::default(), TrustPolicy::default()),
        package(
            "lib",
            PythonConfig {
                requires_imports: vec!["numpy".to_string()],
                ..PythonConfig::default()
            },
            TrustPolicy {
                python: vec!["numpy".to_string()],
                ..TrustPolicy::default()
            },
        ),
    ]);

    let diagnostics = resolve_python_environment(&graph, &package_id("app"))
        .expect_err("dependency authorization must fail");

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::PYENV_INVALID_CONFIG));
}

#[test]
fn native_trust_requires_a_canonical_requirement() {
    let graph = graph(vec![package(
        "app",
        PythonConfig::default(),
        TrustPolicy {
            python_native: vec!["numpy".to_string()],
            ..TrustPolicy::default()
        },
    )]);

    let diagnostics = resolve_python_environment(&graph, &package_id("app"))
        .expect_err("stale native trust must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYTRUST_UNTRUSTED_NATIVE_IMPORT
    );
}
