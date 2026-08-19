use super::{
    build_error_type_items, file_handle_read_bytes_method, file_handle_readlines_method,
    file_handle_write_bytes_method, RustExpr, RustItem, RustMatchArm, RustParam, RustStmt,
    RustType, Visibility,
};
use crate::RustTypeParam;
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
        type_params: vec![RustTypeParam {
            name: "E".to_string(),
            bounds: vec!["std::fmt::Display".to_string(), "'static".to_string()],
        }],
        params: vec![RustParam::Named {
            name: "e".to_string(),
            ty: RustType::Named("E".to_string()),
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
                value: io_error_kind_expr(),
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

fn io_error_kind_expr() -> RustExpr {
    let any_ref = RustExpr::Cast {
        expr: Box::new(RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Ident("e".to_string())),
        }),
        ty: RustType::Ref {
            mutable: false,
            inner: Box::new(RustType::DynTrait {
                trait_: crate::RustTrait::Named {
                    name: "std::any::Any".to_string(),
                    params: Vec::new(),
                    associated_types: Vec::new(),
                },
                auto_traits: Vec::new(),
            }),
        },
    };
    let error_kind = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(any_ref),
            method: "downcast_ref::<std::io::Error>".to_string(),
            args: vec![],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Path(vec![
            "std".to_string(),
            "io".to_string(),
            "Error".to_string(),
            "kind".to_string(),
        ])],
    };
    let cases = [
        ("NotFound", "FileNotFound"),
        ("PermissionDenied", "PermissionDenied"),
        ("AlreadyExists", "FileExists"),
        ("IsADirectory", "IsADirectory"),
        ("NotADirectory", "NotADirectory"),
        ("DirectoryNotEmpty", "DirectoryNotEmpty"),
    ];
    let mut arms = cases
        .into_iter()
        .map(|(kind, label)| crate::RustMatchArm {
            pattern: format!("Some(::std::io::ErrorKind::{kind})"),
            bindings: vec![],
            guard: None,
            body: vec![RustStmt::TailExpr(RustExpr::Literal(
                crate::RustLiteral::Str(label.to_string()),
            ))],
        })
        .collect::<Vec<_>>();
    arms.push(crate::RustMatchArm {
        pattern: "_".to_string(),
        bindings: vec![],
        guard: None,
        body: vec![RustStmt::TailExpr(RustExpr::Literal(
            crate::RustLiteral::Str("Other".to_string()),
        ))],
    });
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_io_kind".to_string(),
            ty: None,
            value: error_kind,
        }],
        expr: Some(Box::new(RustExpr::Match {
            expr: Box::new(RustExpr::Ident("__sifr_io_kind".to_string())),
            arms,
        })),
    }
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
        RustItem::Static {
            name: "__SIFR_NEXT_FILE_HANDLE_ID".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named("std::sync::atomic::AtomicI64".to_string()),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "atomic".to_string(),
                    "AtomicI64".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Literal(crate::RustLiteral::Int(1))],
            },
        },
        RustItem::Fn {
            name: "__sifr_next_file_handle_id".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::I64),
            body: vec![RustStmt::Return(Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__SIFR_NEXT_FILE_HANDLE_ID".to_string())),
                method: "fetch_add".to_string(),
                args: vec![
                    RustExpr::Literal(crate::RustLiteral::Int(1)),
                    RustExpr::Path(vec![
                        "std".to_string(),
                        "sync".to_string(),
                        "atomic".to_string(),
                        "Ordering".to_string(),
                        "SeqCst".to_string(),
                    ]),
                ],
            }))],
            is_async: false,
        },
    ]
}

pub fn build_file_handle_struct_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "__SifrIoFileHandle".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields: vec![
                ("_handle".to_string(), RustType::I64),
                ("_mode".to_string(), RustType::String_),
            ],
        },
        RustItem::Impl {
            target: "__SifrIoFileHandle".to_string(),
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
                            (
                                "_handle".to_string(),
                                RustExpr::Ident("_handle".to_string()),
                            ),
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
                            receiver: Box::new(file_handles_lock_expr()),
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

pub(crate) fn file_handles_lock_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__SIFR_FILE_HANDLES".to_string())),
            method: "lock".to_string(),
            args: vec![],
        }),
        method: "unwrap_or_else".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__err".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__err".to_string())),
                method: "into_inner".to_string(),
                args: vec![],
            }),
            is_move: false,
        }],
    }
}

pub(crate) fn file_handle_read_method() -> RustItem {
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
                value: file_handles_lock_expr(),
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

pub(crate) fn file_handle_write_method() -> RustItem {
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
                value: file_handles_lock_expr(),
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

pub(crate) fn file_handle_readline_method() -> RustItem {
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
                value: file_handles_lock_expr(),
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
                                            receiver: Box::new(RustExpr::Ident(
                                                "__line".to_string(),
                                            )),
                                            method: "ends_with".to_string(),
                                            args: vec![RustExpr::Literal(
                                                crate::RustLiteral::Char('\r'),
                                            )],
                                        },
                                        then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__line".to_string(),
                                            )),
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
