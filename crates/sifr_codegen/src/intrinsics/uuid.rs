//! UUID intrinsic lowerers for registry migration.

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

pub(super) fn lower_uuid4(args: &[RustExpr]) -> Option<RustExpr> {
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
