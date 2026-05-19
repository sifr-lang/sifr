use crate::cargo::metadata::{
    CargoDependency, CargoMetadata, CargoPackage, CargoPackageId, NormalizedCargoMetadata,
};
use crate::diag::PackageDiagnostic;
use crate::manifest::metadata::CargoSifrAliasMetadata;
use crate::manifest::sifr::{ImportRoot, SifrManifest, SifrPackageName};
use crate::manifest::validate::{validate_exports_match_sources, validate_source_roots_exist};
use crate::source::layout::{validate_pure_marker_file, MarkerValidation};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SifrPackageId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrPackageGraph {
    pub packages: BTreeMap<SifrPackageId, SifrPackageMetadata>,
    pub cargo_edges: BTreeMap<SifrPackageId, BTreeSet<SifrPackageId>>,
    pub backend_crates: BTreeMap<SifrPackageId, Vec<BackendCrateMetadata>>,
    pub classifications: BTreeMap<CargoPackageId, PackageClassification>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrPackageMetadata {
    pub package_id: SifrPackageId,
    pub cargo_package_id: CargoPackageId,
    pub cargo_package_name: String,
    pub cargo_version: String,
    pub cargo_source: Option<String>,
    pub package_root: PathBuf,
    pub sifr_manifest: PathBuf,
    pub sifr_name: SifrPackageName,
    pub manifest: SifrManifest,
    pub aliases: BTreeMap<String, CargoSifrAliasMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendCrateMetadata {
    pub cargo_package_id: CargoPackageId,
    pub cargo_package_name: String,
    pub cargo_version: String,
    pub cargo_source: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageClassification {
    SifrSource(SifrPackageId),
    RustBackedSifr(SifrPackageId),
    BackendRust,
}

pub fn derive_package_graph(
    metadata: CargoMetadata,
) -> Result<SifrPackageGraph, Vec<PackageDiagnostic>> {
    derive_package_graph_from_normalized(&metadata.normalize())
}

pub fn derive_package_graph_from_normalized(
    metadata: &NormalizedCargoMetadata,
) -> Result<SifrPackageGraph, Vec<PackageDiagnostic>> {
    let mut diagnostics = Vec::new();
    let mut packages = BTreeMap::new();
    let mut classifications = BTreeMap::new();

    for package in metadata.packages.values() {
        let Some(sifr_metadata) = &package.sifr_metadata else {
            classifications.insert(package.id.clone(), PackageClassification::BackendRust);
            continue;
        };

        let package_root = package_root(package);
        let manifest_path = package_root.join(&sifr_metadata.manifest);
        match load_sifr_package(package, &package_root, &manifest_path) {
            Ok(package_metadata) => {
                let package_id = package_metadata.package_id.clone();
                let classification = if package_metadata.manifest.declares_rust_backend() {
                    PackageClassification::RustBackedSifr(package_id.clone())
                } else {
                    PackageClassification::SifrSource(package_id.clone())
                };
                classifications.insert(package.id.clone(), classification);
                packages.insert(package_id, package_metadata);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let cargo_edges = derive_sifr_edges(&metadata.packages, &classifications);
    let backend_crates = derive_backend_crates(&metadata.packages, &packages, &classifications);

    Ok(SifrPackageGraph {
        packages,
        cargo_edges,
        backend_crates,
        classifications,
    })
}

fn load_sifr_package(
    package: &CargoPackage,
    package_root: &Path,
    manifest_path: &Path,
) -> Result<SifrPackageMetadata, PackageDiagnostic> {
    let manifest = SifrManifest::load(&package.id, manifest_path)?;
    validate_source_roots_exist(&package.id, manifest_path, package_root, &manifest)?;
    validate_exports_match_sources(&package.id, manifest_path, package_root, &manifest)?;
    if !manifest.declares_rust_backend() {
        validate_pure_markers(package, &package.id)?;
    }

    let package_id = SifrPackageId(format!(
        "{}@{}#{}",
        package.name,
        package.version,
        package.source.as_deref().unwrap_or("path")
    ));

    Ok(SifrPackageMetadata {
        package_id,
        cargo_package_id: package.id.clone(),
        cargo_package_name: package.name.clone(),
        cargo_version: package.version.clone(),
        cargo_source: package.source.clone(),
        package_root: package_root.to_path_buf(),
        sifr_manifest: manifest_path.to_path_buf(),
        sifr_name: manifest.package_name.clone(),
        manifest,
        aliases: package
            .sifr_metadata
            .as_ref()
            .map(|metadata| metadata.aliases.clone())
            .unwrap_or_default(),
    })
}

fn package_root(package: &CargoPackage) -> PathBuf {
    package
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn validate_pure_markers(
    package: &CargoPackage,
    cargo_package_id: &CargoPackageId,
) -> Result<(), PackageDiagnostic> {
    for target in &package.targets {
        if !target.kind.contains("lib") {
            continue;
        }
        match validate_pure_marker_file(&target.src_path) {
            Ok(MarkerValidation::PureMarker) => {}
            Ok(MarkerValidation::NonTrivialRust { reason }) => {
                return Err(PackageDiagnostic::non_trivial_pure_marker(
                    cargo_package_id,
                    target.src_path.clone(),
                    reason,
                ));
            }
            Err(error) => {
                return Err(PackageDiagnostic::non_trivial_pure_marker(
                    cargo_package_id,
                    target.src_path.clone(),
                    format!("marker target could not be read: {error}"),
                ));
            }
        }
    }
    Ok(())
}

fn derive_sifr_edges(
    cargo_packages: &BTreeMap<CargoPackageId, CargoPackage>,
    classifications: &BTreeMap<CargoPackageId, PackageClassification>,
) -> BTreeMap<SifrPackageId, BTreeSet<SifrPackageId>> {
    let by_name = package_ids_by_name(cargo_packages);
    let mut edges = BTreeMap::new();

    for package in cargo_packages.values() {
        let Some(from) = classification_package_id(classifications.get(&package.id)) else {
            continue;
        };
        let targets = package
            .dependencies
            .iter()
            .filter_map(|dependency| resolve_dependency_package_id(dependency, &by_name))
            .filter_map(|id| classification_package_id(classifications.get(id)))
            .collect::<BTreeSet<_>>();
        edges.insert(from, targets);
    }
    edges
}

fn derive_backend_crates(
    cargo_packages: &BTreeMap<CargoPackageId, CargoPackage>,
    packages: &BTreeMap<SifrPackageId, SifrPackageMetadata>,
    classifications: &BTreeMap<CargoPackageId, PackageClassification>,
) -> BTreeMap<SifrPackageId, Vec<BackendCrateMetadata>> {
    let by_name = package_ids_by_name(cargo_packages);
    let by_cargo_id = packages
        .values()
        .map(|package| (&package.cargo_package_id, &package.package_id))
        .collect::<BTreeMap<_, _>>();
    let mut backend = BTreeMap::new();

    for cargo_package in cargo_packages.values() {
        let Some(sifr_package_id) = by_cargo_id.get(&cargo_package.id).copied() else {
            continue;
        };
        let mut backend_crates = cargo_package
            .dependencies
            .iter()
            .filter_map(|dependency| resolve_dependency_package_id(dependency, &by_name))
            .filter(|cargo_id| {
                matches!(
                    classifications.get(*cargo_id),
                    Some(PackageClassification::BackendRust)
                )
            })
            .filter_map(|cargo_id| cargo_packages.get(cargo_id))
            .map(|dependency| BackendCrateMetadata {
                cargo_package_id: dependency.id.clone(),
                cargo_package_name: dependency.name.clone(),
                cargo_version: dependency.version.clone(),
                cargo_source: dependency.source.clone(),
            })
            .collect::<Vec<_>>();
        backend_crates.sort_by(|left, right| {
            (&left.cargo_package_id, &left.cargo_package_name)
                .cmp(&(&right.cargo_package_id, &right.cargo_package_name))
        });
        backend.insert(sifr_package_id.clone(), backend_crates);
    }
    backend
}

fn package_ids_by_name(
    cargo_packages: &BTreeMap<CargoPackageId, CargoPackage>,
) -> BTreeMap<String, Vec<&CargoPackageId>> {
    let mut by_name: BTreeMap<String, Vec<&CargoPackageId>> = BTreeMap::new();
    for package in cargo_packages.values() {
        by_name
            .entry(package.name.clone())
            .or_default()
            .push(&package.id);
    }
    by_name
}

fn resolve_dependency_package_id<'a>(
    dependency: &CargoDependency,
    by_name: &'a BTreeMap<String, Vec<&'a CargoPackageId>>,
) -> Option<&'a CargoPackageId> {
    let package_name = dependency.package.as_ref().unwrap_or(&dependency.name);
    by_name
        .get(package_name)
        .and_then(|ids| ids.first().copied())
}

fn classification_package_id(
    classification: Option<&PackageClassification>,
) -> Option<SifrPackageId> {
    match classification {
        Some(
            PackageClassification::SifrSource(package_id)
            | PackageClassification::RustBackedSifr(package_id),
        ) => Some(package_id.clone()),
        Some(PackageClassification::BackendRust) | None => None,
    }
}

#[must_use]
pub fn exported_roots(package: &SifrPackageMetadata) -> BTreeSet<ImportRoot> {
    package.manifest.exports.iter().cloned().collect()
}
