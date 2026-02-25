//! File-handle intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustMatchArm, RustStmt};

fn owned_str(arg: &RustExpr) -> RustExpr {
    RustExpr::RawCode(format!("({}).to_string()", crate::render_expr(arg)))
}

fn io_other_error_expr(message: &str) -> String {
    format!("IOError {{ message: \"{message}\".to_string(), kind: \"Other\".to_string() }}")
}

fn next_handle_id_expr(static_name: &str) -> RustExpr {
    RustExpr::RawCode(format!(
        "{{ use std::sync::atomic::{{AtomicI64, Ordering}}; \
         static {static_name}: AtomicI64 = AtomicI64::new(1); \
         {static_name}.fetch_add(1, Ordering::SeqCst) }}"
    ))
}

fn wrap_handle_result(
    hid_expr: RustExpr,
    imports: &[&str],
    arm_pattern: &str,
    arm_body: Vec<RustStmt>,
    err_message: &str,
) -> RustExpr {
    let err_expr = format!("Err({})", io_other_error_expr(err_message));
    let mut body = Vec::new();
    for import in imports {
        if !import.trim().is_empty() {
            body.push(RustStmt::RawCode((*import).to_string()));
        }
    }
    body.push(RustStmt::Let {
        mutable: false,
        name: "__hid".to_string(),
        ty: None,
        value: hid_expr,
    });
    body.push(RustStmt::Let {
        mutable: true,
        name: "__handles".to_string(),
        ty: None,
        value: file_handles_lock_expr(),
    });
    body.push(RustStmt::Match {
        expr: RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__handles".to_string())),
            method: "get_mut".to_string(),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Ident("__hid".to_string())),
            }],
        },
        arms: vec![
            RustMatchArm {
                pattern: format!("Some(SifrFileHandle::{arm_pattern})"),
                bindings: vec![],
                guard: None,
                body: arm_body,
            },
            RustMatchArm {
                pattern: "_".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![RustStmt::RawCode(err_expr)],
            },
        ],
    });
    RustExpr::FnCall {
        func: Box::new(RustExpr::ClosureBlock {
            params: vec![],
            body,
            is_move: false,
        }),
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
        method: "unwrap".to_string(),
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

fn invalid_mode_error_expr(with_return: bool) -> RustStmt {
    let err_expr = RustExpr::FnCall {
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
    };
    if with_return {
        RustStmt::Return(Some(err_expr))
    } else {
        RustStmt::RawCode(crate::render_expr(&err_expr))
    }
}

fn open_arm(
    pattern: &str,
    open_expr: RustExpr,
    variant: &str,
    success_expr: &RustExpr,
) -> RustMatchArm {
    let (buffer_ty, buffer_var) = if variant.ends_with("Read") {
        ("BufReader", "__reader")
    } else {
        ("BufWriter", "__writer")
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
                    func: Box::new(RustExpr::Path(vec![
                        buffer_ty.to_string(),
                        "new".to_string(),
                    ])),
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
            RustStmt::RawCode(crate::render_expr(success_expr)),
        ],
    }
}

fn build_open_match(success_expr: &RustExpr, invalid_stmt: RustStmt) -> RustStmt {
    let path_ref = path_as_str_expr();
    RustStmt::Match {
        expr: mode_as_str_expr(),
        arms: vec![
            open_arm(
                "\"r\" | \"rt\"",
                open_file_expr(path_ref.clone()),
                "TextRead",
                success_expr,
            ),
            open_arm(
                "\"w\" | \"wt\"",
                create_file_expr(path_ref.clone()),
                "TextWrite",
                success_expr,
            ),
            open_arm(
                "\"a\" | \"at\"",
                append_file_expr(path_ref.clone()),
                "TextWrite",
                success_expr,
            ),
            open_arm(
                "\"rb\"",
                open_file_expr(path_ref.clone()),
                "BinaryRead",
                success_expr,
            ),
            open_arm(
                "\"wb\"",
                create_file_expr(path_ref.clone()),
                "BinaryWrite",
                success_expr,
            ),
            open_arm(
                "\"ab\"",
                append_file_expr(path_ref),
                "BinaryWrite",
                success_expr,
            ),
            RustMatchArm {
                pattern: "_".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![invalid_stmt],
            },
        ],
    }
}

pub(super) fn lower_builtin_open(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let success_expr = RustExpr::StructInit {
        name: "FileHandle".to_string(),
        fields: vec![
            (
                "_handle".to_string(),
                RustExpr::Ident("__handle_id".to_string()),
            ),
            (
                "_mode".to_string(),
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__mode".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            ),
        ],
    };
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::RawCode("use std::io::{BufReader, BufWriter};".to_string()),
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
                name: "__handle_id".to_string(),
                ty: None,
                value: next_handle_id_expr("__NEXT_FH_ID"),
            },
            build_open_match(&success_expr, invalid_mode_error_expr(true)),
        ],
        expr: None,
    })
}

