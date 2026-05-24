//! Environment intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

pub(crate) fn lower_env_get(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__k".to_string(),
            ty: None,
            value: args[0].clone(),
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__k".to_string())),
                    method: "is_empty".to_string(),
                    args: vec![],
                }),
                op: "||".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__k".to_string())),
                        method: "contains".to_string(),
                        args: vec![RustExpr::Literal(RustLiteral::Char('='))],
                    }),
                    op: "||".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__k".to_string())),
                            method: "as_bytes".to_string(),
                            args: vec![],
                        }),
                        method: "contains".to_string(),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                        }],
                    }),
                }),
            }),
            then_expr: Box::new(RustExpr::Literal(RustLiteral::None)),
            else_expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "env".to_string(),
                        "var".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__k".to_string())],
                }),
                method: "ok".to_string(),
                args: vec![],
            })),
        })),
    })
}

pub(crate) fn lower_env_set(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__k".to_string(),
                ty: None,
                value: args[0].clone(),
            },
            RustStmt::Let {
                mutable: false,
                name: "__v".to_string(),
                ty: None,
                value: args[1].clone(),
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__k".to_string())),
                            method: "is_empty".to_string(),
                            args: vec![],
                        }),
                    }),
                    op: "&&".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__k".to_string())),
                                method: "contains".to_string(),
                                args: vec![RustExpr::Literal(RustLiteral::Char('='))],
                            }),
                        }),
                        op: "&&".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::UnaryOp {
                                op: "!".to_string(),
                                operand: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__k".to_string())),
                                        method: "as_bytes".to_string(),
                                        args: vec![],
                                    }),
                                    method: "contains".to_string(),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                                    }],
                                }),
                            }),
                            op: "&&".to_string(),
                            right: Box::new(RustExpr::UnaryOp {
                                op: "!".to_string(),
                                operand: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__v".to_string())),
                                        method: "as_bytes".to_string(),
                                        args: vec![],
                                    }),
                                    method: "contains".to_string(),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                                    }],
                                }),
                            }),
                        }),
                    }),
                },
                then_body: vec![RustStmt::Expr(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "env".to_string(),
                        "set_var".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("__k".to_string()),
                        RustExpr::Ident("__v".to_string()),
                    ],
                })],
                else_body: None,
            },
        ],
        expr: None,
    })
}

pub(crate) fn lower_env_unset(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__k".to_string(),
                ty: None,
                value: args[0].clone(),
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__k".to_string())),
                            method: "is_empty".to_string(),
                            args: vec![],
                        }),
                    }),
                    op: "&&".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__k".to_string())),
                                method: "contains".to_string(),
                                args: vec![RustExpr::Literal(RustLiteral::Char('='))],
                            }),
                        }),
                        op: "&&".to_string(),
                        right: Box::new(RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__k".to_string())),
                                    method: "as_bytes".to_string(),
                                    args: vec![],
                                }),
                                method: "contains".to_string(),
                                args: vec![RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                                }],
                            }),
                        }),
                    }),
                },
                then_body: vec![RustStmt::Expr(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "env".to_string(),
                        "remove_var".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__k".to_string())],
                })],
                else_body: None,
            },
        ],
        expr: None,
    })
}

pub(crate) fn lower_env_keys(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "env".to_string(),
                    "vars_os".to_string(),
                ])),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__kv".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__kv".to_string())),
                            field: "0".to_string(),
                        }),
                        method: "to_string_lossy".to_string(),
                        args: vec![],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<String>>".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_env_values(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "env".to_string(),
                    "vars_os".to_string(),
                ])),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__kv".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__kv".to_string())),
                            field: "1".to_string(),
                        }),
                        method: "to_string_lossy".to_string(),
                        args: vec![],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<String>>".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_env_items(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "env".to_string(),
                    "vars_os".to_string(),
                ])),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__kv".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{}={}".to_string(),
                    args: vec![
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__kv".to_string())),
                                field: "0".to_string(),
                            }),
                            method: "to_string_lossy".to_string(),
                            args: vec![],
                        },
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__kv".to_string())),
                                field: "1".to_string(),
                            }),
                            method: "to_string_lossy".to_string(),
                            args: vec![],
                        },
                    ],
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<String>>".to_string(),
        args: vec![],
    })
}
