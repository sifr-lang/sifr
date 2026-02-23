//! IR-backed helpers for generating codegen preamble items.

use crate::{RustExpr, RustItem, RustMatchArm, RustParam, RustStmt, RustType, Type, Visibility};

pub fn sifr_type_to_rust_type(ty: &Type) -> RustType {
    match ty {
        Type::Int | Type::LiteralInt(_) => RustType::I64,
        Type::Float => RustType::F64,
        Type::Bool | Type::LiteralBool(_) => RustType::Bool,
        Type::Str | Type::LiteralStr(_) => RustType::String_,
        Type::None => RustType::Unit,
        Type::List(inner) => RustType::Vec(Box::new(sifr_type_to_rust_type(inner))),
        Type::Dict(key, value) => RustType::HashMap(
            Box::new(sifr_type_to_rust_type(key)),
            Box::new(sifr_type_to_rust_type(value)),
        ),
        Type::Set(inner) => RustType::HashSet(Box::new(sifr_type_to_rust_type(inner))),
        Type::Tuple(items) => RustType::Tuple(items.iter().map(sifr_type_to_rust_type).collect()),
        Type::Result(ok, err) => RustType::Result(
            Box::new(sifr_type_to_rust_type(ok)),
            Box::new(sifr_type_to_rust_type(err)),
        ),
        Type::Union(members) => {
            let non_none: Vec<&Type> = members
                .iter()
                .filter(|m| !matches!(m, Type::None))
                .collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                RustType::Option(Box::new(sifr_type_to_rust_type(non_none[0])))
            } else {
                RustType::Named(ty.rust_type())
            }
        }
        _ => RustType::Named(ty.rust_type()),
    }
}

pub fn build_error_type_items(
    name: &str,
    extra_fields: &[(String, RustType)],
    constructor_defaults: &[(String, RustExpr)],
) -> Vec<RustItem> {
    let mut fields = vec![("message".to_string(), RustType::String_)];
    fields.extend(extra_fields.iter().cloned());

    let mut init_fields = vec![("message".to_string(), RustExpr::Ident("message".to_string()))];
    init_fields.extend(constructor_defaults.iter().cloned());

    vec![
        RustItem::Struct {
            name: name.to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields,
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "new".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![RustParam::Named {
                    name: "message".to_string(),
                    ty: RustType::String_,
                }],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                    name: "Self".to_string(),
                    fields: init_fields,
                }))],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: Some("std::fmt::Display".to_string()),
            items: vec![RustItem::Fn {
                name: "fmt".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![
                    RustParam::SelfParam { mutable: false },
                    RustParam::Named {
                        name: "f".to_string(),
                        ty: RustType::Ref {
                            mutable: true,
                            inner: Box::new(RustType::Named("std::fmt::Formatter<'_>".to_string())),
                        },
                    },
                ],
                ret: Some(RustType::Named("std::fmt::Result".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fmt".to_string(),
                        "Display".to_string(),
                        "fmt".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "message".to_string(),
                            }),
                        },
                        RustExpr::Ident("f".to_string()),
                    ],
                }))],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: Some("std::error::Error".to_string()),
            items: vec![],
        },
    ]
}

