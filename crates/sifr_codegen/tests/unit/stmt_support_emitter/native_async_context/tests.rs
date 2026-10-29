use super::*;

fn error_type(name: &str, identity: &str) -> Type {
    Type::Class {
        identity: Some(identity.to_string()),
        type_args: Vec::new(),
        name: name.to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    }
}

#[test]
fn union_body_error_is_classified_before_conversion() {
    let timeout = error_type("TimeoutError", "sifr.builtin.TimeoutError");
    let ordinary = error_type("ResourceError", "project.ResourceError");
    let union = Type::Union(vec![timeout.clone(), ordinary.clone()]);

    let cause = native_error_cause(&union, "body_error");

    assert!(cause.contains(&format!(
        "{}::{}(variant_error) => AsyncExitCause::Timeout",
        union.union_enum_name(),
        timeout.union_variant_name()
    )));
    assert!(cause.contains(&format!(
        "{}::{}(variant_error) => AsyncExitCause::OrdinaryError",
        union.union_enum_name(),
        ordinary.union_variant_name()
    )));
}
