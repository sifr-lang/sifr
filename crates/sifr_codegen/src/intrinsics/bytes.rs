//! Bytes intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn int(v: i64) -> RustExpr {
    RustExpr::Literal(RustLiteral::Int(v))
}

fn string_lit(v: &str) -> RustExpr {
    RustExpr::Literal(RustLiteral::Str(v.to_string()))
}

fn parse_error_expr(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "ParseError".to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn err_parse_expr(message: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![parse_error_expr(message)],
    }
}

fn parse_map_err(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "e".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(parse_error_expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("e".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            })),
            is_move: false,
        }],
    }
}

pub(super) fn lower_encode_utf8(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__s".to_string(),
            ty: None,
            value: arg_expr(args, 0),
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__s".to_string())),
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
                        expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident("b".to_string())))),
                        ty: RustType::I64,
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Vec<i64>>".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_decode_utf8(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(args[0].clone()),
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

pub(super) fn lower_bytes_to_hex(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(args[0].clone()),
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
                args: vec![RustExpr::Ident("\"\"".to_string())],
            }),
            is_move: false,
        }],
    })
}

pub(super) fn lower_bytes_from_hex(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "s".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: true,
                name: "cleaned".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "ch".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("s".to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                },
                body: vec![
                    RustStmt::If {
                        cond: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("ch".to_string())),
                            method: "is_ascii_whitespace".to_string(),
                            args: vec![],
                        },
                        then_body: vec![RustStmt::Continue],
                        else_body: None,
                    },
                    RustStmt::If {
                        cond: RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("ch".to_string())),
                                method: "is_ascii_hexdigit".to_string(),
                                args: vec![],
                            }),
                        },
                        then_body: vec![RustStmt::Return(Some(err_parse_expr(
                            RustExpr::FormatMacro {
                                name: "format".to_string(),
                                format_str: "invalid hex character: {}".to_string(),
                                args: vec![RustExpr::Ident("ch".to_string())],
                            },
                        )))],
                        else_body: None,
                    },
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("cleaned".to_string())),
                        method: "push".to_string(),
                        args: vec![RustExpr::Ident("ch".to_string())],
                    }),
                ],
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("cleaned".to_string())),
                            method: "len".to_string(),
                            args: vec![],
                        }),
                        op: "%".to_string(),
                        right: Box::new(int(2)),
                    }),
                    op: "!=".to_string(),
                    right: Box::new(int(0)),
                },
                then_body: vec![RustStmt::Return(Some(err_parse_expr(
                    RustExpr::MethodCall {
                        receiver: Box::new(string_lit(
                            "fromhex() arg must contain an even number of hexadecimal digits",
                        )),
                        method: "to_string".to_string(),
                        args: vec![],
                    },
                )))],
                else_body: None,
            },
            RustStmt::Let {
                mutable: true,
                name: "result".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "pair".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("cleaned".to_string())),
                        method: "as_bytes".to_string(),
                        args: vec![],
                    }),
                    method: "chunks".to_string(),
                    args: vec![int(2)],
                },
                body: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "pair_str".to_string(),
                        ty: None,
                        value: RustExpr::Try(Box::new(parse_map_err(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "str".to_string(),
                                "from_utf8".to_string(),
                            ])),
                            args: vec![RustExpr::Ident("pair".to_string())],
                        }))),
                    },
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("result".to_string())),
                        method: "push".to_string(),
                        args: vec![RustExpr::Try(Box::new(parse_map_err(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "i64".to_string(),
                                "from_str_radix".to_string(),
                            ])),
                            args: vec![RustExpr::Ident("pair_str".to_string()), int(16)],
                        })))],
                    }),
                ],
            },
        ],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![RustExpr::Ident("result".to_string())],
        })),
    })
}
