//! Time intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn str_lit(v: &str) -> RustExpr {
    RustExpr::Ident(format!("{v:?}"))
}

fn value_error_from_ident(name: &str) -> RustExpr {
    RustExpr::StructInit {
        name: "ValueError".to_string(),
        fields: vec![(
            "message".to_string(),
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(name.to_string())),
                method: "to_string".to_string(),
                args: vec![],
            },
        )],
    }
}

fn format_iso8601(dt_expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(dt_expr),
            method: "format".to_string(),
            args: vec![str_lit("%Y-%m-%dT%H:%M:%S")],
        }),
        method: "to_string".to_string(),
        args: vec![],
    }
}

pub(crate) fn lower_time_now(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "time".to_string(),
                        "SystemTime".to_string(),
                        "now".to_string(),
                    ])),
                    args: vec![],
                }),
                method: "duration_since".to_string(),
                args: vec![RustExpr::Path(vec![
                    "std".to_string(),
                    "time".to_string(),
                    "UNIX_EPOCH".to_string(),
                ])],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        }),
        method: "as_secs_f64".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_sleep(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__secs".to_string(),
            ty: None,
            value: args[0].clone(),
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__secs".to_string())),
                    method: "is_finite".to_string(),
                    args: vec![],
                }),
                op: "&&".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__secs".to_string())),
                    op: ">".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                }),
            }),
            then_expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "thread".to_string(),
                    "sleep".to_string(),
                ])),
                args: vec![RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "time".to_string(),
                        "Duration".to_string(),
                        "from_nanos".to_string(),
                    ])),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__secs".to_string())),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::Literal(RustLiteral::Float(1_000_000_000.0))),
                        }),
                        ty: RustType::Named("u64".to_string()),
                    }],
                }],
            }),
            else_expr: Some(Box::new(RustExpr::Literal(RustLiteral::Unit))),
        })),
    })
}

pub(crate) fn lower_time_format(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "secs".to_string(),
                ty: None,
                value: RustExpr::Cast {
                    expr: Box::new(args[0].clone()),
                    ty: RustType::I64,
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "dt".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "chrono".to_string(),
                            "DateTime".to_string(),
                            "from_timestamp".to_string(),
                        ])),
                        args: vec![
                            RustExpr::Ident("secs".to_string()),
                            RustExpr::Literal(RustLiteral::Int(0)),
                        ],
                    }),
                    method: "unwrap_or_default".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("dt".to_string())),
                method: "format".to_string(),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(arg_expr(args, 1)),
                }],
            }),
            method: "to_string".to_string(),
            args: vec![],
        })),
    })
}

pub(crate) fn lower_perf_counter(args: &[RustExpr]) -> Option<RustExpr> {
    lower_time_now(args)
}

pub(crate) fn lower_monotonic(args: &[RustExpr]) -> Option<RustExpr> {
    lower_perf_counter(args)
}

pub(crate) fn lower_strptime(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "chrono".to_string(),
                    "NaiveDateTime".to_string(),
                    "parse_from_str".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(arg_expr(args, 0)),
                    },
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(arg_expr(args, 1)),
                    },
                ],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "dt".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(format_iso8601(RustExpr::Ident("dt".to_string()))),
                is_move: false,
            }],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "e".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(value_error_from_ident("e")),
            is_move: false,
        }],
    })
}

pub(crate) fn lower_gmtime(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__ts".to_string(),
            ty: None,
            value: RustExpr::Cast {
                expr: Box::new(arg_expr(args, 0)),
                ty: RustType::I64,
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "DateTime::<chrono::Utc>".to_string(),
                        "from_timestamp".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("__ts".to_string()),
                        RustExpr::Literal(RustLiteral::Int(0)),
                    ],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "dt".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(format_iso8601(RustExpr::Ident("dt".to_string()))),
                    is_move: false,
                }],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        })),
    })
}

