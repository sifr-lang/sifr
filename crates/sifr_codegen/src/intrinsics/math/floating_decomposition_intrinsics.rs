pub(super) fn lower_frexp(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__x".to_string(),
            ty: Some(RustType::F64),
            value: RustExpr::Cast {
                expr: Box::new(args[0].clone()),
                ty: RustType::F64,
            },
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__x".to_string())),
                op: "==".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
            }),
            then_expr: Box::new(RustExpr::Vec(vec![
                RustExpr::Ident("__x".to_string()),
                RustExpr::Literal(RustLiteral::Float(0.0)),
            ])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__x".to_string())),
                        method: "is_finite".to_string(),
                        args: vec![],
                    }),
                }),
                then_expr: Box::new(RustExpr::Vec(vec![
                    RustExpr::Ident("__x".to_string()),
                    RustExpr::Literal(RustLiteral::Float(0.0)),
                ])),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__bits".to_string(),
                            ty: Some(RustType::Named("u64".to_string())),
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                                method: "to_bits".to_string(),
                                args: vec![],
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__sign_mask".to_string(),
                            ty: Some(RustType::Named("u64".to_string())),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                    ty: RustType::Named("u64".to_string()),
                                }),
                                op: "<<".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Int(63))),
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__frac_mask".to_string(),
                            ty: Some(RustType::Named("u64".to_string())),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                        ty: RustType::Named("u64".to_string()),
                                    }),
                                    op: "<<".to_string(),
                                    right: Box::new(RustExpr::Literal(RustLiteral::Int(52))),
                                }),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                    ty: RustType::Named("u64".to_string()),
                                }),
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__sign".to_string(),
                            ty: Some(RustType::Named("u64".to_string())),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__bits".to_string())),
                                op: "&".to_string(),
                                right: Box::new(RustExpr::Ident("__sign_mask".to_string())),
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__exp".to_string(),
                            ty: Some(RustType::Named("i32".to_string())),
                            value: RustExpr::Cast {
                                expr: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__bits".to_string())),
                                        op: ">>".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Int(52))),
                                    }),
                                    op: "&".to_string(),
                                    right: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(2047))),
                                        ty: RustType::Named("u64".to_string()),
                                    }),
                                }),
                                ty: RustType::Named("i32".to_string()),
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__frac".to_string(),
                            ty: Some(RustType::Named("u64".to_string())),
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__bits".to_string())),
                                op: "&".to_string(),
                                right: Box::new(RustExpr::Ident("__frac_mask".to_string())),
                            },
                        },
                    ],
                    expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__exp".to_string())),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(RustExpr::Block {
                            stmts: vec![
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__scaled".to_string(),
                                    ty: Some(RustType::F64),
                                    value: RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__x".to_string())),
                                        op: "*".to_string(),
                                        right: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Cast {
                                                expr: Box::new(RustExpr::Literal(
                                                    RustLiteral::Float(2.0),
                                                )),
                                                ty: RustType::F64,
                                            }),
                                            method: "powi".to_string(),
                                            args: vec![RustExpr::Literal(RustLiteral::Int(54))],
                                        }),
                                    },
                                },
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__sbits".to_string(),
                                    ty: Some(RustType::Named("u64".to_string())),
                                    value: RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__scaled".to_string())),
                                        method: "to_bits".to_string(),
                                        args: vec![],
                                    },
                                },
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__sexp".to_string(),
                                    ty: Some(RustType::Named("i32".to_string())),
                                    value: RustExpr::Cast {
                                        expr: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Ident(
                                                    "__sbits".to_string(),
                                                )),
                                                op: ">>".to_string(),
                                                right: Box::new(RustExpr::Literal(
                                                    RustLiteral::Int(52),
                                                )),
                                            }),
                                            op: "&".to_string(),
                                            right: Box::new(RustExpr::Cast {
                                                expr: Box::new(RustExpr::Literal(
                                                    RustLiteral::Int(2047),
                                                )),
                                                ty: RustType::Named("u64".to_string()),
                                            }),
                                        }),
                                        ty: RustType::Named("i32".to_string()),
                                    },
                                },
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__sfrac".to_string(),
                                    ty: Some(RustType::Named("u64".to_string())),
                                    value: RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__sbits".to_string())),
                                        op: "&".to_string(),
                                        right: Box::new(RustExpr::Ident("__frac_mask".to_string())),
                                    },
                                },
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__mant".to_string(),
                                    ty: Some(RustType::F64),
                                    value: RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "f64".to_string(),
                                            "from_bits".to_string(),
                                        ])),
                                        args: vec![RustExpr::BinOp {
                                            left: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Ident(
                                                    "__sign".to_string(),
                                                )),
                                                op: "|".to_string(),
                                                right: Box::new(RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Cast {
                                                        expr: Box::new(RustExpr::Literal(
                                                            RustLiteral::Int(1022),
                                                        )),
                                                        ty: RustType::Named("u64".to_string()),
                                                    }),
                                                    op: "<<".to_string(),
                                                    right: Box::new(RustExpr::Literal(
                                                        RustLiteral::Int(52),
                                                    )),
                                                }),
                                            }),
                                            op: "|".to_string(),
                                            right: Box::new(RustExpr::Ident("__sfrac".to_string())),
                                        }],
                                    },
                                },
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__e".to_string(),
                                    ty: Some(RustType::Named("i32".to_string())),
                                    value: RustExpr::BinOp {
                                        left: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__sexp".to_string())),
                                            op: "-".to_string(),
                                            right: Box::new(RustExpr::Literal(RustLiteral::Int(
                                                1022,
                                            ))),
                                        }),
                                        op: "-".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Int(54))),
                                    },
                                },
                            ],
                            expr: Some(Box::new(RustExpr::Vec(vec![
                                RustExpr::Ident("__mant".to_string()),
                                RustExpr::Cast {
                                    expr: Box::new(RustExpr::Ident("__e".to_string())),
                                    ty: RustType::F64,
                                },
                            ]))),
                        }),
                        else_expr: Some(Box::new(RustExpr::Block {
                            stmts: vec![
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__mant".to_string(),
                                    ty: Some(RustType::F64),
                                    value: RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "f64".to_string(),
                                            "from_bits".to_string(),
                                        ])),
                                        args: vec![RustExpr::BinOp {
                                            left: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Ident(
                                                    "__sign".to_string(),
                                                )),
                                                op: "|".to_string(),
                                                right: Box::new(RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Cast {
                                                        expr: Box::new(RustExpr::Literal(
                                                            RustLiteral::Int(1022),
                                                        )),
                                                        ty: RustType::Named("u64".to_string()),
                                                    }),
                                                    op: "<<".to_string(),
                                                    right: Box::new(RustExpr::Literal(
                                                        RustLiteral::Int(52),
                                                    )),
                                                }),
                                            }),
                                            op: "|".to_string(),
                                            right: Box::new(RustExpr::Ident("__frac".to_string())),
                                        }],
                                    },
                                },
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__e".to_string(),
                                    ty: Some(RustType::Named("i32".to_string())),
                                    value: RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__exp".to_string())),
                                        op: "-".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Int(1022))),
                                    },
                                },
                            ],
                            expr: Some(Box::new(RustExpr::Vec(vec![
                                RustExpr::Ident("__mant".to_string()),
                                RustExpr::Cast {
                                    expr: Box::new(RustExpr::Ident("__e".to_string())),
                                    ty: RustType::F64,
                                },
                            ]))),
                        })),
                    })),
                })),
            })),
        })),
    })
}

