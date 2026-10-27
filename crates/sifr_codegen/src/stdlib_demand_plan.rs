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
        let mut stdlib = crate::StdlibCode::default();
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
            &crate::StdlibEmissionView::new(&stdlib),
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

    fn source(module: &str, rust: &str) -> crate::StdlibRustSource {
        crate::StdlibRustSource {
            module: module.to_string(),
            source_path: format!("{module}.sifr"),
            source_sha256: "test".to_string(),
            nominal_types: HashSet::new(),
            rust: rust.to_string(),
        }
    }

    #[test]
    fn growing_demands_cross_multiple_rounds_and_a_dependency_cycle() {
        let mut stdlib = crate::StdlibCode::default();
        for (module, rust, dependency) in [
            (
                "sifr.a",
                "fn start() { middle(); } fn cycle_leaf() {} fn unused_a() {}",
                "sifr.b",
            ),
            (
                "sifr.b",
                "fn middle() { finish(); } fn unused_b() {}",
                "sifr.c",
            ),
            (
                "sifr.c",
                "fn finish() { cycle_leaf(); } fn unused_c() {}",
                "sifr.a",
            ),
        ] {
            stdlib
                .module_rust_code
                .insert(module.into(), source(module, rust));
            stdlib
                .transitive_deps
                .insert(module.into(), HashSet::from([dependency.into()]));
        }
        let order = ["sifr.c", "sifr.b", "sifr.a"].map(String::from);
        let direct = HashSet::from(["sifr.a".to_string()]);
        let view = crate::StdlibEmissionView::new(&stdlib);
        let selected = plan_demanded_stdlib_sources(
            &view,
            &order,
            &direct,
            &HashMap::from([("sifr.a".into(), HashSet::from(["start".into()]))]),
            &HashSet::new(),
        );
        for (module, required) in [
            ("sifr.a", vec!["start", "cycle_leaf"]),
            ("sifr.b", vec!["middle"]),
            ("sifr.c", vec!["finish"]),
        ] {
            let names = rust_source_defined_item_names(&selected[module]);
            assert_eq!(names, required.into_iter().map(String::from).collect());
        }

        // An independent request must not inherit roots from the preceding closure.
        let next = plan_demanded_stdlib_sources(
            &view,
            &order,
            &direct,
            &HashMap::from([("sifr.a".into(), HashSet::from(["cycle_leaf".into()]))]),
            &HashSet::new(),
        );
        assert_eq!(
            rust_source_defined_item_names(&next["sifr.a"]),
            HashSet::from(["cycle_leaf".into()])
        );
        assert!(next["sifr.b"].is_empty());
        assert!(next["sifr.c"].is_empty());
    }

    #[test]
    fn canonical_references_constants_and_suppressed_types_survive_selection() {
        let mut stdlib = crate::StdlibCode::default();
        let module = "sifr.dependency";
        let canonical = stdlib_class_rust_name(module, "Value");
        let rust = "struct Value; enum SharedUnion { Empty } const LIMIT: i64 = 3; fn unused() {}";
        let mut dependency = source(module, rust);
        dependency.nominal_types.insert("Value".into());
        stdlib.module_rust_code.insert(module.into(), dependency);
        stdlib.module_rust_code.insert(
            "sifr.owner".into(),
            source(
                "sifr.owner",
                &format!("fn selected() -> {canonical} {{ {canonical} }} fn unused_owner() {{}}"),
            ),
        );
        stdlib
            .transitive_deps
            .insert("sifr.owner".into(), HashSet::from([module.into()]));
        stdlib.module_constants.insert(
            module.into(),
            HashMap::from([(
                "limit".into(),
                (sifr_type_system::Type::Int, "LIMIT".into()),
            )]),
        );
        let selected = plan_demanded_stdlib_sources(
            &crate::StdlibEmissionView::new(&stdlib),
            &[module.into(), "sifr.owner".into()],
            &HashSet::from(["sifr.owner".into()]),
            &HashMap::from([(
                "sifr.owner".into(),
                HashSet::from(["selected".into(), "limit".into()]),
            )]),
            &HashSet::from(["SharedUnion".into()]),
        );
        assert_eq!(
            rust_source_defined_item_names(&selected[module]),
            HashSet::from(["Value".into(), "SharedUnion".into(), "LIMIT".into()])
        );
        assert_eq!(
            rust_source_defined_item_names(&selected["sifr.owner"]),
            HashSet::from(["selected".into()])
        );
    }

    #[test]
    fn whole_module_roots_keep_all_items_while_propagating_dependencies() {
        let mut stdlib = crate::StdlibCode::default();
        stdlib.module_rust_code.insert(
            "sifr.root".into(),
            source("sifr.root", "fn first() {} fn second() {}"),
        );
        stdlib
            .module_rust_code
            .insert("sifr.dep".into(), source("sifr.dep", "fn dependency() {}"));
        stdlib
            .transitive_deps
            .insert("sifr.root".into(), HashSet::from(["sifr.dep".into()]));
        let selected = plan_demanded_stdlib_sources(
            &crate::StdlibEmissionView::new(&stdlib),
            &["sifr.dep".into(), "sifr.root".into()],
            &HashSet::from(["sifr.root".into()]),
            &HashMap::new(),
            &HashSet::new(),
        );
        assert_eq!(
            selected["sifr.root"],
            stdlib.module_rust_code["sifr.root"].rust
        );
        assert!(rust_source_defined_item_names(&selected["sifr.dep"]).contains("dependency"));
    }
}
