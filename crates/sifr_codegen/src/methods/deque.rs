//! Deque-specific list method lowerers.

use crate::RustExpr;

pub(super) fn lower_append(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "push_back".to_string(),
        args: vec![args[0].clone()],
    })
}

pub(super) fn lower_appendleft(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "push_front".to_string(),
        args: vec![args[0].clone()],
    })
}

pub(super) fn lower_pop(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "pop_back".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_popleft(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "pop_front".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_reverse(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "make_contiguous".to_string(),
            args: Vec::new(),
        }),
        method: "reverse".to_string(),
        args: Vec::new(),
    })
}
