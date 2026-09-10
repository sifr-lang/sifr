use super::*;

#[test]
fn stdlib_bootstrap_syntax_session_preserves_full_inventory_and_source_order() {
    let compiled = compile_stdlib_uncached().expect("complete bootstrap");
    let session = sifr_codegen::StdlibSyntaxSession::default();
    let mut names = compiled.code.hir_modules.keys().collect::<Vec<_>>();
    names.reverse();
    assert!(names.iter().any(|name| name.starts_with("_sifr.")));
    assert!(names.iter().any(|name| name.starts_with("sifr.")));
    for name in names {
        let module = &compiled.code.hir_modules[name];
        let reference =
            sifr_codegen::generate_stdlib_module_body(module, &compiled.code.emission, name);
        let generated = session.generate_module(module, &compiled.code.emission, name);
        assert_eq!(generated.rust_source, reference.rust_source, "{name}");
        assert_eq!(
            generated.required_features, reference.required_features,
            "{name}"
        );
        assert_eq!(
            generated.used_stdlib_modules, reference.used_stdlib_modules,
            "{name}"
        );
        syn::parse_file(&generated.rust_source).expect("trusted full inventory syntax oracle");
    }
}

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

#[test]
fn recursive_json_structural_contracts_follow_the_shared_project_owner() {
    let compiled = compile_stdlib().expect("real complete stdlib");
    let source = include_str!(
        "../../../../verification/areas/rust_interop/fixtures/structural_bridge_calls/examples/structural_bridge_runtime/src/main.sifr"
    );
    let parsed = parse_module_raw(source, None).expect("original structural fixture");
    let lowered = sifr_lowering::lower_module_with_externals(parsed.suite(), &compiled.defs)
        .expect("original structural fixture lowers");
    let generated = sifr_codegen::generate_rust_multi_with_metadata(
        &[("main", &lowered.module)],
        &compiled.code,
    );
    assert_json_contract_owner(
        &generated.project_union_prelude,
        &generated.rust_files["main"],
    );
    let assembled = format!(
        "{}\n{}",
        generated.project_union_prelude, generated.rust_files["main"]
    );
    let canonical = sifr_codegen::canonicalize_generated_rust_source(&assembled)
        .expect("complete generated assembly");
    assert_eq!(json_contract_count(&canonical), 3, "{canonical}");
    // The real recursive seven-field template, not the historical one-int stub.
    for field in [
        "kind",
        "bool_value",
        "int_value",
        "float_value",
        "str_value",
        "array_items",
        "object_items",
    ] {
        assert!(
            canonical.contains(&format!("RecordField(\"{field}\")")),
            "{field}"
        );
    }

    // Two importing support modules and a root test must not create duplicate
    // impls or silently omit the test-only imported contract.
    let tests = sifr_codegen::generate_rust_test_project_with_metadata(
        &[("alpha", &lowered.module), ("zeta", &lowered.module)],
        &[("test_root", &lowered.module)],
        &compiled.code,
    );
    assert_eq!(json_contract_count(&tests.project_union_prelude), 3);
    for body in tests
        .support_rust_files
        .values()
        .chain(tests.test_rust_files.values())
    {
        assert_eq!(
            json_contract_count(body),
            0,
            "contracts belong to shared nominal"
        );
    }
    let tests_only = sifr_codegen::generate_rust_test_project_with_metadata(
        &[],
        &[("test_root", &lowered.module)],
        &compiled.code,
    );
    assert_eq!(json_contract_count(&tests_only.project_union_prelude), 3);
}

fn assert_json_contract_owner(prelude: &str, body: &str) {
    assert_eq!(json_contract_count(prelude), 3, "{prelude}");
    assert_eq!(json_contract_count(body), 0, "{body}");
}

fn json_contract_count(source: &str) -> usize {
    fn count(items: &[syn::Item]) -> usize {
        items
            .iter()
            .map(|item| match item {
                syn::Item::Mod(module) => {
                    module.content.as_ref().map_or(0, |(_, items)| count(items))
                }
                syn::Item::Impl(implementation) => {
                    let syn::Type::Path(owner) = implementation.self_ty.as_ref() else {
                        return 0;
                    };
                    let json =
                        owner.path.segments.last().is_some_and(|segment| {
                            segment.ident.to_string().ends_with("JsonValue")
                        });
                    let structural = implementation.trait_.as_ref().is_some_and(|(path, _)| {
                        path.segments.last().is_some_and(|segment| {
                            matches!(
                                segment.ident.to_string().as_str(),
                                "StructuralType" | "StructuralConstruct" | "StructuralProject"
                            )
                        })
                    });
                    usize::from(json && structural)
                }
                _ => 0,
            })
            .sum()
    }
    count(
        &syn::parse_file(source)
            .expect("generated Rust syntax")
            .items,
    )
}
