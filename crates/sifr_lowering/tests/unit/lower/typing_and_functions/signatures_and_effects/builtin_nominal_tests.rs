use super::*;
#[test]
fn registered_error_classes_and_constructors_share_canonical_identity() {
    let mut context = LowerCtx::new();
    register_builtins(&mut context);
    let mut checked = 0;
    for (name, ty) in &context.class_types {
        if name == "Error" {
            continue;
        }
        let Some(canonical) = sifr_type_system::builtin_error_identity(name) else {
            continue;
        };
        let Type::Class { identity, .. } = ty else {
            panic!("builtin error class")
        };
        assert_eq!(identity.as_deref(), Some(canonical.as_str()), "{name}");
        let constructor = &context.functions[name];
        assert_eq!(constructor.return_type.as_ref(), ty, "{name}");
        checked += 1;
    }
    assert!(checked > 20);
    assert!(context.class_types["Error"].is_builtin_error_base());
    for (child, parent) in [
        ("FileNotFoundError", "IOError"),
        ("PermissionError", "IOError"),
        ("FloatOverflowError", "OverflowError"),
        ("FloatPrecisionLossError", "OverflowError"),
        ("ArithmeticLimitError", "OverflowError"),
    ] {
        assert!(
            context.class_types[child].is_assignable_to(&context.class_types[parent]),
            "{child} -> {parent}"
        );
        assert!(
            context.class_types[child].is_assignable_to(&context.class_types["Error"]),
            "{child} -> Error"
        );
    }
}
