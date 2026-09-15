use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirImport, HirModule, MethodKind,
    RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
    RustInteropEffect,
};
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

const STRUCTURAL_IMPL_TARGET: &str = "impl ::sifr_runtime::interop::structural::StructuralType";

#[test]
fn unused_import_only_stdlib_structural_impls_are_pruned_after_relocation() {
    let mut alpha = module(Vec::new(), Vec::new());
    alpha.imports.push(json_import());
    let mut zeta = module(Vec::new(), Vec::new());
    zeta.imports.push(json_import());
    let main = module(vec![structural_function()], Vec::new());

    let generated = crate::generate_rust_multi_with_metadata(
        &[("zeta", &zeta), ("main", &main), ("alpha", &alpha)],
        &json_stdlib(),
    );
    let count = generated
        .rust_files
        .values()
        .map(|source| source.matches(STRUCTURAL_IMPL_TARGET).count())
        .sum::<usize>();

    assert_eq!(count, 0, "{:?}", generated.rust_files);
    assert!(!generated.rust_files["alpha"].contains(STRUCTURAL_IMPL_TARGET));
    assert!(!generated.rust_files["zeta"].contains(STRUCTURAL_IMPL_TARGET));
}

#[test]
fn project_identity_with_stdlib_prefix_gets_no_origin_bypass() {
    let mut payload = payload_class();
    payload.identity = Some("sifr.fake.Payload".to_string());
    let module = module(vec![structural_function()], vec![payload]);
    let supported = HashSet::new();

    let generated = crate::generate_rust_with_stdlib_for_module_with_project_policy(
        &module,
        &crate::StdlibCode::default().emission_view(),
        Some("main"),
        Some("main"),
        true,
        None,
        None,
        None,
        Some(&supported),
        crate::ProjectStructuralLayoutLocation::Local,
        None,
        crate::SupportEmission::Inline,
    )
    .rust_source;

    assert!(
        !generated.contains("StructuralType for Payload"),
        "{generated}"
    );
}

fn json_import() -> HirImport {
    HirImport {
        module: "sifr.json".to_string(),
        names: vec!["JsonValue".to_string()],
        aliases: Vec::new(),
    }
}

fn json_stdlib() -> crate::StdlibCode {
    let mut template = payload_class();
    template.name = "JsonValue".to_string();
    template.identity = Some("sifr.json.JsonValue".to_string());
    let mut stdlib = crate::StdlibCode::default();
    stdlib.module_class_templates.insert(
        "sifr.json".to_string(),
        HashMap::from([("JsonValue".to_string(), template)]),
    );
    stdlib.module_rust_code.insert(
        "sifr.json".to_string(),
        crate::StdlibRustSource {
            module: "sifr.json".to_string(),
            source_path: "stdlib/sifr/json.sifr".to_string(),
            source_sha256: "fixture".to_string(),
            nominal_types: HashSet::from(["JsonValue".to_string()]),
            rust: "struct JsonValue { value: SifrInt }".to_string(),
        },
    );
    stdlib
}

