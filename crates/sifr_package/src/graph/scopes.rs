use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use crate::manifest::sifr::ImportRoot;
use std::collections::BTreeMap;

#[must_use]
pub fn direct_dependency_export_scopes(
    graph: &SifrPackageGraph,
) -> BTreeMap<SifrPackageId, BTreeMap<ImportRoot, SifrPackageId>> {
    let mut scopes = BTreeMap::new();
    for (from, dependencies) in &graph.cargo_edges {
        let mut scope = BTreeMap::new();
        for dependency_id in dependencies {
            if let Some(package) = graph.packages.get(dependency_id) {
                for export in &package.manifest.exports {
                    scope.insert(export.clone(), dependency_id.clone());
                }
            }
        }
        scopes.insert(from.clone(), scope);
    }
    scopes
}
