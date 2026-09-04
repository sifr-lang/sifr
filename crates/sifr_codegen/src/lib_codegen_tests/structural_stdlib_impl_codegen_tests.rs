use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirImport, HirModule, MethodKind,
    RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
    RustInteropEffect,
};
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

const STRUCTURAL_IMPL_TARGET: &str = "impl ::sifr_runtime::interop::structural::StructuralType";

#[test]
fn multi_module_stdlib_structural_impl_is_emitted_for_each_imported_nominal() {
    let mut alpha = module(Vec::new(), Vec::new());
    alpha.imports.push(json_import());
    let mut zeta = module(Vec::new(), Vec::new());
    zeta.imports.push(json_import());
    let main = module(vec![structural_function()], Vec::new());

    let generated = crate::generate_rust_multi_with_metadata(
        &[("zeta", &zeta), ("main", &main), ("alpha", &alpha)],
        &json_stdlib(),
    )
    .expect("project generation should succeed");
    let count = generated
        .rust_files
        .values()
        .map(|source| source.matches(STRUCTURAL_IMPL_TARGET).count())
        .sum::<usize>();

    assert_eq!(count, 2, "{:?}", generated.rust_files);
    assert!(generated.rust_files["alpha"].contains(STRUCTURAL_IMPL_TARGET));
    assert!(generated.rust_files["zeta"].contains(STRUCTURAL_IMPL_TARGET));
}

#[test]
fn project_identity_with_stdlib_prefix_gets_no_origin_bypass() {
    let mut payload = payload_class();
    payload.identity = Some("sifr.fake.Payload".to_string());
    let module = module(vec![structural_function()], vec![payload]);
    let supported = HashSet::new();

    let generated = crate::generate_rust_with_stdlib_for_module_with_project_policy(
        &module,
        &crate::StdlibCode::default(),
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
