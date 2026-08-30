use super::leaves_and_plain_calls::try_lower_leaf_or_name_expr;
use crate::{RustExpr, RustLiteral, RustStmt, RustType};
use sifr_ir::HirExpr;
use sifr_type_system::Type;

pub(super) fn await_call_needs_convention_aware_lowering(args: &[HirExpr]) -> bool {
    args.iter().any(|arg| {
        !crate::helpers::is_copy_type_for_codegen(arg.ty())
            || matches!(
                arg.ty().resolve_alias(),
                Type::Function(_) | Type::AsyncFunction(_) | Type::AsyncCallable(..)
            )
    })
}

pub(super) fn try_lower_task_sleep_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [duration] = args else {
        return None;
    };
    let duration_expr = try_lower_task_duration_expr(duration, "__sifr_task_sleep_seconds")?;
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "tokio".to_string(),
            "time".to_string(),
            "sleep".to_string(),
        ])),
        args: vec![duration_expr],
    })
}

pub(crate) fn try_lower_task_duration_expr(
    duration: &HirExpr,
    seconds_name: &str,
) -> Option<RustExpr> {
    let seconds = RustExpr::Cast {
        expr: Box::new(try_lower_leaf_or_name_expr(duration)?),
        ty: RustType::F64,
    };
    let seconds_name = seconds_name.to_string();
    let finite_positive = RustExpr::BinOp {
        left: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(seconds_name.clone())),
            method: "is_finite".to_string(),
            args: vec![],
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident(seconds_name.clone())),
            op: ">".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
        }),
    };
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: seconds_name.clone(),
            ty: Some(RustType::F64),
            value: seconds,
        }],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "time".to_string(),
                "Duration".to_string(),
                "from_secs_f64".to_string(),
            ])),
            args: vec![RustExpr::If {
                cond: Box::new(finite_positive),
                then_expr: Box::new(RustExpr::Ident(seconds_name)),
                else_expr: Some(Box::new(RustExpr::Literal(RustLiteral::Float(0.0)))),
            }],
        })),
    })
}
