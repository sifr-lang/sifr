use super::*;

#[test]
fn stdlib_codegen_selects_only_explicitly_imported_class_templates() {
    let available = HashMap::from([
        (
            "sifr.alpha".to_string(),
            HashMap::from([("First".to_string(), 1), ("Second".to_string(), 2)]),
        ),
        (
            "sifr.unused".to_string(),
            HashMap::from([("Unused".to_string(), 3)]),
        ),
    ]);
    let imports = vec![
        sifr_ir::HirImport {
            module: "sifr.alpha".to_string(),
            names: vec!["Second".to_string(), "Missing".to_string()],
            aliases: vec![("Second".to_string(), "Renamed".to_string())],
        },
        sifr_ir::HirImport {
            module: "sifr.alpha".to_string(),
            names: vec!["Second".to_string()],
            aliases: Vec::new(),
        },
    ];

    let selected = select_imported_class_templates(&imports, &available);

    assert_eq!(
        selected,
        HashMap::from([(
            "sifr.alpha".to_string(),
            HashMap::from([("Second".to_string(), 2)]),
        )])
    );
}

#[test]
fn stdlib_structural_templates_retain_signatures_without_bodies() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let json_value = compiled
        .code
        .module_class_templates
        .get("sifr.json")
        .and_then(|classes| classes.get("JsonValue"))
        .expect("sifr.json.JsonValue should retain a structural template");

    assert_eq!(json_value.identity.as_deref(), Some("sifr.json.JsonValue"));
    assert!(!json_value.methods.is_empty());
    assert!(json_value
        .methods
        .iter()
        .all(|method| method.body.is_empty()));
    assert!(json_value
        .operator_impls
        .iter()
        .all(|(_, method)| method.body.is_empty()));
}
