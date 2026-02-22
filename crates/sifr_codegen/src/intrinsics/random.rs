//! Random intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[String], idx: usize) -> RustExpr {
    RustExpr::Ident(args[idx].clone())
}

fn thread_rng_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "rand".to_string(),
            "thread_rng".to_string(),
        ])),
        args: vec![],
    }
}

pub(super) fn lower_random_int(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let start = arg_expr(args, 0);
    let end = arg_expr(args, 1);
    Some(RustExpr::BinOp {
        left: Box::new(start.clone()),
        op: "+".to_string(),
        right: Box::new(RustExpr::MethodCall {
            receiver: Box::new(thread_rng_expr()),
            method: "gen_range".to_string(),
            args: vec![RustExpr::Range {
                start: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                end: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(end),
                        op: "-".to_string(),
                        right: Box::new(start),
                    }),
                    op: "+".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                }),
            }],
        }),
    })
}

pub(super) fn lower_random_float(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(thread_rng_expr()),
        method: "gen::<f64>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_random_choice(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
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
                name: "__idx".to_string(),
                ty: None,
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(thread_rng_expr()),
                                method: "gen::<f64>".to_string(),
                                args: vec![],
                            }),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::Cast {
                                expr: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__items".to_string())),
                                    method: "len".to_string(),
                                    args: vec![],
                                }),
                                ty: RustType::F64,
                            }),
                        }),
                        method: "floor".to_string(),
                        args: vec![],
                    }),
                    ty: RustType::Named("usize".to_string()),
                },
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Index {
                expr: Box::new(RustExpr::Ident("__items".to_string())),
                index: Box::new(RustExpr::Ident("__idx".to_string())),
            }),
            method: "clone".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_random_uniform(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let start = arg_expr(args, 0);
    let end = arg_expr(args, 1);
    Some(RustExpr::BinOp {
        left: Box::new(start.clone()),
        op: "+".to_string(),
        right: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::BinOp {
                left: Box::new(end),
                op: "-".to_string(),
                right: Box::new(start),
            }),
            op: "*".to_string(),
            right: Box::new(RustExpr::MethodCall {
                receiver: Box::new(thread_rng_expr()),
                method: "gen::<f64>".to_string(),
                args: vec![],
            }),
        }),
    })
}

pub(super) fn lower_random_shuffle(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "__v".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(arg_expr(args, 0))),
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__v".to_string())),
                method: "shuffle".to_string(),
                args: vec![RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(thread_rng_expr()),
                }],
            }),
        ],
        expr: Some(Box::new(RustExpr::Ident("__v".to_string()))),
    })
}

pub(super) fn lower_random_sample(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__items".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(arg_expr(args, 0)),
                },
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
            then_expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![RustExpr::StructInit {
                    name: "ValueError".to_string(),
                    fields: vec![(
                        "message".to_string(),
                        RustExpr::FormatMacro {
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
                        },
                    )],
                }],
            }),
            else_expr: Some(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__items".to_string())),
                            method: "choose_multiple".to_string(),
                            args: vec![
                                RustExpr::Ref {
                                    mutable: true,
                                    expr: Box::new(thread_rng_expr()),
                                },
                                RustExpr::Ident("__k".to_string()),
                            ],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    }),
                    method: "collect::<Vec<_>>".to_string(),
                    args: vec![],
                }],
            })),
        })),
    })
}

pub(super) fn lower_random_randrange(args: &[String]) -> Option<RustExpr> {
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
                right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
            }),
            then_expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![RustExpr::StructInit {
                    name: "ValueError".to_string(),
                    fields: vec![(
                        "message".to_string(),
                        RustExpr::Literal(RustLiteral::Str(
                            "randrange: step must not be zero".to_string(),
                        )),
                    )],
                }],
            }),
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
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    }),
                }),
                then_expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                    args: vec![RustExpr::StructInit {
                        name: "ValueError".to_string(),
                        fields: vec![(
                            "message".to_string(),
                            RustExpr::Literal(RustLiteral::Str(
                                "randrange: empty range".to_string(),
                            )),
                        )],
                    }],
                }),
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
                                        right: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                    }),
                                }),
                                op: "/".to_string(),
                                right: Box::new(RustExpr::Ident("__step".to_string())),
                            }),
                            method: "abs".to_string(),
                            args: vec![],
                        },
                    }],
                    expr: Some(Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__start".to_string())),
                            op: "+".to_string(),
                            right: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(thread_rng_expr()),
                                    method: "gen_range".to_string(),
                                    args: vec![RustExpr::Range {
                                        start: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                                        end: Box::new(RustExpr::Ident("__n".to_string())),
                                    }],
                                }),
                                op: "*".to_string(),
                                right: Box::new(RustExpr::Ident("__step".to_string())),
                            }),
                        }],
                    })),
                })),
            })),
        })),
    })
}

pub(super) fn lower_random_gauss(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mu = arg_expr(args, 0);
    let sigma = arg_expr(args, 1);
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "rand_distr".to_string(),
                    "Normal".to_string(),
                    "new".to_string(),
                ])),
                args: vec![mu.clone(), sigma],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "d".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("d".to_string())),
                    method: "sample".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(thread_rng_expr()),
                    }],
                }),
                is_move: false,
            }],
        }),
        method: "unwrap_or".to_string(),
        args: vec![mu],
    })
}
