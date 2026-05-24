//! Base64 intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn ref_expr(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

fn ref_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    ref_expr(arg_expr(args, idx))
}

fn string_lit(s: &str) -> RustExpr {
    RustExpr::Literal(RustLiteral::Str(s.to_string()))
}

fn int_lit(v: i64) -> RustExpr {
    RustExpr::Literal(RustLiteral::Int(v))
}

fn bool_lit(v: bool) -> RustExpr {
    RustExpr::Literal(RustLiteral::Bool(v))
}

fn char_lit(v: char) -> RustExpr {
    RustExpr::Literal(RustLiteral::Char(v))
}

fn parse_error(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "ParseError".to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn err_parse(message: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![parse_error(message)],
    }
}

fn to_string_expr(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn as_bytes(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "as_bytes".to_string(),
        args: vec![],
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
            body: Box::new(parse_error(to_string_expr(RustExpr::Ident(
                "e".to_string(),
            )))),
            is_move: false,
        }],
    }
}

fn ok_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![expr],
    }
}

fn base64_engine_standard() -> RustExpr {
    RustExpr::Path(vec![
        "base64".to_string(),
        "engine".to_string(),
        "general_purpose".to_string(),
        "STANDARD".to_string(),
    ])
}

fn base64_engine_url_safe() -> RustExpr {
    RustExpr::Path(vec![
        "base64".to_string(),
        "engine".to_string(),
        "general_purpose".to_string(),
        "URL_SAFE".to_string(),
    ])
}

fn engine_encode(engine: RustExpr, bytes: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "base64".to_string(),
            "Engine".to_string(),
            "encode".to_string(),
        ])),
        args: vec![ref_expr(engine), bytes],
    }
}

fn engine_decode(engine: RustExpr, bytes: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "base64".to_string(),
            "Engine".to_string(),
            "decode".to_string(),
        ])),
        args: vec![ref_expr(engine), bytes],
    }
}

fn string_from_utf8(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "String".to_string(),
            "from_utf8".to_string(),
        ])),
        args: vec![expr],
    }
}

fn char_between(var_name: &str, start: char, end: char) -> RustExpr {
    RustExpr::BinOp {
        left: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident(var_name.to_string())),
            op: ">=".to_string(),
            right: Box::new(char_lit(start)),
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident(var_name.to_string())),
            op: "<=".to_string(),
            right: Box::new(char_lit(end)),
        }),
    }
}

fn is_base64_char(var_name: &str) -> RustExpr {
    RustExpr::BinOp {
        left: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::BinOp {
                left: Box::new(char_between(var_name, 'A', 'Z')),
                op: "||".to_string(),
                right: Box::new(char_between(var_name, 'a', 'z')),
            }),
            op: "||".to_string(),
            right: Box::new(char_between(var_name, '0', '9')),
        }),
        op: "||".to_string(),
        right: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident(var_name.to_string())),
                    op: "==".to_string(),
                    right: Box::new(char_lit('+')),
                }),
                op: "||".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident(var_name.to_string())),
                    op: "==".to_string(),
                    right: Box::new(char_lit('/')),
                }),
            }),
            op: "||".to_string(),
            right: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident(var_name.to_string())),
                op: "==".to_string(),
                right: Box::new(char_lit('=')),
            }),
        }),
    }
}

pub(crate) fn lower_base64_encode(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(engine_encode(
        base64_engine_standard(),
        as_bytes(ref_arg(args, 0)),
    ))
}

pub(crate) fn lower_base64_encode_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(engine_encode(base64_engine_standard(), ref_arg(args, 0))),
        method: "into_bytes".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_base64_decode(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__bytes".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(parse_map_err(engine_decode(
                base64_engine_standard(),
                as_bytes(ref_arg(args, 0)),
            )))),
        }],
        expr: Some(Box::new(parse_map_err(string_from_utf8(RustExpr::Ident(
            "__bytes".to_string(),
        ))))),
    })
}

