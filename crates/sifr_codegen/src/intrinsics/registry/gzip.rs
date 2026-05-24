//! `GZip` intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn ref_expr(expr: RustExpr, mutable: bool) -> RustExpr {
    RustExpr::Ref {
        mutable,
        expr: Box::new(expr),
    }
}

pub(crate) fn lower_gzip_compress(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__data".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(ref_expr(arg_expr(args, 0), false)),
                    method: "as_bytes".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__enc".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "flate2".to_string(),
                        "write".to_string(),
                        "GzEncoder".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "Vec".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        },
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "flate2".to_string(),
                                "Compression".to_string(),
                                "default".to_string(),
                            ])),
                            args: vec![],
                        },
                    ],
                },
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "io".to_string(),
                        "Write".to_string(),
                        "write_all".to_string(),
                    ])),
                    args: vec![
                        ref_expr(RustExpr::Ident("__enc".to_string()), true),
                        RustExpr::Ident("__data".to_string()),
                    ],
                }),
                method: "unwrap_or".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Unit)],
            }),
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__enc".to_string())),
                            method: "finish".to_string(),
                            args: vec![],
                        }),
                        method: "unwrap_or_default".to_string(),
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

pub(crate) fn lower_gzip_decompress(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__bytes".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(arg_expr(args, 0)),
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
                                ty: RustType::Named("u8".to_string()),
                            }),
                            is_move: false,
                        }],
                    }),
                    method: "collect::<Vec<u8>>".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__dec".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "flate2".to_string(),
                        "read".to_string(),
                        "GzDecoder".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__bytes".to_string())),
                        method: "as_slice".to_string(),
                        args: vec![],
                    }],
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
            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "io".to_string(),
                        "Read".to_string(),
                        "read_to_string".to_string(),
                    ])),
                    args: vec![
                        ref_expr(RustExpr::Ident("__dec".to_string()), true),
                        ref_expr(RustExpr::Ident("__out".to_string()), true),
                    ],
                }),
                method: "map_err".to_string(),
                args: vec![RustExpr::Ident("__io_err".to_string())],
            }))),
        ],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![RustExpr::Ident("__out".to_string())],
        })),
    })
}
