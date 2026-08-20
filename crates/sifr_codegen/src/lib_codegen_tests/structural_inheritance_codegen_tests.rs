use super::structural_impl_demand_codegen_tests::{module, payload_class, structural_function};
use sifr_ir::HirClass;
use sifr_structural_identity::{metadata, nominal_record, primitive, NominalField};
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
