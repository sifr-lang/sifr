//! Random intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn rng_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["rand".to_string(), "rng".to_string()])),
        args: vec![],
    }
}

fn random_f64_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "rand".to_string(),
            "random::<f64>".to_string(),
        ])),
        args: vec![],
    }
}

fn int(v: i64) -> RustExpr {
    RustExpr::Literal(RustLiteral::Int(v))
}

fn value_error(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "ValueError".to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn err_value_error(message: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![value_error(message)],
    }
}

fn ok_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![expr],
    }
}

fn random_range_expr(start: RustExpr, end: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "rand".to_string(),
            "RngExt".to_string(),
            "random_range".to_string(),
        ])),
        args: vec![
            RustExpr::Ref {
                mutable: true,
                expr: Box::new(rng_expr()),
            },
            RustExpr::Range {
                start: Box::new(start),
                end: Box::new(end),
            },
        ],
    }
}

fn module_state_lock_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__SIFR_RANDOM_MODULE_STATE".to_string())),
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

pub(crate) fn lower_random_int(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__start".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__end".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
        ],
        expr: Some(Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("__start".to_string())),
            op: "+".to_string(),
            right: Box::new(random_range_expr(
                int(0),
                RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__end".to_string())),
                        op: "-".to_string(),
                        right: Box::new(RustExpr::Ident("__start".to_string())),
                    }),
                    op: "+".to_string(),
                    right: Box::new(int(1)),
                },
            )),
        })),
    })
}

pub(crate) fn lower_random_float(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(random_f64_expr())
}

pub(crate) fn lower_random_choice(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "items".to_string(),
            ty: None,
            value: arg_expr(args, 0),
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Index {
                expr: Box::new(RustExpr::Ident("items".to_string())),
                index: Box::new(random_range_expr(
                    int(0),
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("items".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    },
                )),
            }),
            method: "clone".to_string(),
            args: vec![],
        })),
    })
}

pub(crate) fn lower_random_uniform(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__start".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__end".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
        ],
        expr: Some(Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("__start".to_string())),
            op: "+".to_string(),
            right: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__end".to_string())),
                    op: "-".to_string(),
                    right: Box::new(RustExpr::Ident("__start".to_string())),
                }),
                op: "*".to_string(),
                right: Box::new(random_f64_expr()),
            }),
        })),
    })
}

pub(crate) fn lower_random_shuffle(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "__v".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(arg_expr(args, 0)),
                    method: "clone".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "rand".to_string(),
                    "seq".to_string(),
                    "SliceRandom".to_string(),
                    "shuffle".to_string(),
                ])),
                args: vec![
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__v".to_string())),
                        method: "as_mut_slice".to_string(),
                        args: vec![],
                    },
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(rng_expr()),
                    },
                ],
            }),
        ],
        expr: Some(Box::new(RustExpr::Ident("__v".to_string()))),
    })
}

pub(crate) fn lower_random_sample(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__items".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__k".to_string(),
                ty: None,
                value: RustExpr::Cast {
                    expr: Box::new(arg_expr(args, 1)),
                    ty: RustType::Named("usize".to_string()),
                },
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__k".to_string())),
                op: ">".to_string(),
                right: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__items".to_string())),
                    method: "len".to_string(),
                    args: vec![],
                }),
            }),
            then_expr: Box::new(err_value_error(RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: "sample larger than population: {} > {}".to_string(),
                args: vec![
                    RustExpr::Ident("__k".to_string()),
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__items".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    },
                ],
            })),
            else_expr: Some(Box::new(ok_expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "rand".to_string(),
                            "seq".to_string(),
                            "IndexedRandom".to_string(),
                            "sample".to_string(),
                        ])),
                        args: vec![
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__items".to_string())),
                                method: "as_slice".to_string(),
                                args: vec![],
                            },
                            RustExpr::Ref {
                                mutable: true,
                                expr: Box::new(rng_expr()),
                            },
                            RustExpr::Ident("__k".to_string()),
                        ],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                }),
                method: "collect::<Vec<_>>".to_string(),
                args: vec![],
            }))),
        })),
    })
}

