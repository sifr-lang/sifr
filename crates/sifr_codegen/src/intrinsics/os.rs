//! OS intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn borrow_expr(expr: &str) -> String {
    format!("&({expr})")
}

fn lower_cfg_windows_string(args: &[String], windows: &str, other: &str) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::If {
        cond: Box::new(RustExpr::MacroCall {
            name: "cfg".to_string(),
            args: vec![RustExpr::Ident("target_os = \"windows\"".to_string())],
        }),
        then_expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Literal(RustLiteral::Str(windows.to_string()))),
            method: "to_string".to_string(),
            args: vec![],
        }),
        else_expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Literal(RustLiteral::Str(other.to_string()))),
            method: "to_string".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_run_command(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, IOError> {{ let __cmd = {}; let output = std::process::Command::new(\"sh\").args([\"-c\", &__cmd]).output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) }})()",
        args[0]
    )))
}

pub(super) fn lower_get_args(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "env".to_string(),
                "args".to_string(),
            ])),
            args: vec![],
        }),
        method: "collect::<Vec<String>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_chdir(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "env".to_string(),
                "set_current_dir".to_string(),
            ])),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Ident(format!("({})", args[0]))),
            }],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    })
}

pub(super) fn lower_getpid(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "process".to_string(),
                "id".to_string(),
            ])),
            args: vec![],
        }),
        ty: crate::RustType::I64,
    })
}

pub(super) fn lower_cpu_count(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__n".to_string(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "thread".to_string(),
                            "available_parallelism".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "n".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("n".to_string())),
                            method: "get".to_string(),
                            args: vec![],
                        }),
                        is_move: false,
                    }],
                }),
                method: "unwrap_or".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Int(1))],
            },
        }],
        expr: Some(Box::new(RustExpr::Cast {
            expr: Box::new(RustExpr::Ident("__n".to_string())),
            ty: RustType::I64,
        })),
    })
}

pub(super) fn lower_stat_size(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "fs".to_string(),
                    "metadata".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident(format!("({})", args[0]))),
                }],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "m".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("m".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                    ty: RustType::I64,
                }),
                is_move: false,
            }],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    })
}

pub(super) fn lower_which(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "env".to_string(),
                    "var".to_string(),
                ])),
                args: vec![RustExpr::Ident("\"PATH\"".to_string())],
            }),
            method: "ok".to_string(),
            args: vec![],
        }),
        method: "and_then".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__path".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__path".to_string())),
                            method: "split".to_string(),
                            args: vec![RustExpr::Literal(RustLiteral::Char(':'))],
                        }),
                        method: "map".to_string(),
                        args: vec![RustExpr::Closure {
                            params: vec![RustParam::Named {
                                name: "d".to_string(),
                                ty: RustType::Named("_".to_string()),
                            }],
                            body: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "path".to_string(),
                                        "Path".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ident("d".to_string())],
                                }),
                                method: "join".to_string(),
                                args: vec![RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(RustExpr::Ident(format!("({})", args[0]))),
                                }],
                            }),
                            is_move: false,
                        }],
                    }),
                    method: "find".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "p".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("p".to_string())),
                            method: "is_file".to_string(),
                            args: vec![],
                        }),
                        is_move: false,
                    }],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "p".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("p".to_string())),
                            method: "to_string_lossy".to_string(),
                            args: vec![],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                }],
            }),
            is_move: false,
        }],
    })
}

pub(super) fn lower_disk_usage(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __path = {}; let __stat = std::fs::metadata(__path); match __stat {{ Ok(_) => {{ let __out = std::process::Command::new(\"df\").args([\"-k\", __path]).output(); match __out {{ Ok(__o) => {{ let __s = String::from_utf8_lossy(&__o.stdout); let __lines: Vec<&str> = __s.lines().collect(); if __lines.len() >= 2 {{ let __parts: Vec<&str> = __lines[1].split_whitespace().collect(); if __parts.len() >= 4 {{ let __total = __parts[1].parse::<i64>().unwrap_or(0) * 1024; let __used = __parts[2].parse::<i64>().unwrap_or(0) * 1024; let __free = __parts[3].parse::<i64>().unwrap_or(0) * 1024; vec![__total, __used, __free] }} else {{ vec![0i64, 0, 0] }} }} else {{ vec![0i64, 0, 0] }} }}, Err(_) => vec![0i64, 0, 0] }} }}, Err(_) => vec![0i64, 0, 0] }} }}",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_os_sep(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "path".to_string(),
            "MAIN_SEPARATOR".to_string(),
        ])),
        method: "to_string".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_os_linesep(args: &[String]) -> Option<RustExpr> {
    lower_cfg_windows_string(args, "\r\n", "\n")
}

pub(super) fn lower_os_name(args: &[String]) -> Option<RustExpr> {
    lower_cfg_windows_string(args, "nt", "posix")
}
