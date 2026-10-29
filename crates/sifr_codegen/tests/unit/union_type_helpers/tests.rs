use super::*;

#[test]
fn affine_union_uses_debug_without_clone_or_display_bounds() {
    let buffer =
        Type::PythonBuffer(Box::new(Type::FixedInt(sifr_type_system::FixedIntType::U8)));
    let members = vec![Type::None, Type::Int, buffer.clone()];
    let mut emitter = RustEmitter::new();
    emitter.register_union_type(&Type::Union(members));
    emitter.generate_enum_definitions();

    let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
        panic!("first generated item should be the union enum");
    };
    assert_eq!(derives, &["Debug"]);
    let rendered = crate::render::render_items(&emitter.enum_items);
    assert!(rendered.contains(&format!("{}(v) =>", buffer.union_variant_name())));
    assert!(rendered.contains("write!(f, \"{:?}\", v)"));
    let none_arm = rendered
        .split(&format!("{}(v) =>", Type::None.union_variant_name()))
        .nth(1)
        .expect("None union arm should be rendered");
    assert!(none_arm[..none_arm.len().min(120)].contains("write!(f, \"{:?}\", v)"));
}

#[test]
fn equality_capable_union_derives_the_required_rust_traits() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    let mut emitter = RustEmitter::new();
    emitter.register_union_type(&union);
    emitter.generate_enum_definitions();

    let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
        panic!("first generated item should be the union enum");
    };
    assert_eq!(derives, &["Debug", "Clone", "PartialEq", "Eq", "Hash"]);
    let rendered = crate::render::render_items(&emitter.enum_items);
    assert!(rendered.contains(&format!(
        "impl ::std::fmt::Display for {}",
        union.union_enum_name()
    )));
}

#[test]
fn try_carrier_reuse_keeps_ordinary_union_traits() {
    let first_error = Type::Class {
        identity: Some("tests.first.Error".to_string()),
        type_args: Vec::new(),
        name: "Error".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let second_error = Type::Class {
        identity: Some("tests.second.Error".to_string()),
        type_args: Vec::new(),
        name: "Error".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let union = Type::Union(vec![first_error, second_error]);
    let mut emitter = RustEmitter::new();
    emitter.register_union_type(&union);
    emitter.register_try_error_carrier(&union);
    emitter.generate_enum_definitions();

    let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
        panic!("first generated item should be the union enum");
    };
    assert_eq!(derives, &["Debug", "Clone", "PartialEq", "Eq", "Hash"]);
}

#[test]
fn try_carrier_and_ordinary_union_registration_keep_one_member_order() {
    let error = |name: &str| Type::Class {
        identity: Some(format!("a.{name}")),
        type_args: Vec::new(),
        name: name.to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let os_error = error("OSError");
    let zero_division = error("ZeroDivisionError");
    let ordinary = sifr_type_system::make_union(vec![os_error.clone(), zero_division.clone()]);
    let raw = Type::Union(vec![zero_division.clone(), os_error.clone()]);
    let carrier = crate::try_error_carrier::exact_try_error_carrier(&[zero_division, os_error])
        .expect("carrier should contain both errors");

    let render = |first: &Type| {
        let mut emitter = RustEmitter::new();
        emitter.register_union_type(first);
        emitter.register_try_error_carrier(&carrier);
        emitter.register_union_type(&ordinary);
        emitter.generate_enum_definitions();
        crate::render::render_items(&emitter.enum_items)
    };

    assert_eq!(render(&raw), render(&ordinary));
    assert_eq!(render(&carrier), render(&ordinary));
}

#[test]
fn carrier_only_union_omits_unneeded_value_traits() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    let mut emitter = RustEmitter::new();
    emitter.register_try_error_carrier(&union);
    emitter.generate_enum_definitions();

    let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
        panic!("first generated item should be the union enum");
    };
    assert_eq!(derives, &["Debug", "Clone"]);
}

#[test]
fn collapsed_try_carrier_does_not_record_a_plain_type_as_an_enum() {
    let class = Type::Class {
        identity: Some("tests.Error".to_string()),
        type_args: Vec::new(),
        name: "Error".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let snapshot = Type::Class {
        identity: Some("tests.Error".to_string()),
        type_args: Vec::new(),
        name: "Error".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let mut emitter = RustEmitter::new();

    emitter.register_try_error_carrier(&Type::Union(vec![class, snapshot]));

    assert!(emitter.try_error_carrier_enums.is_empty());
}

#[test]
fn nested_optional_union_registers_its_grouped_payload_enum() {
    let inner = sifr_type_system::make_union(vec![Type::Int, Type::Str]);
    let raw = Type::Union(vec![inner.clone(), Type::None]);
    let expected_name = inner.union_enum_name();
    let mut emitter = RustEmitter::new();

    emitter.register_union_type(&raw);

    assert_eq!(
        crate::render_type(&crate::sifr_type_to_rust_type(&raw)),
        format!("Option<{expected_name}>")
    );
    assert!(emitter.union_enums.contains_key(&expected_name));
    assert_eq!(emitter.union_enums.len(), 1);
}

#[test]
fn callable_bearing_union_requires_no_unavailable_formatting_trait() {
    let holder = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "CallbackHolder".to_string(),
        fields: vec![(
            "callback".to_string(),
            Type::Callable(
                vec![Type::Int],
                vec![ParamConvention::own()],
                Box::new(Type::Int),
            ),
        )]
        .into(),
        methods: vec![].into(),
        parent_class: None,
    };
    let mut emitter = RustEmitter::new();
    emitter.register_union_type(&Type::Union(vec![Type::Int, holder]));
    emitter.generate_enum_definitions();

    let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
        panic!("first generated item should be the union enum");
    };
    assert!(derives.is_empty());
    assert_eq!(emitter.enum_items.len(), 1);
}

#[test]
fn nested_result_error_union_is_registered() {
    let handler_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "HandlerError".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let python_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "PythonError".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let error = Type::Union(vec![handler_error, python_error]);
    let mut emitter = RustEmitter::new();

    emitter.register_union_type(&Type::Result(Box::new(Type::None), Box::new(error.clone())));

    assert_eq!(
        emitter.union_enums.get(&error.union_enum_name()),
        match error {
            Type::Union(ref members) => Some(members),
            _ => None,
        }
    );
}
#[test]
#[should_panic(expected = "requires its canonical nominal identity")]
fn builtin_project_member_requires_explicit_identity() {
    let mut emitter = RustEmitter::new();
    emitter
        .project_nominal_type_paths
        .insert("ValueError".to_string(), "crate::ValueError".to_string());
    let _ = emitter.project_nominal_path(None, "ValueError");
}
#[test]
fn builtin_root_error_contract_resolves_before_project_nominal_lookup() {
    let mut emitter = RustEmitter::new();
    emitter.project_nominal_type_paths.insert(
        "sifr.builtin.Error".to_string(),
        "crate::shared::Error".to_string(),
    );
    let root = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Error".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    assert_eq!(
        emitter.project_union_member_rust_type(&root),
        RustType::Named("crate::shared::Error".to_string())
    );
}

#[test]
#[should_panic(expected = "requires its canonical nominal identity")]
fn noncanonical_identityless_root_name_does_not_gain_builtin_identity() {
    let mut emitter = RustEmitter::new();
    emitter.project_nominal_type_paths.insert(
        "sifr.builtin.Error".to_string(),
        "crate::shared::Error".to_string(),
    );
    let lookalike = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Error".to_string(),
        fields: vec![("payload".to_string(), Type::Int)].into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    let _ = emitter.project_union_member_rust_type(&lookalike);
}
