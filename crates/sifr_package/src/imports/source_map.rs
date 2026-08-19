use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use crate::imports::namespace_api::NamespaceApi;
use crate::manifest::sifr::ImportRoot;
use sifr_frontend::SourceProvider;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod discovery;
use discovery::{discover_namespace_apis, discover_package_modules, package_source_root, root_for};
mod resolution;
use resolution::{is_private_dependency_module, matching_scoped_import, remap_import_path};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageSourceMap {
    pub roots: BTreeMap<(SifrPackageId, ImportRoot), PathBuf>,
    pub modules: BTreeMap<PackageModuleKey, PackageModuleSource>,
    pub ambiguous_modules: BTreeMap<PackageModuleKey, Vec<PackageModuleSource>>,
    pub public_apis: BTreeMap<PackageModuleKey, NamespaceApi>,
    pub fatal_diagnostics: Vec<PackageDiagnostic>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageImportAmbiguity {
    pub importing_package_id: SifrPackageId,
    pub import_path: DottedModulePath,
    pub package_id: SifrPackageId,
    pub cargo_package_id: crate::CargoPackageId,
    pub module_path: DottedModulePath,
    pub candidates: Vec<PackageModuleSource>,
    pub origin: PackageImportOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageImportResolutionResult {
    Resolved(PackageImportResolution),
    Ambiguous(PackageImportAmbiguity),
    Unresolved(PackageDiagnostic),
    PrivateAccess(PackageDiagnostic),
    FatalPackageMapFailure(Vec<PackageDiagnostic>),
}

impl PackageSourceMap {
    pub fn build(
        graph: &SifrPackageGraph,
        provider: &mut impl SourceProvider,
    ) -> Result<Self, Vec<PackageDiagnostic>> {
        let mut source_map = Self::default();
        let mut diagnostics = Vec::new();

        for package in graph.packages.values() {
            let source_root = package_source_root(package);
            source_map.roots.insert(
                (package.package_id.clone(), root_for(package)),
                source_root.clone(),
            );
            match discover_package_modules(package, &source_root, provider) {
                Ok(modules) => {
                    for module in modules {
                        source_map.insert_module(module);
                    }
                }
                Err(error) => diagnostics.push(PackageDiagnostic::invalid_sifr_manifest(
                    &package.cargo_package_id,
                    package.sifr_manifest.clone(),
                    "source.root",
                    error,
                )),
            }
            match discover_namespace_apis(package, &source_root, provider) {
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

        if diagnostics.is_empty() {
            Ok(source_map)
        } else {
            source_map.fatal_diagnostics.clone_from(&diagnostics);
            Err(diagnostics)
        }
    }

    #[must_use]
    pub fn from_fatal_diagnostics(diagnostics: Vec<PackageDiagnostic>) -> Self {
        Self {
            fatal_diagnostics: diagnostics,
            ..Self::default()
        }
    }

    pub fn resolve_import_result(
        &self,
        graph: &SifrPackageGraph,
        importing_package_id: &SifrPackageId,
        import_path: &DottedModulePath,
    ) -> PackageImportResolutionResult {
        if !self.fatal_diagnostics.is_empty() {
            return PackageImportResolutionResult::FatalPackageMapFailure(
                self.fatal_diagnostics.clone(),
            );
        }
        let importer = graph
            .packages
            .get(importing_package_id)
            .ok_or_else(|| PackageDiagnostic::cargo_metadata_parse("unknown importing package"));
        let importer = match importer {
            Ok(importer) => importer,
            Err(diagnostic) => return PackageImportResolutionResult::Unresolved(diagnostic),
        };

        match self.module_resolution(importing_package_id, import_path) {
            ModuleResolution::Resolved(module) => {
                return PackageImportResolutionResult::Resolved(PackageImportResolution {
                    importing_package_id: importing_package_id.clone(),
                    import_path: import_path.clone(),
                    resolved_module: module.clone(),
                    origin: PackageImportOrigin::OwnPackage,
                });
            }
            ModuleResolution::Ambiguous(candidates) => {
                let Some(first) = candidates.first() else {
                    return PackageImportResolutionResult::Unresolved(
                        PackageDiagnostic::undeclared_direct_import(
                            &importer.cargo_package_id,
                            importing_package_id,
                            &import_path.0,
                        ),
                    );
                };
                return PackageImportResolutionResult::Ambiguous(PackageImportAmbiguity {
                    importing_package_id: importing_package_id.clone(),
                    import_path: import_path.clone(),
                    package_id: importing_package_id.clone(),
                    cargo_package_id: first.cargo_package_id.clone(),
                    module_path: import_path.clone(),
                    candidates: candidates.to_vec(),
                    origin: PackageImportOrigin::OwnPackage,
                });
            }
            ModuleResolution::Missing => {}
        }

        let Some(scoped_import) = graph
            .direct_dependency_scopes
            .get(importing_package_id)
            .and_then(|scope| matching_scoped_import(&scope.imports, import_path))
        else {
            return PackageImportResolutionResult::Unresolved(
                PackageDiagnostic::undeclared_direct_import(
                    &importer.cargo_package_id,
                    importing_package_id,
                    &import_path.0,
                ),
            );
        };

        let target_path = remap_import_path(
            import_path,
            &scoped_import.import_root,
            &scoped_import.target_export_root,
        );
        let module = match self.module_resolution(&scoped_import.package_id, &target_path) {
            ModuleResolution::Resolved(module) => module,
            ModuleResolution::Ambiguous(candidates) => {
                let Some(first) = candidates.first() else {
                    return PackageImportResolutionResult::Unresolved(
                        PackageDiagnostic::undeclared_direct_import(
                            &importer.cargo_package_id,
                            importing_package_id,
                            &import_path.0,
                        ),
                    );
                };
                return PackageImportResolutionResult::Ambiguous(PackageImportAmbiguity {
                    importing_package_id: importing_package_id.clone(),
                    import_path: import_path.clone(),
                    package_id: scoped_import.package_id.clone(),
                    cargo_package_id: first.cargo_package_id.clone(),
                    module_path: target_path,
                    candidates: candidates.to_vec(),
                    origin: PackageImportOrigin::DirectDependency {
                        import_root: scoped_import.import_root.clone(),
                        target_export_root: scoped_import.target_export_root.clone(),
                        dependency_package_id: scoped_import.package_id.clone(),
                    },
                });
            }
            ModuleResolution::Missing => {
                return PackageImportResolutionResult::Unresolved(
                    PackageDiagnostic::undeclared_direct_import(
                        &importer.cargo_package_id,
                        importing_package_id,
                        &import_path.0,
                    ),
                );
            }
        };

        if is_private_dependency_module(self, module, &target_path) {
            return PackageImportResolutionResult::PrivateAccess(
                PackageDiagnostic::private_module_access(
                    &importer.cargo_package_id,
                    importing_package_id,
                    &import_path.0,
                    &scoped_import.package_id,
                ),
            );
        }

        PackageImportResolutionResult::Resolved(PackageImportResolution {
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

    fn insert_module(&mut self, module: PackageModuleSource) {
        let key = PackageModuleKey {
            package_id: module.package_id.clone(),
            module_path: module.module_path.clone(),
        };
        if let Some(existing) = self.modules.remove(&key) {
            self.ambiguous_modules
                .entry(key)
                .or_insert_with(|| vec![existing])
                .push(module);
        } else if let Some(candidates) = self.ambiguous_modules.get_mut(&key) {
            candidates.push(module);
        } else {
            self.modules.insert(key, module);
        }
    }

    fn module_resolution(
        &self,
        package_id: &SifrPackageId,
        module_path: &DottedModulePath,
    ) -> ModuleResolution<'_> {
        let key = PackageModuleKey {
            package_id: package_id.clone(),
            module_path: module_path.clone(),
        };
        if let Some(candidates) = self.ambiguous_modules.get(&key) {
            return ModuleResolution::Ambiguous(candidates);
        }
        self.modules
            .get(&key)
            .map_or(ModuleResolution::Missing, ModuleResolution::Resolved)
    }

    pub fn module_for_file(
        &self,
        package_id: &SifrPackageId,
        file_path: &Path,
    ) -> Option<&PackageModuleSource> {
        let normalized_file = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());
        self.modules
            .values()
            .chain(
                self.ambiguous_modules
                    .values()
                    .flat_map(|modules| modules.iter()),
            )
            .find(|module| {
                &module.package_id == package_id
                    && module
                        .file_path
                        .canonicalize()
                        .unwrap_or_else(|_| module.file_path.clone())
                        == normalized_file
            })
    }
}

enum ModuleResolution<'a> {
    Resolved(&'a PackageModuleSource),
    Ambiguous(&'a [PackageModuleSource]),
    Missing,
}
