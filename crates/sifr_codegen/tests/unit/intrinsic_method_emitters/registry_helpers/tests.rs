use super::*;

#[test]
fn owned_callable_argument_uses_union_representation_conversion() {
    let source = sifr_type_system::make_union(vec![Type::Str, Type::None]);
    let target = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    let callable = HirExpr::Name {
        name: "handler".to_string(),
        binding_id: None,
        ty: Type::Callable(
            vec![target.clone()],
            vec![ParamConvention::own()],
            Box::new(Type::Bool),
        ),
    };
    let mut emitter = RustEmitter::new();

    let lowered = registry_call_callable_with_owned_args(
        &mut emitter,
        &callable,
        &[("value".to_string(), source)],
    )
    .expect("owned callable argument should lower");
    let rendered = crate::render_expr(&lowered);

    assert!(rendered.contains(").map("), "{rendered}");
    assert!(rendered.contains(".unwrap_or("), "{rendered}");
    assert!(rendered.contains(&target.union_enum_name()), "{rendered}");
    assert!(
        !rendered.contains("compiler-verified callable argument should be Some"),
        "{rendered}"
    );
}

#[test]
fn owned_callable_argument_rejects_unadapted_optional_input() {
    let source = sifr_type_system::make_union(vec![Type::Int, Type::None]);
    let callable = HirExpr::Name {
        name: "handler".to_string(),
        binding_id: None,
        ty: Type::Callable(
            vec![Type::Int],
            vec![ParamConvention::own()],
            Box::new(Type::Bool),
        ),
    };
    let mut emitter = RustEmitter::new();

    assert!(
        registry_call_callable_with_owned_args(
            &mut emitter,
            &callable,
            &[("value".to_string(), source)],
        )
        .is_none()
    );
}

#[test]
fn owned_callable_argument_preserves_class_upcasts() {
    let parent = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Parent".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    let child = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Child".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: Some("Parent".to_string()),
    };
    let callable = HirExpr::Name {
        name: "handler".to_string(),
        binding_id: None,
        ty: Type::Callable(
            vec![parent],
            vec![ParamConvention::own()],
            Box::new(Type::Bool),
        ),
    };
    let mut emitter = RustEmitter::new();

    let lowered = registry_call_callable_with_owned_args(
        &mut emitter,
        &callable,
        &[("value".to_string(), child)],
    )
    .expect("class upcast should lower");
    let rendered = crate::render_expr(&lowered);

    assert!(
        rendered.contains("::std::convert::Into::<Parent>::into(value)"),
        "{rendered}"
    );
}
