//! Native process intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn ref_expr(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

fn ref_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    ref_expr(arg_expr(args, idx))
}

fn string_lit(value: &str) -> RustExpr {
    RustExpr::Literal(RustLiteral::Str(value.to_string()))
}

fn bool_lit(value: bool) -> RustExpr {
    RustExpr::Literal(RustLiteral::Bool(value))
}

fn path_call(parts: &[&str], args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(
            parts.iter().map(|part| (*part).to_string()).collect(),
        )),
        args,
    }
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

fn next_child_handle_id_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "__sifr_next_process_child_id".to_string()
        ])),
        args: vec![],
    }
}

fn missing_child_error_expr() -> RustExpr {
    process_error_expr(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "process child handle is closed or unknown: {}".to_string(),
        args: vec![RustExpr::Ident("__handle".to_string())],
    })
}

fn command_new(program: RustExpr) -> RustExpr {
    path_call(&["std", "process", "Command", "new"], vec![program])
}

fn piped_stdio() -> RustExpr {
    path_call(&["std", "process", "Stdio", "piped"], vec![])
}

fn instant_now() -> RustExpr {
    path_call(&["std", "time", "Instant", "now"], vec![])
}

fn duration_try_from_secs_f64(seconds: RustExpr) -> RustExpr {
    RustExpr::Try(Box::new(process_map_err(path_call(
        &["std", "time", "Duration", "try_from_secs_f64"],
        vec![seconds],
    ))))
}

fn timeout_deadline_expr() -> RustExpr {
    RustExpr::Try(Box::new(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(instant_now()),
            method: "checked_add".to_string(),
            args: vec![duration_try_from_secs_f64(RustExpr::Ident(
                "__timeout_seconds".to_string(),
            ))],
        }),
        method: "ok_or_else".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![],
            body: Box::new(process_error_expr(string_lit(
                "process timeout is too large for this host clock",
            ))),
            is_move: false,
        }],
    }))
}

fn duration_from_millis(millis: i64) -> RustExpr {
    path_call(
        &["std", "time", "Duration", "from_millis"],
        vec![RustExpr::Literal(RustLiteral::Int(millis))],
    )
}

fn command_status_tuple(output_ident: &str) -> RustExpr {
    status_tuple(RustExpr::Field {
        expr: Box::new(RustExpr::Ident(output_ident.to_string())),
        field: "status".to_string(),
    })
}

fn status_tuple(status_expr: RustExpr) -> RustExpr {
    RustExpr::Tuple(vec![
        status_code(status_expr.clone()),
        status_signal(status_expr),
    ])
}

fn status_code(status_expr: RustExpr) -> RustExpr {
    RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(status_expr),
                method: "code".to_string(),
                args: vec![],
            }),
            method: "unwrap_or".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Int(-1))],
        }),
        ty: RustType::I64,
    }
}

fn status_signal(status_expr: RustExpr) -> RustExpr {
    path_call(
        &["__sifr_process_exit_signal"],
        vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(status_expr),
        }],
    )
}

fn normal_command_setup(args: &[RustExpr]) -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: true,
            name: "__cmd".to_string(),
            ty: None,
            value: command_new(ref_arg(args, 0)),
        },
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "args".to_string(),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(arg_expr(args, 1)),
                method: "iter".to_string(),
                args: vec![],
            }],
        }),
        RustStmt::For {
            var: "__sifr_process_env".to_string(),
            iter: RustExpr::MethodCall {
                receiver: Box::new(arg_expr(args, 2)),
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
            cond: arg_expr(args, 4),
            then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                method: "current_dir".to_string(),
                args: vec![ref_arg(args, 3)],
            })],
            else_body: None,
        },
    ]
}

fn shell_command_setup(args: &[RustExpr]) -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: true,
            name: "__cmd".to_string(),
            ty: None,
            value: command_new(string_lit("sh")),
        },
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "arg".to_string(),
            args: vec![string_lit("-c")],
        }),
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "arg".to_string(),
            args: vec![ref_arg(args, 0)],
        }),
    ]
}

fn output_setup_stmts() -> Vec<RustStmt> {
    vec![
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "stdin".to_string(),
            args: vec![piped_stdio()],
        }),
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "stdout".to_string(),
            args: vec![piped_stdio()],
        }),
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "stderr".to_string(),
            args: vec![piped_stdio()],
        }),
    ]
}

