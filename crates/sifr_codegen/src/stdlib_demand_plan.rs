use crate::StdlibCode;
use crate::stdlib_filter::{
    filter_canonical_stdlib_ir_to_needed, rust_source_defined_item_names,
    rust_source_identifier_names,
};
use sifr_type_system::stdlib_class_rust_name;
use std::collections::{HashMap, HashSet};

pub(crate) fn plan_demanded_stdlib_sources(
    stdlib_code: &StdlibCode,
    module_order: &[String],
    directly_used_modules: &HashSet<String>,
    imported_names: &HashMap<String, HashSet<String>>,
    suppressed_union_definitions: &HashSet<String>,
) -> HashMap<String, String> {
    let mut demands = imported_names
        .iter()
        .filter(|(module, _)| directly_used_modules.contains(*module))
        .map(|(module, names)| (module.clone(), names.clone()))
        .collect::<HashMap<_, _>>();
    let whole_module_roots = directly_used_modules
        .iter()
        .filter(|module| !imported_names.contains_key(*module))
        .cloned()
        .collect::<HashSet<_>>();
    let mut selected = HashMap::new();

    loop {
        let mut selection_changed = false;
        for module_name in module_order {
            let Some(module_source) = stdlib_code.module_rust_code.get(module_name) else {
                continue;
            };
            let source = if whole_module_roots.contains(module_name) {
                module_source.rust.clone()
            } else {
                let mut roots = demands.get(module_name).cloned().unwrap_or_default();
                let definitions = rust_source_defined_item_names(&module_source.rust);
                roots.extend(
                    suppressed_union_definitions
                        .iter()
                        .filter(|name| definitions.contains(*name))
                        .cloned(),
                );
                if let Some(constants) = stdlib_code.module_constants.get(module_name) {
                    let imported = roots.clone();
                    for name in imported {
                        if let Some((_, rust_reference)) = constants.get(&name) {
                            roots.insert(
                                rust_reference
                                    .strip_suffix("()")
                                    .unwrap_or(rust_reference)
                                    .to_string(),
                            );
                        }
                    }
                }
                if roots.is_empty() {
                    String::new()
                } else {
                    filter_canonical_stdlib_ir_to_needed(
                        &module_source.rust,
                        &roots,
                        &module_source.module,
                        &module_source.nominal_types,
                    )
                }
            };
            if selected.get(module_name) != Some(&source) {
                selected.insert(module_name.clone(), source);
                selection_changed = true;
            }
        }

        let mut demand_changed = false;
        for owner_module in module_order {
            let Some(dependencies) = stdlib_code.transitive_deps.get(owner_module) else {
                continue;
            };
            let owner_roots = demands.get(owner_module).cloned().unwrap_or_default();
            let owner_is_whole_root = whole_module_roots.contains(owner_module);
            if owner_roots.is_empty() && !owner_is_whole_root {
                continue;
            }
            for dependency in dependencies {
                let Some(dependency_source) = stdlib_code.module_rust_code.get(dependency) else {
                    continue;
                };
                let definitions = rust_source_defined_item_names(&dependency_source.rust);
                let dependency_demands = demands.entry(dependency.clone()).or_default();
                if let Some(constants) = stdlib_code.module_constants.get(dependency) {
                    for root in &owner_roots {
                        if let Some((_, rust_reference)) = constants.get(root) {
                            let definition =
                                rust_reference.strip_suffix("()").unwrap_or(rust_reference);
                            if definitions.contains(definition)
                                && dependency_demands.insert(definition.to_string())
                            {
                                demand_changed = true;
                            }
                        }
                    }
                }
                for definition in definitions {
                    let canonical = stdlib_class_rust_name(&dependency_source.module, &definition);
                    if (owner_is_whole_root
                        || owner_roots.contains(&definition)
                        || owner_roots.contains(&canonical))
                        && dependency_demands.insert(definition)
                    {
                        demand_changed = true;
                    }
                }
            }
        }
        for (owner_module, source) in &selected {
            if source.is_empty() {
                continue;
            }
            let identifiers = rust_source_identifier_names(source);
            let Some(dependencies) = stdlib_code.transitive_deps.get(owner_module) else {
                continue;
            };
            for dependency in dependencies {
                let Some(dependency_source) = stdlib_code.module_rust_code.get(dependency) else {
                    continue;
                };
                let definitions = rust_source_defined_item_names(&dependency_source.rust);
                let dependency_demands = demands.entry(dependency.clone()).or_default();
                for definition in definitions {
                    let canonical = stdlib_class_rust_name(&dependency_source.module, &definition);
                    if (identifiers.contains(&definition) || identifiers.contains(&canonical))
                        && dependency_demands.insert(definition)
                    {
                        demand_changed = true;
                    }
                }
            }
        }

        if !selection_changed && !demand_changed {
            break;
        }
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_roots_are_exact_identifier_intersections() {
        let identifiers = rust_source_identifier_names(
            "fn selected() { dependency(); let _: Option<DependencyType> = None; }",
        );
        let definitions = rust_source_defined_item_names(
            "fn dependency() {} fn unused() {} struct DependencyType;",
        );
        let demanded = definitions
            .intersection(&identifiers)
            .cloned()
            .collect::<HashSet<_>>();

        assert_eq!(
            demanded,
            HashSet::from(["DependencyType".to_string(), "dependency".to_string()])
        );
    }

    #[test]
    fn public_import_roots_seed_matching_transitive_private_definitions() {
        let mut stdlib = StdlibCode::default();
        stdlib.transitive_deps.insert(
            "sifr.io".to_string(),
            HashSet::from(["_sifr.fs".to_string()]),
        );
        stdlib.module_rust_code.insert(
            "_sifr.fs".to_string(),
            crate::StdlibRustSource {
                module: "_sifr.fs".to_string(),
                source_path: "stdlib/_sifr/fs.sifr".to_string(),
                source_sha256: "test".to_string(),
                nominal_types: HashSet::new(),
                rust: "fn read_text() {} const INF: f64 = f64::INFINITY; fn unused() {}"
                    .to_string(),
            },
        );
        stdlib.module_constants.insert(
            "_sifr.fs".to_string(),
            HashMap::from([(
                "inf".to_string(),
                (sifr_type_system::Type::Float, "INF".to_string()),
            )]),
        );

        let selected = plan_demanded_stdlib_sources(
            &stdlib,
            &["_sifr.fs".to_string(), "sifr.io".to_string()],
            &HashSet::from(["sifr.io".to_string()]),
            &HashMap::from([(
                "sifr.io".to_string(),
                HashSet::from(["read_text".to_string(), "inf".to_string()]),
            )]),
            &HashSet::new(),
        );

        let private = selected.get("_sifr.fs").expect("private source planned");
        assert!(private.contains("fn read_text"));
        assert!(private.contains("const INF"));
        assert!(!private.contains("fn unused"));
    }
}
