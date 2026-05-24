//! Zipfile intrinsic lowerers for registry lowering.

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

fn ref_ident(name: &str) -> RustExpr {
    ref_expr(RustExpr::Ident(name.to_string()))
}

fn io_map_err(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    }
}

fn zip_map_err(expr: RustExpr) -> RustExpr {
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

pub(crate) fn lower_zip_create(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__f".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fs".to_string(),
                        "File".to_string(),
                        "create".to_string(),
                    ])),
                    args: vec![ref_arg(args, 0)],
                }))),
            },
            RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["drop".to_string()])),
                args: vec![RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "zip".to_string(),
                        "ZipWriter".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__f".to_string())],
                }],
            }),
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Literal(RustLiteral::Unit)))),
    })
}

pub(crate) fn lower_zip_add_file(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let open_file = RustExpr::MethodCall {
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
                method: "read".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Bool(true))],
            }),
            method: "write".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Bool(true))],
        }),
        method: "open".to_string(),
        args: vec![ref_ident("__path")],
    };

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__path".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(arg_expr(args, 0))),
            },
            RustStmt::Let {
                mutable: false,
                name: "__name".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__content".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
            RustStmt::Let {
                mutable: false,
                name: "__f".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(open_file))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__zip".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(zip_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "zip".to_string(),
                        "ZipWriter".to_string(),
                        "new_append".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__f".to_string())],
                }))),
            },
            RustStmt::Let {
                mutable: false,
                name: "__opts".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "zip".to_string(),
                        "write".to_string(),
                        "SimpleFileOptions".to_string(),
                        "default".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::Try(Box::new(zip_map_err(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__zip".to_string())),
                method: "start_file".to_string(),
                args: vec![
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__name".to_string())),
                        method: "to_string".to_string(),
                        args: vec![],
                    },
                    RustExpr::Ident("__opts".to_string()),
                ],
            })))),
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
                        expr: Box::new(RustExpr::Ident("__zip".to_string())),
                    },
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__content".to_string())),
                        method: "as_bytes".to_string(),
                        args: vec![],
                    },
                ],
            })))),
            RustStmt::Expr(RustExpr::Try(Box::new(zip_map_err(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__zip".to_string())),
                method: "finish".to_string(),
                args: vec![],
            })))),
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Literal(RustLiteral::Unit)))),
    })
}

pub(crate) fn lower_zip_add_file_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let open_file = RustExpr::MethodCall {
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
                method: "read".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Bool(true))],
            }),
            method: "write".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Bool(true))],
        }),
        method: "open".to_string(),
        args: vec![ref_ident("__path")],
    };

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__path".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(arg_expr(args, 0))),
            },
            RustStmt::Let {
                mutable: false,
                name: "__name".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__content".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
            RustStmt::Let {
                mutable: false,
                name: "__f".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(open_file))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__zip".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(zip_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "zip".to_string(),
                        "ZipWriter".to_string(),
                        "new_append".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__f".to_string())],
                }))),
            },
            RustStmt::Let {
                mutable: false,
                name: "__opts".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "zip".to_string(),
                        "write".to_string(),
                        "SimpleFileOptions".to_string(),
                        "default".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::Try(Box::new(zip_map_err(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__zip".to_string())),
                method: "start_file".to_string(),
                args: vec![
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__name".to_string())),
                        method: "to_string".to_string(),
                        args: vec![],
                    },
                    RustExpr::Ident("__opts".to_string()),
                ],
            })))),
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
                        expr: Box::new(RustExpr::Ident("__zip".to_string())),
                    },
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__content".to_string())),
                    },
                ],
            })))),
            RustStmt::Expr(RustExpr::Try(Box::new(zip_map_err(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__zip".to_string())),
                method: "finish".to_string(),
                args: vec![],
            })))),
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Literal(RustLiteral::Unit)))),
    })
}

pub(crate) fn lower_zip_read_file(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__f".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fs".to_string(),
                        "File".to_string(),
                        "open".to_string(),
                    ])),
                    args: vec![ref_arg(args, 0)],
                }))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__zip".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(zip_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "zip".to_string(),
                        "ZipArchive".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__f".to_string())],
                }))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__file".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(zip_map_err(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__zip".to_string())),
                    method: "by_name".to_string(),
                    args: vec![ref_arg(args, 1)],
                }))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__content".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "io".to_string(),
                    "Read".to_string(),
                    "read_to_string".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("__file".to_string())),
                    },
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("__content".to_string())),
                    },
                ],
            })))),
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Ident("__content".to_string())))),
    })
}

pub(crate) fn lower_zip_read_file_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__f".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fs".to_string(),
                        "File".to_string(),
                        "open".to_string(),
                    ])),
                    args: vec![ref_arg(args, 0)],
                }))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__zip".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(zip_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "zip".to_string(),
                        "ZipArchive".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__f".to_string())],
                }))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__file".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(zip_map_err(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__zip".to_string())),
                    method: "by_name".to_string(),
                    args: vec![ref_arg(args, 1)],
                }))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__content".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "io".to_string(),
                    "Read".to_string(),
                    "read_to_end".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("__file".to_string())),
                    },
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident("__content".to_string())),
                    },
                ],
            })))),
        ],
        expr: Some(Box::new(ok_expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__content".to_string())),
            method: "to_vec".to_string(),
            args: vec![],
        }))),
    })
}

pub(crate) fn lower_zip_namelist(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__f".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(io_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fs".to_string(),
                        "File".to_string(),
                        "open".to_string(),
                    ])),
                    args: vec![ref_arg(args, 0)],
                }))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__zip".to_string(),
                ty: None,
                value: RustExpr::Try(Box::new(zip_map_err(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "zip".to_string(),
                        "ZipArchive".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("__f".to_string())],
                }))),
            },
            RustStmt::Let {
                mutable: true,
                name: "__names".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "__i".to_string(),
                iter: RustExpr::Range {
                    start: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    end: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__zip".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                },
                body: vec![RustStmt::IfLet {
                    pattern: "Ok(__file)".to_string(),
                    expr: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__zip".to_string())),
                        method: "by_index".to_string(),
                        args: vec![RustExpr::Ident("__i".to_string())],
                    },
                    then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__names".to_string())),
                        method: "push".to_string(),
                        args: vec![RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__file".to_string())),
                                method: "name".to_string(),
                                args: vec![],
                            }),
                            method: "to_string".to_string(),
                            args: vec![],
                        }],
                    })],
                    else_body: None,
                }],
            },
        ],
        expr: Some(Box::new(ok_expr(RustExpr::Ident("__names".to_string())))),
    })
}
