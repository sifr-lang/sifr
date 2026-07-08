use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn tls_error_result(ok: Type) -> Type {
    result_ty(ok, "TlsError")
}

fn await_tls_result(ok: Type) -> Type {
    Type::Awaitable(Box::new(tls_error_result(ok)))
}

fn tls_client_config_class() -> Type {
    Type::Class {
        name: "TlsClientConfig".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn tls_server_config_class() -> Type {
    Type::Class {
        name: "TlsServerConfig".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn tls_stream_class() -> Type {
    Type::Class {
        name: "TlsStream".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn tls_read_half_class() -> Type {
    Type::Class {
        name: "TlsReadHalf".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn tls_write_half_class() -> Type {
    Type::Class {
        name: "TlsWriteHalf".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

pub(super) fn intrinsic_tls() -> IntrinsicModule {
    let mut functions = HashMap::new();

    functions.insert(
        "tls_client_config_platform".to_string(),
        FunctionType::all_borrow(
            vec![(
                "alpn_protocols".to_string(),
                Type::List(Box::new(Type::Bytes)),
            )],
            tls_error_result(tls_client_config_class()),
        ),
    );
    functions.insert(
        "tls_client_config_with_roots".to_string(),
        FunctionType::all_borrow(
            vec![
                ("root_pem".to_string(), Type::Bytes),
                (
                    "alpn_protocols".to_string(),
                    Type::List(Box::new(Type::Bytes)),
                ),
            ],
            tls_error_result(tls_client_config_class()),
        ),
    );
    functions.insert(
        "tls_client_config_with_roots_and_client_auth".to_string(),
        FunctionType::all_borrow(
            vec![
                ("root_pem".to_string(), Type::Bytes),
                ("cert_pem".to_string(), Type::Bytes),
                ("key_pem".to_string(), Type::Bytes),
                (
                    "alpn_protocols".to_string(),
                    Type::List(Box::new(Type::Bytes)),
                ),
            ],
            tls_error_result(tls_client_config_class()),
        ),
    );
    functions.insert(
        "tls_server_config".to_string(),
        FunctionType::all_borrow(
            vec![
                ("cert_pem".to_string(), Type::Bytes),
                ("key_pem".to_string(), Type::Bytes),
                (
                    "alpn_protocols".to_string(),
                    Type::List(Box::new(Type::Bytes)),
                ),
            ],
            tls_error_result(tls_server_config_class()),
        ),
    );
    functions.insert(
        "tls_server_config_require_client_auth".to_string(),
        FunctionType::all_borrow(
            vec![
                ("cert_pem".to_string(), Type::Bytes),
                ("key_pem".to_string(), Type::Bytes),
                ("client_ca_pem".to_string(), Type::Bytes),
                (
                    "alpn_protocols".to_string(),
                    Type::List(Box::new(Type::Bytes)),
                ),
            ],
            tls_error_result(tls_server_config_class()),
        ),
    );
    functions.insert(
        "tls_client_config_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            tls_error_result(Type::None),
        ),
    );
    functions.insert(
        "tls_server_config_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            tls_error_result(Type::None),
        ),
    );
    functions.insert(
        "tls_connect".to_string(),
        FunctionType::all_borrow(
            vec![
                ("config_handle".to_string(), Type::Int),
                ("tcp_handle".to_string(), Type::Int),
                ("server_name".to_string(), Type::Str),
                ("timeout_seconds".to_string(), Type::Float),
                ("has_timeout".to_string(), Type::Bool),
            ],
            await_tls_result(tls_stream_class()),
        ),
    );
    functions.insert(
        "tls_accept".to_string(),
        FunctionType::all_borrow(
            vec![
                ("config_handle".to_string(), Type::Int),
                ("tcp_handle".to_string(), Type::Int),
                ("timeout_seconds".to_string(), Type::Float),
                ("has_timeout".to_string(), Type::Bool),
            ],
            await_tls_result(tls_stream_class()),
        ),
    );
    functions.insert(
        "tls_stream_read_chunk".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("max_bytes".to_string(), Type::Int),
            ],
            await_tls_result(Type::Union(vec![Type::Bytes, Type::None])),
        ),
    );
    functions.insert(
        "tls_stream_write".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            await_tls_result(Type::Int),
        ),
    );
    functions.insert(
        "tls_stream_write_all".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            await_tls_result(Type::None),
        ),
    );
    functions.insert(
        "tls_stream_flush".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_tls_result(Type::None),
        ),
    );
    functions.insert(
        "tls_stream_close_notify".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_tls_result(Type::None),
        ),
    );
    functions.insert(
        "tls_stream_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_tls_result(Type::None),
        ),
    );
    functions.insert(
        "tls_stream_split".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            Type::Tuple(vec![tls_read_half_class(), tls_write_half_class()]),
        ),
    );
    functions.insert(
        "tls_stream_alpn_protocol".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            tls_error_result(Type::Union(vec![Type::Bytes, Type::None])),
        ),
    );
    functions.insert(
        "tls_stream_protocol_version".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            tls_error_result(Type::Union(vec![Type::Str, Type::None])),
        ),
    );
    functions.insert(
        "tls_read_half_read_chunk".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("max_bytes".to_string(), Type::Int),
            ],
            await_tls_result(Type::Union(vec![Type::Bytes, Type::None])),
        ),
    );
    functions.insert(
        "tls_read_half_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            tls_error_result(Type::None),
        ),
    );
    functions.insert(
        "tls_write_half_write".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            await_tls_result(Type::Int),
        ),
    );
    functions.insert(
        "tls_write_half_write_all".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            await_tls_result(Type::None),
        ),
    );
    functions.insert(
        "tls_write_half_flush".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_tls_result(Type::None),
        ),
    );
    functions.insert(
        "tls_write_half_close_notify".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_tls_result(Type::None),
        ),
    );
    functions.insert(
        "tls_write_half_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_tls_result(Type::None),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
