//! Sys intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustType};

fn parenthesized(expr: &RustExpr) -> RustExpr {
    RustExpr::Paren(Box::new(expr.clone()))
}

pub(crate) fn lower_sys_exit(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "process".to_string(),
            "exit".to_string(),
        ])),
        args: vec![RustExpr::Cast {
            expr: Box::new(parenthesized(&args[0])),
            ty: RustType::Named("i32".to_string()),
        }],
    })
}

pub(crate) fn lower_sys_version(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Literal(RustLiteral::Str(
        "sifr 0.1.0".to_string(),
    )))
}

pub(crate) fn lower_sys_platform(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "env".to_string(),
            "consts".to_string(),
            "OS".to_string(),
        ])),
        method: "to_string".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_sys_maxsize(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Path(vec!["i64".to_string(), "MAX".to_string()]))
}
