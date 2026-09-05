use sifr_stdlib_manifest::{
    StdlibFeature, feature_for_codegen_requirement, features_for_stdlib_module,
    planned_sifr_stdlib_features,
};
use std::collections::HashSet;
use syn::visit::{self, Visit};

/// Remove dependency requests whose generated Rust owners were pruned.
///
/// Project assembly can discard methods and whole support modules after module-level
/// demand has been merged. The manifest inputs must describe the final assembled
/// crate, not that conservative intermediate demand.
pub(crate) fn retain_generated_dependency_metadata<'source>(
    sources: impl IntoIterator<Item = &'source str>,
    used_stdlib_modules: &mut HashSet<String>,
    required_features: &mut HashSet<StdlibFeature>,
) -> Result<(), String> {
    let mut paths = GeneratedDependencyPaths::default();
    for source in sources {
        let file = syn::parse_file(source)
            .map_err(|error| format!("failed to parse generated dependency owner: {error}"))?;
        paths.visit_file(&file);
    }

    used_stdlib_modules.retain(|module| paths.retains_stdlib_module(module));

    let mut retained_features = paths.direct_features();
    for module in used_stdlib_modules.iter() {
        retained_features.extend(features_for_stdlib_module(module));
    }
    *required_features = retained_features;
    Ok(())
}

#[derive(Default)]
struct GeneratedDependencyPaths {
    roots: HashSet<String>,
    sifr_stdlib_namespaces: HashSet<String>,
    sifr_runtime_namespaces: HashSet<String>,
}

impl GeneratedDependencyPaths {
    fn retains_stdlib_module(&self, module: &str) -> bool {
        let module_set = HashSet::from([module.to_string()]);
        let module_features = planned_sifr_stdlib_features(&module_set, &HashSet::new());
        let module_suffix = module
            .rsplit_once('.')
            .map_or(module, |(_, suffix)| suffix)
            .trim_start_matches('_');
        module_features.iter().any(|feature| {
            self.sifr_stdlib_namespaces
                .contains(&feature.replace('-', "_"))
        }) || self.sifr_stdlib_namespaces.contains(module_suffix)
    }

    fn direct_features(&self) -> HashSet<StdlibFeature> {
        let mut features = self
            .roots
            .iter()
            .filter_map(|root| {
                feature_for_codegen_requirement(root)
                    .or_else(|| feature_for_codegen_requirement(&root.replace('_', "-")))
            })
            .collect::<HashSet<_>>();
        features.extend(
            self.sifr_stdlib_namespaces
                .iter()
                .filter_map(|namespace| feature_for_codegen_requirement(namespace)),
        );
        if self.roots.contains("sifr_runtime")
            || self.roots.contains("SifrInt")
            || self.roots.contains("SifrRange")
        {
            features.insert(StdlibFeature::SifrRuntime);
        }
        if self.sifr_runtime_namespaces.contains("structural") {
            features.insert(StdlibFeature::StructuralRuntime);
        }
        if self.roots.contains("pyo3")
            || self.sifr_runtime_namespaces.contains("python")
            || self.sifr_stdlib_namespaces.contains("python")
        {
            features.insert(StdlibFeature::PythonRuntime);
        }
        features
    }
}

impl<'ast> Visit<'ast> for GeneratedDependencyPaths {
    fn visit_item_use(&mut self, item_use: &'ast syn::ItemUse) {
        collect_use_roots(&item_use.tree, &mut self.roots);
        visit::visit_item_use(self, item_use);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(root) = path.segments.first() {
            let root = root.ident.to_string();
            self.roots.insert(root.clone());
            if root == "sifr_stdlib" {
                if let Some(namespace) = path.segments.iter().nth(1) {
                    self.sifr_stdlib_namespaces
                        .insert(namespace.ident.to_string());
                }
            } else if root == "sifr_runtime" {
                self.sifr_runtime_namespaces.extend(
                    path.segments
                        .iter()
                        .skip(1)
                        .map(|segment| segment.ident.to_string()),
                );
            }
        }
        visit::visit_path(self, path);
    }
}

fn collect_use_roots(tree: &syn::UseTree, roots: &mut HashSet<String>) {
    match tree {
        syn::UseTree::Path(path) => {
            roots.insert(path.ident.to_string());
        }
        syn::UseTree::Name(name) => {
            roots.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            roots.insert(rename.ident.to_string());
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_use_roots(item, roots);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_pruned_stdlib_and_runtime_metadata() {
        let mut modules = HashSet::from(["sifr.pathlib".to_string(), "_sifr.fs".to_string()]);
        let mut features = HashSet::from([StdlibFeature::Fs, StdlibFeature::SifrRuntime]);

        retain_generated_dependency_metadata(
            ["struct Path { value: String }"],
            &mut modules,
            &mut features,
        )
        .expect("retained generated Rust should parse");

        assert!(modules.is_empty());
        assert!(features.is_empty());
    }

    #[test]
    fn retains_dependencies_referenced_by_final_generated_paths() {
        let mut modules = HashSet::from(["sifr.io".to_string(), "_sifr.fs".to_string()]);
        let mut features = HashSet::from([
            StdlibFeature::Fs,
            StdlibFeature::SifrRuntime,
            StdlibFeature::Tokio,
        ]);

        retain_generated_dependency_metadata(
            ["fn read(path: &str) { let _ = ::sifr_stdlib::fs::read_text(path); let _ = ::sifr_runtime::SifrInt::from(1); }"],
            &mut modules,
            &mut features,
        )
        .expect("retained generated Rust should parse");

        assert_eq!(
            modules,
            HashSet::from(["sifr.io".to_string(), "_sifr.fs".to_string()])
        );
        assert_eq!(
            features,
            HashSet::from([StdlibFeature::Fs, StdlibFeature::SifrRuntime])
        );
    }

    #[test]
    fn reconstructs_direct_crate_features_from_final_generated_paths() {
        let mut modules = HashSet::new();
        let mut features = HashSet::from([StdlibFeature::BigDecimal]);

        retain_generated_dependency_metadata(
            [r#"
                use ::bigdecimal::BigDecimal;
                use ::num_bigint::BigInt;
                use ::rust_decimal::Decimal;
            "#],
            &mut modules,
            &mut features,
        )
        .expect("retained generated Rust should parse");

        assert_eq!(
            features,
            HashSet::from([
                StdlibFeature::BigDecimal,
                StdlibFeature::NumBigint,
                StdlibFeature::RustDecimal,
            ])
        );
    }

    #[test]
    fn retains_hyphenated_stdlib_features_for_rust_module_paths() {
        let mut modules = HashSet::from([
            "sifr.runtime".to_string(),
            "_sifr.runtime".to_string(),
            "sifr.json".to_string(),
        ]);
        let mut features = HashSet::new();

        retain_generated_dependency_metadata(
            ["fn emit() { let _ = ::sifr_stdlib::runtime_observability::emit_diagnostic(\"info\", \"demo\", \"event\", \"message\"); }"],
            &mut modules,
            &mut features,
        )
        .expect("observability bridge should parse");

        assert_eq!(
            modules,
            HashSet::from(["sifr.runtime".to_string(), "_sifr.runtime".to_string()])
        );
        assert_eq!(
            planned_sifr_stdlib_features(&modules, &features),
            std::collections::BTreeSet::from(["runtime-observability"])
        );

        retain_generated_dependency_metadata(["fn main() {}"], &mut modules, &mut features)
            .expect("empty program should parse");
        assert!(modules.is_empty());
        assert!(features.is_empty());
    }
}
