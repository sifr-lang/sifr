//! UUID intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustStmt, RustType};

fn random_expr(ty: &str) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "rand".to_string(),
            format!("random::<{ty}>"),
        ])),
        args: vec![],
    }
}

pub(crate) fn lower_uuid4(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "seg1".to_string(),
                ty: None,
                value: random_expr("u32"),
            },
            RustStmt::Let {
                mutable: false,
                name: "seg2".to_string(),
                ty: None,
                value: random_expr("u16"),
            },
            RustStmt::Let {
                mutable: false,
                name: "seg3".to_string(),
                ty: None,
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(random_expr("u16")),
                        op: "&".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(4095))),
                    }),
                    op: "|".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(16384))),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "seg4".to_string(),
                ty: None,
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(random_expr("u16")),
                        op: "&".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(16383))),
                    }),
                    op: "|".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(32768))),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "seg5_hi".to_string(),
                ty: None,
                value: random_expr("u32"),
            },
            RustStmt::Let {
                mutable: false,
                name: "seg5_lo".to_string(),
                ty: None,
                value: random_expr("u16"),
            },
            RustStmt::Let {
                mutable: false,
                name: "seg5".to_string(),
                ty: None,
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::Ident("seg5_hi".to_string())),
                            ty: RustType::Named("u64".to_string()),
                        }),
                        op: "<<".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(16))),
                    }),
                    op: "|".to_string(),
                    right: Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("seg5_lo".to_string())),
                        ty: RustType::Named("u64".to_string()),
                    }),
                },
            },
        ],
        expr: Some(Box::new(RustExpr::FormatMacro {
            name: "format".to_string(),
            format_str: "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}".to_string(),
            args: vec![
                RustExpr::Ident("seg1".to_string()),
                RustExpr::Ident("seg2".to_string()),
                RustExpr::Ident("seg3".to_string()),
                RustExpr::Ident("seg4".to_string()),
                RustExpr::Ident("seg5".to_string()),
            ],
        })),
    })
}

fn lower_name_based_uuid(args: &[RustExpr], method_name: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__ns".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "uuid".to_string(),
                            "Uuid".to_string(),
                            "parse_str".to_string(),
                        ])),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(args[0].clone()),
                        }],
                    }),
                    method: "unwrap_or".to_string(),
                    args: vec![RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "uuid".to_string(),
                            "Uuid".to_string(),
                            "nil".to_string(),
                        ])),
                        args: vec![],
                    }],
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__id".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "uuid".to_string(),
                        "Uuid".to_string(),
                        method_name.to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident("__ns".to_string())),
                        },
                        RustExpr::MethodCall {
                            receiver: Box::new(args[1].clone()),
                            method: "as_bytes".to_string(),
                            args: vec![],
                        },
                    ],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__id".to_string())),
                method: "hyphenated".to_string(),
                args: vec![],
            }),
            method: "to_string".to_string(),
            args: vec![],
        })),
    })
}

pub(crate) fn lower_uuid3(args: &[RustExpr]) -> Option<RustExpr> {
    lower_name_based_uuid(args, "new_v3")
}

pub(crate) fn lower_uuid5(args: &[RustExpr]) -> Option<RustExpr> {
    lower_name_based_uuid(args, "new_v5")
}
