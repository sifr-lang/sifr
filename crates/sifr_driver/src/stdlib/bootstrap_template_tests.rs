use super::*;

#[test]
fn stdlib_bootstrap_borrowed_emission_preserves_imports_and_generic_templates() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let source = "from sifr.collections import deque as Queue\nfrom sifr.calendar import isleap as leap\ndef example() -> bool:\n    values: Queue[int] = Queue([1, 2])\n    return leap(len(values))\n";
    let parsed = parse_module_raw(source, None).expect("parse imported generic");
    let lowered = lower_module_sysroot_public_stdlib_with_externals(parsed.suite(), &compiled.defs)
        .expect("lower borrowed generic and alias signatures");
    let template = std::sync::Arc::clone(
        compiled
            .code
            .generic_class_templates
            .get("deque")
            .expect("generic deque template"),
    );
    let before = format!("{:?}", lowered.module);
    let generated = sifr_codegen::generate_stdlib_module_body(
        &lowered.module,
        &compiled.code.emission,
        "sifr.borrowed_example",
    );
    assert!(syn::parse_file(&generated.rust_source).is_ok());
    assert!(!generated.rust_source.contains("// --- stdlib:"));
    assert!(generated.used_stdlib_modules.contains("sifr.calendar"));
    assert!(!generated.rust_source.is_empty());
    assert_eq!(format!("{:?}", lowered.module), before);
    assert!(std::sync::Arc::ptr_eq(
        &template,
        &compiled.code.generic_class_templates["deque"],
    ));
    // The same owner still supplies full application support after bootstrap emission.
    let application = sifr_codegen::generate_rust_with_stdlib(&lowered.module, &compiled.code);
    assert!(application.rust_source.contains("// --- stdlib:"));
    assert!(
        application
            .interop
            .stdlib_demand
            .declarations
            .iter()
            .any(|declaration| declaration.module_name.as_deref() == Some("_sifr.calendar"),)
    );
    assert!(syn::parse_file(&application.rust_source).is_ok());
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
    assert!(
        json_value
            .methods
            .iter()
            .all(|method| method.body.is_empty())
    );
    assert!(
        json_value
            .operator_impls
            .iter()
            .all(|(_, method)| method.body.is_empty())
    );
}
