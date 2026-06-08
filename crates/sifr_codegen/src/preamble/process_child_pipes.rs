//! Generated process pipe-reader preamble item builders.

use crate::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};

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

fn poisoned_lock_expr(static_name: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(static_name.to_string())),
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

fn process_child_handles_lock_expr() -> RustExpr {
    poisoned_lock_expr("__SIFR_PROCESS_CHILDREN")
}

fn process_pipe_readers_lock_expr() -> RustExpr {
    poisoned_lock_expr("__SIFR_PROCESS_PIPE_READERS")
}

fn process_pipe_writers_lock_expr() -> RustExpr {
    poisoned_lock_expr("__SIFR_PROCESS_PIPE_WRITERS")
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

fn process_handle_error(message: &str) -> RustExpr {
    process_error_expr(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: message.to_string(),
        args: vec![RustExpr::Ident("__handle".to_string())],
    })
}

pub(super) fn process_child_pipe_item(function_name: &str, field_name: &str) -> RustItem {
    RustItem::Fn {
        name: function_name.to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "handle".to_string(),
            ty: RustType::I64,
        }],
        ret: Some(RustType::Named("Result<i64, ProcessError>".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__handle".to_string(),
                ty: None,
                value: RustExpr::Ident("handle".to_string()),
            },
            RustStmt::Let {
                mutable: true,
                name: "__children".to_string(),
                ty: None,
                value: process_child_handles_lock_expr(),
            },
            RustStmt::Let {
                mutable: true,
                name: "__child".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__children".to_string())),
                        method: "get_mut".to_string(),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident("__handle".to_string())),
                        }],
                    }),
                    method: "ok_or_else".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![],
                        body: Box::new(process_handle_error(
                            "process child handle is closed or unknown: {}",
                        )),
                        is_move: false,
                    }],
                })),
            },
            RustStmt::Let {
                mutable: false,
                name: "__pipe".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__child".to_string())),
                            field: field_name.to_string(),
                        }),
                        method: "take".to_string(),
                        args: vec![],
                    }),
                    method: "ok_or_else".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![],
                        body: Box::new(process_handle_error(&format!(
                            "process {field_name} pipe is not available or already taken for child handle: {{}}"
                        ))),
                        is_move: false,
                    }],
                })),
            },
            RustStmt::Let {
                mutable: false,
                name: "__pipe_handle".to_string(),
                ty: None,
                value: next_process_id_expr(),
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(process_pipe_readers_lock_expr()),
                method: "insert".to_string(),
                args: vec![
                    RustExpr::Ident("__pipe_handle".to_string()),
                    RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                        args: vec![RustExpr::Ident("__pipe".to_string())],
                    },
                ],
            }),
            RustStmt::Return(Some(ok_expr(RustExpr::Ident("__pipe_handle".to_string())))),
        ],
        is_async: false,
    }
}

pub(super) fn process_child_stdin_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_child_stdin".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "handle".to_string(),
            ty: RustType::I64,
        }],
        ret: Some(RustType::Named("Result<i64, ProcessError>".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__handle".to_string(),
                ty: None,
                value: RustExpr::Ident("handle".to_string()),
            },
            RustStmt::Let {
                mutable: true,
                name: "__children".to_string(),
                ty: None,
                value: process_child_handles_lock_expr(),
            },
            RustStmt::Let {
                mutable: true,
                name: "__child".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__children".to_string())),
                        method: "get_mut".to_string(),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident("__handle".to_string())),
                        }],
                    }),
                    method: "ok_or_else".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![],
                        body: Box::new(process_handle_error(
                            "process child handle is closed or unknown: {}",
                        )),
                        is_move: false,
                    }],
                })),
            },
            RustStmt::Let {
                mutable: false,
                name: "__pipe".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__child".to_string())),
                            field: "stdin".to_string(),
                        }),
                        method: "take".to_string(),
                        args: vec![],
                    }),
                    method: "ok_or_else".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![],
                        body: Box::new(process_handle_error(
                            "process stdin pipe is not available or already taken for child handle: {}",
                        )),
                        is_move: false,
                    }],
                })),
            },
            RustStmt::Let {
                mutable: false,
                name: "__pipe_handle".to_string(),
                ty: None,
                value: next_process_id_expr(),
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(process_pipe_writers_lock_expr()),
                method: "insert".to_string(),
                args: vec![
                    RustExpr::Ident("__pipe_handle".to_string()),
                    RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                        args: vec![RustExpr::Ident("__pipe".to_string())],
                    },
                ],
            }),
            RustStmt::Return(Some(ok_expr(RustExpr::Ident("__pipe_handle".to_string())))),
        ],
        is_async: false,
    }
}

