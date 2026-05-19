use crate::diag::PackageDiagnostic;
use crate::graph::derive::{BackendCrateMetadata, SifrPackageGraph, SifrPackageId};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackendTrustSummary {
    pub trusted_native_dependencies: Vec<String>,
    pub untrusted_native_dependencies: Vec<String>,
    pub stale_trust_entries: Vec<String>,
}

pub fn validate_backend_trust(
    graph: &SifrPackageGraph,
) -> Result<Vec<(SifrPackageId, BackendTrustSummary)>, Vec<PackageDiagnostic>> {
    let mut summaries = Vec::new();
    let mut diagnostics = Vec::new();

    for package in graph.packages.values() {
        let backend_crates = graph
            .backend_crates
            .get(&package.package_id)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let trusted_names = package
            .manifest
            .trust
            .native
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let direct_backend_names = backend_crates
            .iter()
            .map(|backend| backend.cargo_package_name.clone())
            .collect::<BTreeSet<_>>();

        let untrusted_native_dependencies = untrusted_backend_names(backend_crates, &trusted_names);
        for backend_name in &untrusted_native_dependencies {
            diagnostics.push(PackageDiagnostic::backend_trust_violation(
                &package.cargo_package_id,
                &package.package_id,
                backend_name,
            ));
        }

        let stale_trust_entries = trusted_names
            .difference(&direct_backend_names)
            .cloned()
            .collect::<Vec<_>>();
        for backend_name in &stale_trust_entries {
            diagnostics.push(PackageDiagnostic::trust_non_direct_dependency(
                &package.cargo_package_id,
                &package.package_id,
                backend_name,
            ));
        }

        let trusted_native_dependencies = direct_backend_names
            .intersection(&trusted_names)
            .cloned()
            .collect::<Vec<_>>();
        summaries.push((
            package.package_id.clone(),
            BackendTrustSummary {
                trusted_native_dependencies,
                untrusted_native_dependencies,
                stale_trust_entries,
            },
        ));
    }

    if diagnostics.is_empty() {
        Ok(summaries)
    } else {
        Err(diagnostics)
    }
}

fn untrusted_backend_names(
    backend_crates: &[BackendCrateMetadata],
    trusted_names: &BTreeSet<String>,
) -> Vec<String> {
    backend_crates
        .iter()
        .filter(|backend| !trusted_names.contains(&backend.cargo_package_name))
        .map(|backend| backend.cargo_package_name.clone())
        .collect()
}
