//! JSON intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_json_loads(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "serde_json::from_str::<serde_json::Value>(({}).as_ref()).map(|v| v.to_string()).map_err(|e| JSONDecodeError {{ message: e.to_string(), line: e.line() as i64, column: e.column() as i64 }})",
        args[0]
    )))
}

pub(super) fn lower_json_dumps(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "serde_json".to_string(),
                "to_string".to_string(),
            ])),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Ident(args[0].clone())),
            }],
        }),
        method: "unwrap_or_default".to_string(),
        args: vec![],
    })
}
