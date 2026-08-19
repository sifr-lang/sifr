use super::structural_record_identities_for_project;
use sifr_ir::{HirClass, HirClassKind, HirModule};
use sifr_type_system::Type;
use std::collections::HashMap;

fn class(name: &str, fields: Vec<(String, Type)>) -> HirClass {
    HirClass {
        name: name.to_string(),
        identity: Some(format!("main.{name}")),
        fields,
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

#[test]
fn nested_generic_union_topology_change_is_not_structurally_supported() {
    let mut generic = class(
        "Generic",
        vec![(
            "value".to_string(),
            Type::Union(vec![Type::None, Type::TypeVar("T".to_string())]),
        )],
    );
    generic.type_params = vec!["T".to_string()];
    let concrete = Type::Class {
        identity: Some("main.Generic".to_string()),
        type_args: vec![sifr_type_system::make_union(vec![Type::None, Type::Str])],
        name: "Generic".to_string(),
        fields: vec![(
            "value".to_string(),
            sifr_type_system::make_union(vec![Type::None, Type::Str]),
        )],
        methods: Vec::new(),
        parent_class: None,
    };
    let owner = class("Owner", vec![("payload".to_string(), concrete.clone())]);
    let mut child = class("Child", vec![("label".to_string(), Type::Str)]);
    child.parent_class = Some("Generic".to_string());
    child.parent_type = Some(concrete);
    let module = HirModule {
        functions: Vec::new(),
        classes: vec![generic, owner, child],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };
    let modules = [("main", &module)];

    let supported = structural_record_identities_for_project(&modules);

    assert!(supported.contains("main.Generic"));
    assert!(!supported.contains("main.Owner"));
    assert!(!supported.contains("main.Child"));
}
