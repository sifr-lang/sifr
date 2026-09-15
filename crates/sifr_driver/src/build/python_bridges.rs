use super::project_codegen::GeneratedBinaryProject;
use super::python_runtime::{EmbeddedPythonBridgeSource, PackagePythonRuntime};
use sifr_codegen::{PythonBridgeImportPlan, PythonBridgeModulePlan, PythonBridgePackagePlan};
use sifr_lowering::{LoweringOptions, PythonBridgeTargetAuthority};
use sifr_package::{
    ResolvedPythonBridgeGraph, ResolvedPythonBridgeImport, ResolvedPythonBridgeModule,
    ResolvedPythonBridgePackage,
};
use std::collections::{BTreeMap, HashMap};

pub(super) fn apply_package_python_bridge_metadata(
    mut generated: GeneratedBinaryProject,
    bridges: Option<&ResolvedPythonBridgeGraph>,
) -> GeneratedBinaryProject {
    if let Some(graph) = bridges {
        generated.interop.python.required_import_roots.extend(
            graph
                .requirements
                .iter()
                .map(|requirement| requirement.root.clone()),
        );
        generated.interop.python.required_import_roots.sort();
        generated.interop.python.required_import_roots.dedup();
    }
    generated.interop.python.bridge_packages = bridges
        .map(|graph| graph.packages.iter().map(package_plan).collect())
        .unwrap_or_default();
    generated
}

fn package_plan(package: &ResolvedPythonBridgePackage) -> PythonBridgePackagePlan {
    PythonBridgePackagePlan {
        package_id: package.package_id.0.clone(),
        resolved_package_key: package.resolved_package_key.clone(),
        runtime_package: package.runtime_package.clone(),
        inventory_digest: package.inventory_digest.clone(),
        modules: package.modules.iter().map(module_plan).collect(),
    }
}

fn module_plan(module: &ResolvedPythonBridgeModule) -> PythonBridgeModulePlan {
    PythonBridgeModulePlan {
        module: module.module.clone(),
        runtime_module: module.runtime_module.clone(),
        source_path: module.source_path.clone(),
        source_digest: module.source_digest.clone(),
        source: module.source.clone(),
        is_package: module.is_package,
        imports: module
            .imports
            .iter()
            .map(|import| match import {
                ResolvedPythonBridgeImport::SamePackage {
                    module,
                    runtime_module,
                } => PythonBridgeImportPlan::SamePackage {
                    module: module.clone(),
                    runtime_module: runtime_module.clone(),
                },
                ResolvedPythonBridgeImport::ThirdParty { root } => {
                    PythonBridgeImportPlan::ThirdParty { root: root.clone() }
                }
            })
            .collect(),
    }
}

pub(super) fn bridge_authorities_by_module(
    module_packages: &HashMap<String, sifr_package::SifrPackageId>,
    bridges: &ResolvedPythonBridgeGraph,
) -> BTreeMap<String, PythonBridgeTargetAuthority> {
    let packages = bridges
        .packages
        .iter()
        .map(|package| (&package.package_id, package))
        .collect::<HashMap<_, _>>();
    module_packages
        .iter()
        .filter_map(|(module, package_id)| {
            packages.get(package_id).map(|package| {
                (
                    module.clone(),
                    PythonBridgeTargetAuthority {
                        runtime_package: package.runtime_package.clone(),
                        modules: package
                            .modules
                            .iter()
                            .map(|module| module.module.clone())
                            .collect(),
                    },
                )
            })
        })
        .collect()
}

pub(super) fn package_bridge_lowering_options(
    runtime: Option<&PackagePythonRuntime>,
    module_packages: &HashMap<String, sifr_package::SifrPackageId>,
    bridges: &ResolvedPythonBridgeGraph,
) -> LoweringOptions {
    let mut options = runtime.map_or_else(
        LoweringOptions::default,
        PackagePythonRuntime::lowering_options,
    );
    options.python_bridge_authorities = bridge_authorities_by_module(module_packages, bridges);
    options
}