pub(crate) fn lower_base64_decode_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(parse_map_err(engine_decode(
        base64_engine_standard(),
        ref_arg(args, 0),
    )))
}

pub(crate) fn lower_base64_encode_opts(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__s".to_string(),
                ty: None,
                value: ref_arg(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__alt".to_string(),
                ty: None,
                value: ref_arg(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__wrap".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__wrap".to_string())),
                    op: "<".to_string(),
                    right: Box::new(int_lit(0)),
                },
                then_body: vec![RustStmt::Return(Some(err_parse(string_lit(
                    "wrapcol must be >= 0",
                ))))],
                else_body: None,
            },
            RustStmt::Let {
                mutable: true,
                name: "__encoded".to_string(),
                ty: None,
                value: engine_encode(
                    base64_engine_standard(),
                    as_bytes(RustExpr::Ident("__s".to_string())),
                ),
            },
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__alt".to_string())),
                        method: "is_empty".to_string(),
                        args: vec![],
                    }),
                },
                then_body: vec![
                    RustStmt::If {
                        cond: RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__alt".to_string())),
                                    method: "chars".to_string(),
                                    args: vec![],
                                }),
                                method: "count".to_string(),
                                args: vec![],
                            }),
                            op: "!=".to_string(),
                            right: Box::new(int_lit(2)),
                        },
                        then_body: vec![RustStmt::Return(Some(err_parse(RustExpr::FormatMacro {
                            name: "format".to_string(),
                            format_str: "invalid altchars: {}".to_string(),
                            args: vec![RustExpr::Ident("__alt".to_string())],
                        })))],
                        else_body: None,
                    },
                    RustStmt::Let {
                        mutable: true,
                        name: "__it".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__alt".to_string())),
                            method: "chars".to_string(),
                            args: vec![],
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__a".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__it".to_string())),
                                method: "next".to_string(),
                                args: vec![],
                            }),
                            method: "unwrap_or".to_string(),
                            args: vec![char_lit('+')],
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__b".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__it".to_string())),
                                method: "next".to_string(),
                                args: vec![],
                            }),
                            method: "unwrap_or".to_string(),
                            args: vec![char_lit('/')],
                        },
                    },
                    RustStmt::Assign {
                        target: RustExpr::Ident("__encoded".to_string()),
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__encoded".to_string())),
                                    method: "chars".to_string(),
                                    args: vec![],
                                }),
                                method: "map".to_string(),
                                args: vec![RustExpr::Closure {
                                    params: vec![RustParam::Named {
                                        name: "c".to_string(),
                                        ty: RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(RustExpr::If {
                                        cond: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("c".to_string())),
                                            op: "==".to_string(),
                                            right: Box::new(char_lit('+')),
                                        }),
                                        then_expr: Box::new(RustExpr::Ident("__a".to_string())),
                                        else_expr: Some(Box::new(RustExpr::If {
                                            cond: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Ident("c".to_string())),
                                                op: "==".to_string(),
                                                right: Box::new(char_lit('/')),
                                            }),
                                            then_expr: Box::new(RustExpr::Ident("__b".to_string())),
                                            else_expr: Some(Box::new(RustExpr::Ident(
                                                "c".to_string(),
                                            ))),
                                        })),
                                    }),
                                    is_move: false,
                                }],
                            }),
                            method: "collect::<String>".to_string(),
                            args: vec![],
                        },
                    },
                ],
                else_body: None,
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__wrap".to_string())),
                    op: "==".to_string(),
                    right: Box::new(int_lit(0)),
                },
                then_body: vec![RustStmt::Return(Some(ok_expr(RustExpr::Ident(
                    "__encoded".to_string(),
                ))))],
                else_body: None,
            },
            RustStmt::Let {
                mutable: false,
                name: "__w".to_string(),
                ty: None,
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident("__wrap".to_string())),
                    ty: RustType::Named("usize".to_string()),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__wrapped".to_string(),
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
                var: "__pair".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__encoded".to_string())),
                        method: "chars".to_string(),
                        args: vec![],
                    }),
                    method: "enumerate".to_string(),
                    args: vec![],
                },
                body: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__i".to_string(),
                        ty: None,
                        value: RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__pair".to_string())),
                            field: "0".to_string(),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__ch".to_string(),
                        ty: None,
                        value: RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__pair".to_string())),
                            field: "1".to_string(),
                        },
                    },
                    RustStmt::If {
                        cond: RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__i".to_string())),
                                op: ">".to_string(),
                                right: Box::new(int_lit(0)),
                            }),
                            op: "&&".to_string(),
                            right: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__i".to_string())),
                                    op: "%".to_string(),
                                    right: Box::new(RustExpr::Ident("__w".to_string())),
                                }),
                                op: "==".to_string(),
                                right: Box::new(int_lit(0)),
                            }),
                        },
                        then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__wrapped".to_string())),
                            method: "push".to_string(),
                            args: vec![char_lit('\n')],
                        })],
                        else_body: None,
                    },
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__wrapped".to_string())),
                        method: "push".to_string(),
                        args: vec![RustExpr::Ident("__ch".to_string())],
                    }),
                ],
            },
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Ident("__wrapped".to_string())))),
    })
}

