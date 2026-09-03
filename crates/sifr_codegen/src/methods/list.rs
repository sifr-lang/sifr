//! List method lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustStmt, RustType};

use super::common::exact_int_to_usize_expr;

fn is_already_borrowed_rendered_expr(arg: &RustExpr) -> bool {
    match arg {
        RustExpr::Ref { .. } => true,
        RustExpr::MethodCall { method, .. } => method == "as_str",
        RustExpr::Paren(inner)
        | RustExpr::Try(inner)
        | RustExpr::Await(inner)
        | RustExpr::Clone(inner) => is_already_borrowed_rendered_expr(inner),
        _ => false,
    }
}

fn render_borrowed_arg_expr(arg: &RustExpr) -> RustExpr {
    match arg {
        RustExpr::Ref { .. } => arg.clone(),
        _ if is_already_borrowed_rendered_expr(arg) => arg.clone(),
        _ => RustExpr::Ref {
            mutable: false,
            expr: Box::new(arg.clone()),
        },
    }
}

fn list_bound_expr(arg: Option<&RustExpr>, default: usize) -> RustExpr {
    let Some(arg) = arg else {
        return RustExpr::Verbatim(format!("{default}usize"));
    };
    RustExpr::MethodCall {
        receiver: Box::new(arg.clone()),
        method: "clamp_slice_bound".to_string(),
        args: vec![RustExpr::Ident("__len".to_string())],
    }
}

pub(super) fn lower_append(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "push".to_string(),
        args: vec![args[0].clone()],
    })
}

pub(super) fn lower_extend(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "extend".to_string(),
        args: vec![args[0].clone()],
    })
}

pub(super) fn lower_insert(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "insert".to_string(),
        args: vec![exact_int_to_usize_expr(args[0].clone()), args[1].clone()],
    })
}

pub(super) fn lower_clear(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "clear".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_copy(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "clone".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_reverse(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "reverse".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_sort(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    match args {
        [] => Some(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "sort".to_string(),
            args: vec![],
        }),
        [reverse] => Some(RustExpr::If {
            cond: Box::new(reverse.clone()),
            then_expr: Box::new(RustExpr::Block {
                stmts: vec![
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(object.clone()),
                        method: "sort".to_string(),
                        args: vec![],
                    }),
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(object.clone()),
                        method: "reverse".to_string(),
                        args: vec![],
                    }),
                ],
                expr: None,
            }),
            else_expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "sort".to_string(),
                args: vec![],
            })),
        }),
        _ => None,
    }
}

pub(super) fn lower_count(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "SifrInt".to_string(),
            "from".to_string(),
        ])),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "filter".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "x".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Deref(Box::new(RustExpr::Deref(Box::new(
                            RustExpr::Ident("x".to_string()),
                        ))))),
                        op: "==".to_string(),
                        right: Box::new(args[0].clone()),
                    }),
                    is_move: false,
                }],
            }),
            method: "count".to_string(),
            args: vec![],
        }],
    })
}

pub(super) fn lower_contains(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "contains".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_pop(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    match args {
        [] => Some(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "pop".to_string(),
            args: vec![],
        }),
        [index] => Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__len".to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(object.clone()),
                        method: "len".to_string(),
                        args: vec![],
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__index".to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(index.clone()),
                        method: "normalize_index_or_len".to_string(),
                        args: vec![RustExpr::Ident("__len".to_string())],
                    },
                },
            ],
            expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__index".to_string())),
                    op: ">=".to_string(),
                    right: Box::new(RustExpr::Ident("__len".to_string())),
                }),
                then_expr: Box::new(RustExpr::Path(vec!["None".to_string()])),
                else_expr: Some(Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![RustExpr::MethodCall {
                        receiver: Box::new(object.clone()),
                        method: "remove".to_string(),
                        args: vec![RustExpr::Ident("__index".to_string())],
                    }],
                })),
            })),
        }),
        _ => None,
    }
}

pub(super) fn lower_remove(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::IfLet {
            pattern: "Some(__pos)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "position".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__x".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__x".to_string())),
                        method: "eq".to_string(),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(args[0].clone()),
                        }],
                    }),
                    is_move: false,
                }],
            },
            then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "remove".to_string(),
                args: vec![RustExpr::Ident("__pos".to_string())],
            })],
            else_body: None,
        }],
        expr: None,
    })
}

pub(super) fn lower_index(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.is_empty() || args.len() > 3 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__len".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "len".to_string(),
                    args: vec![],
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
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__result".to_string())),
                        method: "is_none".to_string(),
                        args: Vec::new(),
                    }),
                },
                body: vec![
                    RustStmt::IfLet {
                        pattern: "Some(__x)".to_string(),
                        expr: RustExpr::MethodCall {
                            receiver: Box::new(object.clone()),
                            method: "get".to_string(),
                            args: vec![RustExpr::Ident("__i".to_string())],
                        },
                        then_body: vec![RustStmt::If {
                            cond: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                                method: "eq".to_string(),
                                args: vec![RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(args[0].clone()),
                                }],
                            },
                            then_body: vec![RustStmt::Assign {
                                target: RustExpr::Ident("__result".to_string()),
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                                    args: vec![RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "SifrInt".to_string(),
                                            "from".to_string(),
                                        ])),
                                        args: vec![RustExpr::Ident("__i".to_string())],
                                    }],
                                },
                            }],
                            else_body: None,
                        }],
                        else_body: None,
                    },
                    RustStmt::AugAssign {
                        target: RustExpr::Ident("__i".to_string()),
                        op: "+".to_string(),
                        value: RustExpr::Literal(crate::RustLiteral::Int(1)),
                    },
                ],
            },
        ],
        expr: Some(Box::new(RustExpr::Ident("__result".to_string()))),
    })
}