fn spawn_child_stmt() -> RustStmt {
    RustStmt::Let {
        mutable: true,
        name: "__child".to_string(),
        ty: None,
        value: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "spawn".to_string(),
            args: vec![],
        }))),
    }
}

fn write_stdin_stmt(stdin_expr: RustExpr, has_stdin_expr: RustExpr) -> RustStmt {
    RustStmt::If {
        cond: has_stdin_expr,
        then_body: vec![RustStmt::IfLet {
            pattern: "Some(mut __sifr_process_stdin)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("__child".to_string())),
                    field: "stdin".to_string(),
                }),
                method: "take".to_string(),
                args: vec![],
            },
            then_body: vec![RustStmt::Expr(RustExpr::Try(Box::new(process_map_err(
                path_call(
                    &["std", "io", "Write", "write_all"],
                    vec![
                        RustExpr::Ref {
                            mutable: true,
                            expr: Box::new(RustExpr::Ident("__sifr_process_stdin".to_string())),
                        },
                        RustExpr::MethodCall {
                            receiver: Box::new(stdin_expr),
                            method: "as_slice".to_string(),
                            args: vec![],
                        },
                    ],
                ),
            ))))],
            else_body: None,
        }],
        else_body: None,
    }
}

fn wait_output_stmt() -> RustStmt {
    RustStmt::Let {
        mutable: false,
        name: "__output".to_string(),
        ty: None,
        value: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__child".to_string())),
            method: "wait_with_output".to_string(),
            args: vec![],
        }))),
    }
}

fn spawn_output_stmts(stdin_expr: RustExpr, has_stdin_expr: RustExpr) -> Vec<RustStmt> {
    vec![
        spawn_child_stmt(),
        write_stdin_stmt(stdin_expr, has_stdin_expr),
        wait_output_stmt(),
    ]
}

fn output_tuple_expr() -> RustExpr {
    RustExpr::Tuple(vec![
        RustExpr::Field {
            expr: Box::new(RustExpr::Ident("__output".to_string())),
            field: "stdout".to_string(),
        },
        RustExpr::Field {
            expr: Box::new(RustExpr::Ident("__output".to_string())),
            field: "stderr".to_string(),
        },
        command_status_tuple("__output"),
    ])
}

fn output_timeout_tuple_expr() -> RustExpr {
    RustExpr::Tuple(vec![
        RustExpr::Field {
            expr: Box::new(RustExpr::Ident("__output".to_string())),
            field: "stdout".to_string(),
        },
        RustExpr::Field {
            expr: Box::new(RustExpr::Ident("__output".to_string())),
            field: "stderr".to_string(),
        },
        command_status_tuple("__output"),
        RustExpr::Ident("__timed_out".to_string()),
    ])
}

fn output_text_tuple_expr() -> RustExpr {
    RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__stdout".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(process_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "from_utf8".to_string(),
                    ])),
                    args: vec![RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("__output".to_string())),
                        field: "stdout".to_string(),
                    }],
                }))),
            },
            RustStmt::Let {
                mutable: false,
                name: "__stderr".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(process_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "from_utf8".to_string(),
                    ])),
                    args: vec![RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("__output".to_string())),
                        field: "stderr".to_string(),
                    }],
                }))),
            },
        ],
        expr: Some(Box::new(RustExpr::Tuple(vec![
            RustExpr::Ident("__stdout".to_string()),
            RustExpr::Ident("__stderr".to_string()),
            command_status_tuple("__output"),
        ]))),
    }
}

fn utf8_encoding_guard(encoding: RustExpr, success: RustExpr) -> RustExpr {
    RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__encoding".to_string(),
                ty: None,
                value: encoding,
            },
            RustStmt::Let {
                mutable: false,
                name: "__encoding_lower".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__encoding".to_string())),
                    method: "to_ascii_lowercase".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__encoding_lower".to_string())),
                    op: "!=".to_string(),
                    right: Box::new(string_lit("utf-8")),
                }),
                op: "&&".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__encoding_lower".to_string())),
                    op: "!=".to_string(),
                    right: Box::new(string_lit("utf8")),
                }),
            }),
            then_expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![process_error_expr(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str:
                        "process text output currently supports only UTF-8 encoding, got {}"
                            .to_string(),
                    args: vec![RustExpr::Ident("__encoding".to_string())],
                })],
            }),
            else_expr: Some(Box::new(success)),
        })),
    }
}