pub(crate) fn lower_random_randrange(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__start".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__stop".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__step".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__step".to_string())),
                op: "==".to_string(),
                right: Box::new(int(0)),
            }),
            then_expr: Box::new(err_value_error(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(
                    "\"randrange: step must not be zero\"".to_string(),
                )),
                method: "to_string".to_string(),
                args: vec![],
            })),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__start".to_string())),
                        op: ">=".to_string(),
                        right: Box::new(RustExpr::Ident("__stop".to_string())),
                    }),
                    op: "&&".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__step".to_string())),
                        op: ">".to_string(),
                        right: Box::new(int(0)),
                    }),
                }),
                then_expr: Box::new(err_value_error(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("\"randrange: empty range\"".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                })),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![RustStmt::Let {
                        mutable: false,
                        name: "__n".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__stop".to_string())),
                                        op: "-".to_string(),
                                        right: Box::new(RustExpr::Ident("__start".to_string())),
                                    }),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__step".to_string())),
                                        op: "-".to_string(),
                                        right: Box::new(int(1)),
                                    }),
                                }),
                                op: "/".to_string(),
                                right: Box::new(RustExpr::Ident("__step".to_string())),
                            }),
                            method: "abs".to_string(),
                            args: vec![],
                        },
                    }],
                    expr: Some(Box::new(ok_expr(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__start".to_string())),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(random_range_expr(
                                int(0),
                                RustExpr::Ident("__n".to_string()),
                            )),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::Ident("__step".to_string())),
                        }),
                    }))),
                })),
            })),
        })),
    })
}

pub(crate) fn lower_random_gauss(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__mu".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__sigma".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "rand_distr".to_string(),
                        "Normal".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("__mu".to_string()),
                        RustExpr::Ident("__sigma".to_string()),
                    ],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "d".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "rand_distr".to_string(),
                            "Distribution".to_string(),
                            "sample".to_string(),
                        ])),
                        args: vec![
                            RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(RustExpr::Ident("d".to_string())),
                            },
                            RustExpr::Ref {
                                mutable: true,
                                expr: Box::new(rng_expr()),
                            },
                        ],
                    }),
                    is_move: false,
                }],
            }),
            method: "unwrap_or".to_string(),
            args: vec![RustExpr::Ident("__mu".to_string())],
        })),
    })
}

pub(crate) fn lower_random_module_state_words(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__state".to_string(),
            ty: None,
            value: module_state_lock_expr(),
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Field {
                expr: Box::new(RustExpr::Ident("__state".to_string())),
                field: "words".to_string(),
            }),
            method: "clone".to_string(),
            args: vec![],
        })),
    })
}

pub(crate) fn lower_random_module_state_index(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__state".to_string(),
            ty: None,
            value: module_state_lock_expr(),
        }],
        expr: Some(Box::new(RustExpr::Field {
            expr: Box::new(RustExpr::Ident("__state".to_string())),
            field: "index".to_string(),
        })),
    })
}

pub(crate) fn lower_random_module_state_gauss_next(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__state".to_string(),
            ty: None,
            value: module_state_lock_expr(),
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Field {
                expr: Box::new(RustExpr::Ident("__state".to_string())),
                field: "gauss_next".to_string(),
            }),
            method: "clone".to_string(),
            args: vec![],
        })),
    })
}

pub(crate) fn lower_random_module_set_state(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__words".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__index".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__gauss_next".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__index".to_string())),
                    op: "<".to_string(),
                    right: Box::new(int(0)),
                }),
                op: "||".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__index".to_string())),
                    op: ">".to_string(),
                    right: Box::new(int(624)),
                }),
            }),
            then_expr: Box::new(err_value_error(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(
                    "\"random module state index must be in range [0, 624]\"".to_string(),
                )),
                method: "to_string".to_string(),
                args: vec![],
            })),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__words".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                    op: "!=".to_string(),
                    right: Box::new(int(624)),
                }),
                then_expr: Box::new(err_value_error(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(
                        "\"random module state words must have length 624\"".to_string(),
                    )),
                    method: "to_string".to_string(),
                    args: vec![],
                })),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: true,
                            name: "__state".to_string(),
                            ty: None,
                            value: module_state_lock_expr(),
                        },
                        RustStmt::Assign {
                            target: RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__state".to_string())),
                                field: "words".to_string(),
                            },
                            value: RustExpr::Ident("__words".to_string()),
                        },
                        RustStmt::Assign {
                            target: RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__state".to_string())),
                                field: "index".to_string(),
                            },
                            value: RustExpr::Ident("__index".to_string()),
                        },
                        RustStmt::Assign {
                            target: RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__state".to_string())),
                                field: "gauss_next".to_string(),
                            },
                            value: RustExpr::Ident("__gauss_next".to_string()),
                        },
                    ],
                    expr: Some(Box::new(ok_expr(RustExpr::Literal(RustLiteral::Unit)))),
                })),
            })),
        })),
    })
}
