//! Shared method lowerers that are not container-specific.

use crate::{RustExpr, RustLiteral, RustStmt, RustType};

pub(super) fn lower_tuple_len(elem_count: usize, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    let elem_count = i64::try_from(elem_count).ok()?;
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::Literal(RustLiteral::Int(elem_count))),
        ty: RustType::I64,
    })
}

fn tuple_bound_expr(arg: Option<&RustExpr>, len: i64, default: i64) -> RustExpr {
    let len_expr = RustExpr::Literal(RustLiteral::Int(len));
    let default_expr = RustExpr::Literal(RustLiteral::Int(default));
    let Some(arg) = arg else {
        return default_expr;
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
                right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
            }),
            then_expr: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(RustExpr::BinOp {
                        left: Box::new(len_expr.clone()),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::Ident("__bound".to_string())),
                    }))),
                    method: "max".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Int(0))],
                }),
                method: "min".to_string(),
                args: vec![len_expr],
            }),
            else_expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__bound".to_string())),
                method: "min".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Int(len))],
            })),
        })),
    }
}

pub(super) fn lower_tuple_count(
    elem_count: usize,
    object: &RustExpr,
    args: &[RustExpr],
) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let mut stmts = vec![RustStmt::Let {
        mutable: true,
        name: "__count".to_string(),
        ty: None,
        value: RustExpr::Literal(RustLiteral::Int(0)),
    }];
    for index in 0..elem_count {
        stmts.push(RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Field {
                        expr: Box::new(object.clone()),
                        field: index.to_string(),
                    }),
                }),
                op: "==".to_string(),
                right: Box::new(RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(args[0].clone()),
                }),
            },
            then_body: vec![RustStmt::AugAssign {
                target: RustExpr::Ident("__count".to_string()),
                op: "+".to_string(),
                value: RustExpr::Literal(RustLiteral::Int(1)),
            }],
            else_body: None,
        });
    }
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Cast {
            expr: Box::new(RustExpr::Ident("__count".to_string())),
            ty: RustType::I64,
        })),
    })
}

pub(super) fn lower_tuple_index(
    elem_count: usize,
    object: &RustExpr,
    args: &[RustExpr],
) -> Option<RustExpr> {
    if args.is_empty() || args.len() > 3 {
        return None;
    }
    let len = i64::try_from(elem_count).ok()?;
    let mut stmts = vec![
        RustStmt::Let {
            mutable: false,
            name: "__start".to_string(),
            ty: None,
            value: tuple_bound_expr(args.get(1), len, 0),
        },
        RustStmt::Let {
            mutable: false,
            name: "__stop".to_string(),
            ty: None,
            value: tuple_bound_expr(args.get(2), len, len),
        },
        RustStmt::Let {
            mutable: true,
            name: "__result".to_string(),
            ty: None,
            value: RustExpr::Path(vec!["None".to_string()]),
        },
    ];
    for index in 0..elem_count {
        let index_expr = RustExpr::Literal(RustLiteral::Int(i64::try_from(index).ok()?));
        stmts.push(RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__result".to_string())),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Path(vec!["None".to_string()])),
                    }),
                    op: "&&".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::BinOp {
                            left: Box::new(index_expr.clone()),
                            op: ">=".to_string(),
                            right: Box::new(RustExpr::Ident("__start".to_string())),
                        }),
                        op: "&&".to_string(),
                        right: Box::new(RustExpr::BinOp {
                            left: Box::new(index_expr.clone()),
                            op: "<".to_string(),
                            right: Box::new(RustExpr::Ident("__stop".to_string())),
                        }),
                    }),
                }),
                op: "&&".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Field {
                            expr: Box::new(object.clone()),
                            field: index.to_string(),
                        }),
                    }),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(args[0].clone()),
                    }),
                }),
            },
            then_body: vec![RustStmt::Assign {
                target: RustExpr::Ident("__result".to_string()),
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![index_expr],
                },
            }],
            else_body: None,
        });
    }
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident("__result".to_string()))),
    })
}

pub(super) fn lower_string_char_len(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "count".to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}

pub(super) fn lower_option_len(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "as_ref".to_string(),
                args: vec![],
            }),
            method: "map_or".to_string(),
            args: vec![
                RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    ty: RustType::Named("usize".to_string()),
                },
                RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "v".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("v".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                },
            ],
        }),
        ty: RustType::I64,
    })
}

pub(super) fn lower_len(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "len".to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}