fn timeout_invalid_expr(timeout_seconds: RustExpr) -> RustExpr {
    RustExpr::BinOp {
        left: Box::new(RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(RustExpr::MethodCall {
                receiver: Box::new(timeout_seconds.clone()),
                method: "is_finite".to_string(),
                args: vec![],
            }),
        }),
        op: "||".to_string(),
        right: Box::new(RustExpr::BinOp {
            left: Box::new(timeout_seconds),
            op: "<".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
        }),
    }
}

fn timeout_guard(timeout_expr: RustExpr, success: RustExpr) -> RustExpr {
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__timeout_seconds".to_string(),
            ty: Some(RustType::F64),
            value: timeout_expr,
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(timeout_invalid_expr(RustExpr::Ident(
                "__timeout_seconds".to_string(),
            ))),
            then_expr: Box::new(err_expr(process_error_expr(RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: "process timeout must be finite and non-negative, got {}".to_string(),
                args: vec![RustExpr::Ident("__timeout_seconds".to_string())],
            }))),
            else_expr: Some(Box::new(success)),
        })),
    }
}

fn timeout_poll_stmts() -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: false,
            name: "__deadline".to_string(),
            ty: None,
            value: timeout_deadline_expr(),
        },
        RustStmt::Let {
            mutable: true,
            name: "__timed_out".to_string(),
            ty: Some(RustType::Bool),
            value: bool_lit(false),
        },
        RustStmt::Loop {
            body: vec![
                RustStmt::IfLet {
                    pattern: "Some(_)".to_string(),
                    expr: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__child".to_string())),
                        method: "try_wait".to_string(),
                        args: vec![],
                    }))),
                    then_body: vec![RustStmt::Break],
                    else_body: None,
                },
                RustStmt::If {
                    cond: RustExpr::BinOp {
                        left: Box::new(instant_now()),
                        op: ">=".to_string(),
                        right: Box::new(RustExpr::Ident("__deadline".to_string())),
                    },
                    then_body: vec![
                        RustStmt::Expr(RustExpr::Try(Box::new(process_map_err(
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__child".to_string())),
                                method: "kill".to_string(),
                                args: vec![],
                            },
                        )))),
                        RustStmt::Assign {
                            target: RustExpr::Ident("__timed_out".to_string()),
                            value: bool_lit(true),
                        },
                        RustStmt::Break,
                    ],
                    else_body: None,
                },
                RustStmt::Expr(path_call(
                    &["std", "thread", "sleep"],
                    vec![duration_from_millis(1)],
                )),
            ],
        },
    ]
}

pub(crate) fn lower_process_run(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 5 {
        return None;
    }
    let mut stmts = normal_command_setup(args);
    stmts.push(RustStmt::Let {
        mutable: false,
        name: "__status".to_string(),
        ty: None,
        value: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "status".to_string(),
            args: vec![],
        }))),
    });
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(ok_expr(status_tuple(RustExpr::Ident(
            "__status".to_string(),
        ))))),
    })
}

pub(crate) fn lower_process_spawn(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 5 {
        return None;
    }
    let mut stmts = normal_command_setup(args);
    stmts.extend([
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
            value: next_child_handle_id_expr(),
        },
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(process_child_handles_lock_expr()),
            method: "insert".to_string(),
            args: vec![
                RustExpr::Ident("__handle".to_string()),
                RustExpr::Ident("__child".to_string()),
            ],
        }),
    ]);
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(ok_expr(RustExpr::Ident("__handle".to_string())))),
    })
}

pub(crate) fn lower_process_wait(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let handle_expr = arg_expr(args, 0);
    let stmts = vec![
        RustStmt::Let {
            mutable: false,
            name: "__handle".to_string(),
            ty: None,
            value: handle_expr,
        },
        RustStmt::Let {
            mutable: false,
            name: "__maybe_child".to_string(),
            ty: None,
            value: RustExpr::Block {
                stmts: vec![RustStmt::Let {
                    mutable: true,
                    name: "__children".to_string(),
                    ty: None,
                    value: process_child_handles_lock_expr(),
                }],
                expr: Some(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__children".to_string())),
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
            name: "__child".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__maybe_child".to_string())),
                method: "ok_or_else".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![],
                    body: Box::new(missing_child_error_expr()),
                    is_move: false,
                }],
            })),
        },
        RustStmt::Let {
            mutable: false,
            name: "__status".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__child".to_string())),
                method: "wait".to_string(),
                args: vec![],
            }))),
        },
    ];
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(ok_expr(status_tuple(RustExpr::Ident(
            "__status".to_string(),
        ))))),
    })
}

