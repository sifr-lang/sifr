//! File-handle intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType};

fn owned_str(arg: &RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(arg.clone()),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn io_other_error_expr(message: &str) -> RustExpr {
    RustExpr::StructInit {
        name: "IOError".to_string(),
        fields: vec![
            (
                "message".to_string(),
                RustExpr::Literal(RustLiteral::Str(message.to_string())),
            ),
            (
                "kind".to_string(),
                RustExpr::Literal(RustLiteral::Str("Other".to_string())),
            ),
        ],
    }
}

fn std_io_trait_call(trait_name: &str, method_name: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "io".to_string(),
            trait_name.to_string(),
            method_name.to_string(),
        ])),
        args,
    }
}

fn map_io_err_try(expr: RustExpr) -> RustExpr {
    RustExpr::Try(Box::new(RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    }))
}

fn ok_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![value],
    }
}

fn trim_trailing_crlf_stmt(name: &str) -> RustStmt {
    RustStmt::If {
        cond: RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(name.to_string())),
            method: "ends_with".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Char('\n'))],
        },
        then_body: vec![
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(name.to_string())),
                method: "pop".to_string(),
                args: vec![],
            }),
            RustStmt::If {
                cond: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(name.to_string())),
                    method: "ends_with".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Char('\r'))],
                },
                then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(name.to_string())),
                    method: "pop".to_string(),
                    args: vec![],
                })],
                else_body: None,
            },
        ],
        else_body: None,
    }
}

fn next_handle_id_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "__sifr_next_file_handle_id".to_string()
        ])),
        args: vec![],
    }
}

fn wrap_handle_result(
    hid_expr: RustExpr,
    arm_pattern: &str,
    arm_body: Vec<RustStmt>,
    err_message: &str,
) -> RustExpr {
    let err_expr = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![io_other_error_expr(err_message)],
    };
    let mut body = Vec::new();
    body.push(RustStmt::Let {
        mutable: false,
        name: "__hid".to_string(),
        ty: None,
        value: hid_expr,
    });
    body.push(RustStmt::Let {
        mutable: true,
        name: "__handles".to_string(),
        ty: None,
        value: file_handles_lock_expr(),
    });
    body.push(RustStmt::Match {
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
                pattern: format!("Some(SifrFileHandle::{arm_pattern})"),
                bindings: vec![],
                guard: None,
                body: arm_body,
            },
            RustMatchArm {
                pattern: "_".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![RustStmt::Return(Some(err_expr))],
            },
        ],
    });
    RustExpr::FnCall {
        func: Box::new(RustExpr::ClosureBlock {
            params: vec![],
            body,
            is_move: false,
            is_async: false,
        }),
        args: vec![],
    }
}

fn file_handles_lock_expr() -> RustExpr {
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

fn path_as_str_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__path".to_string())),
        method: "as_str".to_string(),
        args: vec![],
    }
}

fn mode_as_str_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__mode".to_string())),
        method: "as_str".to_string(),
        args: vec![],
    }
}

fn open_file_expr(path_expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "File".to_string(),
            "open".to_string(),
        ])),
        args: vec![path_expr],
    }
}

fn create_file_expr(path_expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "File".to_string(),
            "create".to_string(),
        ])),
        args: vec![path_expr],
    }
}

fn append_file_expr(path_expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fs".to_string(),
                        "OpenOptions".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                }),
                method: "append".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Bool(true))],
            }),
            method: "create".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Bool(true))],
        }),
        method: "open".to_string(),
        args: vec![path_expr],
    }
}

fn invalid_mode_error_expr() -> RustStmt {
    let err_expr = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![RustExpr::StructInit {
            name: "IOError".to_string(),
            fields: vec![
                (
                    "message".to_string(),
                    RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: "invalid mode: {}".to_string(),
                        args: vec![RustExpr::Ident("__mode".to_string())],
                    },
                ),
                (
                    "kind".to_string(),
                    RustExpr::Literal(RustLiteral::Str("Other".to_string())),
                ),
            ],
        }],
    };
    RustStmt::Return(Some(err_expr))
}

