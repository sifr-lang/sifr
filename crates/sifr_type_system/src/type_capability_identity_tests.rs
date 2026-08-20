use crate::{FixedIntType, Type};

fn class(identity: &str, type_args: Vec<Type>, fields: Vec<(&str, Type)>) -> Type {
    Type::Class {
        identity: Some(identity.to_string()),
        type_args,
        name: "Root".to_string(),
        fields: fields
            .into_iter()
            .map(|(name, ty)| (name.to_string(), ty))
            .collect(),
        methods: Vec::new(),
        parent_class: None,
    }
}

fn buffer() -> Type {
    Type::PythonBuffer(Box::new(Type::FixedInt(FixedIntType::U8)))
}

#[test]
fn repeated_class_basenames_do_not_short_circuit_recursive_capabilities() {
    let inner = class("inner.Root", Vec::new(), vec![("view", buffer())]);
    let outer = class("outer.Root", Vec::new(), vec![("inner", inner)]);

    assert!(outer.contains_affine_resource());
    assert!(!outer.supports_derived_clone());
    assert!(!outer.supports_structural_equality());
    assert!(!outer.supports_hash_key());
    assert!(!outer.supports_derived_hash());

    let callable = Type::Callable(Vec::new(), Vec::new(), Box::new(Type::Int));
    let non_debug_inner = class("inner.DebugRoot", Vec::new(), vec![("callback", callable)]);
    let non_debug_outer = class(
        "outer.DebugRoot",
        Vec::new(),
        vec![("inner", non_debug_inner)],
    );
    assert!(!non_debug_outer.supports_debug_formatting());
}

#[test]
fn recursive_capability_keys_distinguish_concrete_specializations() {
    let affine_specialization = class("generic.Root", vec![buffer()], vec![("view", buffer())]);
    let outer_specialization = class(
        "generic.Root",
        vec![Type::Int],
        vec![("nested", affine_specialization)],
    );

    assert!(outer_specialization.contains_affine_resource());
    assert!(!outer_specialization.supports_derived_clone());
}

#[test]
fn derived_hash_accepts_type_variables_but_rejects_transitive_non_send_ancestry() {
    assert!(Type::TypeVar("T".to_string()).supports_derived_hash());
    let non_send_child = Type::Class {
        identity: Some("main.Child".to_string()),
        type_args: Vec::new(),
        name: "Child".to_string(),
        fields: vec![("value".to_string(), Type::TypeVar("T".to_string()))],
        methods: Vec::new(),
        parent_class: Some("Parent|NonSend".to_string()),
    };
    assert!(!non_send_child.supports_derived_hash());
}

#[test]
fn unknown_or_any_query_recurses_through_supported_type_shapes() {
    let optional_any = Type::Union(vec![Type::None, Type::Any]);
    let nested = Type::Result(
        Box::new(Type::List(Box::new(optional_any))),
        Box::new(Type::Tuple(vec![
            Type::Int,
            Type::Iterable(Box::new(Type::Unknown)),
        ])),
    );
    let alias = Type::alias("Payload", nested);

    assert!(alias.contains_unknown_or_any());
    assert!(Type::Iterator(Box::new(Type::Any)).contains_unknown_or_any());
    assert!(
        !Type::List(Box::new(Type::Union(vec![Type::None, Type::Int,]))).contains_unknown_or_any()
    );
}
