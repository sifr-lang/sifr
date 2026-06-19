use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use sifr_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn declared_python_imports(graph: &SifrPackageGraph) -> Vec<String> {
    allowed_python_imports(graph)
        .into_iter()
        .chain(required_python_imports(graph))
        .filter(|root| root != "*")
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn allowed_python_imports(graph: &SifrPackageGraph) -> Vec<String> {
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
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn native_python_imports(graph: &SifrPackageGraph) -> Vec<String> {
    trusted_python_native_imports(graph)
}

pub(super) fn trusted_python_imports(graph: &SifrPackageGraph) -> Vec<String> {
    graph
        .packages
        .values()
        .flat_map(|package| package.manifest.trust.python.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn trusted_python_native_policy_imports(graph: &SifrPackageGraph) -> Vec<String> {
    graph
        .packages
        .values()
        .flat_map(|package| package.manifest.trust.python_native.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn validate_python_trust_policy(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
    declared_imports: &[String],
    native_imports: &[String],
    trusted_imports: &[String],
) -> Result<(), Vec<PackageDiagnostic>> {
    let mut diagnostics = Vec::new();
    for package in graph.packages.values() {
        if &package.package_id == root_package_id {
            continue;
        }
        for (key, roots) in [
            (
                "python.allow-imports",
                &package.manifest.python.allow_imports,
            ),
            (
                "python.requires-imports",
                &package.manifest.python.requires_imports,
            ),
            ("trust.python", &package.manifest.trust.python),
            ("trust.python-native", &package.manifest.trust.python_native),
        ] {
            if roots.iter().any(|root| root == "*") {
                diagnostics.push(PackageDiagnostic::python_trust_graph(
                    DiagnosticCode::PYTRUST_WILDCARD_REJECTED,
                    format!(
                        "Python wildcard import root is rejected in {key} for package '{}'",
                        package.cargo_package_name
                    ),
                    Some(package.cargo_package_id.clone()),
                    "list each Python import root explicitly so package review can audit it",
                ));
            }
        }
    }
    let trusted = trusted_imports.iter().collect::<BTreeSet<_>>();
    let trust_all = trusted.contains(&"*".to_string());
    for root in declared_imports {
        if !trust_all && !trusted.contains(root) {
            diagnostics.push(PackageDiagnostic::python_trust_graph(
                DiagnosticCode::PYTRUST_UNTRUSTED_IMPORT,
                format!("Python import root '{root}' is allowed but not trusted"),
                None,
                "add the root to [trust].python or remove it from [python].allow-imports/[python].requires-imports",
            ));
        }
    }
    let declared = declared_imports.iter().collect::<BTreeSet<_>>();
    for root in native_imports {
        if !declared.contains(root) {
            diagnostics.push(PackageDiagnostic::python_trust_graph(
                DiagnosticCode::PYTRUST_UNTRUSTED_NATIVE_IMPORT,
                format!(
                    "native Python import root '{root}' is trusted without an allow-imports entry"
                ),
                None,
                "add the native root to [python].allow-imports so it is probed and audited",
            ));
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn required_python_imports(graph: &SifrPackageGraph) -> Vec<String> {
    graph
        .packages
        .values()
        .flat_map(|package| package.manifest.python.requires_imports.iter())
        .filter(|root| root.as_str() != "*")
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn trusted_python_native_imports(graph: &SifrPackageGraph) -> Vec<String> {
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