pub fn build_io_error_items() -> Vec<RustItem> {
    let mut items = build_error_type_items(
        "IOError",
        &[("kind".to_string(), RustType::String_)],
        &[(
            "kind".to_string(),
            RustExpr::Literal(crate::RustLiteral::Str("Other".to_string())),
        )],
    );

    items.push(RustItem::Fn {
        name: "__io_err".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "e".to_string(),
            ty: RustType::Named("std::io::Error".to_string()),
        }],
        ret: Some(RustType::Named("IOError".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "msg".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("e".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "kind".to_string(),
                ty: None,
                value: RustExpr::If {
                    cond: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("e".to_string())),
                            method: "kind".to_string(),
                            args: vec![],
                        }),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "io".to_string(),
                            "ErrorKind".to_string(),
                            "NotFound".to_string(),
                        ])),
                    }),
                    then_expr: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                        "FileNotFound".to_string(),
                    ))),
                    else_expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "kind".to_string(),
                                args: vec![],
                            }),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "io".to_string(),
                                "ErrorKind".to_string(),
                                "PermissionDenied".to_string(),
                            ])),
                        }),
                        then_expr: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                            "PermissionDenied".to_string(),
                        ))),
                        else_expr: Some(Box::new(RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("e".to_string())),
                                    method: "kind".to_string(),
                                    args: vec![],
                                }),
                                op: "==".to_string(),
                                right: Box::new(RustExpr::Path(vec![
                                    "std".to_string(),
                                    "io".to_string(),
                                    "ErrorKind".to_string(),
                                    "AlreadyExists".to_string(),
                                ])),
                            }),
                            then_expr: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                                "FileExists".to_string(),
                            ))),
                            else_expr: Some(Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                                "Other".to_string(),
                            )))),
                        })),
                    })),
                },
            },
            RustStmt::Return(Some(RustExpr::StructInit {
                name: "IOError".to_string(),
                fields: vec![
                    ("message".to_string(), RustExpr::Ident("msg".to_string())),
                    ("kind".to_string(), RustExpr::Ident("kind".to_string())),
                ],
            })),
        ],
        is_async: false,
    });

    items
}

pub fn build_file_handle_infra_items() -> Vec<RustItem> {
    vec![
        RustItem::Enum {
            name: "SifrFileHandle".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            repr: None,
            variants: vec![
                crate::RustEnumVariant {
                    name: "TextRead".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "std::io::BufReader<std::fs::File>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "TextWrite".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "std::io::BufWriter<std::fs::File>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "BinaryRead".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "std::io::BufReader<std::fs::File>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "BinaryWrite".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "std::io::BufWriter<std::fs::File>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
            ],
        },
        RustItem::Static {
            name: "__SIFR_FILE_HANDLES".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, SifrFileHandle>>>"
                    .to_string(),
            ),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "LazyLock".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Closure {
                    params: vec![],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "sync".to_string(),
                            "Mutex".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "collections".to_string(),
                                "HashMap".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        }],
                    }),
                    is_move: false,
                }],
            },
        },
    ]
}

pub fn build_file_handle_struct_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "FileHandle".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields: vec![
                ("_handle".to_string(), RustType::I64),
                ("_mode".to_string(), RustType::String_),
            ],
        },
        RustItem::Impl {
            target: "FileHandle".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "new".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::Named {
                            name: "_handle".to_string(),
                            ty: RustType::I64,
                        },
                        RustParam::Named {
                            name: "_mode".to_string(),
                            ty: RustType::String_,
                        },
                    ],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            ("_handle".to_string(), RustExpr::Ident("_handle".to_string())),
                            ("_mode".to_string(), RustExpr::Ident("_mode".to_string())),
                        ],
                    }))],
                    is_async: false,
                },
                file_handle_read_method(),
                file_handle_write_method(),
                file_handle_readline_method(),
                file_handle_readlines_method(),
                RustItem::Fn {
                    name: "close".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: None,
                    body: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__hid".to_string(),
                            ty: None,
                            value: RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "_handle".to_string(),
                            },
                        },
                        RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "__SIFR_FILE_HANDLES".to_string(),
                                        "lock".to_string(),
                                    ])),
                                    args: vec![],
                                }),
                                method: "unwrap".to_string(),
                                args: vec![],
                            }),
                            method: "remove".to_string(),
                            args: vec![RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(RustExpr::Ident("__hid".to_string())),
                            }],
                        }),
                    ],
                    is_async: false,
                },
                file_handle_read_bytes_method(),
                file_handle_write_bytes_method(),
                RustItem::Fn {
                    name: "__enter__".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::Ref {
                        mutable: false,
                        inner: Box::new(RustType::Named("Self".to_string())),
                    }),
                    body: vec![RustStmt::Return(Some(RustExpr::Ident("self".to_string())))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__exit__".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: None,
                    body: vec![RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("self".to_string())),
                        method: "close".to_string(),
                        args: vec![],
                    })],
                    is_async: false,
                },
            ],
        },
    ]
}

