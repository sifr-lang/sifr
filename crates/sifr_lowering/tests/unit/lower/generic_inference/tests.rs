
use super::infer_type_var_bindings;
use sifr_type_system::Type;
use std::collections::HashMap;

#[test]
fn infers_iterable_typevar_from_list_argument() {
    let param = Type::Iterable(Box::new(Type::TypeVar("T".to_string())));
    let arg = Type::List(Box::new(Type::Int));
    let mut bindings = HashMap::new();
    infer_type_var_bindings(&param, &arg, &mut bindings);
    assert_eq!(bindings.get("T"), Some(&Type::Int));
}

#[test]
fn infers_nested_iterable_typevar_from_list_of_lists() {
    let param = Type::Iterable(Box::new(Type::Iterable(Box::new(Type::TypeVar(
        "T".to_string(),
    )))));
    let arg = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let mut bindings = HashMap::new();
    infer_type_var_bindings(&param, &arg, &mut bindings);
    assert_eq!(bindings.get("T"), Some(&Type::Int));
}

#[test]
fn infers_iterable_typevar_from_iterator_argument() {
    let param = Type::Iterable(Box::new(Type::TypeVar("T".to_string())));
    let arg = Type::Iterator(Box::new(Type::Str));
    let mut bindings = HashMap::new();
    infer_type_var_bindings(&param, &arg, &mut bindings);
    assert_eq!(bindings.get("T"), Some(&Type::Str));
}

#[test]
fn infers_typevar_from_optional_union_parameter_non_none_branch() {
    let param = Type::Union(vec![
        Type::List(Box::new(Type::TypeVar("T".to_string()))),
        Type::None,
    ]);
    let arg = Type::List(Box::new(Type::Int));
    let mut bindings = HashMap::new();
    infer_type_var_bindings(&param, &arg, &mut bindings);
    assert_eq!(bindings.get("T"), Some(&Type::Int));
}

#[test]
fn optional_union_parameter_does_not_bind_typevar_from_none_argument() {
    let param = Type::Union(vec![
        Type::List(Box::new(Type::TypeVar("T".to_string()))),
        Type::None,
    ]);
    let arg = Type::None;
    let mut bindings = HashMap::new();
    infer_type_var_bindings(&param, &arg, &mut bindings);
    assert!(bindings.is_empty());
}

#[test]
fn optional_union_context_binds_typevar_to_payload_only() {
    let param = Type::Union(vec![Type::TypeVar("T".to_string()), Type::None]);
    let arg = Type::Union(vec![Type::Int, Type::None]);
    let mut bindings = HashMap::new();
    infer_type_var_bindings(&param, &arg, &mut bindings);
    assert_eq!(bindings.get("T"), Some(&Type::Int));
}
