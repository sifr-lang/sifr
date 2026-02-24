//! List method lowerers for registry migration.

use crate::{RustExpr, RustParam, RustStmt, RustType};

fn render_borrowed_arg_expr(arg: &RustExpr) -> RustExpr {
    match arg {
        RustExpr::Ref { .. } => arg.clone(),
        RustExpr::RawCode(code) if code.ends_with(".as_str()") || code.starts_with('&') => {
            RustExpr::RawCode(code.clone())
        }
        _ => RustExpr::Ref {
            mutable: false,
            expr: Box::new(arg.clone()),
        },
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
        args: vec![
            RustExpr::Cast {
                expr: Box::new(args[0].clone()),
                ty: RustType::Named("usize".to_string()),
            },
            args[1].clone(),
        ],
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
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "sort".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_count(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Cast {
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
        }),
        ty: RustType::I64,
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
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "pop".to_string(),
        args: vec![],
    })
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
                    body: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                            "__x".to_string(),
                        )))),
                        op: "==".to_string(),
                        right: Box::new(args[0].clone()),
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
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
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
                body: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                        "__x".to_string(),
                    )))),
                    op: "==".to_string(),
                    right: Box::new(args[0].clone()),
                }),
                is_move: false,
            }],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__p".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Ident("__p".to_string())),
                ty: RustType::I64,
            }),
            is_move: false,
        }],
    })
}
