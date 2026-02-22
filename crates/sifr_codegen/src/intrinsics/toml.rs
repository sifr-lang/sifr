//! TOML intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[String], idx: usize) -> RustExpr {
    RustExpr::Ident(args[idx].clone())
}

fn ref_expr(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

pub(super) fn lower_toml_parse(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__toml_str".to_string(),
            ty: None,
            value: ref_expr(arg_expr(args, 0)),
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__toml_str".to_string())),
                    method: "parse::<toml::Value>".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "v".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: "{}".to_string(),
                        args: vec![RustExpr::Ident("v".to_string())],
                    }),
                    is_move: false,
                }],
            }),
            method: "map_err".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "e".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::StructInit {
                    name: "TOMLDecodeError".to_string(),
                    fields: vec![
                        (
                            "message".to_string(),
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                        ),
                        ("line".to_string(), RustExpr::Literal(RustLiteral::Int(0))),
                        ("column".to_string(), RustExpr::Literal(RustLiteral::Int(0))),
                    ],
                }),
                is_move: false,
            }],
        })),
    })
}
