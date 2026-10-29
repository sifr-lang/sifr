use crate::stdlib_filter::{
    filter_canonical_stdlib_ir_to_needed, rust_source_defined_item_names,
    rust_source_identifier_names,
};
use sifr_type_system::stdlib_class_rust_name;
use std::collections::{HashMap, HashSet};

pub(crate) fn plan_demanded_stdlib_sources(
    stdlib_code: &crate::StdlibEmissionView<'_>,
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
    // All source inputs are immutably borrowed for this call. Retain analyses
    // across fixed-point rounds only; a later request builds its own state.
    let mut module_definitions = HashMap::new();
    let mut selected_roots = HashMap::new();
    let mut selected_identifiers = HashMap::new();

    loop {
        let mut selection_changed = false;
        for module_name in module_order {
            let Some(module_source) = stdlib_code.module_rust_code.get(module_name) else {
                continue;
            };
            let source = if whole_module_roots.contains(module_name) {
                if selected.contains_key(module_name) {
                    continue;
                }
                module_source.rust.clone()
            } else {
                let mut roots = demands.get(module_name).cloned().unwrap_or_default();
                let definitions = module_definitions
                    .entry(module_name.clone())
                    .or_insert_with(|| rust_source_defined_item_names(&module_source.rust));
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
                if selected_roots.get(module_name) == Some(&roots) {
                    continue;
                }
                selected_roots.insert(module_name.clone(), roots.clone());
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
                selected_identifiers.remove(module_name);
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
                let definitions = module_definitions
                    .entry(dependency.clone())
                    .or_insert_with(|| rust_source_defined_item_names(&dependency_source.rust));
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
                for definition in definitions.iter() {
                    let canonical = stdlib_class_rust_name(&dependency_source.module, definition);
                    if (owner_is_whole_root
                        || owner_roots.contains(definition)
                        || owner_roots.contains(&canonical))
                        && dependency_demands.insert(definition.clone())
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
            let identifiers = selected_identifiers
                .entry(owner_module.clone())
                .or_insert_with(|| rust_source_identifier_names(source));
            let Some(dependencies) = stdlib_code.transitive_deps.get(owner_module) else {
                continue;
            };
            for dependency in dependencies {
                let Some(dependency_source) = stdlib_code.module_rust_code.get(dependency) else {
                    continue;
                };
                let definitions = module_definitions
                    .entry(dependency.clone())
                    .or_insert_with(|| rust_source_defined_item_names(&dependency_source.rust));
                let dependency_demands = demands.entry(dependency.clone()).or_default();
                for definition in definitions.iter() {
                    let canonical = stdlib_class_rust_name(&dependency_source.module, definition);
                    if (identifiers.contains(definition) || identifiers.contains(&canonical))
                        && dependency_demands.insert(definition.clone())
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
#[path = "../tests/unit/stdlib_demand_plan/tests.rs"]
mod tests;
