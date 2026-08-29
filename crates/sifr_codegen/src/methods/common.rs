//! Shared method lowerers that are not container-specific.

use crate::{RustExpr, RustLiteral, RustStmt, RustType};

fn exact_int_from_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "SifrInt".to_string(),
            "from".to_string(),
        ])),
        args: vec![expr],
    }
}

pub(super) fn exact_int_to_usize_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "to_usize_proven".to_string(),
        ])),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(expr),
        }],
    }
}

fn exact_int_literal(value: i64) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "SifrInt".to_string(),
            "from_i64".to_string(),
        ])),
        args: vec![RustExpr::Literal(RustLiteral::Int(value))],
    }
}

pub(super) fn lower_tuple_len(elem_count: usize, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    let elem_count = i64::try_from(elem_count).ok()?;
    Some(exact_int_literal(elem_count))
}

fn tuple_bound_expr(arg: Option<&RustExpr>, len: usize, default: usize) -> RustExpr {
    let len_expr = RustExpr::Verbatim(format!("{len}usize"));
    let default_expr = RustExpr::Verbatim(format!("{default}usize"));
    let Some(arg) = arg else {
        return default_expr;
    };
    RustExpr::MethodCall {
        receiver: Box::new(arg.clone()),
        method: "clamp_slice_bound".to_string(),
        args: vec![len_expr],
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
        expr: Some(Box::new(exact_int_from_expr(RustExpr::Ident(
            "__count".to_string(),
        )))),
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
    let len = elem_count;
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
        let index_expr = RustExpr::Verbatim(format!("{index}usize"));
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
                    args: vec![exact_int_from_expr(index_expr)],
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
    Some(exact_int_from_expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "chars".to_string(),
            args: vec![],
        }),
        method: "count".to_string(),
        args: vec![],
    }))
}

pub(super) fn lower_option_len(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(exact_int_from_expr(RustExpr::MethodCall {
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
    }))
}

pub(super) fn lower_len(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(exact_int_from_expr(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "len".to_string(),
        args: vec![],
    }))
}
