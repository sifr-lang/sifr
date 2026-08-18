//! Bytes method lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustStmt, RustType};

fn int(v: i64) -> RustExpr {
    RustExpr::Literal(crate::RustLiteral::Int(v))
}

fn list_bound_expr(arg: Option<&RustExpr>, default: i64) -> RustExpr {
    let Some(arg) = arg else {
        return int(default);
    };
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__bound".to_string(),
            ty: None,
            value: arg.clone(),
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__bound".to_string())),
                op: "<".to_string(),
                right: Box::new(int(0)),
            }),
            then_expr: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__len".to_string())),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::Ident("__bound".to_string())),
                    }))),
                    method: "max".to_string(),
                    args: vec![int(0)],
                }),
                method: "min".to_string(),
                args: vec![RustExpr::Ident("__len".to_string())],
            }),
            else_expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__bound".to_string())),
                method: "min".to_string(),
                args: vec![RustExpr::Ident("__len".to_string())],
            })),
        })),
    }
}

fn byte_range_guard_expr(
    value: RustExpr,
    valid_expr: RustExpr,
    invalid_expr: RustExpr,
) -> RustExpr {
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__needle".to_string(),
            ty: None,
            value,
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__needle".to_string())),
                    op: "<".to_string(),
                    right: Box::new(int(0)),
                }),
                op: "||".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__needle".to_string())),
                    op: ">".to_string(),
                    right: Box::new(int(255)),
                }),
            }),
            then_expr: Box::new(invalid_expr),
            else_expr: Some(Box::new(valid_expr)),
        })),
    }
}

pub(super) fn lower_count(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(byte_range_guard_expr(
        args[0].clone(),
        RustExpr::Cast {
            expr: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(object.clone()),
                        method: "iter".to_string(),
                        args: vec![],
                    }),
                    method: "filter".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "__x".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Deref(Box::new(RustExpr::Deref(Box::new(
                                RustExpr::Ident("__x".to_string()),
                            ))))),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Cast {
                                expr: Box::new(RustExpr::Ident("__needle".to_string())),
                                ty: RustType::Named("u8".to_string()),
                            }),
                        }),
                        is_move: false,
                    }],
                }),
                method: "count".to_string(),
                args: vec![],
            }),
            ty: RustType::I64,
        },
        int(0),
    ))
}

pub(super) fn lower_contains(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(byte_range_guard_expr(
        args[0].clone(),
        RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "contains".to_string(),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident("__needle".to_string())),
                    ty: RustType::Named("u8".to_string()),
                }),
            }],
        },
        RustExpr::Literal(crate::RustLiteral::Bool(false)),
    ))
}

pub(super) fn lower_find(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.is_empty() || args.len() > 3 {
        return None;
    }
    Some(byte_range_guard_expr(
        args[0].clone(),
        RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__len".to_string(),
                    ty: None,
                    value: RustExpr::Cast {
                        expr: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(object.clone()),
                            method: "len".to_string(),
                            args: vec![],
                        }),
                        ty: RustType::I64,
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__start".to_string(),
                    ty: None,
                    value: list_bound_expr(args.get(1), 0),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__stop".to_string(),
                    ty: None,
                    value: if let Some(stop) = args.get(2) {
                        list_bound_expr(Some(stop), 0)
                    } else {
                        RustExpr::Ident("__len".to_string())
                    },
                },
                RustStmt::Let {
                    mutable: true,
                    name: "__i".to_string(),
                    ty: None,
                    value: RustExpr::Ident("__start".to_string()),
                },
                RustStmt::Let {
                    mutable: true,
                    name: "__result".to_string(),
                    ty: None,
                    value: RustExpr::Path(vec!["None".to_string()]),
                },
                RustStmt::While {
                    cond: RustExpr::BinOp {
                        left: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__i".to_string())),
                            op: "<".to_string(),
                            right: Box::new(RustExpr::Ident("__stop".to_string())),
                        }),
                        op: "&&".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__result".to_string())),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Path(vec!["None".to_string()])),
                        }),
                    },
                    body: vec![
                        RustStmt::IfLet {
                            pattern: "Some(__x)".to_string(),
                            expr: RustExpr::MethodCall {
                                receiver: Box::new(object.clone()),
                                method: "get".to_string(),
                                args: vec![RustExpr::Cast {
                                    expr: Box::new(RustExpr::Ident("__i".to_string())),
                                    ty: RustType::Named("usize".to_string()),
                                }],
                            },
                            then_body: vec![RustStmt::If {
                                cond: RustExpr::BinOp {
                                    left: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                                        "__x".to_string(),
                                    )))),
                                    op: "==".to_string(),
                                    right: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Ident("__needle".to_string())),
                                        ty: RustType::Named("u8".to_string()),
                                    }),
                                },
                                then_body: vec![RustStmt::Assign {
                                    target: RustExpr::Ident("__result".to_string()),
                                    value: RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                                        args: vec![RustExpr::Ident("__i".to_string())],
                                    },
                                }],
                                else_body: None,
                            }],
                            else_body: None,
                        },
                        RustStmt::AugAssign {
                            target: RustExpr::Ident("__i".to_string()),
                            op: "+".to_string(),
                            value: int(1),
                        },
                    ],
                },
            ],
            expr: Some(Box::new(RustExpr::Ident("__result".to_string()))),
        },
        RustExpr::Path(vec!["None".to_string()]),
    ))
}

fn lower_prefix_method(object: &RustExpr, args: &[RustExpr], method: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: method.to_string(),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(args[0].clone()),
        }],
    })
}

pub(super) fn lower_startswith(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_prefix_method(object, args, "starts_with")
}

pub(super) fn lower_endswith(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_prefix_method(object, args, "ends_with")
}

pub(super) fn lower_hex(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "__hex".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "with_capacity".to_string(),
                    ])),
                    args: vec![RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(object.clone()),
                            method: "len".to_string(),
                            args: vec![],
                        }),
                        method: "saturating_mul".to_string(),
                        args: vec![int(2)],
                    }],
                },
            },
            RustStmt::For {
                var: "__byte".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "iter".to_string(),
                    args: vec![],
                },
                body: vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__hex".to_string())),
                    method: "push_str".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::FormatMacro {
                            name: "format".to_string(),
                            format_str: "{:02x}".to_string(),
                            args: vec![RustExpr::Deref(Box::new(RustExpr::Ident(
                                "__byte".to_string(),
                            )))],
                        }),
                    }],
                })],
            },
        ],
        expr: Some(Box::new(RustExpr::Ident("__hex".to_string()))),
    })
}

pub(super) fn lower_to_ints(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "iter".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__byte".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                        "__byte".to_string(),
                    )))),
                    ty: RustType::I64,
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<i64>>".to_string(),
        args: vec![],
    })
}