pub(crate) fn lower_base64_decode_opts(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__s".to_string(),
                ty: None,
                value: ref_arg(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__alt".to_string(),
                ty: None,
                value: ref_arg(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__validate".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
            RustStmt::Let {
                mutable: false,
                name: "__ignore".to_string(),
                ty: None,
                value: ref_arg(args, 3),
            },
            RustStmt::Let {
                mutable: true,
                name: "__has_alt".to_string(),
                ty: None,
                value: bool_lit(false),
            },
            RustStmt::Let {
                mutable: true,
                name: "__alt_a".to_string(),
                ty: None,
                value: char_lit('+'),
            },
            RustStmt::Let {
                mutable: true,
                name: "__alt_b".to_string(),
                ty: None,
                value: char_lit('/'),
            },
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__alt".to_string())),
                        method: "is_empty".to_string(),
                        args: vec![],
                    }),
                },
                then_body: vec![
                    RustStmt::If {
                        cond: RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__alt".to_string())),
                                    method: "chars".to_string(),
                                    args: vec![],
                                }),
                                method: "count".to_string(),
                                args: vec![],
                            }),
                            op: "!=".to_string(),
                            right: Box::new(int_lit(2)),
                        },
                        then_body: vec![RustStmt::Return(Some(err_parse(RustExpr::FormatMacro {
                            name: "format".to_string(),
                            format_str: "invalid altchars: {}".to_string(),
                            args: vec![RustExpr::Ident("__alt".to_string())],
                        })))],
                        else_body: None,
                    },
                    RustStmt::Let {
                        mutable: true,
                        name: "__it".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__alt".to_string())),
                            method: "chars".to_string(),
                            args: vec![],
                        },
                    },
                    RustStmt::Assign {
                        target: RustExpr::Ident("__alt_a".to_string()),
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__it".to_string())),
                                method: "next".to_string(),
                                args: vec![],
                            }),
                            method: "unwrap_or".to_string(),
                            args: vec![char_lit('+')],
                        },
                    },
                    RustStmt::Assign {
                        target: RustExpr::Ident("__alt_b".to_string()),
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__it".to_string())),
                                method: "next".to_string(),
                                args: vec![],
                            }),
                            method: "unwrap_or".to_string(),
                            args: vec![char_lit('/')],
                        },
                    },
                    RustStmt::Assign {
                        target: RustExpr::Ident("__has_alt".to_string()),
                        value: bool_lit(true),
                    },
                ],
                else_body: None,
            },
            RustStmt::Let {
                mutable: true,
                name: "__ignore_set".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "collections".to_string(),
                        "HashSet::<char>".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "ch".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__ignore".to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                },
                body: vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__ignore_set".to_string())),
                    method: "insert".to_string(),
                    args: vec![RustExpr::Ident("ch".to_string())],
                })],
            },
            RustStmt::Let {
                mutable: true,
                name: "__normalized".to_string(),
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
                    receiver: Box::new(RustExpr::Ident("__s".to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                },
                body: vec![
                    RustStmt::If {
                        cond: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__ignore_set".to_string())),
                            method: "contains".to_string(),
                            args: vec![ref_expr(RustExpr::Ident("ch".to_string()))],
                        },
                        then_body: vec![RustStmt::Continue],
                        else_body: None,
                    },
                    RustStmt::Let {
                        mutable: true,
                        name: "mapped".to_string(),
                        ty: None,
                        value: RustExpr::Ident("ch".to_string()),
                    },
                    RustStmt::If {
                        cond: RustExpr::Ident("__has_alt".to_string()),
                        then_body: vec![RustStmt::If {
                            cond: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("ch".to_string())),
                                op: "==".to_string(),
                                right: Box::new(RustExpr::Ident("__alt_a".to_string())),
                            },
                            then_body: vec![RustStmt::Assign {
                                target: RustExpr::Ident("mapped".to_string()),
                                value: char_lit('+'),
                            }],
                            else_body: Some(vec![RustStmt::If {
                                cond: RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("ch".to_string())),
                                    op: "==".to_string(),
                                    right: Box::new(RustExpr::Ident("__alt_b".to_string())),
                                },
                                then_body: vec![RustStmt::Assign {
                                    target: RustExpr::Ident("mapped".to_string()),
                                    value: char_lit('/'),
                                }],
                                else_body: None,
                            }]),
                        }],
                        else_body: None,
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "is_base64".to_string(),
                        ty: None,
                        value: is_base64_char("mapped"),
                    },
                    RustStmt::If {
                        cond: RustExpr::Ident("is_base64".to_string()),
                        then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__normalized".to_string())),
                            method: "push".to_string(),
                            args: vec![RustExpr::Ident("mapped".to_string())],
                        })],
                        else_body: Some(vec![RustStmt::If {
                            cond: RustExpr::Ident("__validate".to_string()),
                            then_body: vec![RustStmt::Return(Some(err_parse(
                                RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str: "invalid base64 character: {}".to_string(),
                                    args: vec![RustExpr::Ident("ch".to_string())],
                                },
                            )))],
                            else_body: None,
                        }]),
                    },
                ],
            },
            RustStmt::Let {
                mutable: false,
                name: "__bytes".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(parse_map_err(engine_decode(
                    base64_engine_standard(),
                    as_bytes(RustExpr::Ident("__normalized".to_string())),
                )))),
            },
        ],
        expr: Some(Box::new(parse_map_err(string_from_utf8(RustExpr::Ident(
            "__bytes".to_string(),
        ))))),
    })
}

pub(crate) fn lower_urlsafe_b64encode(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(engine_encode(
        base64_engine_url_safe(),
        as_bytes(ref_arg(args, 0)),
    ))
}

pub(crate) fn lower_urlsafe_b64encode_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(engine_encode(base64_engine_url_safe(), ref_arg(args, 0))),
        method: "into_bytes".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_urlsafe_b64decode(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__bytes".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(parse_map_err(engine_decode(
                base64_engine_url_safe(),
                as_bytes(ref_arg(args, 0)),
            )))),
        }],
        expr: Some(Box::new(parse_map_err(string_from_utf8(RustExpr::Ident(
            "__bytes".to_string(),
        ))))),
    })
}

pub(crate) fn lower_urlsafe_b64decode_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(parse_map_err(engine_decode(
        base64_engine_url_safe(),
        ref_arg(args, 0),
    )))
}
