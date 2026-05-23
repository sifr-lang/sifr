use crate::{RustExpr, RustLiteral, RustStmt, RustType};

fn parenthesized(expr: &RustExpr) -> RustExpr {
    RustExpr::Paren(Box::new(expr.clone()))
}

fn unary_method(args: &[RustExpr], method: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(parenthesized(&args[0])),
        method: method.to_string(),
        args: vec![],
    })
}

fn unary_method_as_i64(args: &[RustExpr], method: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(parenthesized(&args[0])),
            method: method.to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}

fn binary_method(args: &[RustExpr], method: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(parenthesized(&args[0])),
        method: method.to_string(),
        args: vec![args[1].clone()],
    })
}

pub(super) fn lower_sqrt(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "sqrt")
}

pub(super) fn lower_abs_val(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "abs")
}

pub(super) fn lower_log(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "ln")
}

pub(super) fn lower_cbrt(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "cbrt")
}

pub(super) fn lower_exp2(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "exp2")
}

pub(super) fn lower_sin(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "sin")
}

pub(super) fn lower_cos(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "cos")
}

pub(super) fn lower_tan(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "tan")
}

pub(super) fn lower_asin(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "asin")
}

pub(super) fn lower_acos(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "acos")
}

pub(super) fn lower_atan(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "atan")
}

pub(super) fn lower_sinh(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "sinh")
}

pub(super) fn lower_cosh(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "cosh")
}

pub(super) fn lower_tanh(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "tanh")
}

pub(super) fn lower_asinh(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "asinh")
}

pub(super) fn lower_acosh(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "acosh")
}

pub(super) fn lower_atanh(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "atanh")
}

pub(super) fn lower_floor(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method_as_i64(args, "floor")
}

pub(super) fn lower_ceil(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method_as_i64(args, "ceil")
}

pub(super) fn lower_round(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "round")
}

pub(super) fn lower_trunc(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method_as_i64(args, "trunc")
}

pub(super) fn lower_fract(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "fract")
}

pub(super) fn lower_exp(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "exp")
}

pub(super) fn lower_ln(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "ln")
}

pub(super) fn lower_log10(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "log10")
}

pub(super) fn lower_log2(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "log2")
}

pub(super) fn lower_degrees(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "to_degrees")
}

pub(super) fn lower_radians(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "to_radians")
}

pub(super) fn lower_isnan(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "is_nan")
}

pub(super) fn lower_isinf(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "is_infinite")
}

pub(super) fn lower_copysign(args: &[RustExpr]) -> Option<RustExpr> {
    binary_method(args, "copysign")
}

pub(super) fn lower_signbit(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "is_sign_negative")
}

pub(super) fn lower_fmod(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(parenthesized(&args[0])),
        op: "%".to_string(),
        right: Box::new(parenthesized(&args[1])),
    })
}

pub(super) fn lower_hypot(args: &[RustExpr]) -> Option<RustExpr> {
    binary_method(args, "hypot")
}

pub(super) fn lower_fma(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::BinOp {
            left: Box::new(parenthesized(&args[0])),
            op: "*".to_string(),
            right: Box::new(parenthesized(&args[1])),
        }),
        op: "+".to_string(),
        right: Box::new(parenthesized(&args[2])),
    })
}

pub(super) fn lower_pow_val(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(parenthesized(&args[0])),
        method: "powf".to_string(),
        args: vec![args[1].clone()],
    })
}

pub(super) fn lower_min_val(args: &[RustExpr]) -> Option<RustExpr> {
    binary_method(args, "min")
}

pub(super) fn lower_max_val(args: &[RustExpr]) -> Option<RustExpr> {
    binary_method(args, "max")
}

pub(super) fn lower_fmax(args: &[RustExpr]) -> Option<RustExpr> {
    binary_method(args, "max")
}

pub(super) fn lower_fmin(args: &[RustExpr]) -> Option<RustExpr> {
    binary_method(args, "min")
}

pub(super) fn lower_expm1(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "exp_m1")
}

pub(super) fn lower_round_val(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method_as_i64(args, "round")
}

pub(super) fn lower_atan2(args: &[RustExpr]) -> Option<RustExpr> {
    binary_method(args, "atan2")
}

pub(super) fn lower_log1p(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "ln_1p")
}

pub(super) fn lower_fabs(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "abs")
}

pub(super) fn lower_isfinite(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "is_finite")
}

pub(super) fn lower_isnormal(args: &[RustExpr]) -> Option<RustExpr> {
    unary_method(args, "is_normal")
}

pub(super) fn lower_issubnormal(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let arg = parenthesized(&args[0]);
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::MethodCall {
            receiver: Box::new(arg.clone()),
            method: "is_finite".to_string(),
            args: vec![],
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(RustExpr::MethodCall {
                receiver: Box::new(arg),
                method: "is_normal".to_string(),
                args: vec![],
            }),
        }),
    })
}

