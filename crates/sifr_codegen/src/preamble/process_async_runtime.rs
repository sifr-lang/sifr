//! Runtime support for generated async process helpers.

use crate::{RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, Visibility};

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

fn process_async_params(include_stdin: bool) -> Vec<RustParam> {
    let mut params = vec![
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
    ];
    if include_stdin {
        params.push(RustParam::Named {
            name: "has_stdin".to_string(),
            ty: RustType::Bool,
        });
    }
    params
}

fn process_async_stdin_mode_guard() -> RustStmt {
    RustStmt::If {
        cond: RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("stdin_mode".to_string())),
            op: "!=".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Str("inherit".to_string()))),
        },
        then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
            args: vec![process_error_expr(RustExpr::Literal(RustLiteral::Str(
                "async process stdin mode requires owned pipe support".to_string(),
            )))],
        }))],
        else_body: None,
    }
}

fn process_async_command_setup() -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: true,
            name: "__cmd".to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "tokio".to_string(),
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

fn status_code_expr(status: RustExpr) -> RustExpr {
    RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(status),
                method: "code".to_string(),
                args: vec![],
            }),
            method: "unwrap_or".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Int(-1))],
        }),
        ty: RustType::I64,
    }
}

fn status_signal_expr(status: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "__sifr_process_exit_signal".to_string()
        ])),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(status),
        }],
    }
}

fn process_status_from_parts(code: RustExpr, signal: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "__sifr_process_status_from_exit".to_string()
        ])),
        args: vec![code, signal],
    }
}

fn process_async_ret(name: &str) -> RustType {
    RustType::Named(format!("Result<{name}, ProcessError>"))
}

fn process_async_timeout_params() -> Vec<RustParam> {
    let mut params = process_async_params(false);
    params.push(RustParam::Named {
        name: "timeout_seconds".to_string(),
        ty: RustType::F64,
    });
    params
}

fn process_async_output_timeout_params() -> Vec<RustParam> {
    let mut params = process_async_params(true);
    params.push(RustParam::Named {
        name: "timeout_seconds".to_string(),
        ty: RustType::F64,
    });
    params
}

