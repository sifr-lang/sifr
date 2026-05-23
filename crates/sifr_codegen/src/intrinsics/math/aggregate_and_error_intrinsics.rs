pub(super) fn lower_fsum(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__data".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(args[0].clone()),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__sum".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Literal(RustLiteral::Float(0.0)),
            },
            RustStmt::Let {
                mutable: true,
                name: "__comp".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Literal(RustLiteral::Float(0.0)),
            },
            RustStmt::Let {
                mutable: true,
                name: "__pos_inf".to_string(),
                ty: Some(RustType::Bool),
                value: RustExpr::Literal(RustLiteral::Bool(false)),
            },
            RustStmt::Let {
                mutable: true,
                name: "__neg_inf".to_string(),
                ty: Some(RustType::Bool),
                value: RustExpr::Literal(RustLiteral::Bool(false)),
            },
            RustStmt::Let {
                mutable: true,
                name: "__has_nan".to_string(),
                ty: Some(RustType::Bool),
                value: RustExpr::Literal(RustLiteral::Bool(false)),
            },
            RustStmt::For {
                var: "__x".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__data".to_string())),
                    method: "iter".to_string(),
                    args: vec![],
                },
                body: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__v".to_string(),
                        ty: Some(RustType::F64),
                        value: RustExpr::Deref(Box::new(RustExpr::Ident("__x".to_string()))),
                    },
                    RustStmt::If {
                        cond: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__v".to_string())),
                            method: "is_nan".to_string(),
                            args: vec![],
                        },
                        then_body: vec![
                            RustStmt::Assign {
                                target: RustExpr::Ident("__has_nan".to_string()),
                                value: RustExpr::Literal(RustLiteral::Bool(true)),
                            },
                            RustStmt::Continue,
                        ],
                        else_body: None,
                    },
                    RustStmt::If {
                        cond: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__v".to_string())),
                            method: "is_infinite".to_string(),
                            args: vec![],
                        },
                        then_body: vec![
                            RustStmt::If {
                                cond: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__v".to_string())),
                                    method: "is_sign_positive".to_string(),
                                    args: vec![],
                                },
                                then_body: vec![RustStmt::Assign {
                                    target: RustExpr::Ident("__pos_inf".to_string()),
                                    value: RustExpr::Literal(RustLiteral::Bool(true)),
                                }],
                                else_body: Some(vec![RustStmt::Assign {
                                    target: RustExpr::Ident("__neg_inf".to_string()),
                                    value: RustExpr::Literal(RustLiteral::Bool(true)),
                                }]),
                            },
                            RustStmt::Continue,
                        ],
                        else_body: None,
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__t".to_string(),
                        ty: Some(RustType::F64),
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__sum".to_string())),
                            op: "+".to_string(),
                            right: Box::new(RustExpr::Ident("__v".to_string())),
                        },
                    },
                    RustStmt::If {
                        cond: RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__sum".to_string())),
                                method: "abs".to_string(),
                                args: vec![],
                            }),
                            op: ">=".to_string(),
                            right: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__v".to_string())),
                                method: "abs".to_string(),
                                args: vec![],
                            }),
                        },
                        then_body: vec![RustStmt::AugAssign {
                            target: RustExpr::Ident("__comp".to_string()),
                            op: "+".to_string(),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__sum".to_string())),
                                    op: "-".to_string(),
                                    right: Box::new(RustExpr::Ident("__t".to_string())),
                                }),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::Ident("__v".to_string())),
                            },
                        }],
                        else_body: Some(vec![RustStmt::AugAssign {
                            target: RustExpr::Ident("__comp".to_string()),
                            op: "+".to_string(),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__v".to_string())),
                                    op: "-".to_string(),
                                    right: Box::new(RustExpr::Ident("__t".to_string())),
                                }),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::Ident("__sum".to_string())),
                            },
                        }]),
                    },
                    RustStmt::Assign {
                        target: RustExpr::Ident("__sum".to_string()),
                        value: RustExpr::Ident("__t".to_string()),
                    },
                ],
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__has_nan".to_string())),
                op: "||".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__pos_inf".to_string())),
                    op: "&&".to_string(),
                    right: Box::new(RustExpr::Ident("__neg_inf".to_string())),
                }),
            }),
            then_expr: Box::new(RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::Ident("__pos_inf".to_string())),
                then_expr: Box::new(RustExpr::Path(vec![
                    "f64".to_string(),
                    "INFINITY".to_string(),
                ])),
                else_expr: Some(Box::new(RustExpr::If {
                    cond: Box::new(RustExpr::Ident("__neg_inf".to_string())),
                    then_expr: Box::new(RustExpr::Path(vec![
                        "f64".to_string(),
                        "NEG_INFINITY".to_string(),
                    ])),
                    else_expr: Some(Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__sum".to_string())),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::Ident("__comp".to_string())),
                    })),
                })),
            })),
        })),
    })
}