fn open_arm(
    pattern: &str,
    open_expr: RustExpr,
    variant: &str,
    success_expr: &RustExpr,
) -> RustMatchArm {
    let (buffer_ctor, buffer_var) = if variant.ends_with("Read") {
        (
            vec![
                "std".to_string(),
                "io".to_string(),
                "BufReader".to_string(),
                "new".to_string(),
            ],
            "__reader",
        )
    } else {
        (
            vec![
                "std".to_string(),
                "io".to_string(),
                "BufWriter".to_string(),
                "new".to_string(),
            ],
            "__writer",
        )
    };
    let success_stmt = RustStmt::Return(Some(success_expr.clone()));
    RustMatchArm {
        pattern: pattern.to_string(),
        bindings: vec![],
        guard: None,
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__f".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(open_expr),
                    method: "map_err".to_string(),
                    args: vec![RustExpr::Ident("__io_err".to_string())],
                })),
            },
            RustStmt::Let {
                mutable: false,
                name: buffer_var.to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(buffer_ctor)),
                    args: vec![RustExpr::Ident("__f".to_string())],
                },
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(file_handles_lock_expr()),
                method: "insert".to_string(),
                args: vec![
                    RustExpr::Ident("__handle_id".to_string()),
                    RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "SifrFileHandle".to_string(),
                            variant.to_string(),
                        ])),
                        args: vec![RustExpr::Ident(buffer_var.to_string())],
                    },
                ],
            }),
            success_stmt,
        ],
    }
}

fn build_open_match(success_expr: &RustExpr, invalid_stmt: RustStmt) -> RustStmt {
    let path_ref = path_as_str_expr();
    RustStmt::Match {
        expr: mode_as_str_expr(),
        arms: vec![
            open_arm(
                "\"r\" | \"rt\"",
                open_file_expr(path_ref.clone()),
                "TextRead",
                success_expr,
            ),
            open_arm(
                "\"w\" | \"wt\"",
                create_file_expr(path_ref.clone()),
                "TextWrite",
                success_expr,
            ),
            open_arm(
                "\"a\" | \"at\"",
                append_file_expr(path_ref.clone()),
                "TextWrite",
                success_expr,
            ),
            open_arm(
                "\"rb\"",
                open_file_expr(path_ref.clone()),
                "BinaryRead",
                success_expr,
            ),
            open_arm(
                "\"wb\"",
                create_file_expr(path_ref.clone()),
                "BinaryWrite",
                success_expr,
            ),
            open_arm(
                "\"ab\"",
                append_file_expr(path_ref),
                "BinaryWrite",
                success_expr,
            ),
            RustMatchArm {
                pattern: "_".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![invalid_stmt],
            },
        ],
    }
}

pub(crate) fn lower_builtin_open(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let success_value = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "FileHandle".to_string(),
            "new".to_string(),
        ])),
        args: vec![
            RustExpr::Ident("__handle_id".to_string()),
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__mode".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            },
        ],
    };
    let success_expr = ok_expr(success_value);
    Some(RustExpr::Try(Box::new(RustExpr::FnCall {
        func: Box::new(RustExpr::ClosureBlock {
            params: vec![],
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__path".to_string(),
                    ty: None,
                    value: owned_str(&args[0]),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__mode".to_string(),
                    ty: None,
                    value: owned_str(&args[1]),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__handle_id".to_string(),
                    ty: None,
                    value: next_handle_id_expr(),
                },
                build_open_match(&success_expr, invalid_mode_error_expr()),
            ],
            is_move: false,
            is_async: false,
        }),
        args: vec![],
    })))
}

pub(crate) fn lower_open_file(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let success_expr = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![RustExpr::Ident("__handle_id".to_string())],
    };
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::ClosureBlock {
            params: vec![],
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__path".to_string(),
                    ty: None,
                    value: owned_str(&args[0]),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__mode".to_string(),
                    ty: None,
                    value: owned_str(&args[1]),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__handle_id".to_string(),
                    ty: None,
                    value: next_handle_id_expr(),
                },
                build_open_match(&success_expr, invalid_mode_error_expr()),
            ],
            is_move: false,
            is_async: false,
        }),
        args: vec![],
    })
}

pub(crate) fn lower_file_read(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let read_body = vec![
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
        RustStmt::Expr(map_io_err_try(std_io_trait_call(
            "Read",
            "read_to_string",
            vec![
                RustExpr::Ident("__r".to_string()),
                RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(RustExpr::Ident("__s".to_string())),
                },
            ],
        ))),
        RustStmt::Return(Some(ok_expr(RustExpr::Ident("__s".to_string())))),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        "TextRead(ref mut __r)",
        read_body,
        "file not open for reading",
    ))
}

