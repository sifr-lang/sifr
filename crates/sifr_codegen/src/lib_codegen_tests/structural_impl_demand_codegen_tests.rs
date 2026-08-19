use crate::{generate_rust, generate_rust_multi};
use sifr_ir::{
    DeclarationMetadataTargetKind, HirClass, HirClassKind, HirExpr, HirFunction, HirModule,
    HirParam, MethodKind, RustInteropAbiRequirements, RustInteropDeclaration,
    RustInteropDecoratorKind, RustInteropEffect, TypedDeclarationMetadata,
};
use sifr_structural_identity::{metadata, nominal_record, primitive, NominalField};
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
fn structural_construction_uses_checked_defaults_for_missing_fields() {
    let mut payload = payload_class();
    payload.field_defaults = vec![(0, HirExpr::IntLiteral(7))];
    let module = module(vec![structural_function()], vec![payload]);

    let generated = generate_rust(&module);

    assert!(
        generated.contains("let mut child_nodes: [Option<"),
        "{generated}"
    );
    assert!(
        generated.contains("None => 7"),
        "missing structural fields must evaluate their checked default: {generated}"
    );
    assert!(
        !generated.contains("description.edges().len() != 1"),
        "defaulted structural fields must be omittable: {generated}"
    );
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
fn project_record_eligibility_resolves_nested_imported_members() {
    let mut leaf = payload_class();
    leaf.name = "Leaf".to_string();
    leaf.identity = Some("models.Leaf".to_string());
    let models = module(Vec::new(), vec![leaf]);

    let leaf_type = Type::Class {
        identity: Some("models.Leaf".to_string()),
        type_args: Vec::new(),
        name: "Leaf".to_string(),
        fields: vec![("value".to_string(), Type::Int)],
        methods: Vec::new(),
        parent_class: None,
    };
    let mut payload = payload_class();
    payload.identity = Some("records.Payload".to_string());
    payload.fields = vec![("leaf".to_string(), leaf_type.clone())];
    let payload_type = Type::Class {
        identity: Some("records.Payload".to_string()),
        type_args: Vec::new(),
        name: "Payload".to_string(),
        fields: vec![("leaf".to_string(), leaf_type)],
        methods: Vec::new(),
        parent_class: None,
    };
    let records = module(
        vec![ordinary_function(
            "choose",
            Type::Union(vec![payload_type, Type::Str]),
        )],
        vec![payload],
    );
    let api = module(vec![structural_function()], Vec::new());

    let project = crate::generate_rust_multi_with_metadata(
        &[("models", &models), ("records", &records), ("main", &api)],
        &crate::StdlibCode::default(),
    );
    let records_rust = project
        .rust_files
        .get("records")
        .expect("records module is generated");

    assert!(
        records_rust.contains("StructuralType for Payload"),
        "{records_rust}"
    );
    assert!(
        records_rust.contains("StructuralConstruct for Payload"),
        "{records_rust}"
    );
    assert!(
        project
            .project_union_prelude
            .contains("crate::records::Payload"),
        "{}",
        project.project_union_prelude
    );
}

#[test]
fn project_root_record_keeps_qualified_structural_identity() {
    let module = module(vec![structural_function()], vec![payload_class()]);

    let project = crate::generate_rust_multi_with_metadata(
        &[("main", &module)],
        &crate::StdlibCode::default(),
    );
    let main_rust = project
        .rust_files
        .get("main")
        .expect("main module is generated");

    assert!(main_rust.contains("Some(\"main.Payload\")"), "{main_rust}");
    assert!(!main_rust.contains("Some(\"Payload\")"), "{main_rust}");

    let modules = [("main", &module)];
    let supported =
        crate::structural_impl_codegen::structural_record_identities_for_project(&modules);
    let identities = crate::structural_identity_codegen::static_class_identities_for_project(
        &modules, &supported,
    );
    let expected = nominal_record(
        "main.Payload",
        &[],
        &[NominalField {
            name: "value",
            identity: primitive("int"),
            required: true,
            default_identity: None,
        }],
        metadata(&[]),
    );
    assert_eq!(identities.get("main.Payload"), Some(&expected));
}

#[test]
fn named_single_file_record_keeps_unqualified_structural_identity() {
    let module = module(vec![structural_function()], vec![payload_class()]);

    let generated = crate::generate_rust_with_stdlib_for_module(
        &module,
        &crate::StdlibCode::default(),
        Some("main"),
    )
    .rust_source;

    assert!(generated.contains("Some(\"Payload\")"), "{generated}");
    assert!(!generated.contains("Some(\"main.Payload\")"), "{generated}");
}

#[test]
fn imported_stdlib_record_gets_one_late_canonical_structural_impl() {
    let mut module = module(vec![structural_function()], Vec::new());
    let import = sifr_ir::HirImport {
        module: "sifr.json".to_string(),
        names: vec!["JsonValue".to_string()],
        aliases: Vec::new(),
    };
    module.imports = vec![import.clone(), import];
    let mut template = payload_class();
    template.name = "JsonValue".to_string();
    template.identity = Some("sifr.json.JsonValue".to_string());
    let mut stdlib = crate::StdlibCode::default();
    stdlib.module_class_templates.insert(
        "sifr.json".to_string(),
        std::collections::HashMap::from([("JsonValue".to_string(), template)]),
    );
    stdlib.module_rust_code.insert(
        "sifr.json".to_string(),
        crate::StdlibRustSource {
            module: "sifr.json".to_string(),
            source_path: "stdlib/sifr/json.sifr".to_string(),
            source_sha256: "fixture".to_string(),
            nominal_types: std::collections::HashSet::from(["JsonValue".to_string()]),
            rust: "struct JsonValue { value: i64 }".to_string(),
        },
    );

    let generated =
        crate::generate_rust_with_stdlib_for_module(&module, &stdlib, Some("main")).rust_source;
    let target = "StructuralType for __SifrStdlib_sifr_x2ejson_x2eJsonValue";

    assert_eq!(generated.matches(target).count(), 1, "{generated}");
    assert!(
        generated.contains("Some(\"sifr.json.JsonValue\")"),
        "{generated}"
    );
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
fn test_project_root_record_keeps_qualified_structural_identity() {
    let case = module(vec![structural_function()], vec![payload_class()]);
    let generated = crate::lib_test_project_codegen::generate_rust_test_project_with_metadata(
        &[],
        &[("case", &case)],
        &crate::StdlibCode::default(),
    );
    let case_rust = generated
        .test_rust_files
        .get("case")
        .expect("test module is generated");

    assert!(case_rust.contains("Some(\"case.Payload\")"));
    assert!(case_rust.contains("description.nominal_identity() != Some(\"case.Payload\")"));
    assert!(!case_rust.contains("Some(\"Payload\")"));
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
fn enum_with_unrepresentable_identity_metadata_gets_no_structural_impl() {
    let mut enumeration = payload_class();
    enumeration.name = "Status".to_string();
    enumeration.kind = HirClassKind::Enum;
    enumeration.fields = Vec::new();
    enumeration.enum_variants = vec![("READY".to_string(), Some(1))];
    enumeration.declaration_metadata = vec![TypedDeclarationMetadata {
        owner: "Status".to_string(),
        target_kind: DeclarationMetadataTargetKind::Type,
        target_name: None,
        key: "example.policy".to_string(),
        value_type: Type::Int,
        value: HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(1)),
            op: "+".to_string(),
            right: Box::new(HirExpr::IntLiteral(1)),
            ty: Type::Int,
        },
        range: Default::default(),
    }];
    let module = module(vec![structural_function()], vec![enumeration]);

    let generated = generate_rust(&module);

    assert!(
        !generated.contains("StructuralType for Status"),
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

#[test]
fn plain_static_program_generic_emits_the_sealed_runtime_bound() {
    let mut retain = ordinary_function("retain", Type::TypeVar("T".to_string()));
    retain.return_type = Type::TypeVar("T".to_string());
    retain.type_params = vec!["T".to_string()];
    retain.body = vec![sifr_ir::HirStmt::Return {
        value: Some(HirExpr::Name {
            name: "value".to_string(),
            binding_id: None,
            ty: Type::TypeVar("T".to_string()),
        }),
    }];
    let mut module = module(vec![retain], Vec::new());
    module.type_param_bounds.insert(
        "retain".to_string(),
        std::collections::HashMap::from([("T".to_string(), vec!["StaticProgram".to_string()])]),
    );

    let rust_code = generate_rust(&module);

    assert!(
        rust_code.contains("T: ::sifr_runtime::interop::structural::StaticProgramType + Clone"),
        "{rust_code}"
    );
    assert!(
        !rust_code.contains("T: Clone + std::fmt::Display"),
        "{rust_code}"
    );
}

#[test]
fn plain_string_structural_generic_emits_projection_bounds() {
    let mut retain = ordinary_function("retain", Type::TypeVar("T".to_string()));
    retain.return_type = Type::TypeVar("T".to_string());
    retain.type_params = vec!["T".to_string()];
    retain.body = vec![sifr_ir::HirStmt::Return {
        value: Some(HirExpr::Name {
            name: "value".to_string(),
            binding_id: None,
            ty: Type::TypeVar("T".to_string()),
        }),
    }];
    let mut module = module(vec![retain], Vec::new());
    module.type_param_bounds.insert(
        "retain".to_string(),
        std::collections::HashMap::from([("T".to_string(), vec!["StringStructural".to_string()])]),
    );

    let rust_code = generate_rust(&module);

    assert!(
        rust_code.contains(
            "T: ::sifr_runtime::interop::structural::StructuralConstruct + ::sifr_runtime::interop::structural::StructuralProject + Clone + 'static"
        ),
        "{rust_code}"
    );
    assert!(
        !rust_code.contains("T: Clone + std::fmt::Display"),
        "{rust_code}"
    );
}

#[test]
fn attached_structural_generic_preserves_attached_api_bounds() {
    let mut retain = ordinary_function("retain", Type::TypeVar("T".to_string()));
    retain.return_type = Type::TypeVar("T".to_string());
    retain.type_params = vec!["T".to_string()];
    retain.decorators = vec!["attached_api".to_string()];
    retain.body = vec![sifr_ir::HirStmt::Return {
        value: Some(HirExpr::Name {
            name: "value".to_string(),
            binding_id: None,
            ty: Type::TypeVar("T".to_string()),
        }),
    }];
    let mut module = module(vec![retain], Vec::new());
    module.type_param_bounds.insert(
        "retain".to_string(),
        std::collections::HashMap::from([("T".to_string(), vec!["Structural".to_string()])]),
    );

    let rust_code = generate_rust(&module);

    assert!(rust_code.contains("T: Clone + 'static"), "{rust_code}");
    assert!(
        !rust_code.contains("T: ::sifr_runtime::interop::structural::StructuralConstruct"),
        "{rust_code}"
    );
}

#[test]
fn no_context_method_slots_generic_emits_the_slot_table_bound() {
    let mut retain = ordinary_function("retain", Type::TypeVar("T".to_string()));
    retain.return_type = Type::TypeVar("T".to_string());
    retain.type_params = vec!["T".to_string()];
    retain.body = vec![sifr_ir::HirStmt::Return {
        value: Some(HirExpr::Name {
            name: "value".to_string(),
            binding_id: None,
            ty: Type::TypeVar("T".to_string()),
        }),
    }];
    let mut module = module(vec![retain], Vec::new());
    module.type_param_bounds.insert(
        "retain".to_string(),
        std::collections::HashMap::from([("T".to_string(), vec!["MethodSlots".to_string()])]),
    );

    let rust_code = generate_rust(&module);

    assert!(
        rust_code.contains("MethodSlotTable<::sifr_runtime::interop::structural::NoContext>"),
        "{rust_code}"
    );
}

#[test]
fn project_static_program_owners_include_supported_imported_fields() {
    let imported = HirClass {
        name: "ImportedPayload".to_string(),
        identity: Some("support.ImportedPayload".to_string()),
        ..payload_class()
    };
    let owner = HirClass {
        name: "Owner".to_string(),
        fields: vec![(
            "payload".to_string(),
            Type::Class {
                identity: Some("support.ImportedPayload".to_string()),
                name: "ImportedPayload".to_string(),
                fields: vec![("value".to_string(), Type::Int)],
                methods: Vec::new(),
                parent_class: None,
                type_args: Vec::new(),
            },
        )],
        ..payload_class()
    };
    let main = module(Vec::new(), vec![owner]);
    let support = module(Vec::new(), vec![imported]);
    let modules = [("main", &main), ("support", &support)];

    let local = crate::structural_static_program_owners(&main);
    let project = crate::structural_static_program_owners_for_project(&main, &modules);

    assert!(!local.contains("Owner"));
    assert!(project.contains("Owner"));
}

#[test]
fn concrete_generic_child_flattens_parent_fields_for_structural_bridge() {
    let parent = HirClass {
        name: "GenericParent".to_string(),
        identity: Some("models.GenericParent".to_string()),
        fields: vec![("value".to_string(), Type::TypeVar("T".to_string()))],
        type_params: vec!["T".to_string()],
        is_hashable: true,
        ..payload_class()
    };
    let parent_type = Type::Class {
        identity: Some("models.GenericParent".to_string()),
        type_args: vec![Type::Int],
        name: "GenericParent".to_string(),
        fields: vec![("value".to_string(), Type::Int)],
        methods: Vec::new(),
        parent_class: None,
    };
    let child = HirClass {
        name: "Concrete".to_string(),
        identity: Some("main.Concrete".to_string()),
        fields: vec![("label".to_string(), Type::Str)],
        parent_class: Some("GenericParent".to_string()),
        parent_type: Some(parent_type),
        is_hashable: true,
        ..payload_class()
    };
    let nested = HirClass {
        name: "Nested".to_string(),
        identity: Some("main.Nested".to_string()),
        fields: vec![(
            "payload".to_string(),
            Type::Class {
                identity: Some("main.Concrete".to_string()),
                type_args: Vec::new(),
                name: "Concrete".to_string(),
                fields: vec![
                    ("value".to_string(), Type::Int),
                    ("label".to_string(), Type::Str),
                ],
                methods: Vec::new(),
                parent_class: Some("models.GenericParent".to_string()),
            },
        )],
        is_hashable: true,
        ..payload_class()
    };
    let models = module(Vec::new(), vec![parent]);
    let main = module(vec![structural_function()], vec![child, nested]);
    let modules = [("models", &models), ("main", &main)];

    let project = crate::generate_rust_multi_with_metadata(&modules, &crate::StdlibCode::default());
    let main_rust = project
        .rust_files
        .get("main")
        .expect("main module is generated");
    assert!(
        main_rust.contains("StructuralType for Concrete"),
        "{main_rust}"
    );
    assert!(
        main_rust.contains("StructuralConstruct for Concrete"),
        "{main_rust}"
    );
    assert!(
        main_rust.contains("StructuralProject for Concrete"),
        "{main_rust}"
    );
    assert!(
        main_rust.contains("StructuralType for Nested"),
        "{main_rust}"
    );
    assert!(main_rust.contains("RecordField(\"value\")"), "{main_rust}");
    assert!(main_rust.contains("RecordField(\"label\")"), "{main_rust}");
    assert!(
        main_rust.contains("genericparent: <GenericParent<i64>>::new(value)"),
        "{main_rust}"
    );

    let supported =
        crate::structural_impl_codegen::structural_record_identities_for_project(&modules);
    let identities = crate::structural_identity_codegen::static_class_identities_for_project(
        &modules, &supported,
    );
    let expected = nominal_record(
        "main.Concrete",
        &[],
        &[
            NominalField {
                name: "value",
                identity: primitive("int"),
                required: true,
                default_identity: None,
            },
            NominalField {
                name: "label",
                identity: primitive("str"),
                required: true,
                default_identity: None,
            },
        ],
        metadata(&[]),
    );
    assert_eq!(identities.get("main.Concrete"), Some(&expected));
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
