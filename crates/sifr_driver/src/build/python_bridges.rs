use super::project_codegen::GeneratedBinaryProject;
use sifr_codegen::{PythonBridgeImportPlan, PythonBridgeModulePlan, PythonBridgePackagePlan};
use sifr_package::{
    ResolvedPythonBridgeGraph, ResolvedPythonBridgeImport, ResolvedPythonBridgeModule,
    ResolvedPythonBridgePackage,
};

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
            modules: vec![ResolvedPythonBridgeModule {
                module: "adapter".to_string(),
                runtime_module: "__sifr_bridge__.p_abc123.adapter".to_string(),
                source_path: "src/python_bridges/adapter.py".to_string(),
                source_digest: "source-a".to_string(),
                is_package: false,
                imports: vec![ResolvedPythonBridgeImport::ThirdParty {
                    root: "requests".to_string(),
                }],
            }],
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
        let bridge = &generated.interop.python.bridge_packages[0];

        assert_eq!(bridge.runtime_package, "__sifr_bridge__.p_abc123");
        assert_eq!(
            generated.interop.python.required_import_roots,
            ["requests".to_string()]
        );
        assert!(generated
            .interop
            .cache_key_fragment()
            .contains("inventory-a"));
        assert!(generated.interop.cache_key_fragment().contains("source-a"));
    }
}
