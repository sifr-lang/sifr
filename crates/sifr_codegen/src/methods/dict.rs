//! Dict method lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustType};

fn is_already_borrowed_rendered_expr(arg: &RustExpr) -> bool {
    let rendered = crate::render_expr(arg);
    rendered.ends_with(".as_str()") || rendered.starts_with('&')
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
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "extend".to_string(),
        args: vec![args[0].clone()],
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
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "remove".to_string(),
        args: vec![render_key_arg_expr(&args[0])],
    })
}
