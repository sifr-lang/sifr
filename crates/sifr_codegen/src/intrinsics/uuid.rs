//! UUID intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustStmt, RustType};

fn cast(expr: RustExpr, ty: &str) -> RustExpr {
    RustExpr::Cast {
        expr: Box::new(expr),
        ty: RustType::Named(ty.to_string()),
    }
}

fn gen_call(ty: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("rng".to_string())),
        method: format!("gen::<{ty}>"),
        args: vec![],
    }
}

pub(super) fn lower_uuid4(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "rng".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "rand".to_string(),
                        "thread_rng".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "seg1".to_string(),
                ty: None,
                value: gen_call("u32"),
            },
            RustStmt::Let {
                mutable: false,
                name: "seg2".to_string(),
                ty: None,
                value: gen_call("u16"),
            },
            RustStmt::Let {
                mutable: false,
                name: "seg3".to_string(),
                ty: None,
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(gen_call("u16")),
                        op: "&".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(0x0fff))),
                    }),
                    op: "|".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0x4000))),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "seg4".to_string(),
                ty: None,
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(gen_call("u16")),
                        op: "&".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(0x3fff))),
                    }),
                    op: "|".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0x8000))),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "seg5_hi".to_string(),
                ty: None,
                value: gen_call("u32"),
            },
            RustStmt::Let {
                mutable: false,
                name: "seg5_lo".to_string(),
                ty: None,
                value: gen_call("u16"),
            },
            RustStmt::Let {
                mutable: false,
                name: "seg5".to_string(),
                ty: None,
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(cast(RustExpr::Ident("seg5_hi".to_string()), "u64")),
                        op: "<<".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(16))),
                    }),
                    op: "|".to_string(),
                    right: Box::new(cast(RustExpr::Ident("seg5_lo".to_string()), "u64")),
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
