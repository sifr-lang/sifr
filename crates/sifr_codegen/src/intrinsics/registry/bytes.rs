//! Bytes intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn int(v: i64) -> RustExpr {
    RustExpr::Literal(RustLiteral::Int(v))
}

fn string_lit(v: &str) -> RustExpr {
    RustExpr::Literal(RustLiteral::Str(v.to_string()))
}

fn parse_error_expr(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "ParseError".to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn err_parse_expr(message: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![parse_error_expr(message)],
    }
}

fn parse_map_err(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "e".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(parse_error_expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("e".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            })),
            is_move: false,
        }],
    }
}

fn value_error_expr(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "ValueError".to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn err_value_expr(message: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![value_error_expr(message)],
    }
}

fn typed_ok_expr(expr: RustExpr, ok_ty: &str, err_ty: &str) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![format!("Ok::<{ok_ty}, {err_ty}>")])),
        args: vec![expr],
    }
}

pub(crate) fn lower_bytes_from_hex(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "s".to_string(),
                ty: Some(RustType::String_),
                value: RustExpr::MethodCall {
                    receiver: Box::new(arg_expr(args, 0)),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "cleaned".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "ch".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("s".to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                },
                body: vec![
                    RustStmt::If {
                        cond: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("ch".to_string())),
                            method: "is_ascii_whitespace".to_string(),
                            args: vec![],
                        },
                        then_body: vec![RustStmt::Continue],
                        else_body: None,
                    },
                    RustStmt::If {
                        cond: RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("ch".to_string())),
                                method: "is_ascii_hexdigit".to_string(),
                                args: vec![],
                            }),
                        },
                        then_body: vec![RustStmt::Return(Some(err_parse_expr(
                            RustExpr::FormatMacro {
                                name: "format".to_string(),
                                format_str: "invalid hex character: {}".to_string(),
                                args: vec![RustExpr::Ident("ch".to_string())],
                            },
                        )))],
                        else_body: None,
                    },
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("cleaned".to_string())),
                        method: "push".to_string(),
                        args: vec![RustExpr::Ident("ch".to_string())],
                    }),
                ],
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("cleaned".to_string())),
                            method: "len".to_string(),
                            args: vec![],
                        }),
                        op: "%".to_string(),
                        right: Box::new(int(2)),
                    }),
                    op: "!=".to_string(),
                    right: Box::new(int(0)),
                },
                then_body: vec![RustStmt::Return(Some(err_parse_expr(
                    RustExpr::MethodCall {
                        receiver: Box::new(string_lit(
                            "fromhex() arg must contain an even number of hexadecimal digits",
                        )),
                        method: "to_string".to_string(),
                        args: vec![],
                    },
                )))],
                else_body: None,
            },
            RustStmt::Let {
                mutable: true,
                name: "result".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "pair".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("cleaned".to_string())),
                        method: "as_bytes".to_string(),
                        args: vec![],
                    }),
                    method: "chunks".to_string(),
                    args: vec![int(2)],
                },
                body: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "pair_str".to_string(),
                        ty: None,
                        value: RustExpr::Try(Box::new(parse_map_err(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "str".to_string(),
                                "from_utf8".to_string(),
                            ])),
                            args: vec![RustExpr::Ident("pair".to_string())],
                        }))),
                    },
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("result".to_string())),
                        method: "push".to_string(),
                        args: vec![RustExpr::Try(Box::new(parse_map_err(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "u8".to_string(),
                                "from_str_radix".to_string(),
                            ])),
                            args: vec![RustExpr::Ident("pair_str".to_string()), int(16)],
                        })))],
                    }),
                ],
            },
        ],
        expr: Some(Box::new(typed_ok_expr(
            RustExpr::Ident("result".to_string()),
            "Vec<u8>",
            "ParseError",
        ))),
    })
}

pub(crate) fn lower_bytes_with_size(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__size".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__size".to_string())),
                    op: "<".to_string(),
                    right: Box::new(int(0)),
                },
                then_body: vec![RustStmt::Return(Some(err_value_expr(
                    RustExpr::MethodCall {
                        receiver: Box::new(string_lit("bytes(size) requires a non-negative size")),
                        method: "to_string".to_string(),
                        args: vec![],
                    },
                )))],
                else_body: None,
            },
            RustStmt::Let {
                mutable: false,
                name: "__size".to_string(),
                ty: None,
                value: RustExpr::Match {
                    expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__size".to_string())),
                        method: "try_to_usize".to_string(),
                        args: vec![],
                    }),
                    arms: vec![
                        RustMatchArm {
                            pattern: "Ok(__size)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::TailExpr(RustExpr::Ident("__size".to_string()))],
                        },
                        RustMatchArm {
                            pattern: "Err(_)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(err_value_expr(
                                RustExpr::MethodCall {
                                    receiver: Box::new(string_lit(
                                        "bytes(size) exceeds the addressable size",
                                    )),
                                    method: "to_string".to_string(),
                                    args: vec![],
                                },
                            )))],
                        },
                    ],
                },
            },
        ],
        expr: Some(Box::new(typed_ok_expr(
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Range {
                        start: Box::new(int(0)),
                        end: Box::new(RustExpr::Ident("__size".to_string())),
                    }),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "_".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Cast {
                            expr: Box::new(int(0)),
                            ty: RustType::Named("u8".to_string()),
                        }),
                        is_move: false,
                    }],
                }),
                method: "collect::<Vec<u8>>".to_string(),
                args: vec![],
            },
            "Vec<u8>",
            "ValueError",
        ))),
    })
}

pub(crate) fn lower_bytes_from_ints(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__vals".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: true,
                name: "__out".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "__pair".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__vals".to_string())),
                        method: "iter".to_string(),
                        args: vec![],
                    }),
                    method: "enumerate".to_string(),
                    args: vec![],
                },
                body: vec![
                    RustStmt::If {
                        cond: RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Deref(Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                    field: "1".to_string(),
                                }))),
                                op: "<".to_string(),
                                right: Box::new(int(0)),
                            }),
                            op: "||".to_string(),
                            right: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Deref(Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                    field: "1".to_string(),
                                }))),
                                op: ">".to_string(),
                                right: Box::new(int(255)),
                            }),
                        },
                        then_body: vec![RustStmt::Return(Some(err_value_expr(
                            RustExpr::FormatMacro {
                                name: "format".to_string(),
                                format_str: "byte out of range at index {}: {}".to_string(),
                                args: vec![
                                    RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                        field: "0".to_string(),
                                    },
                                    RustExpr::Deref(Box::new(RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                        field: "1".to_string(),
                                    })),
                                ],
                            },
                        )))],
                        else_body: None,
                    },
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__out".to_string())),
                        method: "push".to_string(),
                        args: vec![RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                field: "1".to_string(),
                            }),
                            method: "to_u8_proven_in_range".to_string(),
                            args: vec![],
                        }],
                    }),
                ],
            },
        ],
        expr: Some(Box::new(typed_ok_expr(
            RustExpr::Ident("__out".to_string()),
            "Vec<u8>",
            "ValueError",
        ))),
    })
}
