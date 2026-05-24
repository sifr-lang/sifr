//! Shared digest formatting helpers for hash intrinsics.

use crate::{RustExpr, RustLiteral, RustParam, RustType};

fn empty_str_slice_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Literal(RustLiteral::Str(String::new()))),
        method: "as_str".to_string(),
        args: vec![],
    }
}

pub(crate) fn bytes_to_hex_expr(bytes: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(bytes),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__byte".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: "{:02x}".to_string(),
                        args: vec![RustExpr::Deref(Box::new(RustExpr::Ident(
                            "__byte".to_string(),
                        )))],
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Vec<String>>".to_string(),
            args: vec![],
        }),
        method: "join".to_string(),
        args: vec![empty_str_slice_expr()],
    }
}