pub(super) fn lower_sumprod(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__p".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(args[0].clone()),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__q".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(args[1].clone()),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__len".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__p".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                    method: "min".to_string(),
                    args: vec![RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__q".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }],
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__sum".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Literal(RustLiteral::Float(0.0)),
            },
            RustStmt::For {
                var: "__i".to_string(),
                iter: RustExpr::Range {
                    start: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    end: Box::new(RustExpr::Ident("__len".to_string())),
                },
                body: vec![RustStmt::AugAssign {
                    target: RustExpr::Ident("__sum".to_string()),
                    op: "+".to_string(),
                    value: RustExpr::BinOp {
                        left: Box::new(RustExpr::Index {
                            expr: Box::new(RustExpr::Ident("__p".to_string())),
                            index: Box::new(RustExpr::Ident("__i".to_string())),
                        }),
                        op: "*".to_string(),
                        right: Box::new(RustExpr::Index {
                            expr: Box::new(RustExpr::Ident("__q".to_string())),
                            index: Box::new(RustExpr::Ident("__i".to_string())),
                        }),
                    },
                }],
            },
        ],
        expr: Some(Box::new(RustExpr::Ident("__sum".to_string()))),
    })
}

pub(super) fn lower_erf(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__x".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Cast {
                    expr: Box::new(args[0].clone()),
                    ty: RustType::F64,
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__t".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                    op: "/".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Literal(RustLiteral::Float(0.327_591_1))),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                                method: "abs".to_string(),
                                args: vec![],
                            }),
                        }),
                    }),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__poly".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__t".to_string())),
                    op: "*".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Literal(RustLiteral::Float(0.254_829_592))),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__t".to_string())),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Literal(RustLiteral::Float(
                                    -0.284_496_736,
                                ))),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__t".to_string())),
                                    op: "*".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Literal(RustLiteral::Float(
                                            1.421_413_741,
                                        ))),
                                        op: "+".to_string(),
                                        right: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__t".to_string())),
                                            op: "*".to_string(),
                                            right: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Literal(
                                                    RustLiteral::Float(-1.453_152_027),
                                                )),
                                                op: "+".to_string(),
                                                right: Box::new(RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Ident(
                                                        "__t".to_string(),
                                                    )),
                                                    op: "*".to_string(),
                                                    right: Box::new(RustExpr::Literal(
                                                        RustLiteral::Float(1.061_405_429),
                                                    )),
                                                }),
                                            }),
                                        }),
                                    }),
                                }),
                            }),
                        }),
                    }),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__r".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                    op: "-".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__poly".to_string())),
                        op: "*".to_string(),
                        right: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::UnaryOp {
                                    op: "-".to_string(),
                                    operand: Box::new(RustExpr::Ident("__x".to_string())),
                                }),
                                op: "*".to_string(),
                                right: Box::new(RustExpr::Ident("__x".to_string())),
                            }),
                            method: "exp".to_string(),
                            args: vec![],
                        }),
                    }),
                },
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__x".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
            }),
            then_expr: Box::new(RustExpr::Ident("__r".to_string())),
            else_expr: Some(Box::new(RustExpr::UnaryOp {
                op: "-".to_string(),
                operand: Box::new(RustExpr::Ident("__r".to_string())),
            })),
        })),
    })
}

pub(super) fn lower_erfc(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__x".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Cast {
                    expr: Box::new(args[0].clone()),
                    ty: RustType::F64,
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__t".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                    op: "/".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Literal(RustLiteral::Float(0.327_591_1))),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                                method: "abs".to_string(),
                                args: vec![],
                            }),
                        }),
                    }),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__poly".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__t".to_string())),
                    op: "*".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Literal(RustLiteral::Float(0.254_829_592))),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__t".to_string())),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Literal(RustLiteral::Float(
                                    -0.284_496_736,
                                ))),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__t".to_string())),
                                    op: "*".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Literal(RustLiteral::Float(
                                            1.421_413_741,
                                        ))),
                                        op: "+".to_string(),
                                        right: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__t".to_string())),
                                            op: "*".to_string(),
                                            right: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Literal(
                                                    RustLiteral::Float(-1.453_152_027),
                                                )),
                                                op: "+".to_string(),
                                                right: Box::new(RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Ident(
                                                        "__t".to_string(),
                                                    )),
                                                    op: "*".to_string(),
                                                    right: Box::new(RustExpr::Literal(
                                                        RustLiteral::Float(1.061_405_429),
                                                    )),
                                                }),
                                            }),
                                        }),
                                    }),
                                }),
                            }),
                        }),
                    }),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__r".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__poly".to_string())),
                    op: "*".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::UnaryOp {
                                op: "-".to_string(),
                                operand: Box::new(RustExpr::Ident("__x".to_string())),
                            }),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::Ident("__x".to_string())),
                        }),
                        method: "exp".to_string(),
                        args: vec![],
                    }),
                },
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__x".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
            }),
            then_expr: Box::new(RustExpr::Ident("__r".to_string())),
            else_expr: Some(Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Literal(RustLiteral::Float(2.0))),
                op: "-".to_string(),
                right: Box::new(RustExpr::Ident("__r".to_string())),
            })),
        })),
    })
}

