use crate::{generate_rust, generate_rust_multi};
use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirModule, HirParam, MethodKind,
    RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
    RustInteropEffect,
};
use sifr_type_system::{ParamConvention, Type};

#[test]
fn ordinary_class_codegen_skips_structural_impls() {
    let module = module(Vec::new(), vec![payload_class()]);

    let generated = generate_rust(&module);

    assert!(!generated.contains("StructuralType"));
    assert!(!generated.contains("StructuralConstruct"));
    assert!(!generated.contains("StructuralProject"));

    let metadata = crate::generate_rust_with_metadata(&module);
    assert!(!metadata
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::StructuralRuntime));
}

#[test]
fn ordinary_union_codegen_skips_structural_impls_without_demand() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    let module = module(vec![ordinary_function("choose", union)], Vec::new());

    let generated = generate_rust(&module);

    assert!(generated.contains("enum __SifrUnion"));
    assert!(!generated.contains("StructuralType"));
    assert!(!generated.contains("StructuralConstruct"));
    assert!(!generated.contains("StructuralProject"));

    let project = crate::generate_rust_multi_with_metadata(
        &[("main", &module)],
        &crate::StdlibCode::default(),
    );
    let prelude = project.project_union_prelude;
    assert!(prelude.contains("enum __SifrUnion"));
    assert!(!prelude.contains("StructuralType"));
    assert!(!prelude.contains("StructuralConstruct"));
    assert!(!prelude.contains("StructuralProject"));
}

#[test]
fn direct_ordinary_union_gets_structural_impls_when_demanded() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    let module = module(
        vec![structural_function(), ordinary_function("choose", union)],
        Vec::new(),
    );

    let generated = generate_rust(&module);

    assert!(generated.contains("StructuralKind::Union"), "{generated}");
    assert!(generated.contains("ActiveMember"), "{generated}");
    assert!(generated.contains("StructuralConstruct"), "{generated}");
    assert!(generated.contains("StructuralProject"), "{generated}");

    let project = crate::generate_rust_multi_with_metadata(
        &[("main", &module)],
        &crate::StdlibCode::default(),
    );
    let prelude = project.project_union_prelude;
    assert!(prelude.contains("StructuralKind::Union"), "{prelude}");
    assert!(prelude.contains("ActiveMember"), "{prelude}");
}

#[test]
fn project_union_resolves_structural_members_from_their_defining_module() {
    let models = module(Vec::new(), vec![payload_class()]);
    let payload = Type::Class {
        identity: Some("models.Payload".to_string()),
        type_args: Vec::new(),
        name: "Payload".to_string(),
        fields: vec![("value".to_string(), Type::Int)],
        methods: Vec::new(),
        parent_class: None,
    };
    let api = module(
        vec![
            structural_function(),
            ordinary_function("choose", Type::Union(vec![payload, Type::Str])),
        ],
        Vec::new(),
    );

    let project = crate::generate_rust_multi_with_metadata(
        &[("models", &models), ("main", &api)],
        &crate::StdlibCode::default(),
    );
    let prelude = project.project_union_prelude;

    assert!(prelude.contains("crate::models::Payload"), "{prelude}");
    assert!(prelude.contains("StructuralKind::Union"), "{prelude}");
    assert!(prelude.contains("StructuralConstruct"), "{prelude}");
    assert!(prelude.contains("StructuralProject"), "{prelude}");
}

#[test]
fn platform_integer_union_does_not_receive_structural_impls() {
    let union = Type::Union(vec![
        Type::Int,
        Type::FixedInt(sifr_type_system::FixedIntType::USize),
    ]);
    let module = module(
        vec![structural_function(), ordinary_function("choose", union)],
        Vec::new(),
    );

    let project = crate::generate_rust_multi_with_metadata(
        &[("main", &module)],
        &crate::StdlibCode::default(),
    );
    let prelude = project.project_union_prelude;

    assert!(prelude.contains("enum __SifrUnion"));
    assert!(!prelude.contains("StructuralType"));
    assert!(!prelude.contains("StructuralConstruct"));
    assert!(!prelude.contains("StructuralProject"));
}

