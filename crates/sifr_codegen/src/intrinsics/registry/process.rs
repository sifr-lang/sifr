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

fn ok_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![expr],
    }
}

fn command_new(program: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "process".to_string(),
            "Command".to_string(),
            "new".to_string(),
        ])),
        args: vec![program],
    }
}

fn piped_stdio() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "process".to_string(),
            "Stdio".to_string(),
            "piped".to_string(),
        ])),
        args: vec![],
    }
}

fn command_status_code(output_ident: &str) -> RustExpr {
    status_code(RustExpr::Field {
        expr: Box::new(RustExpr::Ident(output_ident.to_string())),
        field: "status".to_string(),
    })
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

fn spawn_output_stmts(stdin_expr: RustExpr, has_stdin_expr: RustExpr) -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: true,
            name: "__child".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                method: "spawn".to_string(),
                args: vec![],
            }))),
        },
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
                    RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "io".to_string(),
                            "Write".to_string(),
                            "write_all".to_string(),
                        ])),
                        args: vec![
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
                    },
                ))))],
                else_body: None,
            }],
            else_body: None,
        },
        RustStmt::Let {
            mutable: false,
            name: "__output".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(process_map_err(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__child".to_string())),
                method: "wait_with_output".to_string(),
                args: vec![],
            }))),
        },
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
        command_status_code("__output"),
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
            command_status_code("__output"),
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
        expr: Some(Box::new(ok_expr(status_code(RustExpr::Ident(
            "__status".to_string(),
        ))))),
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
        expr: Some(Box::new(ok_expr(status_code(RustExpr::Ident(
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
