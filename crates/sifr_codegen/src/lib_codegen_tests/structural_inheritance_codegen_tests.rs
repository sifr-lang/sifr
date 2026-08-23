use super::structural_impl_demand_codegen_tests::{module, payload_class, structural_function};
use sifr_ir::HirClass;
use sifr_structural_identity::{NominalField, metadata, nominal_record, primitive};
use sifr_type_system::Type;

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
        main_rust.contains("genericparent: <GenericParent<i64>>::new(__sifr_field_0)"),
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

#[test]
fn plain_data_parent_flattens_into_structural_child_impls() {
    let parent = HirClass {
        name: "Parent".to_string(),
        identity: Some("main.Parent".to_string()),
        fields: vec![("value".to_string(), Type::Int)],
        is_hashable: true,
        ..payload_class()
    };
    let child = HirClass {
        name: "Child".to_string(),
        identity: Some("main.Child".to_string()),
        fields: vec![("label".to_string(), Type::Str)],
        parent_class: Some("Parent".to_string()),
        parent_type: Some(Type::Class {
            identity: Some("main.Parent".to_string()),
            type_args: Vec::new(),
            name: "Parent".to_string(),
            fields: vec![("value".to_string(), Type::Int)],
            methods: Vec::new(),
            parent_class: None,
        }),
        is_hashable: true,
        ..payload_class()
    };
    let module = module(vec![structural_function()], vec![parent, child]);

    let generated = crate::generate_rust(&module);

    assert!(
        generated.contains("StructuralConstruct for Child"),
        "{generated}"
    );
    assert!(generated.contains("RecordField(\"value\")"), "{generated}");
    assert!(
        generated.contains("parent: <Parent>::new(__sifr_field_0)"),
        "{generated}"
    );
}

#[test]
fn recursive_boxed_data_parent_is_not_structurally_emitted_for_child() {
    let recursive_type = Type::Class {
        identity: Some("main.Node".to_string()),
        type_args: Vec::new(),
        name: "Node".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    };
    let recursive_field = Type::Union(vec![Type::None, recursive_type]);
    let parent = HirClass {
        name: "Node".to_string(),
        identity: Some("main.Node".to_string()),
        fields: vec![("next".to_string(), recursive_field.clone())],
        is_hashable: true,
        ..payload_class()
    };
    let child = HirClass {
        name: "RecursiveChild".to_string(),
        identity: Some("main.RecursiveChild".to_string()),
        parent_class: Some("Node".to_string()),
        parent_type: Some(Type::Class {
            identity: Some("main.Node".to_string()),
            type_args: Vec::new(),
            name: "Node".to_string(),
            fields: vec![("next".to_string(), recursive_field)],
            methods: Vec::new(),
            parent_class: None,
        }),
        is_hashable: true,
        ..payload_class()
    };
    let module = module(vec![structural_function()], vec![parent, child]);

    let generated = crate::generate_rust(&module);

    assert!(
        !generated.contains("StructuralType for RecursiveChild"),
        "{generated}"
    );
}
