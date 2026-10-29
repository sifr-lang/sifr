
use super::collect_type_vars;
use sifr_type_system::Type;

#[test]
fn collects_typevars_from_nominal_class_arguments() {
    let ty = Type::Class {
        identity: Some("pkg.Node".to_string()),
        type_args: vec![Type::TypeVar("T".to_string())],
        name: "Node".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    let mut vars = Vec::new();
    collect_type_vars(&ty, &mut vars);
    assert_eq!(vars, vec!["T".to_string()]);
}
