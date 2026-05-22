use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId, SifrPackageMetadata};
use crate::imports::namespace_api::{parse_init_sifr_reexports, NamespaceApi};
use crate::manifest::sifr::ImportRoot;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageSourceMap {
    pub roots: BTreeMap<(SifrPackageId, ImportRoot), PathBuf>,
    pub modules: BTreeMap<PackageModuleKey, PackageModuleSource>,
    pub public_apis: BTreeMap<PackageModuleKey, NamespaceApi>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DottedModulePath(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageModuleKey {
    pub package_id: SifrPackageId,
    pub module_path: DottedModulePath,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageModuleSource {
    pub package_id: SifrPackageId,
    pub cargo_package_id: crate::CargoPackageId,
    pub module_path: DottedModulePath,
    pub file_path: PathBuf,
    pub source_root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageImportResolution {
    pub importing_package_id: SifrPackageId,
    pub import_path: DottedModulePath,
    pub resolved_module: PackageModuleSource,
    pub origin: PackageImportOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageImportOrigin {
    OwnPackage,
    DirectDependency {
        import_root: ImportRoot,
        target_export_root: ImportRoot,
        dependency_package_id: SifrPackageId,
    },
}

impl PackageSourceMap {
    pub fn build(graph: &SifrPackageGraph) -> Result<Self, Vec<PackageDiagnostic>> {
        let mut source_map = Self::default();
        let mut diagnostics = Vec::new();

        for package in graph.packages.values() {
            for source_root in package_source_roots(package) {
                source_map.roots.insert(
                    (package.package_id.clone(), root_for(package)),
                    source_root.clone(),
                );
                match discover_package_modules(package, &source_root) {
                    Ok(modules) => {
                        for module in modules {
                            let key = PackageModuleKey {
                                package_id: package.package_id.clone(),
                                module_path: module.module_path.clone(),
                            };
                            if source_map.modules.insert(key, module).is_some() {
                                diagnostics.push(PackageDiagnostic::invalid_sifr_manifest(
                                    &package.cargo_package_id,
                                    package.sifr_manifest.clone(),
                                    "source.roots",
                                    "multiple source files define the same module path",
                                ));
                            }
                        }
                    }
                    Err(error) => diagnostics.push(PackageDiagnostic::invalid_sifr_manifest(
                        &package.cargo_package_id,
                        package.sifr_manifest.clone(),
                        "source.roots",
                        error,
                    )),
                }
                match discover_namespace_apis(package, &source_root) {
                    Ok(apis) => {
                        for api in apis {
                            source_map.public_apis.insert(
                                PackageModuleKey {
                                    package_id: package.package_id.clone(),
                                    module_path: api.namespace.clone(),
                                },
                                api,
                            );
                        }
                    }
                    Err(errors) => diagnostics.extend(errors),
                }
            }
        }

        if diagnostics.is_empty() {
            Ok(source_map)
        } else {
            Err(diagnostics)
        }
    }

    pub fn resolve_import(
        &self,
        graph: &SifrPackageGraph,
        importing_package_id: &SifrPackageId,
        import_path: &DottedModulePath,
    ) -> Result<PackageImportResolution, PackageDiagnostic> {
        let importer = graph
            .packages
            .get(importing_package_id)
            .ok_or_else(|| PackageDiagnostic::cargo_metadata_parse("unknown importing package"))?;

        if let Some(module) = self.module(importing_package_id, import_path) {
            return Ok(PackageImportResolution {
                importing_package_id: importing_package_id.clone(),
                import_path: import_path.clone(),
                resolved_module: module.clone(),
                origin: PackageImportOrigin::OwnPackage,
            });
        }

        let Some(scoped_import) = graph
            .direct_dependency_scopes
            .get(importing_package_id)
            .and_then(|scope| matching_scoped_import(&scope.imports, import_path))
        else {
            return Err(PackageDiagnostic::undeclared_direct_import(
                &importer.cargo_package_id,
                importing_package_id,
                &import_path.0,
            ));
        };

        let target_path = remap_import_path(
            import_path,
            &scoped_import.import_root,
            &scoped_import.target_export_root,
        );
        let Some(module) = self.module(&scoped_import.package_id, &target_path) else {
            return Err(PackageDiagnostic::undeclared_direct_import(
                &importer.cargo_package_id,
                importing_package_id,
                &import_path.0,
            ));
        };

        if is_private_dependency_module(self, graph, module, &target_path) {
            return Err(PackageDiagnostic::private_module_access(
                &importer.cargo_package_id,
                importing_package_id,
                &import_path.0,
                &scoped_import.package_id,
            ));
        }

        Ok(PackageImportResolution {
            importing_package_id: importing_package_id.clone(),
            import_path: import_path.clone(),
            resolved_module: module.clone(),
            origin: PackageImportOrigin::DirectDependency {
                import_root: scoped_import.import_root.clone(),
                target_export_root: scoped_import.target_export_root.clone(),
                dependency_package_id: scoped_import.package_id.clone(),
            },
        })
    }

    fn module(
        &self,
        package_id: &SifrPackageId,
        module_path: &DottedModulePath,
    ) -> Option<&PackageModuleSource> {
        self.modules.get(&PackageModuleKey {
            package_id: package_id.clone(),
            module_path: module_path.clone(),
        })
    }
}

fn package_source_roots(package: &SifrPackageMetadata) -> Vec<PathBuf> {
    package
        .manifest
        .source_roots
        .iter()
        .map(|root| package.package_root.join(&root.0))
        .collect()
}

fn root_for(package: &SifrPackageMetadata) -> ImportRoot {
    package
        .manifest
        .exports
        .first()
        .cloned()
        .unwrap_or_else(|| ImportRoot(package.sifr_name.0.clone()))
}

fn discover_package_modules(
    package: &SifrPackageMetadata,
    source_root: &Path,
) -> Result<Vec<PackageModuleSource>, String> {
    let mut modules = Vec::new();
    discover_modules_recursive(package, source_root, source_root, &mut modules)?;
    modules.sort_by(|left, right| left.module_path.cmp(&right.module_path));
    Ok(modules)
}

fn discover_modules_recursive(
    package: &SifrPackageMetadata,
    source_root: &Path,
    directory: &Path,
    modules: &mut Vec<PackageModuleSource>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(directory).map_err(|error| {
        format!(
            "could not read source root '{}': {error}",
            directory.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "could not read source entry under '{}': {error}",
                directory.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            discover_modules_recursive(package, source_root, &path, modules)?;
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("sifr") {
            continue;
        }
        let Some(module_path) = module_path_from_file(package, source_root, &path) else {
            continue;
        };
        modules.push(PackageModuleSource {
            package_id: package.package_id.clone(),
            cargo_package_id: package.cargo_package_id.clone(),
            module_path,
            file_path: path,
            source_root: source_root.to_path_buf(),
        });
    }
    Ok(())
}

fn discover_namespace_apis(
    package: &SifrPackageMetadata,
    source_root: &Path,
) -> Result<Vec<NamespaceApi>, Vec<PackageDiagnostic>> {
    let modules = discover_package_modules(package, source_root).map_err(|error| {
        vec![PackageDiagnostic::invalid_sifr_manifest(
            &package.cargo_package_id,
            package.sifr_manifest.clone(),
            if package.manifest.production_schema {
                "source.root"
            } else {
                "source.roots"
            },
            error,
        )]
    })?;
    let mut apis = Vec::new();
    let mut diagnostics = Vec::new();
    for module in modules {
        if module.file_path.file_name().and_then(|name| name.to_str()) != Some("__init__.sifr") {
            continue;
        }
        match parse_init_sifr_reexports(
            &package.cargo_package_id,
            &package.sifr_manifest,
            module.module_path,
            &module.file_path,
            source_root,
        ) {
            Ok(api) => apis.push(api),
            Err(errors) => diagnostics.extend(errors),
        }
    }
    if diagnostics.is_empty() {
        Ok(apis)
    } else {
        Err(diagnostics)
    }
}

fn module_path_from_file(
    package: &SifrPackageMetadata,
    source_root: &Path,
    file_path: &Path,
) -> Option<DottedModulePath> {
    let relative = file_path.strip_prefix(source_root).ok()?;
    let mut parts = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let last = parts.last_mut()?;
    if last == "__init__.sifr" {
        let _ = parts.pop();
    } else if let Some(stripped) = last.strip_suffix(".sifr") {
        *last = stripped.to_string();
    }
    if !parts.iter().all(|part| valid_identifier(part)) {
        return None;
    }
    if package.manifest.production_schema {
        parts.insert(0, package.sifr_name.0.clone());
    }
    if parts.is_empty() {
        return None;
    }
    Some(DottedModulePath(parts.join(".")))
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn matching_scoped_import<'a>(
    imports: &'a BTreeMap<ImportRoot, crate::ScopedImport>,
    import_path: &DottedModulePath,
) -> Option<&'a crate::ScopedImport> {
    imports
        .iter()
        .filter(|(root, _)| import_root_matches(root, import_path))
        .max_by_key(|(root, _)| root.0.split('.').count())
        .map(|(_, import)| import)
}

fn import_root_matches(root: &ImportRoot, import_path: &DottedModulePath) -> bool {
    import_path.0 == root.0
        || import_path
            .0
            .strip_prefix(&root.0)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

fn remap_import_path(
    import_path: &DottedModulePath,
    import_root: &ImportRoot,
    target_export_root: &ImportRoot,
) -> DottedModulePath {
    if import_path.0 == import_root.0 {
        return DottedModulePath(target_export_root.0.clone());
    }
    let suffix = import_path
        .0
        .strip_prefix(&format!("{}.", import_root.0))
        .unwrap_or_default();
    DottedModulePath(format!("{}.{}", target_export_root.0, suffix))
}

fn is_private_dependency_module(
    source_map: &PackageSourceMap,
    graph: &SifrPackageGraph,
    module: &PackageModuleSource,
    module_path: &DottedModulePath,
) -> bool {
    let Some(package) = graph.packages.get(&module.package_id) else {
        return false;
    };
    if !package.manifest.production_schema
        && package.manifest.exports.iter().any(|export| {
            import_root_matches(export, module_path)
                && !module_path.0.split('.').any(|part| part.starts_with('_'))
        })
    {
        return false;
    }
    if source_map.public_apis.contains_key(&PackageModuleKey {
        package_id: module.package_id.clone(),
        module_path: module_path.clone(),
    }) {
        return false;
    }
    true
}
