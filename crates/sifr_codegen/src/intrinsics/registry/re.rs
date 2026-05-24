//! Regex intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn ref_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(arg_expr(args, idx)),
    }
}

fn ref_ident(name: &str) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Ident(name.to_string())),
    }
}

fn replacer_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Deref(Box::new(arg_expr(args, idx)))),
    }
}

fn to_string_expr(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn regex_error_expr(name: &str) -> RustExpr {
    RustExpr::StructInit {
        name: "RegexError".to_string(),
        fields: vec![
            (
                "message".to_string(),
                to_string_expr(RustExpr::Ident(name.to_string())),
            ),
            (
                "detail".to_string(),
                to_string_expr(RustExpr::Ident(name.to_string())),
            ),
        ],
    }
}

fn regex_error_mapper() -> RustExpr {
    RustExpr::Closure {
        params: vec![RustParam::Named {
            name: "e".to_string(),
            ty: RustType::Named("_".to_string()),
        }],
        body: Box::new(regex_error_expr("e")),
        is_move: false,
    }
}

fn map_regex_error(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![regex_error_mapper()],
    }
}

fn regex_new(pattern: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "regex".to_string(),
            "Regex".to_string(),
            "new".to_string(),
        ])),
        args: vec![pattern],
    }
}

fn wrap_ok(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![expr],
    }
}

fn map_match_to_string(expr: RustExpr, method: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "m".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(to_string_expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("m".to_string())),
                method: method.to_string(),
                args: vec![],
            })),
            is_move: false,
        }],
    }
}

fn map_str_to_string(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "s".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(to_string_expr(RustExpr::Ident("s".to_string()))),
            is_move: false,
        }],
    }
}

pub(crate) fn lower_re_match(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(regex_new(ref_arg(args, 0))),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "re".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("re".to_string())),
                method: "is_match".to_string(),
                args: vec![ref_arg(args, 1)],
            }),
            is_move: false,
        }],
    };
    Some(map_regex_error(mapped))
}

pub(crate) fn lower_re_find(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(regex_new(ref_arg(args, 0))),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "re".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(map_match_to_string(
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("re".to_string())),
                    method: "find".to_string(),
                    args: vec![ref_arg(args, 1)],
                },
                "as_str",
            )),
            is_move: false,
        }],
    };
    Some(map_regex_error(mapped))
}

pub(crate) fn lower_re_replace(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(regex_new(ref_arg(args, 0))),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "re".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(to_string_expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("re".to_string())),
                method: "replace_all".to_string(),
                args: vec![ref_arg(args, 2), replacer_arg(args, 1)],
            })),
            is_move: false,
        }],
    };
    Some(map_regex_error(mapped))
}

pub(crate) fn lower_re_findall(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(regex_new(ref_arg(args, 0))),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "re".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(map_match_to_string(
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("re".to_string())),
                        method: "find_iter".to_string(),
                        args: vec![ref_arg(args, 1)],
                    },
                    "as_str",
                )),
                method: "collect::<Vec<String>>".to_string(),
                args: vec![],
            }),
            is_move: false,
        }],
    };
    Some(map_regex_error(mapped))
}

pub(crate) fn lower_re_split(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(regex_new(ref_arg(args, 0))),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "re".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(map_str_to_string(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("re".to_string())),
                    method: "split".to_string(),
                    args: vec![ref_arg(args, 1)],
                })),
                method: "collect::<Vec<String>>".to_string(),
                args: vec![],
            }),
            is_move: false,
        }],
    };
    Some(map_regex_error(mapped))
}

pub(crate) fn lower_re_find_start(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(regex_new(ref_arg(args, 0))),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "re".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("re".to_string())),
                    method: "find".to_string(),
                    args: vec![ref_arg(args, 1)],
                }),
                method: "map_or".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Int(-1)),
                    RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "m".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("m".to_string())),
                                method: "start".to_string(),
                                args: vec![],
                            }),
                            ty: RustType::I64,
                        }),
                        is_move: false,
                    },
                ],
            }),
            is_move: false,
        }],
    };
    Some(map_regex_error(mapped))
}

