//! Decimal and bigdecimal method lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn bigdecimal_default_context_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "bigdecimal".to_string(),
            "Context".to_string(),
            "new".to_string(),
        ])),
        args: vec![
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "num".to_string(),
                    "NonZeroU64".to_string(),
                    "MIN".to_string(),
                ])),
                method: "saturating_add".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Int(27))],
            },
            RustExpr::Path(vec![
                "bigdecimal".to_string(),
                "RoundingMode".to_string(),
                "HalfEven".to_string(),
            ]),
        ],
    }
}

fn round_bigdecimal_with_default_context(value: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(bigdecimal_default_context_expr()),
        method: "round_decimal_ref".to_string(),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Paren(Box::new(value))),
        }],
    }
}

fn decimal_non_negative_scale_u32(scale: RustExpr) -> RustExpr {
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__scale".to_string(),
            ty: None,
            value: scale,
        }],
        expr: Some(Box::new(RustExpr::Cast {
            expr: Box::new(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__scale".to_string())),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                }),
                then_expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                else_expr: Some(Box::new(RustExpr::Ident("__scale".to_string()))),
            }),
            ty: RustType::Named("u32".to_string()),
        })),
    }
}

pub(super) fn lower_decimal_quantize(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }

    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "round_dp_with_strategy".to_string(),
        args: vec![
            decimal_non_negative_scale_u32(args[0].clone()),
            RustExpr::Path(vec![
                "rust_decimal".to_string(),
                "RoundingStrategy".to_string(),
                "MidpointNearestEven".to_string(),
            ]),
        ],
    })
}

pub(super) fn lower_decimal_sqrt(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }

    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "<Decimal as rust_decimal::MathematicalOps>".to_string(),
                "sqrt".to_string(),
            ])),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(object.clone()),
            }],
        }),
        method: "map_or_else".to_string(),
        args: vec![
            RustExpr::Closure {
                params: vec![],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                    args: vec![RustExpr::StructInit {
                        name: "DecimalConversionError".to_string(),
                        fields: vec![(
                            "message".to_string(),
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                                    "decimal.sqrt() is undefined for negative values".to_string(),
                                ))),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                        )],
                    }],
                }),
                is_move: false,
            },
            RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__v".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                    args: vec![RustExpr::Ident("__v".to_string())],
                }),
                is_move: false,
            },
        ],
    })
}

pub(super) fn lower_decimal_round(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    match args {
        [] => Some(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "round_dp_with_strategy".to_string(),
            args: vec![
                RustExpr::Literal(RustLiteral::Int(0)),
                RustExpr::Path(vec![
                    "rust_decimal".to_string(),
                    "RoundingStrategy".to_string(),
                    "MidpointNearestEven".to_string(),
                ]),
            ],
        }),
        [scale] => lower_decimal_quantize(object, std::slice::from_ref(scale)),
        _ => None,
    }
}

pub(super) fn lower_decimal_abs(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "abs".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_decimal_is_zero(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "is_zero".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_decimal_is_finite(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Literal(RustLiteral::Bool(true)))
}

pub(super) fn lower_bigdecimal_quantize(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }

    Some(round_bigdecimal_with_default_context(
        RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "with_scale_round".to_string(),
            args: vec![
                args[0].clone(),
                RustExpr::Path(vec![
                    "bigdecimal".to_string(),
                    "RoundingMode".to_string(),
                    "HalfEven".to_string(),
                ]),
            ],
        },
    ))
}

pub(super) fn lower_bigdecimal_sqrt(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }

    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "sqrt_with_context".to_string(),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(bigdecimal_default_context_expr()),
            }],
        }),
        method: "map_or_else".to_string(),
        args: vec![
            RustExpr::Closure {
                params: vec![],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                    args: vec![RustExpr::StructInit {
                        name: "DecimalConversionError".to_string(),
                        fields: vec![(
                            "message".to_string(),
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                                    "bigdecimal.sqrt() is undefined for negative values"
                                        .to_string(),
                                ))),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                        )],
                    }],
                }),
                is_move: false,
            },
            RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__v".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                    args: vec![round_bigdecimal_with_default_context(RustExpr::Ident(
                        "__v".to_string(),
                    ))],
                }),
                is_move: false,
            },
        ],
    })
}

pub(super) fn lower_bigdecimal_round(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    match args {
        [] => Some(round_bigdecimal_with_default_context(
            RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "with_scale_round".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Int(0)),
                    RustExpr::Path(vec![
                        "bigdecimal".to_string(),
                        "RoundingMode".to_string(),
                        "HalfEven".to_string(),
                    ]),
                ],
            },
        )),
        [scale] => lower_bigdecimal_quantize(object, std::slice::from_ref(scale)),
        _ => None,
    }
}

pub(super) fn lower_bigdecimal_abs(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(round_bigdecimal_with_default_context(
        RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "abs".to_string(),
            args: vec![],
        },
    ))
}

pub(super) fn lower_bigdecimal_is_zero(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(object.clone()),
        op: "==".to_string(),
        right: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "BigDecimal".to_string(),
                "from".to_string(),
            ])),
            args: vec![RustExpr::Literal(RustLiteral::Int(0))],
        }),
    })
}

pub(super) fn lower_bigdecimal_is_finite(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Literal(RustLiteral::Bool(true)))
}
