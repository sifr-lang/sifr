//! Runtime support for generated process status and child handles.

use crate::{RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, Visibility};

use super::process_child_pipes::{
    process_child_pipe_item, process_child_pipe_writer_item, process_pipe_close_item,
    process_pipe_read_all_item, process_pipe_write_all_item,
};

pub(crate) fn build_process_status_items() -> Vec<RustItem> {
    vec![
        RustItem::Attr("#[cfg(unix)]".to_string()),
        RustItem::Fn {
            name: "__sifr_process_exit_signal".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "status".to_string(),
                ty: RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::Named("std::process::ExitStatus".to_string())),
                },
            }],
            ret: Some(RustType::Option(Box::new(RustType::I64))),
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__signal".to_string(),
                    ty: None,
                    value: RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "os".to_string(),
                            "unix".to_string(),
                            "process".to_string(),
                            "ExitStatusExt".to_string(),
                            "signal".to_string(),
                        ])),
                        args: vec![RustExpr::Ident("status".to_string())],
                    },
                },
                RustStmt::Return(Some(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__signal".to_string())),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "__signal".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::Ident("__signal".to_string())),
                            ty: RustType::I64,
                        }),
                        is_move: false,
                    }],
                })),
            ],
            is_async: false,
        },
        RustItem::Attr("#[cfg(not(unix))]".to_string()),
        RustItem::Fn {
            name: "__sifr_process_exit_signal".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "_status".to_string(),
                ty: RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::Named("std::process::ExitStatus".to_string())),
                },
            }],
            ret: Some(RustType::Option(Box::new(RustType::I64))),
            body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))],
            is_async: false,
        },
    ]
}

fn string_ty() -> RustType {
    RustType::String_
}

fn string_vec_ty() -> RustType {
    RustType::Vec(Box::new(string_ty()))
}

fn process_error_expr(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "ProcessError".to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn process_map_err(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_process_error".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(process_error_expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_process_error".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            })),
            is_move: false,
        }],
    }
}

fn process_child_handles_lock_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__SIFR_PROCESS_CHILDREN".to_string())),
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

fn process_pipe_readers_lock_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__SIFR_PROCESS_PIPE_READERS".to_string())),
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

fn next_process_id_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "__sifr_next_process_child_id".to_string()
        ])),
        args: vec![],
    }
}

fn ok_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![expr],
    }
}

fn err_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![expr],
    }
}

fn process_handle_error(message: &str) -> RustExpr {
    process_error_expr(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: message.to_string(),
        args: vec![RustExpr::Ident("__handle".to_string())],
    })
}

fn process_sync_command_setup() -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: true,
            name: "__cmd".to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "process".to_string(),
                    "Command".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident("program".to_string())),
                }],
            },
        },
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "args".to_string(),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("args".to_string())),
                method: "iter".to_string(),
                args: vec![],
            }],
        }),
        RustStmt::For {
            var: "__sifr_process_env".to_string(),
            iter: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("env".to_string())),
                method: "iter".to_string(),
                args: vec![],
            },
            body: vec![RustStmt::IfLet {
                pattern: "Some((__sifr_process_env_key, __sifr_process_env_value))".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_process_env".to_string())),
                    method: "split_once".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Char('='))],
                },
                then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                    method: "env".to_string(),
                    args: vec![
                        RustExpr::Ident("__sifr_process_env_key".to_string()),
                        RustExpr::Ident("__sifr_process_env_value".to_string()),
                    ],
                })],
                else_body: None,
            }],
        },
        RustStmt::If {
            cond: RustExpr::Ident("has_cwd".to_string()),
            then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                method: "current_dir".to_string(),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident("cwd".to_string())),
                }],
            })],
            else_body: None,
        },
    ]
}

fn stdio_mode_return(mode: &str, constructor: &str) -> RustStmt {
    RustStmt::If {
        cond: RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("mode".to_string())),
            op: "==".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Str(mode.to_string()))),
        },
        then_body: vec![RustStmt::Return(Some(ok_expr(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "process".to_string(),
                "Stdio".to_string(),
                constructor.to_string(),
            ])),
            args: vec![],
        })))],
        else_body: None,
    }
}