pub(crate) fn lower_re_find_end(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(regex_new(ref_arg(args, 0))),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "re".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("re".to_string())),
                    method: "find".to_string(),
                    args: vec![ref_arg(args, 1)],
                }),
                method: "map_or".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Int(-1)),
                    RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "m".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("m".to_string())),
                                method: "end".to_string(),
                                args: vec![],
                            }),
                            ty: RustType::I64,
                        }),
                        is_move: false,
                    },
                ],
            }),
            is_move: false,
        }],
    };
    Some(map_regex_error(mapped))
}

fn push_flag_stmt(bit: i64, marker: &str) -> RustStmt {
    RustStmt::If {
        cond: RustExpr::BinOp {
            left: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__flags_val".to_string())),
                op: "&".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Int(bit))),
            }),
            op: "!=".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
        },
        then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__flag_str".to_string())),
            method: "push_str".to_string(),
            args: vec![RustExpr::Ident(format!("{marker:?}"))],
        })],
        else_body: None,
    }
}

fn build_flags_prefix_stmts(args: &[RustExpr], flag_idx: usize) -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: false,
            name: "__flags_val".to_string(),
            ty: None,
            value: arg_expr(args, flag_idx),
        },
        RustStmt::Let {
            mutable: true,
            name: "__flag_str".to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "String".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        },
        push_flag_stmt(2, "(?i)"),
        push_flag_stmt(8, "(?m)"),
        push_flag_stmt(16, "(?s)"),
        push_flag_stmt(64, "(?x)"),
        RustStmt::Let {
            mutable: false,
            name: "__pat".to_string(),
            ty: None,
            value: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__flag_str".to_string())),
                op: "+".to_string(),
                right: Box::new(ref_arg(args, 0)),
            },
        },
        RustStmt::Let {
            mutable: false,
            name: "__re".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(map_regex_error(regex_new(ref_ident("__pat"))))),
        },
    ]
}

fn lower_flags_common(args: &[RustExpr], mode: &str) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }

    let result_expr = match mode {
        "match" => wrap_ok(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__re".to_string())),
            method: "is_match".to_string(),
            args: vec![ref_arg(args, 1)],
        }),
        "find" => wrap_ok(map_match_to_string(
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__re".to_string())),
                method: "find".to_string(),
                args: vec![ref_arg(args, 1)],
            },
            "as_str",
        )),
        "findall" => wrap_ok(RustExpr::MethodCall {
            receiver: Box::new(map_match_to_string(
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__re".to_string())),
                    method: "find_iter".to_string(),
                    args: vec![ref_arg(args, 1)],
                },
                "as_str",
            )),
            method: "collect::<Vec<String>>".to_string(),
            args: vec![],
        }),
        "split" => wrap_ok(RustExpr::MethodCall {
            receiver: Box::new(map_str_to_string(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__re".to_string())),
                method: "split".to_string(),
                args: vec![ref_arg(args, 1)],
            })),
            method: "collect::<Vec<String>>".to_string(),
            args: vec![],
        }),
        _ => return None,
    };

    Some(RustExpr::Block {
        stmts: build_flags_prefix_stmts(args, 2),
        expr: Some(Box::new(result_expr)),
    })
}

pub(crate) fn lower_re_match_flags(args: &[RustExpr]) -> Option<RustExpr> {
    lower_flags_common(args, "match")
}

pub(crate) fn lower_re_find_flags(args: &[RustExpr]) -> Option<RustExpr> {
    lower_flags_common(args, "find")
}

pub(crate) fn lower_re_findall_flags(args: &[RustExpr]) -> Option<RustExpr> {
    lower_flags_common(args, "findall")
}

pub(crate) fn lower_re_split_flags(args: &[RustExpr]) -> Option<RustExpr> {
    lower_flags_common(args, "split")
}

pub(crate) fn lower_re_replace_flags(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: build_flags_prefix_stmts(args, 3),
        expr: Some(Box::new(wrap_ok(to_string_expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__re".to_string())),
            method: "replace_all".to_string(),
            args: vec![ref_arg(args, 2), replacer_arg(args, 1)],
        })))),
    })
}
