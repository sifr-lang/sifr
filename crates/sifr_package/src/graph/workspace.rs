use crate::cargo::metadata::{CargoPackageId, NormalizedCargoMetadata};
use crate::diag::PackageDiagnostic;
use crate::graph::derive::{PackageClassification, SifrPackageGraph, SifrPackageId};
use crate::manifest::sifr::ImportRoot;
use std::collections::{BTreeMap, BTreeSet};

#[must_use]
pub fn selected_workspace_members(metadata: &NormalizedCargoMetadata) -> Vec<CargoPackageId> {
    metadata
        .workspace_members
        .iter()
        .filter(|id| {
            metadata.workspace_sifr.tools_package.as_deref()
                != metadata
                    .packages
                    .get(*id)
                    .map(|package| package.name.as_str())
        })
        .cloned()
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspacePackageSelection {
    pub selected_sifr_packages: BTreeSet<SifrPackageId>,
    pub selected_backend_packages: BTreeSet<CargoPackageId>,
}

pub fn select_sifr_workspace_members(
    metadata: &NormalizedCargoMetadata,
    graph: &SifrPackageGraph,
) -> Result<WorkspacePackageSelection, Vec<PackageDiagnostic>> {
    let mut selection = WorkspacePackageSelection::default();
    let mut diagnostics = Vec::new();

    for cargo_package_id in &metadata.workspace_members {
        match graph.classifications.get(cargo_package_id) {
            Some(
                PackageClassification::SifrSource(package_id)
                | PackageClassification::RustBackedSifr(package_id),
            ) => {
                selection.selected_sifr_packages.insert(package_id.clone());
            }
            Some(PackageClassification::BackendRust) => {
                selection
                    .selected_backend_packages
                    .insert(cargo_package_id.clone());
            }
            Some(PackageClassification::HostTools) => {}
            None => {}
        }
    }

    diagnostics.extend(rust_only_sifr_dependency_diagnostics(metadata, graph));
    diagnostics.extend(duplicate_workspace_sifr_names(
        graph,
        &selection.selected_sifr_packages,
    ));
    diagnostics.extend(duplicate_workspace_import_roots(
        graph,
        &selection.selected_sifr_packages,
    ));

    if diagnostics.is_empty() {
        Ok(selection)
    } else {
        Err(diagnostics)
    }
}

pub fn explicit_package_selection(
    metadata: &NormalizedCargoMetadata,
    graph: &SifrPackageGraph,
    cargo_package_names: &[String],
) -> Result<WorkspacePackageSelection, Vec<PackageDiagnostic>> {
    let by_name = metadata
        .packages
        .values()
        .map(|package| (package.name.as_str(), &package.id))
        .collect::<BTreeMap<_, _>>();
    let mut selection = WorkspacePackageSelection::default();
    let mut diagnostics = Vec::new();

    for name in cargo_package_names {
        let Some(cargo_package_id) = by_name.get(name.as_str()) else {
            diagnostics.push(PackageDiagnostic::selector_ambiguous(name, &[]));
            continue;
        };
        match graph.classifications.get(*cargo_package_id) {
            Some(
                PackageClassification::SifrSource(package_id)
                | PackageClassification::RustBackedSifr(package_id),
            ) => {
                selection.selected_sifr_packages.insert(package_id.clone());
            }
            Some(PackageClassification::BackendRust | PackageClassification::HostTools) => {
                diagnostics.push(PackageDiagnostic::selected_rust_only(
                    cargo_package_id,
                    name,
                ));
            }
            None => diagnostics.push(PackageDiagnostic::selector_ambiguous(name, &[])),
        }
    }

    diagnostics.extend(duplicate_workspace_import_roots(
        graph,
        &selection.selected_sifr_packages,
    ));
    diagnostics.extend(duplicate_workspace_sifr_names(
        graph,
        &selection.selected_sifr_packages,
    ));

    if diagnostics.is_empty() {
        Ok(selection)
    } else {
        Err(diagnostics)
    }
}

fn rust_only_sifr_dependency_diagnostics(
    metadata: &NormalizedCargoMetadata,
    graph: &SifrPackageGraph,
) -> Vec<PackageDiagnostic> {
    metadata
        .resolve_edges
        .iter()
        .filter(|edge| {
            matches!(
                graph.classifications.get(&edge.from),
                Some(PackageClassification::BackendRust)
            ) && matches!(
                graph.classifications.get(&edge.to),
                Some(
                    PackageClassification::SifrSource(_) | PackageClassification::RustBackedSifr(_)
                )
            )
        })
        .map(|edge| PackageDiagnostic::rust_only_depends_on_sifr(&edge.from, &edge.to))
        .collect()
}

fn duplicate_workspace_import_roots(
    graph: &SifrPackageGraph,
    selected: &BTreeSet<SifrPackageId>,
) -> Vec<PackageDiagnostic> {
    let mut by_root: BTreeMap<ImportRoot, Vec<&SifrPackageId>> = BTreeMap::new();
    for package_id in selected {
        let Some(package) = graph.packages.get(package_id) else {
            continue;
        };
        by_root
            .entry(ImportRoot(package.sifr_name.0.clone()))
            .or_default()
            .push(package_id);
    }

    by_root
        .into_iter()
        .filter(|(_, packages)| packages.len() > 1)
        .map(|(root, packages)| {
            let package_names = packages
                .into_iter()
                .map(|package_id| package_id.0.clone())
                .collect::<Vec<_>>();
            PackageDiagnostic::duplicate_workspace_import_root(&root, &package_names)
        })
        .collect()
}

fn duplicate_workspace_sifr_names(
    graph: &SifrPackageGraph,
    selected: &BTreeSet<SifrPackageId>,
) -> Vec<PackageDiagnostic> {
    let mut by_name: BTreeMap<String, Vec<&SifrPackageId>> = BTreeMap::new();
    for package_id in selected {
        let Some(package) = graph.packages.get(package_id) else {
            continue;
        };
        by_name
            .entry(package.sifr_name.0.clone())
            .or_default()
            .push(package_id);
    }

    by_name
        .into_iter()
        .filter(|(_, packages)| packages.len() > 1)
        .map(|(name, packages)| {
            let members = packages
                .into_iter()
                .map(|package_id| package_id.0.clone())
                .collect::<Vec<_>>();
            PackageDiagnostic::duplicate_workspace_sifr_name(&name, &members)
        })
        .collect()
}