pub(crate) fn build_process_async_items(
    needs_run: bool,
    needs_run_timeout: bool,
    needs_output: bool,
    needs_output_timeout: bool,
) -> Vec<RustItem> {
    let mut run_body = vec![process_async_stdin_mode_guard()];
    run_body.extend(process_async_command_setup());
    run_body.push(RustStmt::Let {
        mutable: false,
        name: "__status".to_string(),
        ty: None,
        value: RustExpr::Try(Box::new(process_map_err(RustExpr::Await(Box::new(
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                method: "status".to_string(),
                args: vec![],
            },
        ))))),
    });
    run_body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![process_status_from_parts(
            status_code_expr(RustExpr::Ident("__status".to_string())),
            status_signal_expr(RustExpr::Ident("__status".to_string())),
        )],
    })));

    let mut run_timeout_body = vec![
        process_async_stdin_mode_guard(),
        RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("timeout_seconds".to_string())),
                        method: "is_finite".to_string(),
                        args: vec![],
                    }),
                }),
                op: "||".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("timeout_seconds".to_string())),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                }),
            },
            then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![process_error_expr(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "process timeout must be finite and non-negative, got {}"
                        .to_string(),
                    args: vec![RustExpr::Ident("timeout_seconds".to_string())],
                })],
            }))],
            else_body: None,
        },
    ];
    run_timeout_body.push(RustStmt::Let {
        mutable: false,
        name: "__duration".to_string(),
        ty: None,
        value: RustExpr::Try(Box::new(process_map_err(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "time".to_string(),
                "Duration".to_string(),
                "try_from_secs_f64".to_string(),
            ])),
            args: vec![RustExpr::Ident("timeout_seconds".to_string())],
        }))),
    });
    run_timeout_body.extend(process_async_command_setup());
    run_timeout_body.push(RustStmt::Expr(RustExpr::Ident(
        "let mut __child = __cmd.spawn().map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
        return tokio::select! {
            biased;
            __waited = __child.wait() => {
                let __status = __waited.map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
                Ok(__sifr_process_status_from_exit(
                    __status.code().unwrap_or(-1) as i64,
                    __sifr_process_exit_signal(&__status),
                ))
            }
            _ = tokio::time::sleep(__duration) => {
                __child.kill().await.map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
                let mut __timeout_status = Status::new(-1, \"timeout\".to_string());
                __timeout_status.success = false;
                __timeout_status.timed_out = true;
                Ok(__timeout_status)
            }
        }"
        .to_string(),
    )));

    let mut output_body = vec![
        process_async_stdin_mode_guard(),
        RustStmt::If {
            cond: RustExpr::Ident("has_stdin".to_string()),
            then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![process_error_expr(RustExpr::Literal(RustLiteral::Str(
                    "async process stdin bytes require owned pipe support".to_string(),
                )))],
            }))],
            else_body: None,
        },
    ];
    output_body.extend(process_async_command_setup());
    output_body.push(RustStmt::Let {
        mutable: false,
        name: "__output".to_string(),
        ty: None,
        value: RustExpr::Try(Box::new(process_map_err(RustExpr::Await(Box::new(
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                method: "output".to_string(),
                args: vec![],
            },
        ))))),
    });
    output_body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "Output".to_string(),
                "new".to_string(),
            ])),
            args: vec![
                RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("__output".to_string())),
                    field: "stdout".to_string(),
                },
                RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("__output".to_string())),
                    field: "stderr".to_string(),
                },
                process_status_from_parts(
                    status_code_expr(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("__output".to_string())),
                        field: "status".to_string(),
                    }),
                    status_signal_expr(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("__output".to_string())),
                        field: "status".to_string(),
                    }),
                ),
            ],
        }],
    })));

    let mut output_timeout_body = vec![
        process_async_stdin_mode_guard(),
        RustStmt::If {
            cond: RustExpr::Ident("has_stdin".to_string()),
            then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![process_error_expr(RustExpr::Literal(RustLiteral::Str(
                    "async process stdin bytes require owned pipe support".to_string(),
                )))],
            }))],
            else_body: None,
        },
        RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("timeout_seconds".to_string())),
                        method: "is_finite".to_string(),
                        args: vec![],
                    }),
                }),
                op: "||".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("timeout_seconds".to_string())),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                }),
            },
            then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![process_error_expr(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "process timeout must be finite and non-negative, got {}"
                        .to_string(),
                    args: vec![RustExpr::Ident("timeout_seconds".to_string())],
                })],
            }))],
            else_body: None,
        },
        RustStmt::Let {
            mutable: false,
            name: "__duration".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(process_map_err(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "time".to_string(),
                    "Duration".to_string(),
                    "try_from_secs_f64".to_string(),
                ])),
                args: vec![RustExpr::Ident("timeout_seconds".to_string())],
            }))),
        },
    ];
    output_timeout_body.extend(process_async_command_setup());
    output_timeout_body.push(RustStmt::Expr(RustExpr::Ident(
        "use tokio::io::AsyncReadExt;
        __cmd.stdout(std::process::Stdio::piped());
        __cmd.stderr(std::process::Stdio::piped());
        let mut __child = __cmd.spawn().map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
        let mut __stdout = __child.stdout.take();
        let mut __stderr = __child.stderr.take();
        let mut __stdout_bytes = Vec::new();
        let mut __stderr_bytes = Vec::new();
        return tokio::select! {
            biased;
            __completed = async {
                let __stdout_read = async {
                    if let Some(__pipe) = __stdout.as_mut() {
                        __pipe.read_to_end(&mut __stdout_bytes).await?;
                    }
                    Ok::<(), std::io::Error>(())
                };
                let __stderr_read = async {
                    if let Some(__pipe) = __stderr.as_mut() {
                        __pipe.read_to_end(&mut __stderr_bytes).await?;
                    }
                    Ok::<(), std::io::Error>(())
                };
                let (__status, _, _) = tokio::try_join!(__child.wait(), __stdout_read, __stderr_read)?;
                Ok::<(std::process::ExitStatus, Vec<u8>, Vec<u8>), std::io::Error>((__status, __stdout_bytes, __stderr_bytes))
            } => {
                let (__status, __stdout_done, __stderr_done) = __completed.map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
                Ok(Output::new(
                    __stdout_done,
                    __stderr_done,
                    __sifr_process_status_from_exit(
                        __status.code().unwrap_or(-1) as i64,
                        __sifr_process_exit_signal(&__status),
                    ),
                ))
            }
            _ = tokio::time::sleep(__duration) => {
                __child.kill().await.map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
                let _ = __child.wait().await.map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
                let mut __timeout_status = Status::new(-1, \"timeout\".to_string());
                __timeout_status.success = false;
                __timeout_status.timed_out = true;
                Ok(Output::new(Vec::new(), Vec::new(), __timeout_status))
            }
        }"
        .to_string(),
    )));

    let mut items = vec![RustItem::Fn {
        name: "__sifr_process_status_from_exit".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::Named {
                name: "code".to_string(),
                ty: RustType::I64,
            },
            RustParam::Named {
                name: "signal".to_string(),
                ty: RustType::Option(Box::new(RustType::I64)),
            },
        ],
        ret: Some(RustType::Named("Status".to_string())),
        body: vec![
            RustStmt::IfLet {
                pattern: "Some(__signal)".to_string(),
                expr: RustExpr::Ident("signal".to_string()),
                then_body: vec![
                    RustStmt::Let {
                        mutable: true,
                        name: "__status".to_string(),
                        ty: Some(RustType::Named("Status".to_string())),
                        value: RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "Status".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![
                                RustExpr::Ident("code".to_string()),
                                RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                                        "signal".to_string(),
                                    ))),
                                    method: "to_string".to_string(),
                                    args: vec![],
                                },
                            ],
                        },
                    },
                    RustStmt::Assign {
                        target: RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__status".to_string())),
                            field: "success".to_string(),
                        },
                        value: RustExpr::Literal(RustLiteral::Bool(false)),
                    },
                    RustStmt::Assign {
                        target: RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__status".to_string())),
                            field: "signal".to_string(),
                        },
                        value: RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                            args: vec![RustExpr::Ident("__signal".to_string())],
                        },
                    },
                    RustStmt::Return(Some(RustExpr::Ident("__status".to_string()))),
                ],
                else_body: None,
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("code".to_string())),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                },
                then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "Status".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("code".to_string()),
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                                "success".to_string(),
                            ))),
                            method: "to_string".to_string(),
                            args: vec![],
                        },
                    ],
                }))],
                else_body: None,
            },
            RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "Status".to_string(),
                    "new".to_string(),
                ])),
                args: vec![
                    RustExpr::Ident("code".to_string()),
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                            "nonzero".to_string(),
                        ))),
                        method: "to_string".to_string(),
                        args: vec![],
                    },
                ],
            })),
        ],
        is_async: false,
    }];

    if needs_run {
        items.push(RustItem::Fn {
            name: "__sifr_process_async_run".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_params(false),
            ret: Some(process_async_ret("Status")),
            body: run_body,
            is_async: true,
        });
    }
    if needs_run_timeout {
        items.push(RustItem::Fn {
            name: "__sifr_process_async_run_timeout".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_timeout_params(),
            ret: Some(process_async_ret("Status")),
            body: run_timeout_body,
            is_async: true,
        });
    }
    if needs_output {
        items.push(RustItem::Fn {
            name: "__sifr_process_async_output".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_params(true),
            ret: Some(process_async_ret("Output")),
            body: output_body,
            is_async: true,
        });
    }
    if needs_output_timeout {
        items.push(RustItem::Fn {
            name: "__sifr_process_async_output_timeout".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_output_timeout_params(),
            ret: Some(process_async_ret("Output")),
            body: output_timeout_body,
            is_async: true,
        });
    }

    items
}
