use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn url_error_result(ok: Type) -> Type {
    result_ty(ok, "UrlError")
}

fn url_class() -> Type {
    Type::Class {
        name: "Url".to_string(),
        fields: vec![
            ("scheme".to_string(), Type::Str),
            ("username".to_string(), Type::Str),
            (
                "password".to_string(),
                Type::Union(vec![Type::Str, Type::None]),
            ),
            ("host".to_string(), Type::Str),
            ("port".to_string(), Type::Union(vec![Type::Int, Type::None])),
            ("path".to_string(), Type::Str),
            (
                "query".to_string(),
                Type::Union(vec![Type::Str, Type::None]),
            ),
            (
                "fragment".to_string(),
                Type::Union(vec![Type::Str, Type::None]),
            ),
            ("serialized".to_string(), Type::Str),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn query_pairs() -> Type {
    Type::List(Box::new(Type::Tuple(vec![Type::Str, Type::Str])))
}

pub(super) fn intrinsic_url() -> IntrinsicModule {
    let mut functions = HashMap::new();

    functions.insert(
        "url_parse".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            url_error_result(url_class()),
        ),
    );
    functions.insert(
        "url_build".to_string(),
        FunctionType::all_borrow(
            vec![
                ("scheme".to_string(), Type::Str),
                ("host".to_string(), Type::Str),
                ("path".to_string(), Type::Str),
                (
                    "query".to_string(),
                    Type::Union(vec![Type::Str, Type::None]),
                ),
                ("port".to_string(), Type::Union(vec![Type::Int, Type::None])),
            ],
            url_error_result(url_class()),
        ),
    );
    functions.insert(
        "url_percent_encode".to_string(),
        FunctionType::all_borrow(vec![("value".to_string(), Type::Str)], Type::Str),
    );
    functions.insert(
        "url_percent_decode".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            url_error_result(Type::Str),
        ),
    );
    functions.insert(
        "url_percent_encode_bytes".to_string(),
        FunctionType::all_borrow(vec![("value".to_string(), Type::Bytes)], Type::Str),
    );
    functions.insert(
        "url_percent_decode_bytes".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            url_error_result(Type::Bytes),
        ),
    );
    functions.insert(
        "url_normalize_path".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            url_error_result(Type::Str),
        ),
    );
    functions.insert(
        "url_query_parse".to_string(),
        FunctionType::all_borrow(
            vec![("query".to_string(), Type::Str)],
            url_error_result(query_pairs()),
        ),
    );
    functions.insert(
        "url_query_build".to_string(),
        FunctionType::all_borrow(
            vec![("pairs".to_string(), query_pairs())],
            url_error_result(Type::Str),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
