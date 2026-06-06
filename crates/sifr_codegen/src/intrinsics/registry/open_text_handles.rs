//! Explicit text-open intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType};

fn owned_str(arg: &RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(arg.clone()),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn ok_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![value],
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
    RustStmt::Return(Some(RustExpr::FnCall {
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
    }))
}

fn open_text_arm(
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
            RustStmt::Return(Some(success_expr.clone())),
        ],
    }
}

fn build_open_text_match(success_expr: &RustExpr) -> RustStmt {
    let path_ref = path_as_str_expr();
    RustStmt::Match {
        expr: mode_as_str_expr(),
        arms: vec![
            open_text_arm(
                "\"r\" | \"rt\"",
                open_file_expr(path_ref.clone()),
                "BinaryRead",
                success_expr,
            ),
            open_text_arm(
                "\"w\" | \"wt\"",
                create_file_expr(path_ref.clone()),
                "BinaryWrite",
                success_expr,
            ),
            open_text_arm(
                "\"a\" | \"at\"",
                append_file_expr(path_ref),
                "BinaryWrite",
                success_expr,
            ),
            RustMatchArm {
                pattern: "_".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![invalid_mode_error_expr()],
            },
        ],
    }
}

pub(crate) fn lower_builtin_open_text(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    let binary_handle = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "BinaryFileHandle".to_string(),
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
    let success_expr = ok_expr(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "TextFileHandle".to_string(),
            "new".to_string(),
        ])),
        args: vec![
            binary_handle,
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "Encoding".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Ident("__encoding".to_string())],
            },
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "DecodeErrorHandler".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Clone(Box::new(RustExpr::Ident(
                    "__errors".to_string(),
                )))],
            },
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "EncodeErrorHandler".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Ident("__errors".to_string())],
            },
        ],
    });
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
                    name: "__encoding".to_string(),
                    ty: None,
                    value: owned_str(&args[2]),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__errors".to_string(),
                    ty: None,
                    value: owned_str(&args[3]),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__handle_id".to_string(),
                    ty: None,
                    value: next_handle_id_expr(),
                },
                build_open_text_match(&success_expr),
            ],
            is_move: false,
            is_async: false,
        }),
        args: vec![],
    })))
}