pub(crate) fn lower_process_kill(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let stmts = vec![
        RustStmt::Let {
            mutable: false,
            name: "__handle".to_string(),
            ty: None,
            value: arg_expr(args, 0),
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
                    body: Box::new(missing_child_error_expr()),
                    is_move: false,
                }],
            })),
        },
        RustStmt::Expr(RustExpr::Try(Box::new(process_map_err(
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__child".to_string())),
                method: "kill".to_string(),
                args: vec![],
            },
        )))),
    ];
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(ok_expr(RustExpr::Literal(RustLiteral::Unit)))),
    })
}

pub(crate) fn lower_process_output(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 7 {
        return None;
    }
    let mut stmts = normal_command_setup(args);
    stmts.extend(output_setup_stmts());
    stmts.extend(spawn_output_stmts(arg_expr(args, 5), arg_expr(args, 6)));
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(ok_expr(output_tuple_expr()))),
    })
}

pub(crate) fn lower_process_output_timeout(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 8 {
        return None;
    }
    let mut stmts = normal_command_setup(args);
    stmts.extend(output_setup_stmts());
    stmts.push(spawn_child_stmt());
    stmts.push(write_stdin_stmt(arg_expr(args, 5), arg_expr(args, 6)));
    stmts.extend(timeout_poll_stmts());
    stmts.push(wait_output_stmt());
    Some(timeout_guard(
        arg_expr(args, 7),
        RustExpr::Block {
            stmts,
            expr: Some(Box::new(ok_expr(output_timeout_tuple_expr()))),
        },
    ))
}

pub(crate) fn lower_process_output_text(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 8 {
        return None;
    }
    let mut stmts = normal_command_setup(args);
    stmts.extend(output_setup_stmts());
    stmts.extend(spawn_output_stmts(arg_expr(args, 5), arg_expr(args, 6)));
    Some(utf8_encoding_guard(
        arg_expr(args, 7),
        RustExpr::Block {
            stmts,
            expr: Some(Box::new(ok_expr(output_text_tuple_expr()))),
        },
    ))
}

pub(crate) fn lower_process_shell_run(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let mut stmts = shell_command_setup(args);
    stmts.push(RustStmt::Let {
        mutable: false,
        name: "__status".to_string(),
        ty: None,
        value: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "status".to_string(),
            args: vec![],
        }))),
    });
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(ok_expr(status_tuple(RustExpr::Ident(
            "__status".to_string(),
        ))))),
    })
}

pub(crate) fn lower_process_shell_output(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let mut stmts = shell_command_setup(args);
    stmts.extend(output_setup_stmts());
    stmts.extend(spawn_output_stmts(arg_expr(args, 1), arg_expr(args, 2)));
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(ok_expr(output_tuple_expr()))),
    })
}

pub(crate) fn lower_process_shell_output_timeout(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    let mut stmts = shell_command_setup(args);
    stmts.extend(output_setup_stmts());
    stmts.push(spawn_child_stmt());
    stmts.push(write_stdin_stmt(arg_expr(args, 1), arg_expr(args, 2)));
    stmts.extend(timeout_poll_stmts());
    stmts.push(wait_output_stmt());
    Some(timeout_guard(
        arg_expr(args, 3),
        RustExpr::Block {
            stmts,
            expr: Some(Box::new(ok_expr(output_timeout_tuple_expr()))),
        },
    ))
}

pub(crate) fn lower_process_shell_output_text(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    let mut stmts = shell_command_setup(args);
    stmts.extend(output_setup_stmts());
    stmts.extend(spawn_output_stmts(arg_expr(args, 1), arg_expr(args, 2)));
    Some(utf8_encoding_guard(
        arg_expr(args, 3),
        RustExpr::Block {
            stmts,
            expr: Some(Box::new(ok_expr(output_text_tuple_expr()))),
        },
    ))
}
