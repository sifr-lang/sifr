use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use crate::python::requirements::CanonicalPythonRequirements;
use sifr_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn root_python_trust(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
) -> Vec<String> {
    graph
        .packages
        .get(root_package_id)
        .map(|package| sorted_unique(&package.manifest.trust.python))
        .unwrap_or_default()
}

pub(super) fn root_python_native_trust(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
) -> Vec<String> {
    graph
        .packages
        .get(root_package_id)
        .map(|package| sorted_unique(&package.manifest.trust.python_native))
        .unwrap_or_default()
}

pub(super) fn validate_python_trust_policy(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
    requirements: &CanonicalPythonRequirements,
    trusted_imports: &[String],
    trusted_native_imports: &[String],
) -> Result<(), Vec<PackageDiagnostic>> {
    let required_imports = requirements.import_roots();
    let mut diagnostics = Vec::new();
    for package in graph.packages.values() {
        if &package.package_id == root_package_id {
            continue;
        }
        if package
            .manifest
            .python
            .requires_imports
            .iter()
            .any(|root| root == "*")
        {
            diagnostics.push(PackageDiagnostic::python_trust_graph(
                DiagnosticCode::PYTRUST_WILDCARD_REJECTED,
                format!(
                    "Python wildcard requirement is rejected for dependency package '{}'",
                    package.cargo_package_name
                ),
                Some(package.cargo_package_id.clone()),
                "list every Python import root explicitly so package review can audit it",
            ));
        }
        for (key, roots) in [
            ("trust.python", &package.manifest.trust.python),
            ("trust.python-native", &package.manifest.trust.python_native),
        ] {
            if !roots.is_empty() {
                diagnostics.push(PackageDiagnostic::python_environment_config(
                    &package.cargo_package_id,
                    &package.sifr_manifest,
                    key,
                    "dependency packages may publish Python requirements but cannot authorize Python execution or native extensions",
                ));
            }
        }
    }

    let trusted = trusted_imports.iter().collect::<BTreeSet<_>>();
    let trust_all = trusted.contains(&"*".to_string());
    for requirement in &requirements.roots {
        let root = &requirement.root;
        if !trust_all && !trusted.contains(root) {
            let sources = requirement
                .contributions
                .iter()
                .map(|contribution| contribution.source.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            diagnostics.push(PackageDiagnostic::python_trust_graph(
                DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED,
                format!("required Python import root '{root}' is not authorized by the root application"),
                None,
                format!(
                    "required by {sources}; add the root to the root [trust].python table or remove every requirement source"
                ),
            ));
        }
    }

    let required = required_imports.iter().collect::<BTreeSet<_>>();
    let require_all = required.contains(&"*".to_string());
    for root in trusted_native_imports {
        if root != "*" && !require_all && !required.contains(root) {
            diagnostics.push(PackageDiagnostic::python_trust_graph(
                DiagnosticCode::PYTRUST_UNTRUSTED_NATIVE_IMPORT,
                format!("native Python import root '{root}' is trusted but not required"),
                None,
                "remove the stale root from [trust].python-native or add a reviewed Python requirement",
            ));
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub(super) fn native_probe_imports(
    required_imports: &[String],
    trusted_native_imports: &[String],
) -> Vec<String> {
    let native = trusted_native_imports.iter().collect::<BTreeSet<_>>();
    let trust_all = native.contains(&"*".to_string());
    if required_imports.iter().any(|root| root == "*") {
        return native
            .into_iter()
            .filter(|root| root.as_str() != "*")
            .cloned()
            .collect();
    }
    required_imports
        .iter()
        .filter(|root| trust_all || native.contains(root))
        .cloned()
        .collect()
}

fn sorted_unique(values: &[String]) -> Vec<String> {
    values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