#[test]
fn project_structural_demand_enables_implicit_classes_across_modules() {
    let models = module(Vec::new(), vec![payload_class()]);
    let api = module(vec![structural_function()], Vec::new());

    let generated = generate_rust_multi(&[("models", &models), ("api", &api)]);
    let models_rust = generated.get("models").expect("models module is generated");

    assert!(models_rust.contains("StructuralType"));
    assert!(models_rust.contains("StructuralConstruct"));
    assert!(models_rust.contains("StructuralProject"));
    assert!(models_rust.contains("fn nominal_identity() -> Option<&'static str>"));
    assert!(models_rust.contains("Some(\"models.Payload\")"));

    let metadata = crate::generate_rust_multi_with_metadata(
        &[("models", &models), ("api", &api)],
        &crate::StdlibCode::default(),
    );
    assert!(metadata
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::StructuralRuntime));
}

#[test]
fn structural_impls_escape_rust_keyword_field_identifiers() {
    let mut keyword = payload_class();
    keyword.name = "KeywordPayload".to_string();
    keyword.fields = vec![("type".to_string(), Type::Int)];
    let models = module(Vec::new(), vec![keyword]);
    let api = module(vec![structural_function()], Vec::new());

    let generated = generate_rust_multi(&[("models", &models), ("api", &api)]);
    let models_rust = generated.get("models").expect("models module is generated");

    assert!(models_rust.contains("let r#type ="));
    assert!(models_rust.contains("Ok(Self { r#type })"));
    assert!(models_rust.contains("&self.r#type"));
    assert!(models_rust.contains("RecordField"));
    assert!(models_rust.contains("\"type\""));
}

#[test]
fn structural_demand_emits_checked_enum_and_ordinary_union_impls() {
    let mut enumeration = payload_class();
    enumeration.name = "Status".to_string();
    enumeration.kind = HirClassKind::Enum;
    enumeration.fields = Vec::new();
    enumeration.enum_variants = vec![
        ("READY".to_string(), Some(4)),
        ("WAITING".to_string(), None),
    ];
    let enum_type = Type::Enum {
        identity: Some("main.Status".to_string()),
        name: "Status".to_string(),
        variants: enumeration.enum_variants.clone(),
    };
    let mut container = payload_class();
    container.name = "SumPayload".to_string();
    container.fields = vec![
        (
            "choice".to_string(),
            Type::Union(vec![Type::Int, Type::Str]),
        ),
        ("status".to_string(), enum_type),
    ];
    let module = module(vec![structural_function()], vec![enumeration, container]);

    let generated = generate_rust(&module);

    assert!(generated.contains("StructuralKind::Union"), "{generated}");
    assert!(generated.contains("ActiveMember"), "{generated}");
    assert!(generated.contains("StructuralKind::Enum"), "{generated}");
    assert!(generated.contains("Self::READY"), "{generated}");
    assert!(generated.contains("Self::WAITING"), "{generated}");
    assert!(generated.contains("::structural::union"), "{generated}");
    assert!(
        generated.contains("ShapeIdentity::from_bytes"),
        "{generated}"
    );
}

#[test]
fn static_program_owners_use_structural_implementation_eligibility() {
    let supported = payload_class();
    let mut unsupported = payload_class();
    unsupported.name = "InheritedPayload".to_string();
    unsupported.parent_class = Some("Base".to_string());
    let mut direct_bytes = payload_class();
    direct_bytes.name = "DirectBytes".to_string();
    direct_bytes.fields = vec![("payload".to_string(), Type::Bytes)];
    let mut nested_bytes = payload_class();
    nested_bytes.name = "NestedBytes".to_string();
    nested_bytes.fields = vec![("payloads".to_string(), Type::List(Box::new(Type::Bytes)))];
    let module = module(
        Vec::new(),
        vec![supported, unsupported, direct_bytes, nested_bytes],
    );

    let owners = crate::structural_static_program_owners(&module);

    assert!(owners.contains("Payload"));
    assert!(!owners.contains("InheritedPayload"));
    assert!(owners.contains("DirectBytes"));
    assert!(!owners.contains("NestedBytes"));
}

fn module(functions: Vec<HirFunction>, classes: Vec<HirClass>) -> HirModule {
    HirModule {
        functions,
        classes,
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    }
}

fn payload_class() -> HirClass {
    HirClass {
        name: "Payload".to_string(),
        identity: None,
        fields: vec![("value".to_string(), Type::Int)],
        field_defaults: Vec::new(),
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

fn ordinary_function(name: &str, parameter_type: Type) -> HirFunction {
    HirFunction {
        name: name.to_string(),
        params: vec![HirParam {
            name: "value".to_string(),
            ty: parameter_type,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::None,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    }
}
