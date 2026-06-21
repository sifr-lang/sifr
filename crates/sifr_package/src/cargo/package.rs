#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CargoPackageRole {
    SifrSource,
    BackendRust,
    RustBackedSifr,
}

use crate::cargo::commands::CargoCommandPlan;
use crate::cargo::lock_modes::CargoLockMode;
use crate::cargo::trust::validate_backend_trust;
use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId, SifrPackageMetadata};
use crate::imports::source_map::PackageSourceMap;
use crate::projection_bridge;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageArchiveEntry {
    pub relative_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArchiveValidation {
    pub package_id: SifrPackageId,
    pub required_entries: BTreeSet<PathBuf>,
    pub archive_entries: BTreeSet<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageDryRunPlan {
    pub package_id: SifrPackageId,
    pub cargo_package: CargoCommandPlan,
    pub cargo_publish_dry_run: CargoCommandPlan,
}

pub fn validate_package_archive(
    package: &SifrPackageMetadata,
    source_map: &PackageSourceMap,
    entries: &[PackageArchiveEntry],
) -> Result<PackageArchiveValidation, Vec<PackageDiagnostic>> {
    let mut diagnostics = Vec::new();
    let mut archive_entries = BTreeSet::new();

    for entry in entries {
        if !is_safe_archive_path(&entry.relative_path) {
            diagnostics.push(PackageDiagnostic::archive_traversal(
                &package.cargo_package_id,
                &entry.relative_path,
            ));
            continue;
        }
        archive_entries.insert(entry.relative_path.clone());
    }

    let required_entries = required_archive_entries(package, source_map);
    if !required_entries.iter().any(|entry| {
        entry
            .extension()
            .is_some_and(|extension| extension == "sifr")
    }) {
        diagnostics.push(PackageDiagnostic::archive_missing_sifr_source(
            &package.cargo_package_id,
            &package.package_id,
        ));
    }

    for required in required_entries.difference(&archive_entries) {
        diagnostics.push(PackageDiagnostic::include_exclude_omits_source(
            &package.cargo_package_id,
            required,
        ));
    }

    if diagnostics.is_empty() {
        Ok(PackageArchiveValidation {
            package_id: package.package_id.clone(),
            required_entries,
            archive_entries,
        })
    } else {
        Err(diagnostics)
    }
}

pub fn package_dry_run_plan(
    graph: &SifrPackageGraph,
    source_map: &PackageSourceMap,
    package_id: &SifrPackageId,
    archive_entries: &[PackageArchiveEntry],
    lock_mode: CargoLockMode,
) -> Result<PackageDryRunPlan, Vec<PackageDiagnostic>> {
    let mut diagnostics = Vec::new();
    let Some(package) = graph.packages.get(package_id) else {
        return Err(vec![PackageDiagnostic::cargo_metadata_parse(
            "unknown package selected for package dry-run",
        )]);
    };

    if let Err(errors) = validate_backend_trust(graph) {
        diagnostics.extend(errors);
    }
    if let Err(errors) = validate_package_archive(package, source_map, archive_entries) {
        diagnostics.extend(errors);
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(PackageDryRunPlan {
        package_id: package_id.clone(),
        cargo_package: CargoCommandPlan::package(package.package_root.clone(), lock_mode),
        cargo_publish_dry_run: CargoCommandPlan::publish(
            package.package_root.clone(),
            lock_mode,
            true,
        ),
    })
}

pub fn required_archive_entries(
    package: &SifrPackageMetadata,
    source_map: &PackageSourceMap,
) -> BTreeSet<PathBuf> {
    let mut required = BTreeSet::from([PathBuf::from("sifr.toml")]);
    required.extend(
        source_map
            .modules
            .values()
            .filter(|module| module.package_id == package.package_id)
            .filter_map(|module| {
                relative_to_package_root(&package.package_root, &module.file_path)
            }),
    );
    required.extend(projection_bridge::required_archive_entries(
        &package.package_root,
        &package.manifest,
    ));
    required
}

fn relative_to_package_root(package_root: &Path, path: &Path) -> Option<PathBuf> {
    path.strip_prefix(package_root).ok().map(Path::to_path_buf)
}

fn is_safe_archive_path(path: &Path) -> bool {
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}
