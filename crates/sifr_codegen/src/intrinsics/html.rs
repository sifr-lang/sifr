//! HTML intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral};

fn arg_expr(args: &[String], idx: usize) -> RustExpr {
    RustExpr::Ident(args[idx].clone())
}

fn str_lit(value: &str) -> RustExpr {
    RustExpr::Ident(format!("{value:?}"))
}

fn char_lit(value: char) -> RustExpr {
    RustExpr::Literal(RustLiteral::Char(value))
}

pub(super) fn lower_html_escape(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(arg_expr(args, 0)),
                        method: "replace".to_string(),
                        args: vec![char_lit('&'), str_lit("&amp;")],
                    }),
                    method: "replace".to_string(),
                    args: vec![char_lit('<'), str_lit("&lt;")],
                }),
                method: "replace".to_string(),
                args: vec![char_lit('>'), str_lit("&gt;")],
            }),
            method: "replace".to_string(),
            args: vec![char_lit('"'), str_lit("&quot;")],
        }),
        method: "replace".to_string(),
        args: vec![char_lit('\''), str_lit("&#x27;")],
    })
}

pub(super) fn lower_html_unescape(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(arg_expr(args, 0)),
                            method: "replace".to_string(),
                            args: vec![str_lit("&amp;"), str_lit("&")],
                        }),
                        method: "replace".to_string(),
                        args: vec![str_lit("&lt;"), str_lit("<")],
                    }),
                    method: "replace".to_string(),
                    args: vec![str_lit("&gt;"), str_lit(">")],
                }),
                method: "replace".to_string(),
                args: vec![str_lit("&quot;"), str_lit("\"")],
            }),
            method: "replace".to_string(),
            args: vec![str_lit("&#x27;"), str_lit("'")],
        }),
        method: "replace".to_string(),
        args: vec![str_lit("&#39;"), str_lit("'")],
    })
}