fn process_stdio_from_mode_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_stdio_from_mode".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "mode".to_string(),
            ty: string_ty(),
        }],
        ret: Some(RustType::Named(
            "Result<std::process::Stdio, ProcessError>".to_string(),
        )),
        body: vec![
            stdio_mode_return("pipe", "piped"),
            stdio_mode_return("inherit", "inherit"),
            stdio_mode_return("null", "null"),
            RustStmt::Return(Some(err_expr(process_error_expr(RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: "unsupported process stdio mode: {}".to_string(),
                args: vec![RustExpr::Ident("mode".to_string())],
            })))),
        ],
        is_async: false,
    }
}

fn process_spawn_item() -> RustItem {
    let mut body = process_sync_command_setup();
    body.extend([
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "stdin".to_string(),
            args: vec![RustExpr::Try(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "__sifr_process_stdio_from_mode".to_string()
                ])),
                args: vec![RustExpr::Ident("stdin_mode".to_string())],
            }))],
        }),
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "stdout".to_string(),
            args: vec![RustExpr::Try(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "__sifr_process_stdio_from_mode".to_string()
                ])),
                args: vec![RustExpr::Ident("stdout_mode".to_string())],
            }))],
        }),
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "stderr".to_string(),
            args: vec![RustExpr::Try(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "__sifr_process_stdio_from_mode".to_string()
                ])),
                args: vec![RustExpr::Ident("stderr_mode".to_string())],
            }))],
        }),
        RustStmt::Let {
            mutable: false,
            name: "__child".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                method: "spawn".to_string(),
                args: vec![],
            }))),
        },
        RustStmt::Let {
            mutable: false,
            name: "__handle".to_string(),
            ty: None,
            value: next_process_id_expr(),
        },
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(process_child_handles_lock_expr()),
            method: "insert".to_string(),
            args: vec![
                RustExpr::Ident("__handle".to_string()),
                RustExpr::Ident("__child".to_string()),
            ],
        }),
        RustStmt::Return(Some(ok_expr(RustExpr::Ident("__handle".to_string())))),
    ]);
    RustItem::Fn {
        name: "__sifr_process_spawn".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::Named {
                name: "program".to_string(),
                ty: string_ty(),
            },
            RustParam::Named {
                name: "args".to_string(),
                ty: string_vec_ty(),
            },
            RustParam::Named {
                name: "env".to_string(),
                ty: string_vec_ty(),
            },
            RustParam::Named {
                name: "cwd".to_string(),
                ty: string_ty(),
            },
            RustParam::Named {
                name: "has_cwd".to_string(),
                ty: RustType::Bool,
            },
            RustParam::Named {
                name: "stdin_mode".to_string(),
                ty: string_ty(),
            },
            RustParam::Named {
                name: "stdout_mode".to_string(),
                ty: string_ty(),
            },
            RustParam::Named {
                name: "stderr_mode".to_string(),
                ty: string_ty(),
            },
        ],
        ret: Some(RustType::Named("Result<i64, ProcessError>".to_string())),
        body,
        is_async: false,
    }
}

pub(crate) fn build_process_child_items() -> Vec<RustItem> {
    vec![
        RustItem::Static {
            name: "__SIFR_PROCESS_CHILDREN".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, std::process::Child>>>"
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
            name: "__SIFR_PROCESS_PIPE_READERS".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, Box<dyn std::io::Read + Send>>>>"
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
            name: "__SIFR_PROCESS_PIPE_WRITERS".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, Box<dyn std::io::Write + Send>>>>"
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
            name: "__SIFR_NEXT_PROCESS_CHILD_ID".to_string(),
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
                args: vec![RustExpr::Literal(RustLiteral::Int(1))],
            },
        },
        RustItem::Fn {
            name: "__sifr_next_process_child_id".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::I64),
            body: vec![RustStmt::Return(Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(
                    "__SIFR_NEXT_PROCESS_CHILD_ID".to_string(),
                )),
                method: "fetch_add".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Int(1)),
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
        process_stdio_from_mode_item(),
        process_spawn_item(),
        process_child_pipe_writer_item("__sifr_process_child_stdin", "stdin"),
        process_child_pipe_item("__sifr_process_child_stdout", "stdout"),
        process_child_pipe_item("__sifr_process_child_stderr", "stderr"),
        process_pipe_read_all_item(),
        process_pipe_write_all_item(),
        process_pipe_close_item(),
    ]
}