pub(super) fn embedded_bridge_sources(
    packages: &[PythonBridgePackagePlan],
) -> Vec<EmbeddedPythonBridgeSource> {
    let mut sources = BTreeMap::new();
    if !packages.is_empty() {
        insert_synthetic_package(&mut sources, "__sifr_bridge__", "__sifr_bridge__");
    }
    for package in packages {
        insert_synthetic_package(
            &mut sources,
            &package.runtime_package,
            &package.runtime_package,
        );
        for module in &package.modules {
            let mut prefix = package.runtime_package.clone();
            let components = module.module.split('.').collect::<Vec<_>>();
            for component in components.iter().take(components.len().saturating_sub(1)) {
                prefix.push('.');
                prefix.push_str(component);
                insert_synthetic_package(&mut sources, &prefix, &package.runtime_package);
            }
            sources.insert(
                module.runtime_module.clone(),
                EmbeddedPythonBridgeSource {
                    module: module.runtime_module.clone(),
                    source: module.source.clone(),
                    filename: format!("<{}>", module.runtime_module),
                    is_package: module.is_package,
                    package_prefix: package.runtime_package.clone(),
                },
            );
        }
    }
    sources.into_values().collect()
}

fn insert_synthetic_package(
    sources: &mut BTreeMap<String, EmbeddedPythonBridgeSource>,
    module: &str,
    package_prefix: &str,
) {
    sources
        .entry(module.to_string())
        .or_insert_with(|| EmbeddedPythonBridgeSource {
            module: module.to_string(),
            source: String::new(),
            filename: format!("<{module}>"),
            is_package: true,
            package_prefix: package_prefix.to_string(),
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_package::{
        PythonRequirementContribution, PythonRequirementKind, ResolvedPythonBridgeModule,
        ResolvedPythonBridgePackage, SifrPackageId,
    };
    use std::collections::{BTreeMap, HashSet};

    fn base_project() -> GeneratedBinaryProject {
        GeneratedBinaryProject {
            main_rs: "fn main() {}\n".to_string(),
            support_modules: BTreeMap::new(),
            used_stdlib_modules: HashSet::new(),
            required_features: HashSet::new(),
            interop: sifr_codegen::InteropBuildPlan::default(),
            cache_key_fragment: None,
            bridge_modules: Default::default(),
            python_runtime: None,
        }
    }

    #[test]
    fn resolved_bridge_graph_reaches_codegen_plan_and_cache_identity() {
        let package = ResolvedPythonBridgePackage {
            package_id: SifrPackageId("demo@1.0.0#registry".to_string()),
            resolved_package_key: "abc123".to_string(),
            runtime_package: "__sifr_bridge__.p_abc123".to_string(),
            inventory_digest: "inventory-a".to_string(),
            modules: vec![
                ResolvedPythonBridgeModule {
                    module: "adapter".to_string(),
                    runtime_module: "__sifr_bridge__.p_abc123.adapter".to_string(),
                    source_path: "src/python_bridges/adapter.py".to_string(),
                    source_digest: "source-a".to_string(),
                    source: "def value():\n    return 1\n".to_string(),
                    is_package: false,
                    imports: vec![ResolvedPythonBridgeImport::ThirdParty {
                        root: "requests".to_string(),
                    }],
                },
                ResolvedPythonBridgeModule {
                    module: "unused".to_string(),
                    runtime_module: "__sifr_bridge__.p_abc123.unused".to_string(),
                    source_path: "src/python_bridges/unused.py".to_string(),
                    source_digest: "source-unused".to_string(),
                    source: "VALUE = 2\n".to_string(),
                    is_package: false,
                    imports: Vec::new(),
                },
            ],
        };
        let graph = ResolvedPythonBridgeGraph {
            packages: vec![package],
            requirements: vec![PythonRequirementContribution {
                root: "requests".to_string(),
                package_id: SifrPackageId("demo@1.0.0#registry".to_string()),
                kind: PythonRequirementKind::BridgeImport,
                source: "demo:adapter imports requests".to_string(),
            }],
        };

        let generated = apply_package_python_bridge_metadata(base_project(), Some(&graph));
        assert!(generated.interop.python.declarations.is_empty());
        let bridge = &generated.interop.python.bridge_packages[0];

        assert_eq!(bridge.runtime_package, "__sifr_bridge__.p_abc123");
        assert_eq!(
            generated.interop.python.required_import_roots,
            ["requests".to_string()]
        );
        assert!(
            generated
                .interop
                .cache_key_fragment()
                .contains("inventory-a")
        );
        assert!(generated.interop.cache_key_fragment().contains("source-a"));
        let sources = embedded_bridge_sources(&generated.interop.python.bridge_packages);
        assert_eq!(sources[0].module, "__sifr_bridge__");
        assert_eq!(sources[1].module, "__sifr_bridge__.p_abc123");
        assert_eq!(sources[2].filename, "<__sifr_bridge__.p_abc123.adapter>");
        assert_eq!(sources[2].source, "def value():\n    return 1\n");
        assert!(
            sources
                .iter()
                .any(|source| source.module == "__sifr_bridge__.p_abc123.unused")
        );
    }
}
