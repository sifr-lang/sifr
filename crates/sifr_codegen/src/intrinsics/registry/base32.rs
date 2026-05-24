//! Base32 intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn int(v: i64) -> RustExpr {
    RustExpr::Literal(RustLiteral::Int(v))
}

fn char_lit(v: char) -> RustExpr {
    RustExpr::Literal(RustLiteral::Char(v))
}

fn bytes_lit(v: &str) -> RustExpr {
    RustExpr::Ident(format!("b{v:?}"))
}

fn ok_expr(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![expr],
    }
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

fn cast(expr: RustExpr, ty: &str) -> RustExpr {
    RustExpr::Cast {
        expr: Box::new(expr),
        ty: RustType::Named(ty.to_string()),
    }
}

fn encode_with_alphabet(input: RustExpr, alphabet: &str) -> RustExpr {
    RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__b32_alpha".to_string(),
                ty: None,
                value: bytes_lit(alphabet),
            },
            RustStmt::Let {
                mutable: false,
                name: "__s".to_string(),
                ty: None,
                value: input,
            },
            RustStmt::Let {
                mutable: false,
                name: "__data".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__s".to_string())),
                    method: "as_bytes".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__out".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__i".to_string(),
                ty: None,
                value: cast(int(0), "usize"),
            },
            RustStmt::While {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__i".to_string())),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__data".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                },
                body: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__b0".to_string(),
                        ty: None,
                        value: cast(
                            RustExpr::Index {
                                expr: Box::new(RustExpr::Ident("__data".to_string())),
                                index: Box::new(RustExpr::Ident("__i".to_string())),
                            },
                            "i64",
                        ),
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__b1".to_string(),
                        ty: None,
                        value: RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__i".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(int(1)),
                                }),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__data".to_string())),
                                    method: "len".to_string(),
                                    args: vec![],
                                }),
                            }),
                            then_expr: Box::new(cast(
                                RustExpr::Index {
                                    expr: Box::new(RustExpr::Ident("__data".to_string())),
                                    index: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__i".to_string())),
                                        op: "+".to_string(),
                                        right: Box::new(int(1)),
                                    }),
                                },
                                "i64",
                            )),
                            else_expr: Some(Box::new(int(0))),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__b2".to_string(),
                        ty: None,
                        value: RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__i".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(int(2)),
                                }),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__data".to_string())),
                                    method: "len".to_string(),
                                    args: vec![],
                                }),
                            }),
                            then_expr: Box::new(cast(
                                RustExpr::Index {
                                    expr: Box::new(RustExpr::Ident("__data".to_string())),
                                    index: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__i".to_string())),
                                        op: "+".to_string(),
                                        right: Box::new(int(2)),
                                    }),
                                },
                                "i64",
                            )),
                            else_expr: Some(Box::new(int(0))),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__b3".to_string(),
                        ty: None,
                        value: RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__i".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(int(3)),
                                }),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__data".to_string())),
                                    method: "len".to_string(),
                                    args: vec![],
                                }),
                            }),
                            then_expr: Box::new(cast(
                                RustExpr::Index {
                                    expr: Box::new(RustExpr::Ident("__data".to_string())),
                                    index: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__i".to_string())),
                                        op: "+".to_string(),
                                        right: Box::new(int(3)),
                                    }),
                                },
                                "i64",
                            )),
                            else_expr: Some(Box::new(int(0))),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__b4".to_string(),
                        ty: None,
                        value: RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__i".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(int(4)),
                                }),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__data".to_string())),
                                    method: "len".to_string(),
                                    args: vec![],
                                }),
                            }),
                            then_expr: Box::new(cast(
                                RustExpr::Index {
                                    expr: Box::new(RustExpr::Ident("__data".to_string())),
                                    index: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__i".to_string())),
                                        op: "+".to_string(),
                                        right: Box::new(int(4)),
                                    }),
                                },
                                "i64",
                            )),
                            else_expr: Some(Box::new(int(0))),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__buf".to_string(),
                        ty: None,
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__b0".to_string())),
                                            op: "<<".to_string(),
                                            right: Box::new(int(32)),
                                        }),
                                        op: "|".to_string(),
                                        right: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__b1".to_string())),
                                            op: "<<".to_string(),
                                            right: Box::new(int(24)),
                                        }),
                                    }),
                                    op: "|".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__b2".to_string())),
                                        op: "<<".to_string(),
                                        right: Box::new(int(16)),
                                    }),
                                }),
                                op: "|".to_string(),
                                right: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Ident("__b3".to_string())),
                                    op: "<<".to_string(),
                                    right: Box::new(int(8)),
                                }),
                            }),
                            op: "|".to_string(),
                            right: Box::new(RustExpr::Ident("__b4".to_string())),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__remaining".to_string(),
                        ty: None,
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__data".to_string())),
                                method: "len".to_string(),
                                args: vec![],
                            }),
                            op: "-".to_string(),
                            right: Box::new(RustExpr::Ident("__i".to_string())),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__n".to_string(),
                        ty: None,
                        value: RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__remaining".to_string())),
                                op: "<".to_string(),
                                right: Box::new(int(5)),
                            }),
                            then_expr: Box::new(RustExpr::Ident("__remaining".to_string())),
                            else_expr: Some(Box::new(int(5))),
                        },
                    },
                    RustStmt::For {
                        var: "__j".to_string(),
                        iter: RustExpr::Range {
                            start: Box::new(int(0)),
                            end: Box::new(int(8)),
                        },
                        body: vec![RustStmt::If {
                            cond: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__j".to_string())),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__n".to_string())),
                                            op: "*".to_string(),
                                            right: Box::new(int(8)),
                                        }),
                                        op: "+".to_string(),
                                        right: Box::new(int(4)),
                                    }),
                                    op: "/".to_string(),
                                    right: Box::new(int(5)),
                                }),
                            },
                            then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__out".to_string())),
                                method: "push".to_string(),
                                args: vec![cast(
                                    RustExpr::Index {
                                        expr: Box::new(RustExpr::Ident("__b32_alpha".to_string())),
                                        index: Box::new(cast(
                                            RustExpr::BinOp {
                                                left: Box::new(RustExpr::BinOp {
                                                    left: Box::new(RustExpr::Ident(
                                                        "__buf".to_string(),
                                                    )),
                                                    op: ">>".to_string(),
                                                    right: Box::new(cast(
                                                        RustExpr::BinOp {
                                                            left: Box::new(int(35)),
                                                            op: "-".to_string(),
                                                            right: Box::new(RustExpr::BinOp {
                                                                left: Box::new(RustExpr::Ident(
                                                                    "__j".to_string(),
                                                                )),
                                                                op: "*".to_string(),
                                                                right: Box::new(int(5)),
                                                            }),
                                                        },
                                                        "usize",
                                                    )),
                                                }),
                                                op: "&".to_string(),
                                                right: Box::new(int(0x1f)),
                                            },
                                            "usize",
                                        )),
                                    },
                                    "char",
                                )],
                            })],
                            else_body: Some(vec![RustStmt::Expr(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__out".to_string())),
                                method: "push".to_string(),
                                args: vec![char_lit('=')],
                            })]),
                        }],
                    },
                    RustStmt::AugAssign {
                        target: RustExpr::Ident("__i".to_string()),
                        op: "+".to_string(),
                        value: int(5),
                    },
                ],
            },
        ],
        expr: Some(Box::new(RustExpr::Ident("__out".to_string()))),
    }
}

