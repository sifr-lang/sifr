//! Bytes intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustParam, RustType};

pub(super) fn lower_encode_utf8(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(format!("({})", args[0]))),
                    method: "as_bytes".to_string(),
                    args: vec![],
                }),
                method: "iter".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "b".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                        "b".to_string(),
                    )))),
                    ty: RustType::I64,
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<i64>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_decode_utf8(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(args[0].clone())),
                        method: "iter".to_string(),
                        args: vec![],
                    }),
                    method: "enumerate".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__pair".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Deref(Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                    field: "1".to_string(),
                                }))),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                            }),
                            op: "||".to_string(),
                            right: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Deref(Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                    field: "1".to_string(),
                                }))),
                                op: ">".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Int(255))),
                            }),
                        }),
                        then_expr: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "ParseError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    RustExpr::FormatMacro {
                                        name: "format".to_string(),
                                        format_str: "byte out of range at index {}: {}".to_string(),
                                        args: vec![
                                            RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident(
                                                    "__pair".to_string(),
                                                )),
                                                field: "0".to_string(),
                                            },
                                            RustExpr::Deref(Box::new(RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident(
                                                    "__pair".to_string(),
                                                )),
                                                field: "1".to_string(),
                                            })),
                                        ],
                                    },
                                )],
                            }],
                        }),
                        else_expr: Some(Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![RustExpr::Cast {
                                expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                    field: "1".to_string(),
                                }))),
                                ty: RustType::Named("u8".to_string()),
                            }],
                        })),
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Result<Vec<u8>, ParseError>>".to_string(),
            args: vec![],
        }),
        method: "and_then".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__bytes".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "from_utf8".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__bytes".to_string())],
                }),
                method: "map_err".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "e".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::StructInit {
                        name: "ParseError".to_string(),
                        fields: vec![(
                            "message".to_string(),
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                        )],
                    }),
                    is_move: false,
                }],
            }),
            is_move: false,
        }],
    })
}

pub(super) fn lower_bytes_to_hex(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(args[0].clone())),
                        method: "iter".to_string(),
                        args: vec![],
                    }),
                    method: "enumerate".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__pair".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Deref(Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                    field: "1".to_string(),
                                }))),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                            }),
                            op: "||".to_string(),
                            right: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Deref(Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                    field: "1".to_string(),
                                }))),
                                op: ">".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Int(255))),
                            }),
                        }),
                        then_expr: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "ParseError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    RustExpr::FormatMacro {
                                        name: "format".to_string(),
                                        format_str: "byte out of range at index {}: {}".to_string(),
                                        args: vec![
                                            RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident(
                                                    "__pair".to_string(),
                                                )),
                                                field: "0".to_string(),
                                            },
                                            RustExpr::Deref(Box::new(RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident(
                                                    "__pair".to_string(),
                                                )),
                                                field: "1".to_string(),
                                            })),
                                        ],
                                    },
                                )],
                            }],
                        }),
                        else_expr: Some(Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![RustExpr::FormatMacro {
                                name: "format".to_string(),
                                format_str: "{:02x}".to_string(),
                                args: vec![RustExpr::Cast {
                                    expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                        field: "1".to_string(),
                                    }))),
                                    ty: RustType::Named("u8".to_string()),
                                }],
                            }],
                        })),
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Result<Vec<String>, ParseError>>".to_string(),
            args: vec![],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__parts".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__parts".to_string())),
                method: "join".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Str(String::new()))],
            }),
            is_move: false,
        }],
    })
}

pub(super) fn lower_bytes_from_hex(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<i64>, ParseError> {{ let s = {}; let mut cleaned = String::new(); for ch in s.chars() {{ if ch.is_ascii_whitespace() {{ continue; }} if !ch.is_ascii_hexdigit() {{ return Err(ParseError {{ message: format!(\"invalid hex character: {{}}\", ch) }}); }} cleaned.push(ch); }} if cleaned.len() % 2 != 0 {{ return Err(ParseError {{ message: \"fromhex() arg must contain an even number of hexadecimal digits\".to_string() }}); }} let mut result = Vec::new(); for pair in cleaned.as_bytes().chunks(2) {{ let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError {{ message: e.to_string() }})?; result.push(i64::from_str_radix(pair_str, 16).map_err(|e| ParseError {{ message: e.to_string() }})?); }} Ok(result) }})()",
        args[0]
    )))
}