pub(super) fn lower_ldexp(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::Cast {
            expr: Box::new(args[0].clone()),
            ty: RustType::F64,
        }),
        op: "*".to_string(),
        right: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Float(2.0))),
                ty: RustType::F64,
            }),
            method: "powi".to_string(),
            args: vec![RustExpr::Cast {
                expr: Box::new(args[1].clone()),
                ty: RustType::Named("i32".to_string()),
            }],
        }),
    })
}

pub(super) fn lower_modf(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__x".to_string(),
            ty: Some(RustType::F64),
            value: RustExpr::Cast {
                expr: Box::new(args[0].clone()),
                ty: RustType::F64,
            },
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                method: "is_nan".to_string(),
                args: vec![],
            }),
            then_expr: Box::new(RustExpr::Vec(vec![
                RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
                RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
            ])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__x".to_string())),
                    method: "is_infinite".to_string(),
                    args: vec![],
                }),
                then_expr: Box::new(RustExpr::Vec(vec![
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                            ty: RustType::F64,
                        }),
                        method: "copysign".to_string(),
                        args: vec![RustExpr::Ident("__x".to_string())],
                    },
                    RustExpr::Ident("__x".to_string()),
                ])),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__int".to_string(),
                            ty: None,
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                                method: "trunc".to_string(),
                                args: vec![],
                            },
                        },
                        RustStmt::Let {
                            mutable: true,
                            name: "__frac".to_string(),
                            ty: None,
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__x".to_string())),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Ident("__int".to_string())),
                            },
                        },
                        RustStmt::If {
                            cond: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__frac".to_string())),
                                op: "==".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                            },
                            then_body: vec![RustStmt::Assign {
                                target: RustExpr::Ident("__frac".to_string()),
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                                        ty: RustType::F64,
                                    }),
                                    method: "copysign".to_string(),
                                    args: vec![RustExpr::Ident("__x".to_string())],
                                },
                            }],
                            else_body: None,
                        },
                    ],
                    expr: Some(Box::new(RustExpr::Vec(vec![
                        RustExpr::Ident("__frac".to_string()),
                        RustExpr::Ident("__int".to_string()),
                    ]))),
                })),
            })),
        })),
    })
}

