//! Logging state intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustStmt};

fn global_log_level_lock_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__SIFR_GLOBAL_LOG_LEVEL".to_string())),
            method: "lock".to_string(),
            args: vec![],
        }),
        method: "unwrap_or_else".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![crate::RustParam::Named {
                name: "__err".to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__err".to_string())),
                method: "into_inner".to_string(),
                args: vec![],
            }),
            is_move: false,
        }],
    }
}

pub(crate) fn lower_set_global_level(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Assign {
            target: RustExpr::Deref(Box::new(global_log_level_lock_expr())),
            value: args[0].clone(),
        }],
        expr: Some(Box::new(RustExpr::Literal(RustLiteral::Unit))),
    })
}

pub(crate) fn lower_get_global_level(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Deref(Box::new(global_log_level_lock_expr())))
}
