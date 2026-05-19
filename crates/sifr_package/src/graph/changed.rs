use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedPathSelection {
    pub paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedPackageSelection {
    pub package_ids: BTreeSet<SifrPackageId>,
    pub invalidates_all: bool,
}

pub fn select_changed_packages(
    graph: &SifrPackageGraph,
    changed_paths: &[PathBuf],
) -> Result<ChangedPackageSelection, Vec<PackageDiagnostic>> {
    let mut package_ids = BTreeSet::new();
    let mut invalidates_all = false;
    let mut diagnostics = Vec::new();

    for changed_path in changed_paths {
        if is_global_invalidation_path(changed_path) {
            invalidates_all = true;
            package_ids.extend(graph.packages.keys().cloned());
            continue;
        }

        let owners = graph
            .packages
            .values()
            .filter(|package| changed_path.starts_with(&package.package_root))
            .map(|package| package.package_id.clone())
            .collect::<Vec<_>>();

        match owners.as_slice() {
            [package_id] => {
                package_ids.insert(package_id.clone());
            }
            [] => diagnostics.push(PackageDiagnostic::changed_file_mapping_failed(changed_path)),
            _ => diagnostics.push(PackageDiagnostic::changed_file_mapping_failed(changed_path)),
        }
    }

    if diagnostics.is_empty() {
        Ok(ChangedPackageSelection {
            package_ids,
            invalidates_all,
        })
    } else {
        Err(diagnostics)
    }
}

fn is_global_invalidation_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, "Cargo.toml" | "Cargo.lock" | "sifr.toml"))
}
