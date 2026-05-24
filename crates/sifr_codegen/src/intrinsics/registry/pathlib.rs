//! Pathlib intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType};

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

fn ref_ident(name: &str) -> RustExpr {
    ref_expr(RustExpr::Ident(name.to_string()))
}

fn bool_lit(v: bool) -> RustExpr {
    RustExpr::Literal(RustLiteral::Bool(v))
}

fn str_ref_lit(v: &str) -> RustExpr {
    RustExpr::Ident(format!("{v:?}"))
}

fn io_map_err(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    }
}

fn io_map_err_new(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "e".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "IOError".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("e".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                }],
            }),
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

fn to_string_expr(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn regex_source_expr(pattern_ident: &str) -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "^{}$".to_string(),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "regex".to_string(),
                        "escape".to_string(),
                    ])),
                    args: vec![RustExpr::Ident(pattern_ident.to_string())],
                }),
                method: "replace".to_string(),
                args: vec![str_ref_lit("\\*"), str_ref_lit(".*")],
            }),
            method: "replace".to_string(),
            args: vec![str_ref_lit("\\?"), str_ref_lit(".")],
        }],
    }
}

fn starts_with_dot_expr(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "starts_with".to_string(),
        args: vec![str_ref_lit(".")],
    }
}

fn empty_string_vec_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
        args: vec![],
    }
}

fn regex_new(regex_src_ident: &str) -> RustExpr {
    io_map_err_new(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "regex".to_string(),
            "Regex".to_string(),
            "new".to_string(),
        ])),
        args: vec![ref_ident(regex_src_ident)],
    })
}

fn entry_name_to_string(entry_ident: &str) -> RustExpr {
    to_string_expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(entry_ident.to_string())),
                method: "file_name".to_string(),
                args: vec![],
            }),
            method: "to_string_lossy".to_string(),
            args: vec![],
        }),
        method: "to_string".to_string(),
        args: vec![],
    })
}

fn entry_path_to_string(entry_ident: &str) -> RustExpr {
    to_string_expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(entry_ident.to_string())),
            method: "path".to_string(),
            args: vec![],
        }),
        method: "to_string_lossy".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_touch(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "fs".to_string(),
                                "OpenOptions".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        }),
                        method: "create".to_string(),
                        args: vec![bool_lit(true)],
                    }),
                    method: "truncate".to_string(),
                    args: vec![bool_lit(false)],
                }),
                method: "write".to_string(),
                args: vec![bool_lit(true)],
            }),
            method: "open".to_string(),
            args: vec![ref_arg(args, 0)],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "_".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::Literal(RustLiteral::Unit)),
            is_move: false,
        }],
    }))
}

pub(crate) fn lower_resolve_path(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "fs".to_string(),
                "canonicalize".to_string(),
            ])),
            args: vec![ref_arg(args, 0)],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "p".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(to_string_expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("p".to_string())),
                method: "to_string_lossy".to_string(),
                args: vec![],
            })),
            is_move: false,
        }],
    }))
}

pub(crate) fn lower_iterdir(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__entries".to_string(),
            ty: None,
            value: RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "fs".to_string(),
                    "read_dir".to_string(),
                ])),
                args: vec![ref_arg(args, 0)],
            }))),
        }],
        expr: Some(Box::new(ok_expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__entries".to_string())),
                method: "filter_map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "e".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("e".to_string())),
                            method: "ok".to_string(),
                            args: vec![],
                        }),
                        method: "map".to_string(),
                        args: vec![RustExpr::Closure {
                            params: vec![RustParam::Named {
                                name: "e".to_string(),
                                ty: RustType::Named("_".to_string()),
                            }],
                            body: Box::new(entry_path_to_string("e")),
                            is_move: false,
                        }],
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Vec<String>>".to_string(),
            args: vec![],
        }))),
    })
}

pub(crate) fn lower_glob_pattern(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__dir".to_string(),
                ty: None,
                value: ref_arg(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__pat".to_string(),
                ty: None,
                value: ref_arg(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__include_hidden".to_string(),
                ty: None,
                value: starts_with_dot_expr(RustExpr::Ident("__pat".to_string())),
            },
            RustStmt::Let {
                mutable: false,
                name: "__regex_src".to_string(),
                ty: None,
                value: regex_source_expr("__pat"),
            },
            RustStmt::Let {
                mutable: false,
                name: "__re".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(regex_new("__regex_src"))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__results".to_string(),
                ty: Some(RustType::Vec(Box::new(RustType::String_))),
                value: empty_string_vec_expr(),
            },
            RustStmt::Match {
                expr: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fs".to_string(),
                        "read_dir".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__dir".to_string())],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Ok(__entries)".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::For {
                            var: "__entry".to_string(),
                            iter: RustExpr::Ident("__entries".to_string()),
                            body: vec![RustStmt::IfLet {
                                pattern: "Ok(__e)".to_string(),
                                expr: RustExpr::Ident("__entry".to_string()),
                                then_body: vec![
                                    RustStmt::Let {
                                        mutable: false,
                                        name: "__name".to_string(),
                                        ty: None,
                                        value: entry_name_to_string("__e"),
                                    },
                                    RustStmt::If {
                                        cond: RustExpr::BinOp {
                                            left: Box::new(RustExpr::UnaryOp {
                                                op: "!".to_string(),
                                                operand: Box::new(RustExpr::Ident(
                                                    "__include_hidden".to_string(),
                                                )),
                                            }),
                                            op: "&&".to_string(),
                                            right: Box::new(starts_with_dot_expr(RustExpr::Ident(
                                                "__name".to_string(),
                                            ))),
                                        },
                                        then_body: vec![RustStmt::Continue],
                                        else_body: None,
                                    },
                                    RustStmt::If {
                                        cond: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__re".to_string())),
                                            method: "is_match".to_string(),
                                            args: vec![ref_ident("__name")],
                                        },
                                        then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__results".to_string(),
                                            )),
                                            method: "push".to_string(),
                                            args: vec![entry_path_to_string("__e")],
                                        })],
                                        else_body: None,
                                    },
                                ],
                                else_body: Some(vec![RustStmt::Continue]),
                            }],
                        }],
                    },
                    RustMatchArm {
                        pattern: "Err(_)".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(ok_expr(empty_string_vec_expr())))],
                    },
                ],
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__results".to_string())),
                method: "sort".to_string(),
                args: vec![],
            }),
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Ident("__results".to_string())))),
    })
}

