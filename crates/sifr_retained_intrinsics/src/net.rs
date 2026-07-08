use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn net_error_result(ok: Type) -> Type {
    result_ty(ok, "NetError")
}

fn socket_addr_class() -> Type {
    Type::Class {
        name: "SocketAddr".to_string(),
        fields: vec![("value".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    }
}

fn tcp_stream_class() -> Type {
    Type::Class {
        name: "TcpStream".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn tcp_listener_class() -> Type {
    Type::Class {
        name: "TcpListener".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn tcp_read_half_class() -> Type {
    Type::Class {
        name: "TcpReadHalf".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn tcp_write_half_class() -> Type {
    Type::Class {
        name: "TcpWriteHalf".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_closed".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn await_net_result(ok: Type) -> Type {
    Type::Awaitable(Box::new(net_error_result(ok)))
}

pub(super) fn intrinsic_net() -> IntrinsicModule {
    let mut functions = HashMap::new();

    functions.insert(
        "net_connect_tcp".to_string(),
        FunctionType::all_borrow(
            vec![
                ("address".to_string(), Type::Str),
                ("timeout_seconds".to_string(), Type::Float),
                ("has_timeout".to_string(), Type::Bool),
                ("local_addr".to_string(), Type::Str),
                ("has_local_addr".to_string(), Type::Bool),
            ],
            await_net_result(tcp_stream_class()),
        ),
    );
    functions.insert(
        "net_listen_tcp".to_string(),
        FunctionType::all_borrow(
            vec![
                ("address".to_string(), Type::Str),
                ("backlog".to_string(), Type::Int),
                ("has_backlog".to_string(), Type::Bool),
                ("reuse_addr".to_string(), Type::Bool),
            ],
            await_net_result(tcp_listener_class()),
        ),
    );
    functions.insert(
        "net_listener_accept".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_net_result(Type::Tuple(vec![tcp_stream_class(), socket_addr_class()])),
        ),
    );
    functions.insert(
        "net_listener_local_addr".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            net_error_result(Type::Str),
        ),
    );
    functions.insert(
        "net_listener_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            net_error_result(Type::None),
        ),
    );
    functions.insert(
        "net_tcp_stream_read_chunk".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("max_bytes".to_string(), Type::Int),
            ],
            await_net_result(Type::Union(vec![Type::Bytes, Type::None])),
        ),
    );
    functions.insert(
        "net_tcp_stream_write".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            await_net_result(Type::Int),
        ),
    );
    functions.insert(
        "net_tcp_stream_write_all".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            await_net_result(Type::None),
        ),
    );
    functions.insert(
        "net_tcp_stream_shutdown_write".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_net_result(Type::None),
        ),
    );
    functions.insert(
        "net_tcp_stream_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_net_result(Type::None),
        ),
    );
    functions.insert(
        "net_tcp_stream_split".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            Type::Tuple(vec![tcp_read_half_class(), tcp_write_half_class()]),
        ),
    );
    functions.insert(
        "net_tcp_stream_local_addr".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            net_error_result(Type::Str),
        ),
    );
    functions.insert(
        "net_tcp_stream_peer_addr".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            net_error_result(Type::Str),
        ),
    );
    functions.insert(
        "net_tcp_read_half_read_chunk".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("max_bytes".to_string(), Type::Int),
            ],
            await_net_result(Type::Union(vec![Type::Bytes, Type::None])),
        ),
    );
    functions.insert(
        "net_tcp_read_half_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            net_error_result(Type::None),
        ),
    );
    functions.insert(
        "net_tcp_write_half_write".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            await_net_result(Type::Int),
        ),
    );
    functions.insert(
        "net_tcp_write_half_write_all".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            await_net_result(Type::None),
        ),
    );
    functions.insert(
        "net_tcp_write_half_shutdown_write".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            await_net_result(Type::None),
        ),
    );
    functions.insert(
        "net_tcp_write_half_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            net_error_result(Type::None),
        ),
    );
    functions.insert(
        "net_lookup_host".to_string(),
        FunctionType::all_borrow(
            vec![
                ("address".to_string(), Type::Str),
                ("timeout_seconds".to_string(), Type::Float),
                ("has_timeout".to_string(), Type::Bool),
            ],
            await_net_result(Type::List(Box::new(socket_addr_class()))),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