fn decode_with_alphabet(input: RustExpr, alphabet: &str, invalid_msg: &str) -> RustExpr {
    RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__b32_alpha".to_string(),
                ty: None,
                value: bytes_lit(alphabet),
            },
            RustStmt::Let {
                mutable: false,
                name: "__s_val".to_string(),
                ty: None,
                value: input,
            },
            RustStmt::Let {
                mutable: false,
                name: "__s".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__s_val".to_string())),
                    method: "trim_end_matches".to_string(),
                    args: vec![char_lit('=')],
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__bits".to_string(),
                ty: None,
                value: int(0),
            },
            RustStmt::Let {
                mutable: true,
                name: "__bit_count".to_string(),
                ty: None,
                value: int(0),
            },
            RustStmt::Let {
                mutable: true,
                name: "__out".to_string(),
                ty: Some(RustType::Vec(Box::new(RustType::Named("u8".to_string())))),
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "__c".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__s".to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                },
                body: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__val_opt".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__b32_alpha".to_string())),
                                method: "iter".to_string(),
                                args: vec![],
                            }),
                            method: "position".to_string(),
                            args: vec![RustExpr::Closure {
                                params: vec![RustParam::Named {
                                    name: "b".to_string(),
                                    ty: RustType::Named("_".to_string()),
                                }],
                                body: Box::new(RustExpr::BinOp {
                                    left: Box::new(cast(
                                        RustExpr::Deref(Box::new(RustExpr::Ident("b".to_string()))),
                                        "char",
                                    )),
                                    op: "==".to_string(),
                                    right: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__c".to_string())),
                                        method: "to_ascii_uppercase".to_string(),
                                        args: vec![],
                                    }),
                                }),
                                is_move: false,
                            }],
                        },
                    },
                    RustStmt::Let {
                        mutable: true,
                        name: "__val".to_string(),
                        ty: None,
                        value: int(0),
                    },
                    RustStmt::IfLet {
                        pattern: "Some(__v)".to_string(),
                        expr: RustExpr::Ident("__val_opt".to_string()),
                        then_body: vec![RustStmt::Assign {
                            target: RustExpr::Ident("__val".to_string()),
                            value: cast(RustExpr::Ident("__v".to_string()), "i64"),
                        }],
                        else_body: Some(vec![RustStmt::Return(Some(err_parse(
                            RustExpr::FormatMacro {
                                name: "format".to_string(),
                                format_str: invalid_msg.to_string(),
                                args: vec![RustExpr::Ident("__c".to_string())],
                            },
                        )))]),
                    },
                    RustStmt::Assign {
                        target: RustExpr::Ident("__bits".to_string()),
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__bits".to_string())),
                                op: "<<".to_string(),
                                right: Box::new(int(5)),
                            }),
                            op: "|".to_string(),
                            right: Box::new(RustExpr::Ident("__val".to_string())),
                        },
                    },
                    RustStmt::AugAssign {
                        target: RustExpr::Ident("__bit_count".to_string()),
                        op: "+".to_string(),
                        value: int(5),
                    },
                    RustStmt::If {
                        cond: RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__bit_count".to_string())),
                            op: ">=".to_string(),
                            right: Box::new(int(8)),
                        },
                        then_body: vec![
                            RustStmt::AugAssign {
                                target: RustExpr::Ident("__bit_count".to_string()),
                                op: "-".to_string(),
                                value: int(8),
                            },
                            RustStmt::Expr(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__out".to_string())),
                                method: "push".to_string(),
                                args: vec![cast(
                                    RustExpr::BinOp {
                                        left: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__bits".to_string())),
                                            op: ">>".to_string(),
                                            right: Box::new(cast(
                                                RustExpr::Ident("__bit_count".to_string()),
                                                "usize",
                                            )),
                                        }),
                                        op: "&".to_string(),
                                        right: Box::new(int(0xff)),
                                    },
                                    "u8",
                                )],
                            }),
                        ],
                        else_body: None,
                    },
                ],
            },
        ],
        expr: Some(Box::new(parse_map_err(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "String".to_string(),
                "from_utf8".to_string(),
            ])),
            args: vec![RustExpr::Ident("__out".to_string())],
        }))),
    }
}

pub(crate) fn lower_b32encode(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(encode_with_alphabet(
        arg_expr(args, 0),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
    ))
}

pub(crate) fn lower_b32decode(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(decode_with_alphabet(
        arg_expr(args, 0),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
        "invalid base32 char: {}",
    ))
}

pub(crate) fn lower_b32hexencode(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(encode_with_alphabet(
        arg_expr(args, 0),
        "0123456789ABCDEFGHIJKLMNOPQRSTUV",
    ))
}

pub(crate) fn lower_b32hexdecode(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(decode_with_alphabet(
        arg_expr(args, 0),
        "0123456789ABCDEFGHIJKLMNOPQRSTUV",
        "invalid base32hex char: {}",
    ))
}
