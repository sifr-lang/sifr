//! OS intrinsic lowerers for registry lowering.

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

fn ref_ident(name: &str) -> RustExpr {
    ref_expr(RustExpr::Ident(name.to_string()))
}

fn int(v: i64) -> RustExpr {
    RustExpr::Literal(RustLiteral::Int(v))
}

fn string(v: &str) -> RustExpr {
    RustExpr::Literal(RustLiteral::Str(v.to_string()))
}

fn io_map_err(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    }
}

fn command_new(name: &str) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "process".to_string(),
            "Command".to_string(),
            "new".to_string(),
        ])),
        args: vec![string(name)],
    }
}

fn sh_command_for(cmd_expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(command_new("sh")),
            method: "arg".to_string(),
            args: vec![string("-c")],
        }),
        method: "arg".to_string(),
        args: vec![cmd_expr],
    }
}

fn df_command_for(path_expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(command_new("df")),
            method: "arg".to_string(),
            args: vec![string("-k")],
        }),
        method: "arg".to_string(),
        args: vec![path_expr],
    }
}

fn from_utf8_lossy(bytes_expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "String".to_string(),
            "from_utf8_lossy".to_string(),
        ])),
        args: vec![ref_expr(bytes_expr)],
    }
}

fn trim_to_string(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(expr),
            method: "trim".to_string(),
            args: vec![],
        }),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn parse_i64_or_zero(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(expr),
            method: "parse::<i64>".to_string(),
            args: vec![],
        }),
        method: "unwrap_or".to_string(),
        args: vec![int(0)],
    }
}

fn zero_usage_vec() -> RustExpr {
    RustExpr::Vec(vec![int(0), int(0), int(0)])
}

pub(crate) fn lower_run_command(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__cmd".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__output".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(RustExpr::MethodCall {
                    receiver: Box::new(sh_command_for(ref_ident("__cmd"))),
                    method: "output".to_string(),
                    args: vec![],
                }))),
            },
        ],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![trim_to_string(from_utf8_lossy(RustExpr::Field {
                expr: Box::new(RustExpr::Ident("__output".to_string())),
                field: "stdout".to_string(),
            }))],
        })),
    })
}

pub(crate) fn lower_chdir(args: &[RustExpr]) -> Option<RustExpr> {
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
            args: vec![ref_arg(args, 0)],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    })
}

pub(crate) fn lower_stat_size(args: &[RustExpr]) -> Option<RustExpr> {
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
                args: vec![ref_arg(args, 0)],
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

pub(crate) fn lower_disk_usage(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__path".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__meta_ok".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "fs".to_string(),
                            "metadata".to_string(),
                        ])),
                        args: vec![ref_ident("__path")],
                    }),
                    method: "is_ok".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::Ident("__meta_ok".to_string())),
            then_expr: Box::new(RustExpr::Block {
                stmts: vec![RustStmt::Let {
                    mutable: false,
                    name: "__out".to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(df_command_for(ref_ident("__path"))),
                        method: "output".to_string(),
                        args: vec![],
                    },
                }],
                expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__s".to_string(),
                            ty: None,
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__out".to_string())),
                                    method: "as_ref".to_string(),
                                    args: vec![],
                                }),
                                method: "map_or".to_string(),
                                args: vec![
                                    RustExpr::Literal(RustLiteral::Str(String::new())),
                                    RustExpr::Closure {
                                        params: vec![RustParam::Named {
                                            name: "__o".to_string(),
                                            ty: RustType::Named("_".to_string()),
                                        }],
                                        body: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(from_utf8_lossy(RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident("__o".to_string())),
                                                field: "stdout".to_string(),
                                            })),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        }),
                                        is_move: false,
                                    },
                                ],
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__lines".to_string(),
                            ty: None,
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__s".to_string())),
                                    method: "lines".to_string(),
                                    args: vec![],
                                }),
                                method: "collect::<Vec<&str>>".to_string(),
                                args: vec![],
                            },
                        },
                    ],
                    expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__lines".to_string())),
                                method: "len".to_string(),
                                args: vec![],
                            }),
                            op: ">=".to_string(),
                            right: Box::new(int(2)),
                        }),
                        then_expr: Box::new(RustExpr::Block {
                            stmts: vec![RustStmt::Let {
                                mutable: false,
                                name: "__parts".to_string(),
                                ty: None,
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Index {
                                            expr: Box::new(RustExpr::Ident("__lines".to_string())),
                                            index: Box::new(int(1)),
                                        }),
                                        method: "split_whitespace".to_string(),
                                        args: vec![],
                                    }),
                                    method: "collect::<Vec<&str>>".to_string(),
                                    args: vec![],
                                },
                            }],
                            expr: Some(Box::new(RustExpr::If {
                                cond: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__parts".to_string())),
                                        method: "len".to_string(),
                                        args: vec![],
                                    }),
                                    op: ">=".to_string(),
                                    right: Box::new(int(4)),
                                }),
                                then_expr: Box::new(RustExpr::Block {
                                    stmts: vec![
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "__total".to_string(),
                                            ty: None,
                                            value: RustExpr::BinOp {
                                                left: Box::new(parse_i64_or_zero(
                                                    RustExpr::Index {
                                                        expr: Box::new(RustExpr::Ident(
                                                            "__parts".to_string(),
                                                        )),
                                                        index: Box::new(int(1)),
                                                    },
                                                )),
                                                op: "*".to_string(),
                                                right: Box::new(int(1024)),
                                            },
                                        },
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "__used".to_string(),
                                            ty: None,
                                            value: RustExpr::BinOp {
                                                left: Box::new(parse_i64_or_zero(
                                                    RustExpr::Index {
                                                        expr: Box::new(RustExpr::Ident(
                                                            "__parts".to_string(),
                                                        )),
                                                        index: Box::new(int(2)),
                                                    },
                                                )),
                                                op: "*".to_string(),
                                                right: Box::new(int(1024)),
                                            },
                                        },
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "__free".to_string(),
                                            ty: None,
                                            value: RustExpr::BinOp {
                                                left: Box::new(parse_i64_or_zero(
                                                    RustExpr::Index {
                                                        expr: Box::new(RustExpr::Ident(
                                                            "__parts".to_string(),
                                                        )),
                                                        index: Box::new(int(3)),
                                                    },
                                                )),
                                                op: "*".to_string(),
                                                right: Box::new(int(1024)),
                                            },
                                        },
                                    ],
                                    expr: Some(Box::new(RustExpr::Vec(vec![
                                        RustExpr::Ident("__total".to_string()),
                                        RustExpr::Ident("__used".to_string()),
                                        RustExpr::Ident("__free".to_string()),
                                    ]))),
                                }),
                                else_expr: Some(Box::new(zero_usage_vec())),
                            })),
                        }),
                        else_expr: Some(Box::new(zero_usage_vec())),
                    })),
                })),
            }),
            else_expr: Some(Box::new(zero_usage_vec())),
        })),
    })
}
