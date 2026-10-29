use super::{
    RustExpr, RustItem, RustMatchArm, RustParam, RustStmt, RustType, Visibility,
    file_handles_lock_expr,
};

pub(crate) fn file_handle_read_bytes_method() -> RustItem {
    RustItem::Fn {
        name: "read_bytes".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::Vec(Box::new(RustType::Named("u8".to_string())))),
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
                                args: vec![RustExpr::Ident("__buf".to_string())],
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

pub(crate) fn file_handle_write_bytes_method() -> RustItem {
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
                    inner: Box::new(RustType::Slice(Box::new(RustType::Named("u8".to_string())))),
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
                        pattern: "Some(SifrFileHandle::BinaryWrite(ref mut __w))".to_string(),
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
                                        RustExpr::Ident("data".to_string()),
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

pub(crate) fn file_handle_readlines_method() -> RustItem {
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
#[path = "../../tests/unit/preamble/io_bytes_methods/tests.rs"]
mod tests;
