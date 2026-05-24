//! Hash intrinsic lowerers for registry lowering.

use super::digest_format::bytes_to_hex_expr;
use crate::RustExpr;

fn parenthesized(expr: &RustExpr) -> RustExpr {
    RustExpr::Paren(Box::new(expr.clone()))
}

pub(crate) fn lower_sha256(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(bytes_to_hex_expr(RustExpr::FnCall {
        func: Box::new(RustExpr::Ident(
            "<sha2::Sha256 as sha2::Digest>::digest".to_string(),
        )),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(parenthesized(&args[0])),
            method: "as_bytes".to_string(),
            args: vec![],
        }],
    }))
}

pub(crate) fn lower_md5(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(bytes_to_hex_expr(RustExpr::Field {
        expr: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "md5".to_string(),
                "compute".to_string(),
            ])),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(parenthesized(&args[0])),
                method: "as_bytes".to_string(),
                args: vec![],
            }],
        }),
        field: "0".to_string(),
    }))
}
