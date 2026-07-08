//! Time intrinsic lowerers for retained runtime-sensitive registry lowering.

use crate::{RustExpr, RustLiteral, RustStmt, RustType};

fn wall_clock_seconds_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "time".to_string(),
                        "SystemTime".to_string(),
                        "now".to_string(),
                    ])),
                    args: vec![],
                }),
                method: "duration_since".to_string(),
                args: vec![RustExpr::Path(vec![
                    "std".to_string(),
                    "time".to_string(),
                    "UNIX_EPOCH".to_string(),
                ])],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        }),
        method: "as_secs_f64".to_string(),
        args: vec![],
    }
}

pub(crate) fn lower_sleep(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__secs".to_string(),
            ty: None,
            value: args[0].clone(),
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__secs".to_string())),
                    method: "is_finite".to_string(),
                    args: vec![],
                }),
                op: "&&".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__secs".to_string())),
                    op: ">".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                }),
            }),
            then_expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "thread".to_string(),
                    "sleep".to_string(),
                ])),
                args: vec![RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "time".to_string(),
                        "Duration".to_string(),
                        "from_nanos".to_string(),
                    ])),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__secs".to_string())),
                            op: "*".to_string(),
                            right: Box::new(RustExpr::Literal(RustLiteral::Float(1_000_000_000.0))),
                        }),
                        ty: RustType::Named("u64".to_string()),
                    }],
                }],
            }),
            else_expr: Some(Box::new(RustExpr::Literal(RustLiteral::Unit))),
        })),
    })
}

pub(crate) fn lower_monotonic(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(wall_clock_seconds_expr())
}
