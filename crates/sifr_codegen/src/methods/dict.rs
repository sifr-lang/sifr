//! Dict method lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustType};

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

fn render_key_arg_expr(arg: &RustExpr) -> RustExpr {
    match arg {
        RustExpr::Ref { .. } => arg.clone(),
        _ if is_already_borrowed_rendered_expr(arg) => arg.clone(),
        _ => RustExpr::Ref {
            mutable: false,
            expr: Box::new(arg.clone()),
        },
    }
}

pub(super) fn lower_keys(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "keys".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        method: "collect::<Vec<_>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_values(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "values".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        method: "collect::<Vec<_>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_items(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
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
                    name: "__kv".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::Tuple(vec![
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__kv".to_string())),
                            field: "0".to_string(),
                        }),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__kv".to_string())),
                            field: "1".to_string(),
                        }),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                ])),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<_>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_update(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    match args {
        [] => Some(RustExpr::Literal(crate::RustLiteral::Unit)),
        [other] => Some(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "extend".to_string(),
            args: vec![other.clone()],
        }),
        [other, keyword_dict] => Some(RustExpr::Block {
            stmts: vec![
                crate::RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "extend".to_string(),
                    args: vec![other.clone()],
                }),
                crate::RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "extend".to_string(),
                    args: vec![keyword_dict.clone()],
                }),
            ],
            expr: Some(Box::new(RustExpr::Literal(crate::RustLiteral::Unit))),
        }),
        _ => None,
    }
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

pub(super) fn lower_contains(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "contains_key".to_string(),
        args: vec![render_key_arg_expr(&args[0])],
    })
}

pub(super) fn lower_get(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    match args.len() {
        1 => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "get".to_string(),
                args: vec![render_key_arg_expr(&args[0])],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        2 => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "get".to_string(),
                    args: vec![render_key_arg_expr(&args[0])],
                }),
                method: "cloned".to_string(),
                args: vec![],
            }),
            method: "unwrap_or".to_string(),
            args: vec![args[1].clone()],
        }),
        _ => None,
    }
}

pub(super) fn lower_pop(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    match args {
        [key] => Some(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "remove".to_string(),
            args: vec![render_key_arg_expr(key)],
        }),
        [key, default] => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "remove".to_string(),
                args: vec![render_key_arg_expr(key)],
            }),
            method: "unwrap_or".to_string(),
            args: vec![default.clone()],
        }),
        _ => None,
    }
}

pub(super) fn lower_setdefault(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    match args {
        [key, default] => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "entry".to_string(),
                    args: vec![key.clone()],
                }),
                method: "or_insert".to_string(),
                args: vec![default.clone()],
            }),
            method: "clone".to_string(),
            args: vec![],
        }),
        _ => None,
    }
}