pub(crate) fn lower_localtime(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__ts".to_string(),
            ty: None,
            value: RustExpr::Cast {
                expr: Box::new(arg_expr(args, 0)),
                ty: RustType::I64,
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "DateTime::<chrono::Utc>".to_string(),
                        "from_timestamp".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("__ts".to_string()),
                        RustExpr::Literal(RustLiteral::Int(0)),
                    ],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "dt".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(format_iso8601(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("dt".to_string())),
                        method: "with_timezone".to_string(),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Path(vec![
                                "chrono".to_string(),
                                "Local".to_string(),
                            ])),
                        }],
                    })),
                    is_move: false,
                }],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        })),
    })
}

pub(crate) fn lower_time_strptime_parts(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__parsed".to_string(),
            ty: Some(RustType::Result(
                Box::new(RustType::Vec(Box::new(RustType::I64))),
                Box::new(RustType::Named("ValueError".to_string())),
            )),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "chrono".to_string(),
                            "NaiveDateTime".to_string(),
                            "parse_from_str".to_string(),
                        ])),
                        args: vec![
                            RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(arg_expr(args, 0)),
                            },
                            RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(arg_expr(args, 1)),
                            },
                        ],
                    }),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "dt".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Vec(vec![
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "chrono".to_string(),
                                        "Datelike".to_string(),
                                        "year".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Ident("dt".to_string())),
                                    }],
                                }),
                                ty: RustType::I64,
                            },
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "chrono".to_string(),
                                        "Datelike".to_string(),
                                        "month".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Ident("dt".to_string())),
                                    }],
                                }),
                                ty: RustType::I64,
                            },
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "chrono".to_string(),
                                        "Datelike".to_string(),
                                        "day".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Ident("dt".to_string())),
                                    }],
                                }),
                                ty: RustType::I64,
                            },
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "chrono".to_string(),
                                        "Timelike".to_string(),
                                        "hour".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Ident("dt".to_string())),
                                    }],
                                }),
                                ty: RustType::I64,
                            },
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "chrono".to_string(),
                                        "Timelike".to_string(),
                                        "minute".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Ident("dt".to_string())),
                                    }],
                                }),
                                ty: RustType::I64,
                            },
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "chrono".to_string(),
                                        "Timelike".to_string(),
                                        "second".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Ident("dt".to_string())),
                                    }],
                                }),
                                ty: RustType::I64,
                            },
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("dt".to_string())),
                                        method: "weekday".to_string(),
                                        args: vec![],
                                    }),
                                    method: "num_days_from_monday".to_string(),
                                    args: vec![],
                                }),
                                ty: RustType::I64,
                            },
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "chrono".to_string(),
                                        "Datelike".to_string(),
                                        "ordinal".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Ident("dt".to_string())),
                                    }],
                                }),
                                ty: RustType::I64,
                            },
                        ])),
                        is_move: false,
                    }],
                }),
                method: "map_err".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "e".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::StructInit {
                        name: "ValueError".to_string(),
                        fields: vec![(
                            "message".to_string(),
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                        )],
                    }),
                    is_move: false,
                }],
            },
        }],
        expr: Some(Box::new(RustExpr::Ident("__parsed".to_string()))),
    })
}

pub(crate) fn lower_time_gmtime_parts(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__dt".to_string(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Utc".to_string(),
                        "now".to_string(),
                    ])),
                    args: vec![],
                }),
                method: "naive_utc".to_string(),
                args: vec![],
            },
        }],
        expr: Some(Box::new(RustExpr::Vec(vec![
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "year".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "month".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "day".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "hour".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "minute".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "second".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__dt".to_string())),
                        method: "weekday".to_string(),
                        args: vec![],
                    }),
                    method: "num_days_from_monday".to_string(),
                    args: vec![],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "ordinal".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
        ]))),
    })
}

pub(crate) fn lower_time_localtime_parts(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__dt".to_string(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Local".to_string(),
                        "now".to_string(),
                    ])),
                    args: vec![],
                }),
                method: "naive_local".to_string(),
                args: vec![],
            },
        }],
        expr: Some(Box::new(RustExpr::Vec(vec![
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "year".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "month".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "day".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "hour".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "minute".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "second".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__dt".to_string())),
                        method: "weekday".to_string(),
                        args: vec![],
                    }),
                    method: "num_days_from_monday".to_string(),
                    args: vec![],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "ordinal".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
        ]))),
    })
}