pub(super) fn lower_open_file(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let success_expr = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![RustExpr::Ident("__handle_id".to_string())],
    };
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::ClosureBlock {
            params: vec![],
            body: vec![
                RustStmt::RawCode("use std::io::{BufReader, BufWriter};".to_string()),
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
                    name: "__handle_id".to_string(),
                    ty: None,
                    value: next_handle_id_expr("__NEXT_ID"),
                },
                build_open_match(&success_expr, invalid_mode_error_expr(false)),
            ],
            is_move: false,
        }),
        args: vec![],
    })
}

pub(super) fn lower_file_read(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let read_body = vec![
        RustStmt::RawCode("let mut __s = String::new();".to_string()),
        RustStmt::RawCode("__r.read_to_string(&mut __s).map_err(__io_err)?;".to_string()),
        RustStmt::RawCode("Ok(__s)".to_string()),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        &["use std::io::Read;"],
        "TextRead(ref mut __r)",
        read_body,
        "file not open for reading",
    ))
}

pub(super) fn lower_file_write(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let data_expr = crate::render_expr(&args[1]);
    let write_body = vec![
        RustStmt::RawCode(format!("let __data: &str = ({data_expr}).as_ref();")),
        RustStmt::RawCode("__w.write_all(__data.as_bytes()).map_err(__io_err)?;".to_string()),
        RustStmt::RawCode("Ok(())".to_string()),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        &["use std::io::Write;"],
        "TextWrite(ref mut __w)",
        write_body,
        "file not open for writing",
    ))
}

pub(super) fn lower_file_readline(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let readline_body = vec![
        RustStmt::RawCode("let mut __line = String::new();".to_string()),
        RustStmt::RawCode("let __n = __r.read_line(&mut __line).map_err(__io_err)?;".to_string()),
        RustStmt::RawCode(
            "if __n == 0 { Ok(None) } else { if __line.ends_with('\\n') { __line.pop(); if __line.ends_with('\\r') { __line.pop(); } } Ok(Some(__line)) }"
                .to_string(),
        ),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        &["use std::io::BufRead;"],
        "TextRead(ref mut __r)",
        readline_body,
        "file not open for reading",
    ))
}

pub(super) fn lower_file_readlines(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let readlines_body = vec![
        RustStmt::RawCode("let mut __lines: Vec<String> = Vec::new();".to_string()),
        RustStmt::RawCode(
            "let mut __line = String::new(); loop { __line.clear(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { break; } let mut __l = __line.clone(); if __l.ends_with('\\n') { __l.pop(); if __l.ends_with('\\r') { __l.pop(); } } __lines.push(__l); } Ok(__lines)"
                .to_string(),
        ),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        &["use std::io::BufRead;"],
        "TextRead(ref mut __r)",
        readlines_body,
        "file not open for reading",
    ))
}

pub(super) fn lower_file_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: args[0].clone(),
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(file_handles_lock_expr()),
                method: "remove".to_string(),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident("__hid".to_string())),
                }],
            }),
        ],
        expr: Some(Box::new(RustExpr::Literal(RustLiteral::Unit))),
    })
}

pub(super) fn lower_file_read_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let read_bytes_body = vec![
        RustStmt::RawCode("let mut __buf = Vec::new();".to_string()),
        RustStmt::RawCode("__r.read_to_end(&mut __buf).map_err(__io_err)?;".to_string()),
        RustStmt::RawCode("Ok(__buf.iter().map(|&b| b as i64).collect())".to_string()),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        &["use std::io::Read;"],
        "BinaryRead(ref mut __r)",
        read_bytes_body,
        "file not open for binary reading",
    ))
}

pub(super) fn lower_file_write_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let data_expr = crate::render_expr(&args[1]);
    let write_bytes_body = vec![
        RustStmt::RawCode(format!(
            "let __data: Vec<u8> = ({data_expr}).iter().map(|&b| b as u8).collect();"
        )),
        RustStmt::RawCode("__w.write_all(&__data).map_err(__io_err)?;".to_string()),
        RustStmt::RawCode("Ok(())".to_string()),
    ];
    Some(wrap_handle_result(
        args[0].clone(),
        &["use std::io::Write;"],
        "BinaryWrite(ref mut __w)",
        write_bytes_body,
        "file not open for binary writing",
    ))
}