pub(super) fn process_pipe_read_all_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_pipe_read_all".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "handle".to_string(),
            ty: RustType::I64,
        }],
        ret: Some(RustType::Named("Result<Vec<u8>, ProcessError>".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__handle".to_string(),
                ty: None,
                value: RustExpr::Ident("handle".to_string()),
            },
            RustStmt::Let {
                mutable: false,
                name: "__maybe_pipe".to_string(),
                ty: None,
                value: RustExpr::Block {
                    stmts: vec![RustStmt::Let {
                        mutable: true,
                        name: "__pipes".to_string(),
                        ty: None,
                        value: process_pipe_readers_lock_expr(),
                    }],
                    expr: Some(Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__pipes".to_string())),
                        method: "remove".to_string(),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident("__handle".to_string())),
                        }],
                    })),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__pipe".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__maybe_pipe".to_string())),
                    method: "ok_or_else".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![],
                        body: Box::new(process_handle_error(
                            "process pipe reader handle is closed or unknown: {}",
                        )),
                        is_move: false,
                    }],
                })),
            },
            RustStmt::Let {
                mutable: true,
                name: "__buffer".to_string(),
                ty: Some(RustType::Vec(Box::new(RustType::Named("u8".to_string())))),
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::Try(Box::new(process_map_err(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "io".to_string(),
                    "Read".to_string(),
                    "read_to_end".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("__pipe".to_string())),
                    },
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("__buffer".to_string())),
                    },
                ],
            })))),
            RustStmt::Return(Some(ok_expr(RustExpr::Ident("__buffer".to_string())))),
        ],
        is_async: false,
    }
}

fn remove_writer_stmt() -> RustStmt {
    RustStmt::Let {
        mutable: false,
        name: "__maybe_pipe".to_string(),
        ty: None,
        value: RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: true,
                name: "__pipes".to_string(),
                ty: None,
                value: process_pipe_writers_lock_expr(),
            }],
            expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__pipes".to_string())),
                method: "remove".to_string(),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident("__handle".to_string())),
                }],
            })),
        },
    }
}

fn require_writer_stmt() -> RustStmt {
    RustStmt::Let {
        mutable: true,
        name: "__pipe".to_string(),
        ty: None,
        value: RustExpr::Try(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__maybe_pipe".to_string())),
            method: "ok_or_else".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![],
                body: Box::new(process_handle_error(
                    "process pipe writer handle is closed or unknown: {}",
                )),
                is_move: false,
            }],
        })),
    }
}

pub(super) fn process_pipe_write_all_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_pipe_write_all".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::Named {
                name: "handle".to_string(),
                ty: RustType::I64,
            },
            RustParam::Named {
                name: "data".to_string(),
                ty: RustType::Vec(Box::new(RustType::Named("u8".to_string()))),
            },
        ],
        ret: Some(RustType::Named("Result<(), ProcessError>".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__handle".to_string(),
                ty: None,
                value: RustExpr::Ident("handle".to_string()),
            },
            remove_writer_stmt(),
            require_writer_stmt(),
            RustStmt::Expr(RustExpr::Try(Box::new(process_map_err(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "io".to_string(),
                    "Write".to_string(),
                    "write_all".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("__pipe".to_string())),
                    },
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("data".to_string())),
                        method: "as_slice".to_string(),
                        args: vec![],
                    },
                ],
            })))),
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(process_pipe_writers_lock_expr()),
                method: "insert".to_string(),
                args: vec![
                    RustExpr::Ident("__handle".to_string()),
                    RustExpr::Ident("__pipe".to_string()),
                ],
            }),
            RustStmt::Return(Some(ok_expr(RustExpr::Tuple(vec![])))),
        ],
        is_async: false,
    }
}

pub(super) fn process_pipe_close_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_pipe_close".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "handle".to_string(),
            ty: RustType::I64,
        }],
        ret: Some(RustType::Named("Result<(), ProcessError>".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__handle".to_string(),
                ty: None,
                value: RustExpr::Ident("handle".to_string()),
            },
            remove_writer_stmt(),
            require_writer_stmt(),
            RustStmt::Return(Some(ok_expr(RustExpr::Tuple(vec![])))),
        ],
        is_async: false,
    }
}