pub(crate) fn lower_file_write(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let write_body = vec![
        RustStmt::Let {
            mutable: false,
            name: "__data".to_string(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(args[1].clone()),
                method: "as_str".to_string(),
                args: vec![],
            },
        },
        RustStmt::Expr(map_io_err_try(std_io_trait_call(
            "Write",
            "write_all",
            vec![
                RustExpr::Ident("__w".to_string()),
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__data".to_string())),
                    method: "as_bytes".to_string(),
                    args: vec![],
                },
            ],
        ))),
        RustStmt::Return(Some(ok_expr(RustExpr::Literal(RustLiteral::Unit)))),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        "TextWrite(ref mut __w)",
        write_body,
        "file not open for writing",
    ))
}

pub(crate) fn lower_file_readline(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let readline_body = vec![
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
            value: map_io_err_try(std_io_trait_call(
                "BufRead",
                "read_line",
                vec![
                    RustExpr::Ident("__r".to_string()),
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("__line".to_string())),
                    },
                ],
            )),
        },
        RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__n".to_string())),
                op: "==".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
            },
            then_body: vec![RustStmt::Return(Some(ok_expr(RustExpr::Literal(
                RustLiteral::None,
            ))))],
            else_body: None,
        },
        trim_trailing_crlf_stmt("__line"),
        RustStmt::Return(Some(ok_expr(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![RustExpr::Ident("__line".to_string())],
        }))),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        "TextRead(ref mut __r)",
        readline_body,
        "file not open for reading",
    ))
}

pub(crate) fn lower_file_readlines(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let readlines_body = vec![
        RustStmt::Let {
            mutable: true,
            name: "__lines".to_string(),
            ty: Some(RustType::Vec(Box::new(RustType::String_))),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
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
                    value: map_io_err_try(std_io_trait_call(
                        "BufRead",
                        "read_line",
                        vec![
                            RustExpr::Ident("__r".to_string()),
                            RustExpr::Ref {
                                mutable: true,
                                expr: Box::new(RustExpr::Ident("__line".to_string())),
                            },
                        ],
                    )),
                },
                RustStmt::If {
                    cond: RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__n".to_string())),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    },
                    then_body: vec![RustStmt::Break],
                    else_body: None,
                },
                RustStmt::Let {
                    mutable: true,
                    name: "__l".to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__line".to_string())),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                },
                trim_trailing_crlf_stmt("__l"),
                RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__lines".to_string())),
                    method: "push".to_string(),
                    args: vec![RustExpr::Ident("__l".to_string())],
                }),
            ],
        },
        RustStmt::Return(Some(ok_expr(RustExpr::Ident("__lines".to_string())))),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        "TextRead(ref mut __r)",
        readlines_body,
        "file not open for reading",
    ))
}

pub(crate) fn lower_file_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: args[0].clone(),
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
        expr: Some(Box::new(RustExpr::Literal(RustLiteral::Unit))),
    })
}

pub(crate) fn lower_file_read_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let read_bytes_body = vec![
        RustStmt::Let {
            mutable: true,
            name: "__buf".to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                args: vec![],
            },
        },
        RustStmt::Expr(map_io_err_try(std_io_trait_call(
            "Read",
            "read_to_end",
            vec![
                RustExpr::Ident("__r".to_string()),
                RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(RustExpr::Ident("__buf".to_string())),
                },
            ],
        ))),
        RustStmt::Return(Some(ok_expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__buf".to_string())),
            method: "to_vec".to_string(),
            args: vec![],
        }))),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        "BinaryRead(ref mut __r)",
        read_bytes_body,
        "file not open for binary reading",
    ))
}

pub(crate) fn lower_file_write_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let write_bytes_body = vec![
        RustStmt::Expr(map_io_err_try(std_io_trait_call(
            "Write",
            "write_all",
            vec![
                RustExpr::Ident("__w".to_string()),
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(args[1].clone()),
                },
            ],
        ))),
        RustStmt::Return(Some(ok_expr(RustExpr::Literal(RustLiteral::Unit)))),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        "BinaryWrite(ref mut __w)",
        write_bytes_body,
        "file not open for binary writing",
    ))
}