fn module(functions: Vec<HirFunction>, classes: Vec<HirClass>) -> HirModule {
    HirModule {
        functions,
        classes,
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}

fn payload_class() -> HirClass {
    HirClass {
        name: "Payload".to_string(),
        identity: None,
        fields: vec![("value".to_string(), Type::Int)],
        field_defaults: Vec::new(),
        field_default_identities: Vec::new(),
        declaration_metadata: Vec::new(),
        methods: Vec::new(),
        is_hashable: false,
        is_error_type: false,
        kind: HirClassKind::Regular,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: None,
        parent_type: None,
        type_params: Vec::new(),
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    }
}

fn structural_function() -> HirFunction {
    HirFunction {
        name: "construct".to_string(),
        params: Vec::new(),
        return_type: Type::None,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![RustInteropDeclaration {
            kind: RustInteropDecoratorKind::Structural,
            target: None,
            arguments: Vec::new(),
            span: Default::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
            consumes_receiver: false,
        }],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    }
}

#[test]
fn imported_stdlib_structural_contracts_include_transitive_field_types_only() {
    let mut stdlib = json_stdlib();
    let mut child = payload_class();
    child.name = "Child".to_string();
    child.identity = Some("sifr.json.Child".to_string());
    let child_type = Type::Class {
        name: child.name.clone(),
        identity: child.identity.clone(),
        type_args: Vec::new(),
        fields: child.fields.clone(),
        methods: Vec::new(),
        parent_class: None,
    };
    let templates = stdlib
        .module_class_templates
        .get_mut("sifr.json")
        .expect("json");
    templates.get_mut("JsonValue").expect("root").fields =
        vec![("value".to_string(), Type::List(Box::new(child_type)))];
    templates.insert("Child".to_string(), child.clone());
    let mut other = child.clone();
    other.identity = Some("sifr.other.Child".to_string());
    stdlib.module_class_templates.insert(
        "sifr.other".to_string(),
        HashMap::from([("Child".to_string(), other)]),
    );
    let source = stdlib
        .module_rust_code
        .get_mut("sifr.json")
        .expect("source");
    source.nominal_types.insert("Child".to_string());
    source.rust =
        "struct JsonValue { value: Vec<Child> } struct Child { value: SifrInt }".to_string();
    let mut consumer = module(vec![structural_function()], Vec::new());
    consumer.imports.push(json_import());
    let view = stdlib.emission_view();
    let selected = crate::structural_impl_codegen::imported_stdlib_classes(&consumer, &view);
    assert_eq!(
        selected
            .iter()
            .filter_map(|class| class.identity.as_deref())
            .collect::<Vec<_>>(),
        ["sifr.json.Child", "sifr.json.JsonValue"]
    );
    let mut emitter = crate::RustEmitter::new();
    emitter.structural_interop_enabled = true;
    emitter.emit_imported_stdlib_structural_impls(&consumer, &view);
    let all = crate::Renderer::new().render_file(&crate::RustFile {
        items: emitter.body_items,
    });
    let compact = all.split_whitespace().collect::<String>();
    let child_name = sifr_type_system::class_rust_name(Some("sifr.json.Child"), "Child");
    for contract in ["StructuralType", "StructuralConstruct", "StructuralProject"] {
        assert_eq!(
            compact
                .matches(&format!("structural::{contract}for{child_name}"))
                .count(),
            1,
            "{all}"
        );
    }
}

#[test]
fn import_only_structural_registration_preserves_opaque_runtime_ownership() {
    let mut opaque = payload_class();
    opaque.name = "Object".to_string();
    opaque.identity = Some("_sifr.python.Object".to_string());
    opaque.fields.clear();
    let mut declaration = structural_function().rust_interop.remove(0);
    declaration.kind = RustInteropDecoratorKind::Opaque;
    opaque.rust_interop.push(declaration);
    let mut stdlib = json_stdlib();
    stdlib.module_class_templates.insert(
        "_sifr.python".to_string(),
        HashMap::from([("Object".to_string(), opaque)]),
    );
    let mut consumer = module(vec![structural_function()], Vec::new());
    consumer.imports.push(json_import());
    consumer.imports.push(HirImport {
        module: "_sifr.python".to_string(),
        names: vec!["Object".to_string()],
        aliases: Vec::new(),
    });
    let mut plan = crate::project_stdlib_nominals::ProjectStdlibNominalPlan::empty();
    crate::project_stdlib_nominals::register_imported_structural_nominals(
        &[("consumer", &consumer)],
        &stdlib.emission_view(),
        &mut plan,
    );
    assert!(plan.registry.rust_paths.contains_key("sifr.json.JsonValue"));
    assert!(!plan.registry.rust_paths.contains_key("_sifr.python.Object"));
}
