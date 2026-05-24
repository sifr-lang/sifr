//! HTML intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn str_lit(value: &str) -> RustExpr {
    RustExpr::Ident(format!("{value:?}"))
}

fn char_lit(value: char) -> RustExpr {
    RustExpr::Literal(RustLiteral::Char(value))
}

pub(crate) fn lower_html_escape(args: &[RustExpr]) -> Option<RustExpr> {
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

pub(crate) fn lower_html_unescape(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let mut expr = arg_expr(args, 0);
    let replacements = [
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#x27;", "'"),
        ("&#X27;", "'"),
        ("&#39;", "'"),
        ("&#60;", "<"),
        ("&#x3C;", "<"),
        ("&#x3c;", "<"),
        ("&#X3C;", "<"),
        ("&#X3c;", "<"),
        ("&#62;", ">"),
        ("&#x3E;", ">"),
        ("&#x3e;", ">"),
        ("&#X3E;", ">"),
        ("&#X3e;", ">"),
    ];

    for (from, to) in replacements {
        expr = RustExpr::MethodCall {
            receiver: Box::new(expr),
            method: "replace".to_string(),
            args: vec![str_lit(from), str_lit(to)],
        };
    }

    Some(expr)
}
