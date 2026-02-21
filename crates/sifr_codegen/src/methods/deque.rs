//! Deque-specific list method lowerers.

use crate::RustExpr;

pub(super) fn lower_append(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "push_back".to_string(),
        args: vec![RustExpr::RawCode(args[0].clone())],
    })
}

pub(super) fn lower_appendleft(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "push_front".to_string(),
        args: vec![RustExpr::RawCode(args[0].clone())],
    })
}

pub(super) fn lower_pop(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "pop_back".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_popleft(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "pop_front".to_string(),
        args: vec![],
    })
}