pub fn build_logging_items() -> Vec<RustItem> {
    vec![RustItem::Static {
        name: "__SIFR_GLOBAL_LOG_LEVEL".to_string(),
        visibility: Visibility::Private,
        ty: RustType::Named("std::sync::LazyLock<std::sync::Mutex<i64>>".to_string()),
        value: RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "sync".to_string(),
                "LazyLock".to_string(),
                "new".to_string(),
            ])),
            args: vec![RustExpr::Closure {
                params: vec![],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "sync".to_string(),
                        "Mutex".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpr::Literal(crate::RustLiteral::Int(20))],
                }),
                is_move: false,
            }],
        },
    }]
}

fn file_handle_read_method() -> RustItem {
    RustItem::Fn {
        name: "read".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::String_),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "__SIFR_FILE_HANDLES".to_string(),
                            "lock".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "unwrap".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::TextRead(ref mut __r))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "__s".to_string(),
                                ty: None,
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "String".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "io".to_string(),
                                        "Read".to_string(),
                                        "read_to_string".to_string(),
                                    ])),
                                    args: vec![
                                        RustExpr::Ident("__r".to_string()),
                                        RustExpr::Ref {
                                            mutable: true,
                                            expr: Box::new(RustExpr::Ident("__s".to_string())),
                                        },
                                    ],
                                }),
                                method: "map_err".to_string(),
                                args: vec![RustExpr::Ident("__io_err".to_string())],
                            }))),
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Ident("__s".to_string())],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for reading".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

fn file_handle_write_method() -> RustItem {
    RustItem::Fn {
        name: "write".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::SelfParam { mutable: false },
            RustParam::Named {
                name: "data".to_string(),
                ty: RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::String_),
                },
            },
        ],
        ret: Some(RustType::Result(
            Box::new(RustType::Unit),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "__SIFR_FILE_HANDLES".to_string(),
                            "lock".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "unwrap".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::TextWrite(ref mut __w))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "io".to_string(),
                                        "Write".to_string(),
                                        "write_all".to_string(),
                                    ])),
                                    args: vec![
                                        RustExpr::Ident("__w".to_string()),
                                        RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("data".to_string())),
                                            method: "as_bytes".to_string(),
                                            args: vec![],
                                        },
                                    ],
                                }),
                                method: "map_err".to_string(),
                                args: vec![RustExpr::Ident("__io_err".to_string())],
                            }))),
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for writing".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

fn file_handle_readline_method() -> RustItem {
    RustItem::Fn {
        name: "readline".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::Option(Box::new(RustType::String_))),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "__SIFR_FILE_HANDLES".to_string(),
                            "lock".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "unwrap".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::TextRead(ref mut __r))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "__line".to_string(),
                                ty: None,
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "String".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            RustStmt::Let {
                                mutable: false,
                                name: "__n".to_string(),
                                ty: None,
                                value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "std".to_string(),
                                            "io".to_string(),
                                            "BufRead".to_string(),
                                            "read_line".to_string(),
                                        ])),
                                        args: vec![
                                            RustExpr::Ident("__r".to_string()),
                                            RustExpr::Ref {
                                                mutable: true,
                                                expr: Box::new(RustExpr::Ident("__line".to_string())),
                                            },
                                        ],
                                    }),
                                    method: "map_err".to_string(),
                                    args: vec![RustExpr::Ident("__io_err".to_string())],
                                })),
                            },
                            RustStmt::If {
                                cond: RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__n".to_string())),
                                    op: "==".to_string(),
                                    right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                                },
                                then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                    args: vec![RustExpr::Literal(crate::RustLiteral::None)],
                                }))],
                                else_body: None,
                            },
                            RustStmt::If {
                                cond: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__line".to_string())),
                                    method: "ends_with".to_string(),
                                    args: vec![RustExpr::Literal(crate::RustLiteral::Char('\n'))],
                                },
                                then_body: vec![
                                    RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__line".to_string())),
                                        method: "pop".to_string(),
                                        args: vec![],
                                    }),
                                    RustStmt::If {
                                        cond: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__line".to_string())),
                                            method: "ends_with".to_string(),
                                            args: vec![RustExpr::Literal(
                                                crate::RustLiteral::Char('\r'),
                                            )],
                                        },
                                        then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__line".to_string())),
                                            method: "pop".to_string(),
                                            args: vec![],
                                        })],
                                        else_body: None,
                                    },
                                ],
                                else_body: None,
                            },
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                                    args: vec![RustExpr::Ident("__line".to_string())],
                                }],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for reading".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

