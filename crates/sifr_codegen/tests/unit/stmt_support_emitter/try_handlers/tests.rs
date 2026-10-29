use super::*;

fn error(identity: &str, name: &str, parent_class: Option<&str>) -> Type {
    Type::Class {
        identity: Some(identity.to_string()),
        type_args: Vec::new(),
        name: name.to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: parent_class.map(str::to_string),
    }
}

#[test]
fn handler_matching_uses_exact_user_error_ancestry() {
    let base = error("pkg.BaseError", "BaseError", Some("Error"));
    let child = error("pkg.ChildError", "ChildError", Some("pkg.BaseError|Error"));
    let unrelated = error("other.BaseError", "BaseError", Some("Error"));
    let handler = HirExceptHandler {
        error_type: Some("BaseError".to_string()),
        error_resolved_type: Some(base),
        name: Some("error".to_string()),
        body: Vec::new(),
    };

    assert!(matches!(
        RustEmitter::try_except_handler_condition_expr(
            &handler,
            "error",
            "ChildError",
            Some(&child),
        ),
        HandlerMatchCondition::Always
    ));
    assert!(matches!(
        RustEmitter::try_except_handler_condition_expr(
            &handler,
            "error",
            "BaseError",
            Some(&unrelated),
        ),
        HandlerMatchCondition::Unsupported
    ));
}
