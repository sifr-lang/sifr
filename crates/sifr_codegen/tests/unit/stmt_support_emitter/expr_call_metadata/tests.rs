use super::*;

#[test]
fn plain_call_targets_split_namespaced_functions_into_paths() {
    assert!(matches!(
        plain_call_target_for_ir("Point::origin"),
        RustExpr::Path(parts) if parts == ["Point", "origin"]
    ));
    assert!(matches!(
        plain_call_target_for_ir("compute"),
        RustExpr::Ident(name) if name == "compute"
    ));
}

#[test]
fn constructor_names_leave_nominal_type_arguments_to_rust_inference() {
    let ty = Type::Class {
        identity: None,
        name: "Channel".to_string(),
        type_args: vec![Type::Any],
        fields: Default::default(),
        methods: Default::default(),
        parent_class: None,
    };

    assert_eq!(canonical_constructor_class_name("Channel", &ty), "Channel");
}
