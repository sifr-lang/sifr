pub(super) fn lower_gamma(args: &[RustExpr]) -> Option<RustExpr> {
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
                name: "__g".to_string(),
                ty: Some(RustType::Named("usize".to_string())),
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(7))),
                    ty: RustType::Named("usize".to_string()),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__c".to_string(),
                ty: None,
                value: RustExpr::Vec(vec![
                    RustExpr::Literal(RustLiteral::Float(0.999_999_999_999_809_9)),
                    RustExpr::Literal(RustLiteral::Float(676.520_368_121_885_1)),
                    RustExpr::Literal(RustLiteral::Float(-1_259.139_216_722_402_8)),
                    RustExpr::Literal(RustLiteral::Float(771.323_428_777_653_1)),
                    RustExpr::Literal(RustLiteral::Float(-176.615_029_162_140_6)),
                    RustExpr::Literal(RustLiteral::Float(12.507_343_278_686_905)),
                    RustExpr::Literal(RustLiteral::Float(-0.138_571_095_265_720_12)),
                    RustExpr::Literal(RustLiteral::Float(0.000_009_984_369_578_019_572)),
                    RustExpr::Literal(RustLiteral::Float(0.000_000_150_563_273_514_931_16)),
                ]),
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__x".to_string())),
                    op: "<=".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                }),
                op: "&&".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__x".to_string())),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__x".to_string())),
                        method: "floor".to_string(),
                        args: vec![],
                    }),
                }),
            }),
            then_expr: Box::new(RustExpr::Path(vec![
                "f64".to_string(),
                "INFINITY".to_string(),
            ])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__x".to_string())),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Float(0.5))),
                }),
                then_expr: Box::new(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__xn".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Ident("__x".to_string())),
                            },
                        },
                        RustStmt::Let {
                            mutable: true,
                            name: "__s".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::Index {
                                expr: Box::new(RustExpr::Ident("__c".to_string())),
                                index: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                            },
                        },
                        RustStmt::For {
                            var: "__i".to_string(),
                            iter: RustExpr::Range {
                                start: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                end: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__g".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                                }),
                            },
                            body: vec![RustStmt::AugAssign {
                                target: RustExpr::Ident("__s".to_string()),
                                op: "+".to_string(),
                                value: RustExpr::BinOp {
                                    left: Box::new(RustExpr::Index {
                                        expr: Box::new(RustExpr::Ident("__c".to_string())),
                                        index: Box::new(RustExpr::Ident("__i".to_string())),
                                    }),
                                    op: "/".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__xn".to_string())),
                                            op: "+".to_string(),
                                            right: Box::new(RustExpr::Cast {
                                                expr: Box::new(RustExpr::Ident("__i".to_string())),
                                                ty: RustType::F64,
                                            }),
                                        }),
                                        op: "-".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                                    }),
                                },
                            }],
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__t2".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__xn".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Ident("__g".to_string())),
                                        ty: RustType::F64,
                                    }),
                                }),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.5))),
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__base".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Literal(
                                                    RustLiteral::Float(2.0),
                                                )),
                                                op: "*".to_string(),
                                                right: Box::new(RustExpr::Path(vec![
                                                    "std".to_string(),
                                                    "f64".to_string(),
                                                    "consts".to_string(),
                                                    "PI".to_string(),
                                                ])),
                                            }),
                                            method: "sqrt".to_string(),
                                            args: vec![],
                                        }),
                                        op: "*".to_string(),
                                        right: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__t2".to_string())),
                                            method: "powf".to_string(),
                                            args: vec![RustExpr::BinOp {
                                                left: Box::new(RustExpr::Ident("__xn".to_string())),
                                                op: "-".to_string(),
                                                right: Box::new(RustExpr::Literal(
                                                    RustLiteral::Float(0.5),
                                                )),
                                            }],
                                        }),
                                    }),
                                    op: "*".to_string(),
                                    right: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Literal(RustLiteral::Float(
                                                0.0,
                                            ))),
                                            op: "-".to_string(),
                                            right: Box::new(RustExpr::Ident("__t2".to_string())),
                                        }),
                                        method: "exp".to_string(),
                                        args: vec![],
                                    }),
                                }),
                                op: "*".to_string(),
                                right: Box::new(RustExpr::Ident("__s".to_string())),
                            },
                        },
                    ],
                    expr: Some(Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "f64".to_string(),
                            "consts".to_string(),
                            "PI".to_string(),
                        ])),
                        op: "/".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__x".to_string())),
                                    op: "*".to_string(),
                                    right: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "f64".to_string(),
                                        "consts".to_string(),
                                        "PI".to_string(),
                                    ])),
                                }),
                                method: "sin".to_string(),
                                args: vec![],
                            }),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::Ident("__base".to_string())),
                        }),
                    })),
                }),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__xm".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__x".to_string())),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                            },
                        },
                        RustStmt::Let {
                            mutable: true,
                            name: "__s".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::Index {
                                expr: Box::new(RustExpr::Ident("__c".to_string())),
                                index: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                            },
                        },
                        RustStmt::For {
                            var: "__i".to_string(),
                            iter: RustExpr::Range {
                                start: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                end: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__g".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                                }),
                            },
                            body: vec![RustStmt::AugAssign {
                                target: RustExpr::Ident("__s".to_string()),
                                op: "+".to_string(),
                                value: RustExpr::BinOp {
                                    left: Box::new(RustExpr::Index {
                                        expr: Box::new(RustExpr::Ident("__c".to_string())),
                                        index: Box::new(RustExpr::Ident("__i".to_string())),
                                    }),
                                    op: "/".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__xm".to_string())),
                                        op: "+".to_string(),
                                        right: Box::new(RustExpr::Cast {
                                            expr: Box::new(RustExpr::Ident("__i".to_string())),
                                            ty: RustType::F64,
                                        }),
                                    }),
                                },
                            }],
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__t2".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__xm".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Ident("__g".to_string())),
                                        ty: RustType::F64,
                                    }),
                                }),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.5))),
                            },
                        },
                    ],
                    expr: Some(Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Literal(RustLiteral::Float(2.0))),
                                        op: "*".to_string(),
                                        right: Box::new(RustExpr::Path(vec![
                                            "std".to_string(),
                                            "f64".to_string(),
                                            "consts".to_string(),
                                            "PI".to_string(),
                                        ])),
                                    }),
                                    method: "sqrt".to_string(),
                                    args: vec![],
                                }),
                                op: "*".to_string(),
                                right: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__t2".to_string())),
                                    method: "powf".to_string(),
                                    args: vec![RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__xm".to_string())),
                                        op: "+".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.5))),
                                    }],
                                }),
                            }),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                                    op: "-".to_string(),
                                    right: Box::new(RustExpr::Ident("__t2".to_string())),
                                }),
                                method: "exp".to_string(),
                                args: vec![],
                            }),
                        }),
                        op: "*".to_string(),
                        right: Box::new(RustExpr::Ident("__s".to_string())),
                    })),
                })),
            })),
        })),
    })
}