fn file_handle_read_bytes_method() -> RustItem {
    RustItem::Fn {
        name: "read_bytes".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::Vec(Box::new(RustType::I64))),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "__SIFR_FILE_HANDLES".to_string(),
                            "lock".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "unwrap".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::BinaryRead(ref mut __r))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "__buf".to_string(),
                                ty: None,
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "Vec::<u8>".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "io".to_string(),
                                        "Read".to_string(),
                                        "read_to_end".to_string(),
                                    ])),
                                    args: vec![
                                        RustExpr::Ident("__r".to_string()),
                                        RustExpr::Ref {
                                            mutable: true,
                                            expr: Box::new(RustExpr::Ident("__buf".to_string())),
                                        },
                                    ],
                                }),
                                method: "map_err".to_string(),
                                args: vec![RustExpr::Ident("__io_err".to_string())],
                            }))),
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__buf".to_string(),
                                            )),
                                            method: "into_iter".to_string(),
                                            args: vec![],
                                        }),
                                        method: "map".to_string(),
                                        args: vec![RustExpr::Closure {
                                            params: vec![RustParam::Named {
                                                name: "b".to_string(),
                                                ty: RustType::Named("_".to_string()),
                                            }],
                                            body: Box::new(RustExpr::Cast {
                                                expr: Box::new(RustExpr::Ident("b".to_string())),
                                                ty: RustType::I64,
                                            }),
                                            is_move: false,
                                        }],
                                    }),
                                    method: "collect::<Vec<i64>>".to_string(),
                                    args: vec![],
                                }],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for binary reading".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

fn file_handle_write_bytes_method() -> RustItem {
    RustItem::Fn {
        name: "write_bytes".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::SelfParam { mutable: false },
            RustParam::Named {
                name: "data".to_string(),
                ty: RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::Vec(Box::new(RustType::I64))),
                },
            },
        ],
        ret: Some(RustType::Result(
            Box::new(RustType::Unit),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "__SIFR_FILE_HANDLES".to_string(),
                            "lock".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "unwrap".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::BinaryWrite(ref mut __w))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Let {
                                mutable: false,
                                name: "__bytes".to_string(),
                                ty: Some(RustType::Vec(Box::new(RustType::Named("u8".to_string())))),
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("data".to_string())),
                                        method: "iter".to_string(),
                                        args: vec![],
                                    }),
                                    method: "map".to_string(),
                                    args: vec![RustExpr::Closure {
                                        params: vec![RustParam::Named {
                                            name: "b".to_string(),
                                            ty: RustType::Named("_".to_string()),
                                        }],
                                        body: Box::new(RustExpr::Cast {
                                            expr: Box::new(RustExpr::Deref(Box::new(
                                                RustExpr::Ident("b".to_string()),
                                            ))),
                                            ty: RustType::Named("u8".to_string()),
                                        }),
                                        is_move: false,
                                    }],
                                },
                            },
                            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "io".to_string(),
                                        "Write".to_string(),
                                        "write_all".to_string(),
                                    ])),
                                    args: vec![
                                        RustExpr::Ident("__w".to_string()),
                                        RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(RustExpr::Ident("__bytes".to_string())),
                                        },
                                    ],
                                }),
                                method: "map_err".to_string(),
                                args: vec![RustExpr::Ident("__io_err".to_string())],
                            }))),
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for binary writing".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

