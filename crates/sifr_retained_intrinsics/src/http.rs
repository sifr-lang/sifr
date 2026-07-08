use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn header_error_result(ok: Type) -> Type {
    result_ty(ok, "HeaderError")
}

fn http_error_result(ok: Type) -> Type {
    result_ty(ok, "HttpError")
}

fn method_class() -> Type {
    Type::Class {
        name: "Method".to_string(),
        fields: vec![("value".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    }
}

fn status_class() -> Type {
    Type::Class {
        name: "Status".to_string(),
        fields: vec![("code".to_string(), Type::Int)],
        methods: vec![],
        parent_class: None,
    }
}

fn version_class() -> Type {
    Type::Class {
        name: "Version".to_string(),
        fields: vec![("value".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    }
}

fn header_name_class() -> Type {
    Type::Class {
        name: "HeaderName".to_string(),
        fields: vec![("value".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    }
}

fn header_value_class() -> Type {
    Type::Class {
        name: "HeaderValue".to_string(),
        fields: vec![("value".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    }
}

fn header_entries() -> Type {
    Type::List(Box::new(Type::Tuple(vec![
        header_name_class(),
        header_value_class(),
    ])))
}

fn header_map_class() -> Type {
    Type::Class {
        name: "HeaderMap".to_string(),
        fields: vec![("entries".to_string(), header_entries())],
        methods: vec![],
        parent_class: None,
    }
}

fn raw_header_pairs() -> Type {
    Type::List(Box::new(Type::Tuple(vec![Type::Str, Type::Str])))
}

fn cookie_pairs() -> Type {
    Type::List(Box::new(Type::Tuple(vec![Type::Str, Type::Str])))
}

pub(super) fn intrinsic_http() -> IntrinsicModule {
    let mut functions = HashMap::new();

    functions.insert(
        "http_validate_method".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            http_error_result(method_class()),
        ),
    );
    functions.insert(
        "http_validate_status".to_string(),
        FunctionType::all_borrow(
            vec![("code".to_string(), Type::Int)],
            http_error_result(status_class()),
        ),
    );
    functions.insert(
        "http_validate_version".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            http_error_result(version_class()),
        ),
    );
    functions.insert(
        "http_validate_header_name".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            header_error_result(header_name_class()),
        ),
    );
    functions.insert(
        "http_validate_header_value".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            header_error_result(header_value_class()),
        ),
    );
    functions.insert(
        "http_header_map_from_pairs".to_string(),
        FunctionType::all_borrow(
            vec![("pairs".to_string(), raw_header_pairs())],
            header_error_result(header_map_class()),
        ),
    );
    functions.insert(
        "http_parse_cookie_header".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            header_error_result(cookie_pairs()),
        ),
    );
    functions.insert(
        "http_build_cookie_header".to_string(),
        FunctionType::all_borrow(
            vec![("cookies".to_string(), cookie_pairs())],
            header_error_result(Type::Str),
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
