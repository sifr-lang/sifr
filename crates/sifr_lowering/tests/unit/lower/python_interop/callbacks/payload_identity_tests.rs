
use super::*;

#[test]
fn class_and_builtin_with_same_emitted_name_collide() {
    let class = Type::Class {
        identity: None,
        name: "String".to_string(),
        type_args: Vec::new(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    let channel = Type::Union(vec![class, Type::Str]);
    assert_eq!(
        error_channel_codegen_payload_collision(&channel).as_deref(),
        Some("String")
    );
}

#[test]
fn exact_integer_uses_its_canonical_codegen_payload_identity() {
    let class = Type::Class {
        identity: None,
        name: "SifrInt".to_string(),
        type_args: Vec::new(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    let channel = Type::Union(vec![class, Type::Int]);
    assert_eq!(
        error_channel_codegen_payload_collision(&channel).as_deref(),
        Some("SifrInt")
    );
}
