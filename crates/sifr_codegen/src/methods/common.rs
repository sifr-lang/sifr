//! Shared method lowerers that are not container-specific.

use crate::{RustExpr, RustType};

pub(super) fn lower_tuple_len(elem_count: usize, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{elem_count}_i64")))
}

pub(super) fn lower_tuple_count_placeholder(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(
        "0_i64 /* tuple.count() not fully supported */".to_string(),
    ))
}

pub(super) fn lower_string_char_len(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "count".to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}

pub(super) fn lower_option_len(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.to_string())),
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

pub(super) fn lower_len(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(object.to_string())),
            method: "len".to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}
