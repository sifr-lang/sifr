//! Child process lifecycle intrinsic lowerers.

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

fn clone_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(arg_expr(args, idx)),
        method: "clone".to_string(),
        args: vec![],
    }
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

fn missing_child_error_expr() -> RustExpr {
    process_error_expr(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "process child handle is closed or unknown: {}".to_string(),
        args: vec![RustExpr::Ident("__handle".to_string())],
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
    path_call(&["__sifr_process_exit_signal"], vec![ref_expr(status_expr)])
}

pub(crate) fn lower_process_spawn(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 8 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_spawn"],
        vec![
            clone_arg(args, 0),
            clone_arg(args, 1),
            clone_arg(args, 2),
            clone_arg(args, 3),
            arg_expr(args, 4),
            clone_arg(args, 5),
            clone_arg(args, 6),
            clone_arg(args, 7),
        ],
    ))
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
                    args: vec![ref_expr(RustExpr::Ident("__handle".to_string()))],
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
                    args: vec![ref_expr(RustExpr::Ident("__handle".to_string()))],
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

pub(crate) fn lower_process_terminate(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_terminate"],
        vec![arg_expr(args, 0)],
    ))
}