pub(super) fn lower_lgamma(args: &[RustExpr]) -> Option<RustExpr> {
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
                name: "__g".to_string(),
                ty: Some(RustType::Named("usize".to_string())),
                value: RustExpr::Literal(RustLiteral::Int(7)),
            },
            RustStmt::Let {
                mutable: false,
                name: "__c".to_string(),
                ty: None,
                value: RustExpr::Vec(vec![
                    RustExpr::Literal(RustLiteral::Float(0.999_999_999_999_809_9)),
                    RustExpr::Literal(RustLiteral::Float(676.520_368_121_885_1)),
                    RustExpr::Literal(RustLiteral::Float(-1_259.139_216_722_402_8)),
                    RustExpr::Literal(RustLiteral::Float(771.323_428_777_653_1)),
                    RustExpr::Literal(RustLiteral::Float(-176.615_029_162_140_6)),
                    RustExpr::Literal(RustLiteral::Float(12.507_343_278_686_905)),
                    RustExpr::Literal(RustLiteral::Float(-0.138_571_095_265_720_12)),
                    RustExpr::Literal(RustLiteral::Float(0.000_009_984_369_578_019_572)),
                    RustExpr::Literal(RustLiteral::Float(0.000_000_150_563_273_514_931_16)),
                ]),
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__x".to_string())),
                    op: "<=".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                }),
                op: "&&".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__x".to_string())),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__x".to_string())),
                        method: "floor".to_string(),
                        args: vec![],
                    }),
                }),
            }),
            then_expr: Box::new(RustExpr::Path(vec![
                "f64".to_string(),
                "INFINITY".to_string(),
            ])),
            else_expr: Some(Box::new(RustExpr::Block {
                stmts: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__xm".to_string(),
                        ty: Some(RustType::F64),
                        value: RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__x".to_string())),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.5))),
                            }),
                            then_expr: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Ident("__x".to_string())),
                            }),
                            else_expr: Some(Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__x".to_string())),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Float(1.0))),
                            })),
                        },
                    },
                    RustStmt::Let {
                        mutable: true,
                        name: "__s".to_string(),
                        ty: Some(RustType::F64),
                        value: RustExpr::Index {
                            expr: Box::new(RustExpr::Ident("__c".to_string())),
                            index: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                        },
                    },
                    RustStmt::For {
                        var: "__i".to_string(),
                        iter: RustExpr::Range {
                            start: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                            end: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__g".to_string())),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                            }),
                        },
                        body: vec![RustStmt::AugAssign {
                            target: RustExpr::Ident("__s".to_string()),
                            op: "+".to_string(),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::Index {
                                    expr: Box::new(RustExpr::Ident("__c".to_string())),
                                    index: Box::new(RustExpr::Ident("__i".to_string())),
                                }),
                                op: "/".to_string(),
                                right: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__xm".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Ident("__i".to_string())),
                                        ty: RustType::F64,
                                    }),
                                }),
                            },
                        }],
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__t2".to_string(),
                        ty: Some(RustType::F64),
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__xm".to_string())),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Ident("__g".to_string())),
                                    ty: RustType::F64,
                                }),
                            }),
                            op: "+".to_string(),
                            right: Box::new(RustExpr::Literal(RustLiteral::Float(0.5))),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__r".to_string(),
                        ty: Some(RustType::F64),
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Literal(
                                                    RustLiteral::Float(2.0),
                                                )),
                                                op: "*".to_string(),
                                                right: Box::new(RustExpr::Path(vec![
                                                    "std".to_string(),
                                                    "f64".to_string(),
                                                    "consts".to_string(),
                                                    "PI".to_string(),
                                                ])),
                                            }),
                                            method: "sqrt".to_string(),
                                            args: vec![],
                                        }),
                                        method: "ln".to_string(),
                                        args: vec![],
                                    }),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__xm".to_string())),
                                            op: "+".to_string(),
                                            right: Box::new(RustExpr::Literal(RustLiteral::Float(
                                                0.5,
                                            ))),
                                        }),
                                        op: "*".to_string(),
                                        right: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__t2".to_string())),
                                            method: "ln".to_string(),
                                            args: vec![],
                                        }),
                                    }),
                                }),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Ident("__t2".to_string())),
                            }),
                            op: "+".to_string(),
                            right: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__s".to_string())),
                                method: "ln".to_string(),
                                args: vec![],
                            }),
                        },
                    },
                ],
                expr: Some(Box::new(RustExpr::If {
                    cond: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__x".to_string())),
                        op: "<".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.5))),
                    }),
                    then_expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Path(vec![
                                    "std".to_string(),
                                    "f64".to_string(),
                                    "consts".to_string(),
                                    "PI".to_string(),
                                ])),
                                op: "/".to_string(),
                                right: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__x".to_string())),
                                            op: "*".to_string(),
                                            right: Box::new(RustExpr::Path(vec![
                                                "std".to_string(),
                                                "f64".to_string(),
                                                "consts".to_string(),
                                                "PI".to_string(),
                                            ])),
                                        }),
                                        method: "sin".to_string(),
                                        args: vec![],
                                    }),
                                    op: "*".to_string(),
                                    right: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__r".to_string())),
                                        method: "exp".to_string(),
                                        args: vec![],
                                    }),
                                }),
                            }),
                            method: "abs".to_string(),
                            args: vec![],
                        }),
                        method: "ln".to_string(),
                        args: vec![],
                    }),
                    else_expr: Some(Box::new(RustExpr::Ident("__r".to_string()))),
                })),
            })),
        })),
    })
}