pub(super) fn lower_isqrt(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Cast {
                expr: Box::new(parenthesized(&args[0])),
                ty: RustType::F64,
            }),
            method: "sqrt".to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}

pub(super) fn lower_remainder(args: &[RustExpr]) -> Option<RustExpr> {
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
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__y".to_string())),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                    }),
                    op: "||".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__x".to_string())),
                        method: "is_infinite".to_string(),
                        args: vec![],
                    }),
                }),
                then_expr: Box::new(RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()])),
                else_expr: Some(Box::new(RustExpr::If {
                    cond: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__y".to_string())),
                        method: "is_infinite".to_string(),
                        args: vec![],
                    }),
                    then_expr: Box::new(RustExpr::Ident("__x".to_string())),
                    else_expr: Some(Box::new(RustExpr::Block {
                        stmts: vec![
                            RustStmt::Let {
                                mutable: false,
                                name: "__q".to_string(),
                                ty: Some(RustType::F64),
                                value: RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__x".to_string())),
                                    op: "/".to_string(),
                                    right: Box::new(RustExpr::Ident("__y".to_string())),
                                },
                            },
                            RustStmt::Let {
                                mutable: false,
                                name: "__n0".to_string(),
                                ty: Some(RustType::F64),
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__q".to_string())),
                                    method: "trunc".to_string(),
                                    args: vec![],
                                },
                            },
                            RustStmt::Let {
                                mutable: false,
                                name: "__frac".to_string(),
                                ty: Some(RustType::F64),
                                value: RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__q".to_string())),
                                    op: "-".to_string(),
                                    right: Box::new(RustExpr::Ident("__n0".to_string())),
                                },
                            },
                            RustStmt::Let {
                                mutable: false,
                                name: "__abs_frac".to_string(),
                                ty: Some(RustType::F64),
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__frac".to_string())),
                                    method: "abs".to_string(),
                                    args: vec![],
                                },
                            },
                            RustStmt::Let {
                                mutable: false,
                                name: "__n".to_string(),
                                ty: Some(RustType::F64),
                                value: RustExpr::If {
                                    cond: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__abs_frac".to_string())),
                                        op: "<".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.5))),
                                    }),
                                    then_expr: Box::new(RustExpr::Ident("__n0".to_string())),
                                    else_expr: Some(Box::new(RustExpr::If {
                                        cond: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident(
                                                "__abs_frac".to_string(),
                                            )),
                                            op: ">".to_string(),
                                            right: Box::new(RustExpr::Literal(RustLiteral::Float(
                                                0.5,
                                            ))),
                                        }),
                                        then_expr: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__n0".to_string())),
                                            op: "+".to_string(),
                                            right: Box::new(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "__q".to_string(),
                                                )),
                                                method: "signum".to_string(),
                                                args: vec![],
                                            }),
                                        }),
                                        else_expr: Some(Box::new(RustExpr::If {
                                            cond: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Cast {
                                                        expr: Box::new(RustExpr::Ident(
                                                            "__n0".to_string(),
                                                        )),
                                                        ty: RustType::I64,
                                                    }),
                                                    op: "%".to_string(),
                                                    right: Box::new(RustExpr::Literal(
                                                        RustLiteral::Int(2),
                                                    )),
                                                }),
                                                op: "==".to_string(),
                                                right: Box::new(RustExpr::Literal(
                                                    RustLiteral::Int(0),
                                                )),
                                            }),
                                            then_expr: Box::new(RustExpr::Ident(
                                                "__n0".to_string(),
                                            )),
                                            else_expr: Some(Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Ident("__n0".to_string())),
                                                op: "+".to_string(),
                                                right: Box::new(RustExpr::MethodCall {
                                                    receiver: Box::new(RustExpr::Ident(
                                                        "__q".to_string(),
                                                    )),
                                                    method: "signum".to_string(),
                                                    args: vec![],
                                                }),
                                            })),
                                        })),
                                    })),
                                },
                            },
                            RustStmt::Let {
                                mutable: false,
                                name: "__r".to_string(),
                                ty: Some(RustType::F64),
                                value: RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__x".to_string())),
                                    op: "-".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__n".to_string())),
                                        op: "*".to_string(),
                                        right: Box::new(RustExpr::Ident("__y".to_string())),
                                    }),
                                },
                            },
                        ],
                        expr: Some(Box::new(RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__r".to_string())),
                                op: "==".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                            }),
                            then_expr: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                                    ty: RustType::F64,
                                }),
                                method: "copysign".to_string(),
                                args: vec![RustExpr::Ident("__x".to_string())],
                            }),
                            else_expr: Some(Box::new(RustExpr::Ident("__r".to_string()))),
                        })),
                    })),
                })),
            })),
        })),
    })
}

