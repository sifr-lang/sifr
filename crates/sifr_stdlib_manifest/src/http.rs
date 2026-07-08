use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn header_error_result(ok: Type) -> Type {
    result_ty(ok, "HeaderError")
}

fn http_error_result(ok: Type) -> Type {
    result_ty(ok, "HttpError")
}

fn await_http_result(ok: Type) -> Type {
    Type::Awaitable(Box::new(http_error_result(ok)))
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

fn transport_headers() -> Type {
    Type::List(Box::new(Type::Tuple(vec![Type::Str, Type::Str])))
}

fn transport_response_tuple() -> Type {
    Type::Tuple(vec![Type::Int, Type::Str, transport_headers(), Type::Bytes])
}

fn transport_request_tuple() -> Type {
    Type::Tuple(vec![
        Type::Str,
        Type::Str,
        Type::Str,
        transport_headers(),
        Type::Bytes,
    ])
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
    for name in [
        "http1_client_roundtrip_tcp",
        "http2_client_roundtrip_tcp",
        "http1_client_roundtrip_tls",
        "http2_client_roundtrip_tls",
    ] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![
                    ("handle".to_string(), Type::Int),
                    ("method".to_string(), Type::Str),
                    ("path".to_string(), Type::Str),
                    ("headers".to_string(), transport_headers()),
                    ("body".to_string(), Type::Bytes),
                    ("max_request_bytes".to_string(), Type::Int),
                    ("max_response_bytes".to_string(), Type::Int),
                ],
                await_http_result(transport_response_tuple()),
            ),
        );
    }
    for name in [
        "http1_server_respond_tcp",
        "http2_server_respond_tcp",
        "http1_server_respond_tls",
        "http2_server_respond_tls",
    ] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![
                    ("handle".to_string(), Type::Int),
                    ("status".to_string(), Type::Int),
                    ("headers".to_string(), transport_headers()),
                    ("body".to_string(), Type::Bytes),
                    ("max_request_bytes".to_string(), Type::Int),
                    ("max_response_bytes".to_string(), Type::Int),
                ],
                await_http_result(transport_request_tuple()),
            ),
        );
    }

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
