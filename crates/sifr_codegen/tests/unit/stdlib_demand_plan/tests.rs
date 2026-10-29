use super::*;

#[test]
fn dependency_roots_are_exact_identifier_intersections() {
    let identifiers = rust_source_identifier_names(
        "fn selected() { dependency(); let _: Option<DependencyType> = None; }",
    );
    let definitions =
        rust_source_defined_item_names("fn dependency() {} fn unused() {} struct DependencyType;");
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
            rust: "fn read_text() {} const INF: f64 = f64::INFINITY; fn unused() {}".to_string(),
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
