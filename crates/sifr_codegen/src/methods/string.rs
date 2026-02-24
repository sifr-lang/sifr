//! String method lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn lower_zero_arg_method(object: &str, args: &[String], method: &str) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: method.to_string(),
        args: vec![],
    })
}

fn lower_trim_to_string(object: &str, args: &[String], trim_method: &str) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(object.to_string())),
            method: trim_method.to_string(),
            args: vec![],
        }),
        method: "to_string".to_string(),
        args: vec![],
    })
}

fn render_borrowed_arg_expr(arg: &str) -> RustExpr {
    if arg.ends_with(".as_str()") || arg.starts_with('&') {
        RustExpr::Ident(arg.to_string())
    } else {
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Ident(format!("({arg})"))),
        }
    }
}

fn lower_non_empty_char_all(
    object: &str,
    args: &[String],
    char_predicate_method: &str,
) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "is_empty".to_string(),
                args: vec![],
            }),
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "all".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "c".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("c".to_string())),
                    method: char_predicate_method.to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }),
    })
}

fn char_predicate_closure(method: &str) -> RustExpr {
    RustExpr::Closure {
        params: vec![RustParam::Named {
            name: "c".to_string(),
            ty: RustType::Named("_".to_string()),
        }],
        body: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("c".to_string())),
            method: method.to_string(),
            args: vec![],
        }),
        is_move: false,
    }
}

fn lower_has_alpha_and_filtered_all(
    object: &str,
    args: &[String],
    alpha_case_method: &str,
) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "any".to_string(),
            args: vec![char_predicate_closure("is_alphabetic")],
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "filter".to_string(),
                args: vec![char_predicate_closure("is_alphabetic")],
            }),
            method: "all".to_string(),
            args: vec![char_predicate_closure(alpha_case_method)],
        }),
    })
}

pub(super) fn lower_upper(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_zero_arg_method(object, args, "to_uppercase")
}

pub(super) fn lower_lower(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_zero_arg_method(object, args, "to_lowercase")
}

pub(super) fn lower_strip(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim")
}

pub(super) fn lower_startswith(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "starts_with".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_endswith(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "ends_with".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_split(object: &str, args: &[String]) -> Option<RustExpr> {
    match args.len() {
        0 => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.to_string())),
                    method: "split_whitespace".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "s".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("s".to_string())),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Vec<String>>".to_string(),
            args: vec![],
        }),
        1 => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.to_string())),
                    method: "split".to_string(),
                    args: vec![render_borrowed_arg_expr(&args[0])],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "s".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("s".to_string())),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Vec<String>>".to_string(),
            args: vec![],
        }),
        _ => None,
    }
}

pub(super) fn lower_replace(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "replace".to_string(),
        args: vec![
            render_borrowed_arg_expr(&args[0]),
            render_borrowed_arg_expr(&args[1]),
        ],
    })
}

pub(super) fn lower_find(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(object.to_string())),
            method: "find".to_string(),
            args: vec![render_borrowed_arg_expr(&args[0])],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "i".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Ident("i".to_string())),
                ty: RustType::I64,
            }),
            is_move: false,
        }],
    })
}

pub(super) fn lower_lstrip(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim_start")
}

pub(super) fn lower_rstrip(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim_end")
}

pub(super) fn lower_count(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "matches".to_string(),
                args: vec![render_borrowed_arg_expr(&args[0])],
            }),
            method: "count".to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}

pub(super) fn lower_join(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(args[0].clone())),
        method: "join".to_string(),
        args: vec![render_borrowed_arg_expr(object)],
    })
}

pub(super) fn lower_title(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.to_string())),
                    method: "split_whitespace".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "w".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Block {
                        stmts: vec![RustStmt::Let {
                            mutable: true,
                            name: "c".to_string(),
                            ty: None,
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("w".to_string())),
                                method: "chars".to_string(),
                                args: vec![],
                            },
                        }],
                        expr: Some(Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("c".to_string())),
                                    method: "next".to_string(),
                                    args: vec![],
                                }),
                                method: "map".to_string(),
                                args: vec![RustExpr::Closure {
                                    params: vec![RustParam::Named {
                                        name: "f".to_string(),
                                        ty: RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "f".to_string(),
                                                )),
                                                method: "to_uppercase".to_string(),
                                                args: vec![],
                                            }),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        }),
                                        op: "+".to_string(),
                                        right: Box::new(RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::MethodCall {
                                                    receiver: Box::new(RustExpr::Ident(
                                                        "c".to_string(),
                                                    )),
                                                    method: "as_str".to_string(),
                                                    args: vec![],
                                                }),
                                                method: "to_lowercase".to_string(),
                                                args: vec![],
                                            }),
                                        }),
                                    }),
                                    is_move: false,
                                }],
                            }),
                            method: "unwrap_or_default".to_string(),
                            args: vec![],
                        })),
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Vec<_>>".to_string(),
            args: vec![],
        }),
        method: "join".to_string(),
        args: vec![render_borrowed_arg_expr("\" \"")],
    })
}

