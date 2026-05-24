//! Datetime intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

pub(crate) fn lower_datetime_now(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    // chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "chrono".to_string(),
                    "Local".to_string(),
                    "now".to_string(),
                ])),
                args: vec![],
            }),
            method: "format".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Str(
                "%Y-%m-%dT%H:%M:%S".to_string(),
            ))],
        }),
        method: "to_string".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_datetime_now_struct(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__dt".to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "chrono".to_string(),
                    "Local".to_string(),
                    "now".to_string(),
                ])),
                args: vec![],
            },
        }],
        expr: Some(Box::new(RustExpr::Vec(vec![
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "year".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "month".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Datelike".to_string(),
                        "day".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "hour".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "minute".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
            RustExpr::Cast {
                expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "Timelike".to_string(),
                        "second".to_string(),
                    ])),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__dt".to_string())),
                    }],
                }),
                ty: RustType::I64,
            },
        ]))),
    })
}

pub(crate) fn lower_datetime_format(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    // chrono::NaiveDateTime::parse_from_str(&dt_str, &fmt).map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()).map_err(|e| ValueError { message: e.to_string() })
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "NaiveDateTime".to_string(),
                        "parse_from_str".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(args[0].clone()),
                        },
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(args[1].clone()),
                        },
                    ],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "dt".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("dt".to_string())),
                            method: "format".to_string(),
                            args: vec![RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(RustExpr::Literal(RustLiteral::Str(
                                    "%Y-%m-%dT%H:%M:%S".to_string(),
                                ))),
                            }],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                }],
            }),
            method: "map_err".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "e".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::StructInit {
                    name: "ValueError".to_string(),
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
        method: "ok".to_string(),
        args: vec![],
    })
}

pub(crate) fn lower_datetime_from_timestamp(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__ts".to_string(),
            ty: None,
            value: RustExpr::Cast {
                expr: Box::new(args[0].clone()),
                ty: RustType::I64,
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "DateTime".to_string(),
                        "from_timestamp".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("__ts".to_string()),
                        RustExpr::Literal(RustLiteral::Int(0)),
                    ],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "dt".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("dt".to_string())),
                            method: "format".to_string(),
                            args: vec![RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(RustExpr::Literal(RustLiteral::Str(
                                    "%Y-%m-%dT%H:%M:%S".to_string(),
                                ))),
                            }],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                }],
            }),
            method: "ok_or_else".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![],
                body: Box::new(RustExpr::StructInit {
                    name: "ValueError".to_string(),
                    fields: vec![(
                        "message".to_string(),
                        RustExpr::Literal(RustLiteral::Str("invalid timestamp".to_string())),
                    )],
                }),
                is_move: false,
            }],
        })),
    })
}
