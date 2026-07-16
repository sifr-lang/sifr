use super::{type_check_comparison, Type};
use sifr_diagnostics::DiagnosticCode;

#[test]
fn equality_rejects_container_any_shape_without_partial_eq() {
    assert!(type_check_comparison(
        &Type::List(Box::new(Type::Int)),
        "==",
        &Type::List(Box::new(Type::Any)),
    )
    .is_err());
    assert!(type_check_comparison(
        &Type::List(Box::new(Type::List(Box::new(Type::Int)))),
        "==",
        &Type::List(Box::new(Type::List(Box::new(Type::Any)))),
    )
    .is_err());
    assert!(type_check_comparison(
        &Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        "==",
        &Type::Dict(Box::new(Type::Str), Box::new(Type::Any)),
    )
    .is_err());
}

#[test]
fn equality_rejects_python_buffers_through_nested_aggregates() {
    let buffer = Type::PythonBuffer(Box::new(Type::FixedInt(crate::FixedIntType::U8)));
    let optional = Type::Union(vec![Type::None, buffer.clone()]);
    let collection = Type::List(Box::new(optional.clone()));
    let record = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "BufferRecord".to_string(),
        fields: vec![("views".to_string(), collection.clone())],
        methods: vec![],
        parent_class: None,
    };

    for ty in [buffer, optional, collection, record] {
        let error = type_check_comparison(&ty, "==", &ty).unwrap_err();
        assert_eq!(error.0, DiagnosticCode::TYPE_MISMATCH);
        assert!(error.1.contains("cannot compare affine values"));
    }
}
