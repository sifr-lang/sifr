use super::bridge_inventory::{
    discover_python_bridge_inventory, PythonBridgeImport, PythonBridgeInventory,
};
use super::requirements::{PythonRequirementContribution, PythonRequirementKind};
use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId, SifrPackageMetadata};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;

pub const PYTHON_BRIDGE_RUNTIME_ROOT: &str = "__sifr_bridge__";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolvedPythonBridgeGraph {
    pub packages: Vec<ResolvedPythonBridgePackage>,
    pub requirements: Vec<PythonRequirementContribution>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPythonBridgePackage {
    pub package_id: SifrPackageId,
    pub resolved_package_key: String,
    pub runtime_package: String,
    pub inventory_digest: String,
    pub modules: Vec<ResolvedPythonBridgeModule>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPythonBridgeModule {
    pub module: String,
    pub runtime_module: String,
    pub source_path: String,
    pub source_digest: String,
    pub is_package: bool,
    pub imports: Vec<ResolvedPythonBridgeImport>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolvedPythonBridgeImport {
    SamePackage {
        module: String,
        runtime_module: String,
    },
    ThirdParty {
        root: String,
    },
}

#[must_use]
pub fn resolved_python_bridge_package_key(package_id: &SifrPackageId) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"sifr-python-bridge-package-v1\0");
    hasher.update(package_id.0.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[must_use]
pub fn resolved_python_bridge_runtime_package(package_id: &SifrPackageId) -> String {
    format!(
        "{PYTHON_BRIDGE_RUNTIME_ROOT}.p_{}",
        resolved_python_bridge_package_key(package_id)
    )
}

pub fn resolve_python_bridge_graph(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
) -> Result<ResolvedPythonBridgeGraph, Vec<PackageDiagnostic>> {
    if !graph.packages.contains_key(root_package_id) {
        return Err(vec![PackageDiagnostic::cargo_metadata_parse(
            "root Python bridge package is missing from the package graph",
        )]);
    }
    let selected = selected_runtime_packages(graph, root_package_id);
    let mut packages = Vec::new();
    let mut requirements = Vec::new();
    let mut diagnostics = Vec::new();

    for package_id in selected {
        let Some(package) = graph.packages.get(&package_id) else {
            continue;
        };
        match resolve_package(package) {
            Ok((Some(resolved), mut package_requirements)) => {
                packages.push(resolved);
                requirements.append(&mut package_requirements);
            }
            Ok((None, _)) => {}
            Err(mut errors) => diagnostics.append(&mut errors),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    requirements.sort();
    requirements.dedup();
    Ok(ResolvedPythonBridgeGraph {
        packages,
        requirements,
    })
}

fn resolve_package(
    package: &SifrPackageMetadata,
) -> Result<
    (
        Option<ResolvedPythonBridgePackage>,
        Vec<PythonRequirementContribution>,
    ),
    Vec<PackageDiagnostic>,
> {
    let inventory = discover_python_bridge_inventory(package)?;
    if inventory.modules.is_empty() {
        return Ok((None, Vec::new()));
    }
    let runtime_package = resolved_python_bridge_runtime_package(&package.package_id);
    let known_modules = known_module_names(&inventory);
    let mut diagnostics = Vec::new();
    let mut requirements = Vec::new();
    let modules = inventory
        .modules
        .iter()
        .map(|module| {
            let imports = module
                .imports
                .iter()
                .filter_map(|import| match import {
                    PythonBridgeImport::SamePackage { module: imported } => {
                        if !known_modules.contains(imported) {
                            diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                                &package.cargo_package_id,
                                &package.package_root.join(&module.source_path),
                                format!(
                                    "same-package bridge import '{imported}' does not resolve to an inventoried module or package"
                                ),
                            ));
                            return None;
                        }
                        Some(ResolvedPythonBridgeImport::SamePackage {
                            module: imported.clone(),
                            runtime_module: format!("{runtime_package}.{imported}"),
                        })
                    }
                    PythonBridgeImport::ThirdParty { root } => {
                        requirements.push(PythonRequirementContribution {
                            root: root.clone(),
                            package_id: package.package_id.clone(),
                            kind: PythonRequirementKind::BridgeImport,
                            source: format!(
                                "{}:{} imports {root}",
                                package.package_id.0, module.module
                            ),
                        });
                        Some(ResolvedPythonBridgeImport::ThirdParty { root: root.clone() })
                    }
                })
                .collect();
            ResolvedPythonBridgeModule {
                module: module.module.clone(),
                runtime_module: format!("{runtime_package}.{}", module.module),
                source_path: module.source_path.clone(),
                source_digest: module.source_digest.clone(),
                is_package: module.is_package,
                imports,
            }
        })
        .collect();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok((
        Some(ResolvedPythonBridgePackage {
            package_id: package.package_id.clone(),
            resolved_package_key: resolved_python_bridge_package_key(&package.package_id),
            runtime_package,
            inventory_digest: inventory.inventory_digest,
            modules,
        }),
        requirements,
    ))
}

fn known_module_names(inventory: &PythonBridgeInventory) -> BTreeSet<String> {
    let mut known = BTreeSet::new();
    for module in &inventory.modules {
        let mut prefix = String::new();
        for component in module.module.split('.') {
            if !prefix.is_empty() {
                prefix.push('.');
            }
            prefix.push_str(component);
            known.insert(prefix.clone());
        }
    }
    known
}

fn selected_runtime_packages(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
) -> BTreeSet<SifrPackageId> {
    let mut selected = BTreeSet::new();
    let mut pending = vec![root_package_id.clone()];
    while let Some(package_id) = pending.pop() {
        if !selected.insert(package_id.clone()) {
            continue;
        }
        if let Some(scope) = graph.direct_dependency_scopes.get(&package_id) {
            pending.extend(
                scope
                    .imports
                    .values()
                    .map(|import| import.package_id.clone()),
            );
        }
    }
    selected
}
