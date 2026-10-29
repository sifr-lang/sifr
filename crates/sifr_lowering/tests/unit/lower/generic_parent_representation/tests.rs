use super::{class_candidate, preserves_union_structure};
use sifr_type_system::Type;
use std::collections::HashMap;

fn parent(type_arg: Type, field: Type) -> Type {
    Type::Class {
        identity: Some("models.Parent".to_string()),
        type_args: vec![type_arg],
        name: "Parent".to_string(),
        fields: vec![("value".to_string(), field)].into(),
        methods: Vec::new().into(),
        parent_class: None,
    }
}

#[test]
fn generic_parent_rejects_only_union_topology_changes() {
    let template = parent(
        Type::TypeVar("T".to_string()),
        Type::Union(vec![Type::None, Type::TypeVar("T".to_string())]),
    );
    let safe = parent(Type::Str, Type::Union(vec![Type::None, Type::Str]));
    let unsafe_arg = sifr_type_system::make_union(vec![Type::None, Type::Str]);
    let unsafe_parent = parent(unsafe_arg, Type::Union(vec![Type::None, Type::Str]));
    let classes = HashMap::from([("Parent".to_string(), template)]);
    let type_params = HashMap::from([("Parent".to_string(), vec!["T".to_string()])]);

    assert!(preserves_union_structure(
        &classes,
        &type_params,
        "Parent",
        &safe
    ));
    assert!(!preserves_union_structure(
        &classes,
        &type_params,
        "Parent",
        &unsafe_parent
    ));
}

#[test]
fn declared_parameters_are_the_parent_binding_authority() {
    let template = parent(
        Type::Str,
        Type::Union(vec![Type::None, Type::TypeVar("T".to_string())]),
    );
    let unsafe_arg = sifr_type_system::make_union(vec![Type::None, Type::Str]);
    let concrete = parent(unsafe_arg, Type::Unknown);
    let classes = HashMap::from([("Parent".to_string(), template)]);
    let type_params = HashMap::from([("Parent".to_string(), vec!["T".to_string()])]);

    assert!(!preserves_union_structure(
        &classes,
        &type_params,
        "Parent",
        &concrete
    ));
}

#[test]
fn canonical_identity_selects_the_parent_and_finds_transitive_hazards() {
    let inner = Type::Class {
        identity: Some("models.Inner".to_string()),
        type_args: vec![Type::TypeVar("U".to_string())],
        name: "Inner".to_string(),
        fields: vec![(
            "value".to_string(),
            Type::Union(vec![Type::None, Type::TypeVar("U".to_string())]),
        )]
        .into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    let imported_parent = parent(
        Type::TypeVar("T".to_string()),
        Type::Class {
            identity: Some("models.Inner".to_string()),
            type_args: vec![Type::TypeVar("T".to_string())],
            name: "Inner".to_string(),
            fields: Vec::new().into(),
            methods: Vec::new().into(),
            parent_class: None,
        },
    );
    let mut wrong_local = parent(
        Type::TypeVar("V".to_string()),
        Type::TypeVar("V".to_string()),
    );
    let Type::Class { identity, .. } = &mut wrong_local else {
        unreachable!("the parent test helper always returns a class")
    };
    *identity = Some("main.Parent".to_string());
    let unsafe_arg = sifr_type_system::make_union(vec![Type::None, Type::Str]);
    let concrete = parent(unsafe_arg, Type::Unknown);
    let classes = HashMap::from([
        ("Parent".to_string(), wrong_local),
        ("ImportedParent".to_string(), imported_parent),
        ("Inner".to_string(), inner),
    ]);
    let type_params = HashMap::from([
        ("Parent".to_string(), vec!["V".to_string()]),
        ("ImportedParent".to_string(), vec!["T".to_string()]),
        ("Inner".to_string(), vec!["U".to_string()]),
    ]);

    assert!(!preserves_union_structure(
        &classes,
        &type_params,
        "Parent",
        &concrete
    ));
}

#[test]
fn duplicate_canonical_candidates_use_a_deterministic_key() {
    let candidate = parent(Type::TypeVar("T".to_string()), Type::Str);
    let classes = HashMap::from([
        ("AliasZ".to_string(), candidate.clone()),
        ("AliasA".to_string(), candidate),
    ]);

    let (selected, _) = class_candidate(&classes, Some("models.Parent"), "Missing")
        .expect("canonical candidate exists");
    assert_eq!(selected, "AliasA");
}

#[test]
fn requested_canonical_candidate_wins_over_key_order() {
    let candidate = parent(Type::TypeVar("T".to_string()), Type::Str);
    let classes = HashMap::from([
        ("AliasA".to_string(), candidate.clone()),
        ("Requested".to_string(), candidate),
    ]);

    let (selected, _) = class_candidate(&classes, Some("models.Parent"), "Requested")
        .expect("requested canonical candidate exists");
    assert_eq!(selected, "Requested");
}
