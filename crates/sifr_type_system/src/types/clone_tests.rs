use super::{StructuralRecordType, Type};

#[test]
fn cloned_nominal_type_keeps_identity_and_independent_mutation() {
    let mut original = Type::Class {
        identity: Some("package.Owner".to_owned()),
        type_args: vec![Type::List(Box::new(Type::Str))],
        name: "Owner".to_owned(),
        fields: vec![("items".to_owned(), Type::List(Box::new(Type::Int)))].into(),
        methods: Vec::new().into(),
        parent_class: Some("package.Parent".to_owned()),
    };
    let expected = format!("{original:?}");
    let cloned = original.clone();
    assert_eq!(cloned, original);
    if let (Type::Class { fields: first, .. }, Type::Class { fields: second, .. }) =
        (&original, &cloned)
    {
        assert!(first.shares_storage(second));
    } else {
        panic!("class fixtures");
    }
    let Type::Class {
        identity,
        type_args,
        name,
        fields,
        parent_class,
        ..
    } = &mut original
    else {
        panic!("class fixture");
    };
    *identity = None;
    type_args.clear();
    name.clear();
    fields.clear();
    *parent_class = None;
    assert_eq!(format!("{cloned:?}"), expected);
    assert_ne!(original, cloned);
}

#[test]
fn cloned_structural_record_retains_diagnostic_source_order() {
    let original = Type::StructuralRecord(StructuralRecordType::new(vec![
        ("z".to_owned(), Type::List(Box::new(Type::Int))),
        ("a".to_owned(), Type::Str),
    ]));
    let cloned = original.clone();
    assert_eq!(cloned, original);
    let Type::StructuralRecord(record) = cloned else {
        panic!("record fixture");
    };
    assert_eq!(
        record.fields().iter().map(|f| f.name()).collect::<Vec<_>>(),
        vec!["a", "z"]
    );
    assert_eq!(
        record
            .source_fields()
            .iter()
            .map(|f| f.name())
            .collect::<Vec<_>>(),
        vec!["z", "a"]
    );
}

#[test]
fn cloned_recursive_alias_retains_symbolic_boundary() {
    let original = Type::Alias {
        name: "Chain".to_owned(),
        type_args: vec![Type::Int],
        body: Box::new(Type::Tuple(vec![
            Type::Int,
            Type::Alias {
                name: "Chain".to_owned(),
                type_args: vec![Type::Int],
                body: Box::new(Type::Unknown),
            },
        ])),
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
    assert_eq!(format!("{original:?}"), format!("{cloned:?}"));
}