pub(super) fn lower_capitalize(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "_s".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(object.to_string()))),
            },
            RustStmt::Let {
                mutable: true,
                name: "_c".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("_s".to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("_c".to_string())),
                    method: "next".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "f".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("f".to_string())),
                                method: "to_uppercase".to_string(),
                                args: vec![],
                            }),
                            method: "to_string".to_string(),
                            args: vec![],
                        }),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("_c".to_string())),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }),
                                method: "to_lowercase".to_string(),
                                args: vec![],
                            }),
                        }),
                    }),
                    is_move: false,
                }],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_swapcase(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
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
                    cond: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("c".to_string())),
                        method: "is_uppercase".to_string(),
                        args: vec![],
                    }),
                    then_expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("c".to_string())),
                            method: "to_lowercase".to_string(),
                            args: vec![],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    else_expr: Some(Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("c".to_string())),
                            method: "to_uppercase".to_string(),
                            args: vec![],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    })),
                }),
                is_move: false,
            }],
        }),
        method: "collect::<String>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_isdigit(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_non_empty_char_all(object, args, "is_ascii_digit")
}

pub(super) fn lower_isalpha(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_non_empty_char_all(object, args, "is_alphabetic")
}

pub(super) fn lower_isalnum(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_non_empty_char_all(object, args, "is_alphanumeric")
}

pub(super) fn lower_isspace(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_non_empty_char_all(object, args, "is_whitespace")
}

pub(super) fn lower_isupper(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_has_alpha_and_filtered_all(object, args, "is_uppercase")
}

pub(super) fn lower_islower(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_has_alpha_and_filtered_all(object, args, "is_lowercase")
}

pub(super) fn lower_center(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "_s".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(object.to_string()))),
            },
            RustStmt::Let {
                mutable: false,
                name: "_w".to_string(),
                ty: None,
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident(args[0].clone())),
                    ty: RustType::Named("usize".to_string()),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "_len".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("_s".to_string())),
                        method: "chars".to_string(),
                        args: vec![],
                    }),
                    method: "count".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("_len".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Ident("_w".to_string())),
            }),
            then_expr: Box::new(RustExpr::Ident("_s".to_string())),
            else_expr: Some(Box::new(RustExpr::Block {
                stmts: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "_pad".to_string(),
                        ty: None,
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("_w".to_string())),
                            op: "-".to_string(),
                            right: Box::new(RustExpr::Ident("_len".to_string())),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "_left".to_string(),
                        ty: None,
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("_pad".to_string())),
                            op: "/".to_string(),
                            right: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                        },
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "_right".to_string(),
                        ty: None,
                        value: RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("_pad".to_string())),
                            op: "-".to_string(),
                            right: Box::new(RustExpr::Ident("_left".to_string())),
                        },
                    },
                ],
                expr: Some(Box::new(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{}{}{}".to_string(),
                    args: vec![
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                                " ".to_string(),
                            ))),
                            method: "repeat".to_string(),
                            args: vec![RustExpr::Ident("_left".to_string())],
                        },
                        RustExpr::Ident("_s".to_string()),
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                                " ".to_string(),
                            ))),
                            method: "repeat".to_string(),
                            args: vec![RustExpr::Ident("_right".to_string())],
                        },
                    ],
                })),
            })),
        })),
    })
}

pub(super) fn lower_ljust(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{:<width$}".to_string(),
        args: vec![
            RustExpr::Ident(object.to_string()),
            RustExpr::Ident(format!("width = {} as usize", args[0])),
        ],
    })
}

pub(super) fn lower_rjust(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{:>width$}".to_string(),
        args: vec![
            RustExpr::Ident(object.to_string()),
            RustExpr::Ident(format!("width = {} as usize", args[0])),
        ],
    })
}

pub(super) fn lower_zfill(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{:0>width$}".to_string(),
        args: vec![
            RustExpr::Ident(object.to_string()),
            RustExpr::Ident(format!("width = {} as usize", args[0])),
        ],
    })
}