pub(super) fn lower_nextafter(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
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
                name: "__y".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Cast {
                    expr: Box::new(args[1].clone()),
                    ty: RustType::F64,
                },
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__x".to_string())),
                    method: "is_nan".to_string(),
                    args: vec![],
                }),
                op: "||".to_string(),
                right: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__y".to_string())),
                    method: "is_nan".to_string(),
                    args: vec![],
                }),
            }),
            then_expr: Box::new(RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__x".to_string())),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::Ident("__y".to_string())),
                }),
                then_expr: Box::new(RustExpr::Ident("__y".to_string())),
                else_expr: Some(Box::new(RustExpr::If {
                    cond: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__x".to_string())),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                    }),
                    then_expr: Box::new(RustExpr::Block {
                        stmts: vec![RustStmt::Let {
                            mutable: false,
                            name: "__sign".to_string(),
                            ty: Some(RustType::Named("u64".to_string())),
                            value: RustExpr::If {
                                cond: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__y".to_string())),
                                    method: "is_sign_negative".to_string(),
                                    args: vec![],
                                }),
                                then_expr: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                        ty: RustType::Named("u64".to_string()),
                                    }),
                                    op: "<<".to_string(),
                                    right: Box::new(RustExpr::Literal(RustLiteral::Int(63))),
                                }),
                                else_expr: Some(Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                                    ty: RustType::Named("u64".to_string()),
                                })),
                            },
                        }],
                        expr: Some(Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "f64".to_string(),
                                "from_bits".to_string(),
                            ])),
                            args: vec![RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__sign".to_string())),
                                op: "|".to_string(),
                                right: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                    ty: RustType::Named("u64".to_string()),
                                }),
                            }],
                        })),
                    }),
                    else_expr: Some(Box::new(RustExpr::Block {
                        stmts: vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "__bits".to_string(),
                                ty: Some(RustType::Named("u64".to_string())),
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__x".to_string())),
                                    method: "to_bits".to_string(),
                                    args: vec![],
                                },
                            },
                            RustStmt::If {
                                cond: RustExpr::BinOp {
                                    left: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__x".to_string())),
                                        op: "<".to_string(),
                                        right: Box::new(RustExpr::Ident("__y".to_string())),
                                    }),
                                    op: "==".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__x".to_string())),
                                        op: ">".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                                    }),
                                },
                                then_body: vec![RustStmt::AugAssign {
                                    target: RustExpr::Ident("__bits".to_string()),
                                    op: "+".to_string(),
                                    value: RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                        ty: RustType::Named("u64".to_string()),
                                    },
                                }],
                                else_body: Some(vec![RustStmt::AugAssign {
                                    target: RustExpr::Ident("__bits".to_string()),
                                    op: "-".to_string(),
                                    value: RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                        ty: RustType::Named("u64".to_string()),
                                    },
                                }]),
                            },
                        ],
                        expr: Some(Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "f64".to_string(),
                                "from_bits".to_string(),
                            ])),
                            args: vec![RustExpr::Ident("__bits".to_string())],
                        })),
                    })),
                })),
            })),
        })),
    })
}

pub(super) fn lower_ulp(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__x".to_string(),
            ty: Some(RustType::F64),
            value: RustExpr::Cast {
                expr: Box::new(args[0].clone()),
                ty: RustType::F64,
            },
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                method: "is_nan".to_string(),
                args: vec![],
            }),
            then_expr: Box::new(RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__x".to_string())),
                    method: "is_infinite".to_string(),
                    args: vec![],
                }),
                then_expr: Box::new(RustExpr::Path(vec![
                    "f64".to_string(),
                    "INFINITY".to_string(),
                ])),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![RustStmt::Let {
                        mutable: false,
                        name: "__a".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__x".to_string())),
                            method: "abs".to_string(),
                            args: vec![],
                        },
                    }],
                    expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__a".to_string())),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                        }),
                        then_expr: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "f64".to_string(),
                                "from_bits".to_string(),
                            ])),
                            args: vec![RustExpr::Cast {
                                expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                ty: RustType::Named("u64".to_string()),
                            }],
                        }),
                        else_expr: Some(Box::new(RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__a".to_string())),
                                op: "==".to_string(),
                                right: Box::new(RustExpr::Path(vec![
                                    "f64".to_string(),
                                    "MAX".to_string(),
                                ])),
                            }),
                            then_expr: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__a".to_string())),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "f64".to_string(),
                                        "from_bits".to_string(),
                                    ])),
                                    args: vec![RustExpr::BinOp {
                                        left: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__a".to_string())),
                                            method: "to_bits".to_string(),
                                            args: vec![],
                                        }),
                                        op: "-".to_string(),
                                        right: Box::new(RustExpr::Cast {
                                            expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                            ty: RustType::Named("u64".to_string()),
                                        }),
                                    }],
                                }),
                            }),
                            else_expr: Some(Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "f64".to_string(),
                                        "from_bits".to_string(),
                                    ])),
                                    args: vec![RustExpr::BinOp {
                                        left: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__a".to_string())),
                                            method: "to_bits".to_string(),
                                            args: vec![],
                                        }),
                                        op: "+".to_string(),
                                        right: Box::new(RustExpr::Cast {
                                            expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                            ty: RustType::Named("u64".to_string()),
                                        }),
                                    }],
                                }),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Ident("__a".to_string())),
                            })),
                        })),
                    })),
                })),
            })),
        })),
    })
}
