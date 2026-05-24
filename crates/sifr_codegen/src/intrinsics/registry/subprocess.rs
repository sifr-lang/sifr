//! Subprocess intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustStmt};

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

fn ref_ident(name: &str) -> RustExpr {
    ref_expr(RustExpr::Ident(name.to_string()))
}

fn io_map_err(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    }
}

fn ok_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![expr],
    }
}

fn command_for(cmd_expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "process".to_string(),
                    "Command".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Literal(RustLiteral::Str("sh".to_string()))],
            }),
            method: "arg".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Str("-c".to_string()))],
        }),
        method: "arg".to_string(),
        args: vec![cmd_expr],
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

fn from_utf8_lossy(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "String".to_string(),
            "from_utf8_lossy".to_string(),
        ])),
        args: vec![ref_expr(expr)],
    }
}

fn output_field(output_ident: &str, field: &str) -> RustExpr {
    RustExpr::Field {
        expr: Box::new(RustExpr::Ident(output_ident.to_string())),
        field: field.to_string(),
    }
}

fn output_stdout_trimmed(output_ident: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(from_utf8_lossy(output_field(output_ident, "stdout"))),
            method: "trim".to_string(),
            args: vec![],
        }),
        method: "to_string".to_string(),
        args: vec![],
    }
}

pub(crate) fn lower_subprocess_run(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__output".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(io_map_err(RustExpr::MethodCall {
                receiver: Box::new(command_for(ref_arg(args, 0))),
                method: "output".to_string(),
                args: vec![],
            }))),
        }],
        expr: Some(Box::new(ok_expr(output_stdout_trimmed("__output")))),
    })
}

pub(crate) fn lower_subprocess_run_with_input(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let spawn_expr = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(command_for(ref_arg(args, 0))),
                method: "stdin".to_string(),
                args: vec![piped_stdio()],
            }),
            method: "stdout".to_string(),
            args: vec![piped_stdio()],
        }),
        method: "spawn".to_string(),
        args: vec![],
    };

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "__child".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(spawn_expr))),
            },
            RustStmt::IfLet {
                pattern: "Some(mut stdin)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("__child".to_string())),
                        field: "stdin".to_string(),
                    }),
                    method: "take".to_string(),
                    args: vec![],
                },
                then_body: vec![RustStmt::Expr(RustExpr::Try(Box::new(io_map_err(
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
                                expr: Box::new(RustExpr::Ident("stdin".to_string())),
                            },
                            RustExpr::MethodCall {
                                receiver: Box::new(arg_expr(args, 1)),
                                method: "as_bytes".to_string(),
                                args: vec![],
                            },
                        ],
                    },
                ))))],
                else_body: None,
            },
            RustStmt::Let {
                mutable: false,
                name: "__output".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__child".to_string())),
                    method: "wait_with_output".to_string(),
                    args: vec![],
                }))),
            },
        ],
        expr: Some(Box::new(ok_expr(output_stdout_trimmed("__output")))),
    })
}

pub(crate) fn lower_subprocess_run_structured(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__output".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(RustExpr::MethodCall {
                    receiver: Box::new(command_for(ref_arg(args, 0))),
                    method: "output".to_string(),
                    args: vec![],
                }))),
            },
            RustStmt::Let {
                mutable: false,
                name: "__stdout".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(from_utf8_lossy(output_field("__output", "stdout"))),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__stderr".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(from_utf8_lossy(output_field("__output", "stderr"))),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__returncode".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__output".to_string())),
                                field: "status".to_string(),
                            }),
                            method: "code".to_string(),
                            args: vec![],
                        }),
                        method: "unwrap_or".to_string(),
                        args: vec![RustExpr::Literal(RustLiteral::Int(-1))],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Vec(vec![
            RustExpr::Ident("__stdout".to_string()),
            RustExpr::Ident("__stderr".to_string()),
            RustExpr::Ident("__returncode".to_string()),
        ])))),
    })
}
