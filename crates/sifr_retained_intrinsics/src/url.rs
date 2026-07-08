use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn parse_error_result(ok: Type) -> Type {
    result_ty(ok, "ParseError")
}

fn string_list() -> Type {
    Type::List(Box::new(Type::Str))
}

pub(super) fn intrinsic_url() -> IntrinsicModule {
    let mut functions = HashMap::new();

    functions.insert(
        "url_parse".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            parse_error_result(string_list()),
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
            parse_error_result(string_list()),
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
            parse_error_result(Type::Str),
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
            parse_error_result(Type::Bytes),
        ),
    );
    functions.insert(
        "url_normalize_path".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            parse_error_result(Type::Str),
        ),
    );
    functions.insert(
        "url_query_parse".to_string(),
        FunctionType::all_borrow(
            vec![("query".to_string(), Type::Str)],
            parse_error_result(string_list()),
        ),
    );
    functions.insert(
        "url_query_build".to_string(),
        FunctionType::all_borrow(
            vec![("pairs".to_string(), string_list())],
            parse_error_result(Type::Str),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
