//! Shared method lowerers that are not container-specific.

use crate::{RustExpr, RustLiteral, RustType};

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

pub(super) fn lower_tuple_count_placeholder(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
        ty: RustType::I64,
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
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "as_ref".to_string(),
                    args: vec![],
                }),
                method: "unwrap".to_string(),
                args: vec![],
            }),
            method: "len".to_string(),
            args: vec![],
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