fn file_handle_readlines_method() -> RustItem {
    RustItem::Fn {
        name: "readlines".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::Vec(Box::new(RustType::String_))),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "__SIFR_FILE_HANDLES".to_string(),
                            "lock".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "unwrap".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::TextRead(ref mut __r))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "__lines".to_string(),
                                ty: Some(RustType::Vec(Box::new(RustType::String_))),
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "Vec::<String>".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            RustStmt::Let {
                                mutable: true,
                                name: "__line".to_string(),
                                ty: None,
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "String".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            RustStmt::Loop {
                                body: vec![
                                    RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__line".to_string())),
                                        method: "clear".to_string(),
                                        args: vec![],
                                    }),
                                    RustStmt::Let {
                                        mutable: false,
                                        name: "__n".to_string(),
                                        ty: None,
                                        value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::FnCall {
                                                func: Box::new(RustExpr::Path(vec![
                                                    "std".to_string(),
                                                    "io".to_string(),
                                                    "BufRead".to_string(),
                                                    "read_line".to_string(),
                                                ])),
                                                args: vec![
                                                    RustExpr::Ident("__r".to_string()),
                                                    RustExpr::Ref {
                                                        mutable: true,
                                                        expr: Box::new(RustExpr::Ident(
                                                            "__line".to_string(),
                                                        )),
                                                    },
                                                ],
                                            }),
                                            method: "map_err".to_string(),
                                            args: vec![RustExpr::Ident("__io_err".to_string())],
                                        })),
                                    },
                                    RustStmt::If {
                                        cond: RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__n".to_string())),
                                            op: "==".to_string(),
                                            right: Box::new(RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        },
                                        then_body: vec![RustStmt::Break],
                                        else_body: None,
                                    },
                                    RustStmt::Let {
                                        mutable: true,
                                        name: "__l".to_string(),
                                        ty: None,
                                        value: RustExpr::Clone(Box::new(RustExpr::Ident(
                                            "__line".to_string(),
                                        ))),
                                    },
                                    RustStmt::If {
                                        cond: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__l".to_string())),
                                            method: "ends_with".to_string(),
                                            args: vec![RustExpr::Literal(
                                                crate::RustLiteral::Char('\n'),
                                            )],
                                        },
                                        then_body: vec![
                                            RustStmt::Expr(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "__l".to_string(),
                                                )),
                                                method: "pop".to_string(),
                                                args: vec![],
                                            }),
                                            RustStmt::If {
                                                cond: RustExpr::MethodCall {
                                                    receiver: Box::new(RustExpr::Ident(
                                                        "__l".to_string(),
                                                    )),
                                                    method: "ends_with".to_string(),
                                                    args: vec![RustExpr::Literal(
                                                        crate::RustLiteral::Char('\r'),
                                                    )],
                                                },
                                                then_body: vec![RustStmt::Expr(
                                                    RustExpr::MethodCall {
                                                        receiver: Box::new(RustExpr::Ident(
                                                            "__l".to_string(),
                                                        )),
                                                        method: "pop".to_string(),
                                                        args: vec![],
                                                    },
                                                )],
                                                else_body: None,
                                            },
                                        ],
                                        else_body: None,
                                    },
                                    RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__lines".to_string())),
                                        method: "push".to_string(),
                                        args: vec![RustExpr::Ident("__l".to_string())],
                                    }),
                                ],
                            },
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Ident("__lines".to_string())],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for reading".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_items;
    use std::collections::BTreeSet;

    fn count_raw_in_type(ty: &RustType) -> usize {
        match ty {
            RustType::Vec(inner)
            | RustType::HashSet(inner)
            | RustType::VecDeque(inner)
            | RustType::Option(inner) => count_raw_in_type(inner),
            RustType::HashMap(k, v) | RustType::Result(k, v) => {
                count_raw_in_type(k) + count_raw_in_type(v)
            }
            RustType::Tuple(items) => items.iter().map(count_raw_in_type).sum(),
            RustType::Ref { inner, .. } => count_raw_in_type(inner),
            RustType::Generic { params, .. } | RustType::Fn { params, .. } => {
                params.iter().map(count_raw_in_type).sum()
            }
            RustType::RawCode(_) => 1,
            _ => 0,
        }
    }

    fn count_raw_in_expr(expr: &RustExpr) -> usize {
        match expr {
            RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) => 0,
            RustExpr::MethodCall { receiver, args, .. } | RustExpr::FnCall { func: receiver, args } => {
                count_raw_in_expr(receiver) + args.iter().map(count_raw_in_expr).sum::<usize>()
            }
            RustExpr::MacroCall { args, .. } | RustExpr::Vec(args) | RustExpr::Tuple(args) => {
                args.iter().map(count_raw_in_expr).sum()
            }
            RustExpr::FormatMacro { args, .. } => args.iter().map(count_raw_in_expr).sum(),
            RustExpr::BinOp { left, right, .. } => count_raw_in_expr(left) + count_raw_in_expr(right),
            RustExpr::UnaryOp { operand, .. }
            | RustExpr::Deref(operand)
            | RustExpr::Clone(operand)
            | RustExpr::Try(operand)
            | RustExpr::Await(operand) => count_raw_in_expr(operand),
            RustExpr::Field { expr, .. } => count_raw_in_expr(expr),
            RustExpr::Index { expr, index } => count_raw_in_expr(expr) + count_raw_in_expr(index),
            RustExpr::Ref { expr, .. } => count_raw_in_expr(expr),
            RustExpr::Cast { expr, ty } => count_raw_in_expr(expr) + count_raw_in_type(ty),
            RustExpr::Block { stmts, expr } => {
                stmts.iter().map(count_raw_in_stmt).sum::<usize>()
                    + expr.as_ref().map(|e| count_raw_in_expr(e)).unwrap_or(0)
            }
            RustExpr::If {
                cond,
                then_expr,
                else_expr,
            } => {
                count_raw_in_expr(cond)
                    + count_raw_in_expr(then_expr)
                    + else_expr.as_ref().map(|e| count_raw_in_expr(e)).unwrap_or(0)
            }
            RustExpr::Match { expr, arms } => {
                count_raw_in_expr(expr)
                    + arms
                        .iter()
                        .map(|a| a.body.iter().map(count_raw_in_stmt).sum::<usize>())
                        .sum::<usize>()
            }
            RustExpr::Closure { body, .. } => count_raw_in_expr(body),
            RustExpr::ClosureBlock { body, .. } => body.iter().map(count_raw_in_stmt).sum(),
            RustExpr::StructInit { fields, .. } => fields.iter().map(|(_, v)| count_raw_in_expr(v)).sum(),
            RustExpr::Range { start, end } => count_raw_in_expr(start) + count_raw_in_expr(end),
            RustExpr::RawCode(_) => 1,
        }
    }

    fn count_raw_in_stmt(stmt: &RustStmt) -> usize {
        match stmt {
            RustStmt::Let { ty, value, .. } => {
                ty.as_ref().map(count_raw_in_type).unwrap_or(0) + count_raw_in_expr(value)
            }
            RustStmt::LetPattern { value, .. } => count_raw_in_expr(value),
            RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
                count_raw_in_expr(target) + count_raw_in_expr(value)
            }
            RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => count_raw_in_expr(expr),
            RustStmt::Assert { cond, msg } => {
                count_raw_in_expr(cond) + msg.as_ref().map(count_raw_in_expr).unwrap_or(0)
            }
            RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => 0,
            RustStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                count_raw_in_expr(cond)
                    + then_body.iter().map(count_raw_in_stmt).sum::<usize>()
                    + else_body
                        .as_ref()
                        .map(|b| b.iter().map(count_raw_in_stmt).sum::<usize>())
                        .unwrap_or(0)
            }
            RustStmt::IfLet {
                expr,
                then_body,
                else_body,
                ..
            } => {
                count_raw_in_expr(expr)
                    + then_body.iter().map(count_raw_in_stmt).sum::<usize>()
                    + else_body
                        .as_ref()
                        .map(|b| b.iter().map(count_raw_in_stmt).sum::<usize>())
                        .unwrap_or(0)
            }
            RustStmt::Match { expr, arms } => {
                count_raw_in_expr(expr)
                    + arms
                        .iter()
                        .map(|a| a.body.iter().map(count_raw_in_stmt).sum::<usize>())
                        .sum::<usize>()
            }
            RustStmt::For { iter, body, .. } | RustStmt::While { cond: iter, body } => {
                count_raw_in_expr(iter) + body.iter().map(count_raw_in_stmt).sum::<usize>()
            }
            RustStmt::Loop { body } | RustStmt::Block(body) => body.iter().map(count_raw_in_stmt).sum(),
            RustStmt::RawCode(_) => 1,
        }
    }

    fn count_raw_in_item(item: &RustItem) -> usize {
        match item {
            RustItem::Struct { fields, .. } => fields.iter().map(|(_, t)| count_raw_in_type(t)).sum(),
            RustItem::TupleStruct { inner, .. } => count_raw_in_type(inner),
            RustItem::Enum { variants, .. } => variants
                .iter()
                .map(|v| {
                    v.tuple_fields.iter().map(count_raw_in_type).sum::<usize>()
                        + v.fields.iter().map(|(_, t)| count_raw_in_type(t)).sum::<usize>()
                        + v.value.as_ref().map(count_raw_in_expr).unwrap_or(0)
                })
                .sum(),
            RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
                methods.iter().map(count_raw_in_item).sum()
            }
            RustItem::Fn {
                params, ret, body, ..
            } => {
                params
                    .iter()
                    .map(|p| match p {
                        RustParam::SelfParam { .. } => 0,
                        RustParam::Named { ty, .. } => count_raw_in_type(ty),
                    })
                    .sum::<usize>()
                    + ret.as_ref().map(count_raw_in_type).unwrap_or(0)
                    + body.iter().map(count_raw_in_stmt).sum::<usize>()
            }
            RustItem::Const { ty, value, .. } | RustItem::Static { ty, value, .. } => {
                count_raw_in_type(ty) + count_raw_in_expr(value)
            }
            RustItem::Use(_) | RustItem::Attr(_) => 0,
            RustItem::RawCode(_) => 1,
        }
    }

    #[test]
    fn maps_types_to_structured_rust_types() {
        assert_eq!(sifr_type_to_rust_type(&Type::Int), RustType::I64);
        assert_eq!(
            sifr_type_to_rust_type(&Type::List(Box::new(Type::Str))),
            RustType::Vec(Box::new(RustType::String_))
        );
        assert_eq!(
            sifr_type_to_rust_type(&Type::Union(vec![Type::Int, Type::None])),
            RustType::Option(Box::new(RustType::I64))
        );
    }

    #[test]
    fn error_items_render_expected_shapes() {
        let items = build_error_type_items(
            "RegexError",
            &[("detail".to_string(), RustType::String_)],
            &[(
                "detail".to_string(),
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            )],
        );
        let rendered = render_items(&items);
        assert!(rendered.contains("struct RegexError"));
        assert!(rendered.contains("fn new(message: String) -> Self"));
        assert!(rendered.contains("impl std::error::Error for RegexError"));
    }

    #[test]
    fn file_handle_items_render_core_symbols() {
        let mut items = build_file_handle_infra_items();
        items.extend(build_file_handle_struct_items());
        let rendered = render_items(&items);
        assert!(rendered.contains("enum SifrFileHandle"));
        assert!(rendered.contains("static __SIFR_FILE_HANDLES"));
        assert!(rendered.contains("impl FileHandle"));
        assert!(rendered.contains("fn read(&self) -> Result<String, IOError>"));
    }

    #[test]
    fn preamble_rawcode_is_zero() {
        let mut all = build_io_error_items();
        all.extend(build_file_handle_infra_items());
        all.extend(build_file_handle_struct_items());
        all.extend(build_logging_items());
        let total_raw: usize = all.iter().map(count_raw_in_item).sum();
        assert_eq!(
            total_raw, 0,
            "expected preamble RawCode count to be zero, got {total_raw}"
        );

        let mut raw_method_names = BTreeSet::new();
        for item in build_file_handle_struct_items() {
            if let RustItem::Impl { items, .. } = item {
                for method in items {
                    if let RustItem::Fn { name, body, .. } = method {
                        if body.iter().any(|stmt| match stmt {
                            RustStmt::Return(Some(RustExpr::RawCode(_))) => true,
                            _ => false,
                        }) {
                            raw_method_names.insert(name);
                        }
                    }
                }
            }
        }
        let expected: BTreeSet<String> = BTreeSet::new();
        assert_eq!(raw_method_names, expected);
    }
}
