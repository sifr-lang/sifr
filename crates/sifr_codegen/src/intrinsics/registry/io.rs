//! I/O intrinsic lowerers for registry lowering.

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

fn to_string_expr(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn path_new(expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "path".to_string(),
            "Path".to_string(),
            "new".to_string(),
        ])),
        args: vec![expr],
    }
}

fn io_map_err(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    }
}

fn map_to_unit(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "_".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::Literal(RustLiteral::Unit)),
            is_move: false,
        }],
    }
}

pub(crate) fn lower_read_text(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "read_to_string".to_string(),
        ])),
        args: vec![ref_arg(args, 0)],
    }))
}

pub(crate) fn lower_write_text(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(io_map_err(map_to_unit(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "write".to_string(),
        ])),
        args: vec![
            ref_arg(args, 0),
            RustExpr::MethodCall {
                receiver: Box::new(arg_expr(args, 1)),
                method: "as_bytes".to_string(),
                args: vec![],
            },
        ],
    })))
}

pub(crate) fn lower_exists(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(path_new(ref_arg(args, 0))),
        method: "exists".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_read_lines(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "fs".to_string(),
                "read_to_string".to_string(),
            ])),
            args: vec![ref_arg(args, 0)],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "s".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("s".to_string())),
                        method: "lines".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "l".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(to_string_expr(RustExpr::Ident("l".to_string()))),
                        is_move: false,
                    }],
                }),
                method: "collect::<Vec<String>>".to_string(),
                args: vec![],
            }),
            is_move: false,
        }],
    }))
}

pub(crate) fn lower_append_text(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let open_opts = RustExpr::MethodCall {
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
                method: "append".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Bool(true))],
            }),
            method: "create".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Bool(true))],
        }),
        method: "open".to_string(),
        args: vec![ref_arg(args, 0)],
    };

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "_f".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(open_opts))),
            },
            RustStmt::Expr(RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "io".to_string(),
                    "Write".to_string(),
                    "write_all".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("_f".to_string())),
                    },
                    RustExpr::MethodCall {
                        receiver: Box::new(arg_expr(args, 1)),
                        method: "as_bytes".to_string(),
                        args: vec![],
                    },
                ],
            })))),
        ],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![RustExpr::Literal(RustLiteral::Unit)],
        })),
    })
}

pub(crate) fn lower_getcwd(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(io_map_err(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "env".to_string(),
                "current_dir".to_string(),
            ])),
            args: vec![],
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

pub(crate) fn lower_listdir(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "fs".to_string(),
                "read_dir".to_string(),
            ])),
            args: vec![ref_arg(args, 0)],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "rd".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("rd".to_string())),
                        method: "filter_map".to_string(),
                        args: vec![RustExpr::Closure {
                            params: vec![RustParam::Named {
                                name: "e".to_string(),
                                ty: RustType::Named("_".to_string()),
                            }],
                            body: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "ok".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    }),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "e".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(to_string_expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "file_name".to_string(),
                                args: vec![],
                            }),
                            method: "to_string_lossy".to_string(),
                            args: vec![],
                        })),
                        is_move: false,
                    }],
                }),
                method: "collect::<Vec<String>>".to_string(),
                args: vec![],
            }),
            is_move: false,
        }],
    }))
}

pub(crate) fn lower_walk_dir(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "__stack".to_string(),
                ty: None,
                value: RustExpr::Vec(vec![RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "path".to_string(),
                        "PathBuf".to_string(),
                        "from".to_string(),
                    ])),
                    args: vec![arg_expr(args, 0)],
                }]),
            },
            RustStmt::Let {
                mutable: true,
                name: "__result".to_string(),
                ty: None,
                value: RustExpr::Vec(vec![]),
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
                            name: "__entries".to_string(),
                            ty: None,
                            value: RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "std".to_string(),
                                    "fs".to_string(),
                                    "read_dir".to_string(),
                                ])),
                                args: vec![ref_ident("__current")],
                            }))),
                        },
                        RustStmt::For {
                            var: "__entry_res".to_string(),
                            iter: RustExpr::Ident("__entries".to_string()),
                            body: vec![
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__entry".to_string(),
                                    ty: None,
                                    value: RustExpr::Try(Box::new(io_map_err(RustExpr::Ident(
                                        "__entry_res".to_string(),
                                    )))),
                                },
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__path".to_string(),
                                    ty: None,
                                    value: RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__entry".to_string())),
                                        method: "path".to_string(),
                                        args: vec![],
                                    },
                                },
                                RustStmt::Expr(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__result".to_string())),
                                    method: "push".to_string(),
                                    args: vec![to_string_expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__path".to_string(),
                                            )),
                                            method: "display".to_string(),
                                            args: vec![],
                                        }),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    })],
                                }),
                                RustStmt::If {
                                    cond: RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__path".to_string())),
                                        method: "is_dir".to_string(),
                                        args: vec![],
                                    },
                                    then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__stack".to_string())),
                                        method: "push".to_string(),
                                        args: vec![RustExpr::Ident("__path".to_string())],
                                    })],
                                    else_body: None,
                                },
                            ],
                        },
                    ],
                    else_body: Some(vec![RustStmt::Break]),
                }],
            },
        ],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![RustExpr::Ident("__result".to_string())],
        })),
    })
}

pub(crate) fn lower_mkdir(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(map_to_unit(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "create_dir_all".to_string(),
        ])),
        args: vec![ref_arg(args, 0)],
    })))
}

pub(crate) fn lower_rmdir(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(map_to_unit(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "remove_dir".to_string(),
        ])),
        args: vec![ref_arg(args, 0)],
    })))
}

pub(crate) fn lower_remove_file(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(map_to_unit(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "remove_file".to_string(),
        ])),
        args: vec![ref_arg(args, 0)],
    })))
}

pub(crate) fn lower_rename(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(io_map_err(map_to_unit(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "rename".to_string(),
        ])),
        args: vec![ref_arg(args, 0), ref_arg(args, 1)],
    })))
}

pub(crate) fn lower_is_file(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(path_new(ref_arg(args, 0))),
        method: "is_file".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_is_dir(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(path_new(ref_arg(args, 0))),
        method: "is_dir".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_copy_file(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(io_map_err(map_to_unit(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "copy".to_string(),
        ])),
        args: vec![ref_arg(args, 0), ref_arg(args, 1)],
    })))
}

pub(crate) fn lower_rmdir_all(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(io_map_err(map_to_unit(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "fs".to_string(),
            "remove_dir_all".to_string(),
        ])),
        args: vec![ref_arg(args, 0)],
    })))
}

pub(crate) fn lower_gettempdir(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "env".to_string(),
                    "temp_dir".to_string(),
                ])),
                args: vec![],
            }),
            method: "display".to_string(),
            args: vec![],
        }),
        method: "to_string".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_makedirs(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    lower_mkdir(args)
}
