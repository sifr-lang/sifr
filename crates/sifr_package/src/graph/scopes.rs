use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::graph::derive::{
    DirectCargoDependency, PackageClassification, SifrPackageGraph, SifrPackageId,
    SifrPackageMetadata,
};
use crate::manifest::package_sections::SifrDependency;
use crate::manifest::sifr::ImportRoot;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirectDependencyScope {
    pub imports: BTreeMap<ImportRoot, ScopedImport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedImport {
    pub import_root: ImportRoot,
    pub target_export_root: ImportRoot,
    pub package_id: SifrPackageId,
    pub cargo_package_id: CargoPackageId,
    pub dependency_name: String,
    pub source: ScopedImportSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopedImportSource {
    Export,
    Alias { alias: String },
}

#[must_use]
pub fn direct_dependency_export_scopes(
    graph: &SifrPackageGraph,
) -> BTreeMap<SifrPackageId, BTreeMap<ImportRoot, SifrPackageId>> {
    let mut scopes = BTreeMap::new();
    for (from, dependency_scope) in &graph.direct_dependency_scopes {
        scopes.insert(
            from.clone(),
            dependency_scope
                .imports
                .iter()
                .map(|(root, import)| (root.clone(), import.package_id.clone()))
                .collect(),
        );
    }
    scopes
}

pub(crate) fn derive_direct_dependency_scopes(
    direct_dependencies: &[DirectCargoDependency],
    packages: &BTreeMap<SifrPackageId, SifrPackageMetadata>,
    classifications: &BTreeMap<CargoPackageId, PackageClassification>,
) -> Result<BTreeMap<SifrPackageId, DirectDependencyScope>, Vec<PackageDiagnostic>> {
    let packages_by_cargo_id = packages
        .values()
        .map(|package| (&package.cargo_package_id, package))
        .collect::<BTreeMap<_, _>>();
    let dependencies_by_owner = direct_dependencies_by_owner(direct_dependencies);
    let mut diagnostics = Vec::new();
    let mut scopes = BTreeMap::new();

    for package in packages.values() {
        let mut scope = DirectDependencyScope::default();
        let direct = dependencies_by_owner
            .get(&package.cargo_package_id)
            .map(Vec::as_slice)
            .unwrap_or_default();
        validate_alias_dependencies(package, direct, classifications, &mut diagnostics);

        for dependency in direct
            .iter()
            .copied()
            .filter(|dependency| dependency.dependency_kind.is_none())
        {
            let Some(target) = packages_by_cargo_id.get(&dependency.to).copied() else {
                continue;
            };
            let aliases = aliases_for_dependency(package, &dependency.dependency_name);
            if aliases.is_empty() {
                for export in &target.manifest.exports {
                    insert_scoped_import(
                        package,
                        &mut scope,
                        ScopedImport {
                            import_root: export.clone(),
                            target_export_root: export.clone(),
                            package_id: target.package_id.clone(),
                            cargo_package_id: target.cargo_package_id.clone(),
                            dependency_name: dependency.dependency_name.clone(),
                            source: ScopedImportSource::Export,
                        },
                        &mut diagnostics,
                    );
                }
            } else {
                let Some(target_export_root) = target.manifest.exports.first().cloned() else {
                    continue;
                };
                for alias in aliases {
                    let Some(import_root) = parse_alias_import_root(
                        package,
                        &alias.alias,
                        &alias.import,
                        &mut diagnostics,
                    ) else {
                        continue;
                    };
                    insert_scoped_import(
                        package,
                        &mut scope,
                        ScopedImport {
                            import_root,
                            target_export_root: target_export_root.clone(),
                            package_id: target.package_id.clone(),
                            cargo_package_id: target.cargo_package_id.clone(),
                            dependency_name: dependency.dependency_name.clone(),
                            source: ScopedImportSource::Alias {
                                alias: alias.alias.clone(),
                            },
                        },
                        &mut diagnostics,
                    );
                }
            }
        }
        scopes.insert(package.package_id.clone(), scope);
    }

    if diagnostics.is_empty() {
        Ok(scopes)
    } else {
        Err(diagnostics)
    }
}

fn direct_dependencies_by_owner(
    direct_dependencies: &[DirectCargoDependency],
) -> BTreeMap<&CargoPackageId, Vec<&DirectCargoDependency>> {
    let mut by_owner: BTreeMap<&CargoPackageId, Vec<&DirectCargoDependency>> = BTreeMap::new();
    for dependency in direct_dependencies {
        by_owner
            .entry(&dependency.from)
            .or_default()
            .push(dependency);
    }
    by_owner
}

fn validate_alias_dependencies(
    package: &SifrPackageMetadata,
    direct_dependencies: &[&DirectCargoDependency],
    classifications: &BTreeMap<CargoPackageId, PackageClassification>,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    let direct_sifr_dependencies = direct_dependencies
        .iter()
        .filter(|dependency| {
            matches!(
                classifications.get(&dependency.to),
                Some(
                    PackageClassification::SifrSource(_) | PackageClassification::RustBackedSifr(_),
                )
            )
        })
        .map(|dependency| dependency.dependency_name.as_str())
        .collect::<BTreeSet<_>>();

    for (alias, metadata) in &package.aliases {
        if !direct_sifr_dependencies.contains(metadata.dependency.as_str()) {
            diagnostics.push(PackageDiagnostic::invalid_cargo_sifr_metadata(
                &package.cargo_package_id,
                &package.cargo_package_name,
                format!(
                    "alias `{alias}` references `{}`, which is not a direct Sifr dependency",
                    metadata.dependency
                ),
            ));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DependencyAlias {
    alias: String,
    import: String,
}

fn aliases_for_dependency(
    package: &SifrPackageMetadata,
    dependency_name: &str,
) -> Vec<DependencyAlias> {
    let mut aliases = package
        .aliases
        .iter()
        .filter(|(_, alias)| alias.dependency == dependency_name)
        .map(|(alias, metadata)| DependencyAlias {
            alias: alias.clone(),
            import: metadata.import.clone(),
        })
        .collect::<Vec<_>>();

    if let Some(import) = package
        .manifest
        .dependencies
        .get(dependency_name)
        .and_then(dependency_import_root)
    {
        aliases.push(DependencyAlias {
            alias: dependency_name.to_string(),
            import,
        });
    }

    aliases.sort_by(|left, right| (&left.alias, &left.import).cmp(&(&right.alias, &right.import)));
    aliases.dedup();
    aliases
}

fn dependency_import_root(dependency: &SifrDependency) -> Option<String> {
    let SifrDependency::Table(table) = dependency else {
        return None;
    };
    table
        .get("import")
        .filter(|value| !value.is_empty())
        .cloned()
}

fn parse_alias_import_root(
    package: &SifrPackageMetadata,
    alias: &str,
    import: &str,
    diagnostics: &mut Vec<PackageDiagnostic>,
) -> Option<ImportRoot> {
    if import.split('.').all(valid_identifier) {
        Some(ImportRoot(import.to_string()))
    } else {
        diagnostics.push(PackageDiagnostic::invalid_cargo_sifr_metadata(
            &package.cargo_package_id,
            &package.cargo_package_name,
            format!("alias `{alias}` import `{import}` is not a valid dotted import root"),
        ));
        None
    }
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn insert_scoped_import(
    owner: &SifrPackageMetadata,
    scope: &mut DirectDependencyScope,
    import: ScopedImport,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    if let Some(existing) = scope.imports.get(&import.import_root) {
        if existing.package_id != import.package_id
            || existing.dependency_name != import.dependency_name
        {
            diagnostics.push(PackageDiagnostic::ambiguous_import_root(
                &owner.cargo_package_id,
                &owner.package_id,
                &import.import_root,
                &[
                    scoped_import_candidate(existing),
                    scoped_import_candidate(&import),
                ],
            ));
        }
        return;
    }
    scope.imports.insert(import.import_root.clone(), import);
}

fn scoped_import_candidate(import: &ScopedImport) -> String {
    match &import.source {
        ScopedImportSource::Export => format!(
            "{} via Cargo dependency `{}`",
            import.package_id.0, import.dependency_name
        ),
        ScopedImportSource::Alias { alias } => format!(
            "{} via Cargo dependency `{}` alias `{alias}`",
            import.package_id.0, import.dependency_name
        ),
    }
}
