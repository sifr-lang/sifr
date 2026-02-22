//! HTML intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral};

pub(super) fn lower_html_escape(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    // s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#x27;")
    let s = RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Ident(args[0].clone())),
    };
    Some(RustExpr::Block {
        stmts: vec![],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(s),
                            method: "replace".to_string(),
                            args: vec![
                                RustExpr::Literal(RustLiteral::Char('&')),
                                RustExpr::Literal(RustLiteral::Str("&amp;".to_string())),
                            ],
                        }),
                        method: "replace".to_string(),
                        args: vec![
                            RustExpr::Literal(RustLiteral::Char('<')),
                            RustExpr::Literal(RustLiteral::Str("&lt;".to_string())),
                        ],
                    }),
                    method: "replace".to_string(),
                    args: vec![
                        RustExpr::Literal(RustLiteral::Char('>')),
                        RustExpr::Literal(RustLiteral::Str("&gt;".to_string())),
                    ],
                }),
                method: "replace".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Char('"')),
                    RustExpr::Literal(RustLiteral::Str("&quot;".to_string())),
                ],
            }),
            method: "replace".to_string(),
            args: vec![
                RustExpr::Literal(RustLiteral::Char('\'')),
                RustExpr::Literal(RustLiteral::Str("&#x27;".to_string())),
            ],
        })),
    })
}

pub(super) fn lower_html_unescape(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    // s.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#x27;", "'").replace("&#39;", "'")
    let s = RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Ident(args[0].clone())),
    };
    Some(RustExpr::Block {
        stmts: vec![],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(s),
                                method: "replace".to_string(),
                                args: vec![
                                    RustExpr::Literal(RustLiteral::Str("&amp;".to_string())),
                                    RustExpr::Literal(RustLiteral::Str("&".to_string())),
                                ],
                            }),
                            method: "replace".to_string(),
                            args: vec![
                                RustExpr::Literal(RustLiteral::Str("&lt;".to_string())),
                                RustExpr::Literal(RustLiteral::Str("<".to_string())),
                            ],
                        }),
                        method: "replace".to_string(),
                        args: vec![
                            RustExpr::Literal(RustLiteral::Str("&gt;".to_string())),
                            RustExpr::Literal(RustLiteral::Str(">".to_string())),
                        ],
                    }),
                    method: "replace".to_string(),
                    args: vec![
                        RustExpr::Literal(RustLiteral::Str("&quot;".to_string())),
                        RustExpr::Literal(RustLiteral::Str("\"".to_string())),
                    ],
                }),
                method: "replace".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Str("&#x27;".to_string())),
                    RustExpr::Literal(RustLiteral::Str("'".to_string())),
                ],
            }),
            method: "replace".to_string(),
            args: vec![
                RustExpr::Literal(RustLiteral::Str("&#39;".to_string())),
                RustExpr::Literal(RustLiteral::Str("'".to_string())),
            ],
        })),
    })
}