pub(crate) fn lower_rglob_pattern(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__dir".to_string(),
                ty: None,
                value: ref_arg(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__pat".to_string(),
                ty: None,
                value: ref_arg(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__include_hidden".to_string(),
                ty: None,
                value: starts_with_dot_expr(RustExpr::Ident("__pat".to_string())),
            },
            RustStmt::Let {
                mutable: false,
                name: "__regex_src".to_string(),
                ty: None,
                value: regex_source_expr("__pat"),
            },
            RustStmt::Let {
                mutable: false,
                name: "__re".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(regex_new("__regex_src"))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__results".to_string(),
                ty: Some(RustType::Vec(Box::new(RustType::String_))),
                value: empty_string_vec_expr(),
            },
            RustStmt::Let {
                mutable: true,
                name: "__stack".to_string(),
                ty: Some(RustType::Vec(Box::new(RustType::String_))),
                value: RustExpr::Vec(vec![to_string_expr(RustExpr::Ident("__dir".to_string()))]),
            },
            RustStmt::Loop {
                body: vec![RustStmt::IfLet {
                    pattern: "Some(__current)".to_string(),
                    expr: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__stack".to_string())),
                        method: "pop".to_string(),
                        args: vec![],
                    },
                    then_body: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__entries_result".to_string(),
                            ty: None,
                            value: RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "std".to_string(),
                                    "fs".to_string(),
                                    "read_dir".to_string(),
                                ])),
                                args: vec![ref_ident("__current")],
                            },
                        },
                        RustStmt::IfLet {
                            pattern: "Ok(__entries)".to_string(),
                            expr: RustExpr::Ident("__entries_result".to_string()),
                            then_body: vec![RustStmt::For {
                                var: "__entry".to_string(),
                                iter: RustExpr::Ident("__entries".to_string()),
                                body: vec![RustStmt::IfLet {
                                    pattern: "Ok(__e)".to_string(),
                                    expr: RustExpr::Ident("__entry".to_string()),
                                    then_body: vec![
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "__path".to_string(),
                                            ty: None,
                                            value: RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "__e".to_string(),
                                                )),
                                                method: "path".to_string(),
                                                args: vec![],
                                            },
                                        },
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "__name".to_string(),
                                            ty: None,
                                            value: entry_name_to_string("__e"),
                                        },
                                        RustStmt::If {
                                            cond: RustExpr::BinOp {
                                                left: Box::new(RustExpr::UnaryOp {
                                                    op: "!".to_string(),
                                                    operand: Box::new(RustExpr::Ident(
                                                        "__include_hidden".to_string(),
                                                    )),
                                                }),
                                                op: "&&".to_string(),
                                                right: Box::new(starts_with_dot_expr(
                                                    RustExpr::Ident("__name".to_string()),
                                                )),
                                            },
                                            then_body: vec![RustStmt::Continue],
                                            else_body: None,
                                        },
                                        RustStmt::If {
                                            cond: RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "__path".to_string(),
                                                )),
                                                method: "is_dir".to_string(),
                                                args: vec![],
                                            },
                                            then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "__stack".to_string(),
                                                )),
                                                method: "push".to_string(),
                                                args: vec![to_string_expr(RustExpr::MethodCall {
                                                    receiver: Box::new(RustExpr::Ident(
                                                        "__path".to_string(),
                                                    )),
                                                    method: "to_string_lossy".to_string(),
                                                    args: vec![],
                                                })],
                                            })],
                                            else_body: None,
                                        },
                                        RustStmt::If {
                                            cond: RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "__re".to_string(),
                                                )),
                                                method: "is_match".to_string(),
                                                args: vec![ref_ident("__name")],
                                            },
                                            then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "__results".to_string(),
                                                )),
                                                method: "push".to_string(),
                                                args: vec![to_string_expr(RustExpr::MethodCall {
                                                    receiver: Box::new(RustExpr::Ident(
                                                        "__path".to_string(),
                                                    )),
                                                    method: "to_string_lossy".to_string(),
                                                    args: vec![],
                                                })],
                                            })],
                                            else_body: None,
                                        },
                                    ],
                                    else_body: Some(vec![RustStmt::Continue]),
                                }],
                            }],
                            else_body: Some(vec![RustStmt::Continue]),
                        },
                    ],
                    else_body: Some(vec![RustStmt::Break]),
                }],
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__results".to_string())),
                method: "sort".to_string(),
                args: vec![],
            }),
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Ident("__results".to_string())))),
    })
}
