//! Dict method lowerers for registry migration.

use crate::{RustExpr, RustParam, RustType};

fn render_key_arg_expr(arg: &str) -> RustExpr {
    if arg.ends_with(".as_str()") || arg.starts_with('&') {
        RustExpr::Ident(arg.to_string())
    } else {
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Ident(format!("({arg})"))),
        }
    }
}

pub(super) fn lower_keys(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
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

pub(super) fn lower_values(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
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

pub(super) fn lower_items(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
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

pub(super) fn lower_update(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "extend".to_string(),
        args: vec![RustExpr::Ident(args[0].clone())],
    })
}

pub(super) fn lower_clear(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "clear".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_copy(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "clone".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_contains(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "contains_key".to_string(),
        args: vec![render_key_arg_expr(&args[0])],
    })
}

pub(super) fn lower_get(object: &str, args: &[String]) -> Option<RustExpr> {
    match args.len() {
        1 => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "get".to_string(),
                args: vec![render_key_arg_expr(&args[0])],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        2 => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.to_string())),
                    method: "get".to_string(),
                    args: vec![render_key_arg_expr(&args[0])],
                }),
                method: "cloned".to_string(),
                args: vec![],
            }),
            method: "unwrap_or".to_string(),
            args: vec![RustExpr::Ident(args[1].clone())],
        }),
        _ => None,
    }
}

pub(super) fn lower_pop(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "remove".to_string(),
        args: vec![render_key_arg_expr(&args[0])],
    })
}
