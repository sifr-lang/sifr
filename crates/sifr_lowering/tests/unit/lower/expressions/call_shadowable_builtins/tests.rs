
use super::*;

fn text_handle(methods: Vec<(String, FunctionType)>) -> Type {
    Type::Class {
        identity: Some(TEXT_FILE_HANDLE_IDENTITY.to_string()),
        type_args: Vec::new(),
        name: "TextFileHandle".to_string(),
        fields: Vec::new().into(),
        methods: methods.into(),
        parent_class: None,
    }
}

#[test]
fn canonical_handle_lookup_prefers_the_complete_class_surface() {
    let mut ctx = LowerCtx::new();
    ctx.class_types
        .insert("nested_snapshot".to_string(), text_handle(Vec::new()));
    ctx.class_types.insert(
        "TextFileHandle".to_string(),
        text_handle(vec![(
            "write".to_string(),
            FunctionType::all_borrow(vec![("text".to_string(), Type::Str)], Type::None),
        )]),
    );

    let selected = class_type_with_identity(&ctx, TEXT_FILE_HANDLE_IDENTITY)
        .expect("canonical handle should exist");
    let Type::Class { methods, .. } = selected else {
        panic!("canonical handle should remain a class");
    };
    assert!(methods.iter().any(|(name, _)| name == "write"));
}
