use crate::cargo::metadata::{
    CargoMetadata, CargoPackage, CargoPackageId, NormalizedCargoMetadata,
};
use crate::diag::PackageDiagnostic;
use crate::graph::scopes::{DirectDependencyScope, derive_direct_dependency_scopes};
use crate::manifest::metadata::CargoSifrAliasMetadata;
use crate::manifest::sifr::{SifrManifest, SifrPackageName};
use crate::manifest::validate::validate_source_root_exists;
use crate::source::layout::{MarkerValidation, validate_pure_marker_file};
use sifr_frontend::SourceProvider;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SifrPackageId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrPackageGraph {
    pub packages: BTreeMap<SifrPackageId, SifrPackageMetadata>,
    pub cargo_edges: BTreeMap<SifrPackageId, BTreeSet<SifrPackageId>>,
    pub direct_dependency_scopes: BTreeMap<SifrPackageId, DirectDependencyScope>,
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
    pub dependency_name: String,
    pub dependency_kind: Option<String>,
    pub cargo_package_name: String,
    pub cargo_version: String,
    pub cargo_source: Option<String>,
    pub cargo_manifest_path: PathBuf,
    pub links: Option<String>,
    pub has_build_script: bool,
    pub has_proc_macro: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageClassification {
    SifrSource(SifrPackageId),
    RustBackedSifr(SifrPackageId),
    BackendRust,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DirectCargoDependency {
    pub from: CargoPackageId,
    pub dependency_name: String,
    pub dependency_kind: Option<String>,
    pub to: CargoPackageId,
}

pub fn derive_package_graph(
    metadata: CargoMetadata,
    provider: &mut impl SourceProvider,
) -> Result<SifrPackageGraph, Vec<PackageDiagnostic>> {
    derive_package_graph_from_normalized(&metadata.normalize(), provider)
}

pub fn derive_package_graph_from_normalized(
    metadata: &NormalizedCargoMetadata,
    provider: &mut impl SourceProvider,
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
        match load_sifr_package(package, &package_root, &manifest_path, provider) {
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

    let direct_dependencies = direct_cargo_dependencies(metadata);
    let cargo_edges = derive_sifr_edges(&direct_dependencies, &classifications);
    let mut diagnostics = Vec::new();
    let direct_dependency_scopes =
        match derive_direct_dependency_scopes(&direct_dependencies, &packages, &classifications) {
            Ok(scopes) => scopes,
            Err(errors) => {
                diagnostics.extend(errors);
                BTreeMap::new()
            }
        };
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let backend_crates = derive_backend_crates(
        &metadata.packages,
        &direct_dependencies,
        &packages,
        &classifications,
    );

    Ok(SifrPackageGraph {
        packages,
        cargo_edges,
        direct_dependency_scopes,
        backend_crates,
        classifications,
    })
}

fn load_sifr_package(
    package: &CargoPackage,
    package_root: &Path,
    manifest_path: &Path,
    provider: &mut impl SourceProvider,
) -> Result<SifrPackageMetadata, PackageDiagnostic> {
    let manifest = SifrManifest::load(&package.id, manifest_path, provider)?;
    validate_source_root_exists(
        &package.id,
        manifest_path,
        package_root,
        &manifest,
        provider,
    )?;
    validate_sql_schema_sources(
        &package.id,
        manifest_path,
        package_root,
        &manifest,
        provider,
    )?;
    if !manifest.declares_rust_backend() {
        validate_pure_markers(package, &package.id, provider)?;
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

fn validate_sql_schema_sources(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    package_root: &Path,
    manifest: &SifrManifest,
    provider: &mut impl SourceProvider,
) -> Result<(), PackageDiagnostic> {
    let canonical_root = if manifest.sql.profiles.is_empty() {
        None
    } else {
        Some(provider.canonicalize(package_root).map_err(|error| {
            PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                "sql.profiles",
                format!("cannot resolve package root for schema sources: {error}"),
            )
        })?)
    };
    for (profile_name, profile) in &manifest.sql.profiles {
        for source in &profile.sources {
            let path = package_root.join(source);
            if !provider.is_file(&path) {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    format!("sql.profiles.{profile_name}.source"),
                    format!(
                        "schema source '{}' must be a checked-in file inside the package",
                        source.display()
                    ),
                ));
            }
            let canonical_source = provider.canonicalize(&path).map_err(|error| {
                PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    format!("sql.profiles.{profile_name}.source"),
                    format!(
                        "cannot resolve schema source '{}': {error}",
                        source.display()
                    ),
                )
            })?;
            if !canonical_source.starts_with(canonical_root.as_deref().unwrap_or(package_root)) {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    format!("sql.profiles.{profile_name}.source"),
                    format!(
                        "schema source '{}' resolves outside the package",
                        source.display()
                    ),
                ));
            }
        }
    }
    Ok(())
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
    provider: &mut impl SourceProvider,
) -> Result<(), PackageDiagnostic> {
    for target in &package.targets {
        if !target.kind.contains("lib") {
            continue;
        }
        match validate_pure_marker_file(&target.src_path, provider) {
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

pub(crate) fn direct_cargo_dependencies(
    metadata: &NormalizedCargoMetadata,
) -> Vec<DirectCargoDependency> {
    let mut direct_dependencies = if metadata.resolve_edges.is_empty() {
        fallback_direct_cargo_dependencies(&metadata.packages)
    } else {
        metadata
            .resolve_edges
            .iter()
            .filter(|edge| {
                metadata.packages.contains_key(&edge.from)
                    && metadata.packages.contains_key(&edge.to)
            })
            .map(|edge| DirectCargoDependency {
                from: edge.from.clone(),
                dependency_name: edge.dependency_name.clone(),
                dependency_kind: dependency_kind_for_edge(metadata, edge),
                to: edge.to.clone(),
            })
            .collect()
    };

    direct_dependencies.sort_by(|left, right| {
        (
            &left.from,
            &left.dependency_name,
            left.dependency_kind.as_deref().unwrap_or_default(),
            &left.to,
        )
            .cmp(&(
                &right.from,
                &right.dependency_name,
                right.dependency_kind.as_deref().unwrap_or_default(),
                &right.to,
            ))
    });
    direct_dependencies
}

fn dependency_kind_for_edge(
    metadata: &NormalizedCargoMetadata,
    edge: &crate::cargo::metadata::CargoResolveEdge,
) -> Option<String> {
    let from = metadata.packages.get(&edge.from)?;
    let to = metadata.packages.get(&edge.to)?;
    from.dependencies
        .iter()
        .find(|dependency| {
            dependency.name == edge.dependency_name
                && dependency
                    .package
                    .as_deref()
                    .is_none_or(|package| package == to.name)
        })
        .and_then(|dependency| dependency.kind.clone())
}

fn fallback_direct_cargo_dependencies(
    cargo_packages: &BTreeMap<CargoPackageId, CargoPackage>,
) -> Vec<DirectCargoDependency> {
    let by_name = package_ids_by_name(cargo_packages);
    let mut direct_dependencies = Vec::new();

    for package in cargo_packages.values() {
        direct_dependencies.extend(package.dependencies.iter().filter_map(|dependency| {
            let package_name = dependency.package.as_ref().unwrap_or(&dependency.name);
            by_name.get(package_name).and_then(|ids| {
                ids.first().copied().map(|to| DirectCargoDependency {
                    from: package.id.clone(),
                    dependency_name: dependency.name.clone(),
                    dependency_kind: dependency.kind.clone(),
                    to: to.clone(),
                })
            })
        }));
    }
    direct_dependencies
}

fn derive_sifr_edges(
    direct_dependencies: &[DirectCargoDependency],
    classifications: &BTreeMap<CargoPackageId, PackageClassification>,
) -> BTreeMap<SifrPackageId, BTreeSet<SifrPackageId>> {
    let mut edges = BTreeMap::new();

    for dependency in direct_dependencies {
        let Some(from) = classification_package_id(classifications.get(&dependency.from)) else {
            continue;
        };
        let Some(to) = classification_package_id(classifications.get(&dependency.to)) else {
            continue;
        };
        edges.entry(from).or_insert_with(BTreeSet::new).insert(to);
    }
    edges
}

fn derive_backend_crates(
    cargo_packages: &BTreeMap<CargoPackageId, CargoPackage>,
    direct_dependencies: &[DirectCargoDependency],
    packages: &BTreeMap<SifrPackageId, SifrPackageMetadata>,
    classifications: &BTreeMap<CargoPackageId, PackageClassification>,
) -> BTreeMap<SifrPackageId, Vec<BackendCrateMetadata>> {
    let by_cargo_id = packages
        .values()
        .map(|package| (&package.cargo_package_id, &package.package_id))
        .collect::<BTreeMap<_, _>>();
    let mut backend = BTreeMap::new();

    for cargo_package in cargo_packages.values() {
        let Some(sifr_package_id) = by_cargo_id.get(&cargo_package.id).copied() else {
            continue;
        };
        let mut backend_crates = direct_dependencies
            .iter()
            .filter(|dependency| dependency.from == cargo_package.id)
            .filter(|dependency| {
                matches!(
                    classifications.get(&dependency.to),
                    Some(PackageClassification::BackendRust)
                )
            })
            .filter_map(|dependency| {
                cargo_packages
                    .get(&dependency.to)
                    .map(|cargo_dependency| BackendCrateMetadata {
                        cargo_package_id: cargo_dependency.id.clone(),
                        dependency_name: dependency.dependency_name.clone(),
                        dependency_kind: dependency.dependency_kind.clone(),
                        cargo_package_name: cargo_dependency.name.clone(),
                        cargo_version: cargo_dependency.version.clone(),
                        cargo_source: cargo_dependency.source.clone(),
                        cargo_manifest_path: cargo_dependency.manifest_path.clone(),
                        links: cargo_dependency.links.clone(),
                        has_build_script: cargo_dependency
                            .targets
                            .iter()
                            .any(|target| target.kind.contains("custom-build")),
                        has_proc_macro: cargo_dependency
                            .targets
                            .iter()
                            .any(|target| target.kind.contains("proc-macro")),
                    })
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
