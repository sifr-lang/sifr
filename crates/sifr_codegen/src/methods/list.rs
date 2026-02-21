//! List method lowerers for registry migration.

use crate::{RustExpr, RustParam, RustType};

pub(super) fn lower_append(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "push".to_string(),
        args: vec![RustExpr::RawCode(args[0].clone())],
    })
}

pub(super) fn lower_extend(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "extend".to_string(),
        args: vec![RustExpr::RawCode(args[0].clone())],
    })
}

pub(super) fn lower_insert(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "insert".to_string(),
        args: vec![
            RustExpr::Cast {
                expr: Box::new(RustExpr::RawCode(args[0].clone())),
                ty: RustType::RawCode("usize".to_string()),
            },
            RustExpr::RawCode(args[1].clone()),
        ],
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

pub(super) fn lower_reverse(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "reverse".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_sort(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "sort".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_count(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.iter().filter(|x| **x == {}).count() as i64",
        args[0]
    )))
}

pub(super) fn lower_contains(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "contains".to_string(),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::RawCode(args[0].clone())),
        }],
    })
}

pub(super) fn lower_pop(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "pop".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_remove(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ if let Some(__pos) = {object}.iter().position(|__x| *__x == {}) {{ {object}.remove(__pos); }} }}",
        args[0]
    )))
}

pub(super) fn lower_index(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "iter".to_string(),
                args: vec![],
            }),
            method: "position".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__x".to_string(),
                    ty: RustType::RawCode("_".to_string()),
                }],
                body: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                        "__x".to_string(),
                    )))),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::RawCode(args[0].clone())),
                }),
                is_move: false,
            }],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__p".to_string(),
                ty: RustType::RawCode("_".to_string()),
            }],
            body: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Ident("__p".to_string())),
                ty: RustType::I64,
            }),
            is_move: false,
        }],
    })
}