pub(super) fn lower_dist(args: &[RustExpr]) -> Option<RustExpr> {
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
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__p".to_string())),
                    method: "len".to_string(),
                    args: vec![],
                }),
                op: "!=".to_string(),
                right: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__q".to_string())),
                    method: "len".to_string(),
                    args: vec![],
                }),
            }),
            then_expr: Box::new(RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__p".to_string())),
                    method: "is_empty".to_string(),
                    args: vec![],
                }),
                then_expr: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: true,
                            name: "__scale".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::Literal(RustLiteral::Float(0.0)),
                        },
                        RustStmt::Let {
                            mutable: true,
                            name: "__ssq".to_string(),
                            ty: Some(RustType::F64),
                            value: RustExpr::Literal(RustLiteral::Float(1.0)),
                        },
                        RustStmt::For {
                            var: "__i".to_string(),
                            iter: RustExpr::Range {
                                start: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                                end: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__p".to_string())),
                                    method: "len".to_string(),
                                    args: vec![],
                                }),
                            },
                            body: vec![
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__d".to_string(),
                                    ty: Some(RustType::F64),
                                    value: RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Index {
                                                expr: Box::new(RustExpr::Ident("__p".to_string())),
                                                index: Box::new(RustExpr::Ident("__i".to_string())),
                                            }),
                                            op: "-".to_string(),
                                            right: Box::new(RustExpr::Index {
                                                expr: Box::new(RustExpr::Ident("__q".to_string())),
                                                index: Box::new(RustExpr::Ident("__i".to_string())),
                                            }),
                                        }),
                                        method: "abs".to_string(),
                                        args: vec![],
                                    },
                                },
                                RustStmt::If {
                                    cond: RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__d".to_string())),
                                        op: "!=".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                                    },
                                    then_body: vec![RustStmt::If {
                                        cond: RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__scale".to_string())),
                                            op: "<".to_string(),
                                            right: Box::new(RustExpr::Ident("__d".to_string())),
                                        },
                                        then_body: vec![
                                            RustStmt::Let {
                                                mutable: false,
                                                name: "__r".to_string(),
                                                ty: Some(RustType::F64),
                                                value: RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Ident(
                                                        "__scale".to_string(),
                                                    )),
                                                    op: "/".to_string(),
                                                    right: Box::new(RustExpr::Ident(
                                                        "__d".to_string(),
                                                    )),
                                                },
                                            },
                                            RustStmt::Assign {
                                                target: RustExpr::Ident("__ssq".to_string()),
                                                value: RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Literal(
                                                        RustLiteral::Float(1.0),
                                                    )),
                                                    op: "+".to_string(),
                                                    right: Box::new(RustExpr::BinOp {
                                                        left: Box::new(RustExpr::BinOp {
                                                            left: Box::new(RustExpr::Ident(
                                                                "__ssq".to_string(),
                                                            )),
                                                            op: "*".to_string(),
                                                            right: Box::new(RustExpr::Ident(
                                                                "__r".to_string(),
                                                            )),
                                                        }),
                                                        op: "*".to_string(),
                                                        right: Box::new(RustExpr::Ident(
                                                            "__r".to_string(),
                                                        )),
                                                    }),
                                                },
                                            },
                                            RustStmt::Assign {
                                                target: RustExpr::Ident("__scale".to_string()),
                                                value: RustExpr::Ident("__d".to_string()),
                                            },
                                        ],
                                        else_body: Some(vec![
                                            RustStmt::Let {
                                                mutable: false,
                                                name: "__r".to_string(),
                                                ty: Some(RustType::F64),
                                                value: RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Ident(
                                                        "__d".to_string(),
                                                    )),
                                                    op: "/".to_string(),
                                                    right: Box::new(RustExpr::Ident(
                                                        "__scale".to_string(),
                                                    )),
                                                },
                                            },
                                            RustStmt::AugAssign {
                                                target: RustExpr::Ident("__ssq".to_string()),
                                                op: "+".to_string(),
                                                value: RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Ident(
                                                        "__r".to_string(),
                                                    )),
                                                    op: "*".to_string(),
                                                    right: Box::new(RustExpr::Ident(
                                                        "__r".to_string(),
                                                    )),
                                                },
                                            },
                                        ]),
                                    }],
                                    else_body: None,
                                },
                            ],
                        },
                    ],
                    expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__scale".to_string())),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                        }),
                        then_expr: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                        else_expr: Some(Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__scale".to_string())),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__ssq".to_string())),
                                method: "sqrt".to_string(),
                                args: vec![],
                            }),
                        })),
                    })),
                })),
            })),
        })),
    })
}

