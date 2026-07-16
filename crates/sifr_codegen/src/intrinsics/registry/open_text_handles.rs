//! Explicit text-open intrinsic lowerer.

use crate::{RustExpr, RustLiteral, RustMatchArm, RustStmt};

fn owned_str(arg: &RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(arg.clone()),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn string_literal_expr(value: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Literal(RustLiteral::Str(value.to_string()))),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn path_as_str_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__path".to_string())),
        method: "as_str".to_string(),
        args: vec![],
    }
}

fn binary_mode_as_str_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__binary_mode".to_string())),
        method: "as_str".to_string(),
        args: vec![],
    }
}

fn open_file_handle_expr() -> RustExpr {
    RustExpr::Try(Box::new(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_stdlib".to_string(),
                "fs".to_string(),
                "open_file".to_string(),
            ])),
            args: vec![path_as_str_expr(), binary_mode_as_str_expr()],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    }))
}

fn invalid_mode_error_expr() -> RustExpr {
    RustExpr::FnCall {
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
    }
}

fn native_file_handle_new_expr(handle_id: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "__SifrIoNativeFileHandle".to_string(),
            "new".to_string(),
        ])),
        args: vec![handle_id],
    }
}

fn binary_file_handle_new_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "__SifrIoBinaryFileHandle".to_string(),
            "new".to_string(),
        ])),
        args: vec![
            native_file_handle_new_expr(RustExpr::Ident("__handle_id".to_string())),
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__binary_mode".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            },
        ],
    }
}

fn success_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "Ok::<__SifrIoTextFileHandle, IOError>".to_string(),
        ])),
        args: vec![RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "__SifrIoTextFileHandle".to_string(),
                "new".to_string(),
            ])),
            args: vec![
                binary_file_handle_new_expr(),
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
        }],
    }
}

fn open_text_arm(pattern: &str, binary_mode: &str) -> RustMatchArm {
    RustMatchArm {
        pattern: pattern.to_string(),
        bindings: vec![],
        guard: None,
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__binary_mode".to_string(),
                ty: None,
                value: string_literal_expr(binary_mode),
            },
            RustStmt::Let {
                mutable: false,
                name: "__handle_id".to_string(),
                ty: None,
                value: open_file_handle_expr(),
            },
            RustStmt::Return(Some(success_expr())),
        ],
    }
}

fn build_open_text_match() -> RustStmt {
    RustStmt::Match {
        expr: RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__mode".to_string())),
            method: "as_str".to_string(),
            args: vec![],
        },
        arms: vec![
            open_text_arm("\"r\" | \"rt\"", "rb"),
            open_text_arm("\"w\" | \"wt\"", "wb"),
            open_text_arm("\"a\" | \"at\"", "ab"),
            RustMatchArm {
                pattern: "_".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![RustStmt::Return(Some(invalid_mode_error_expr()))],
            },
        ],
    }
}

pub(crate) fn lower_builtin_open_text(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
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
                build_open_text_match(),
            ],
            is_move: false,
            is_async: false,
        }),
        args: vec![],
    })))
}
