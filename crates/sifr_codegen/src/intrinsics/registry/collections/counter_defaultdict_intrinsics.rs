use super::{arg_expr, RustExpr, RustStmt, RustType};
pub(crate) fn lower_counter_increment(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__counter_json".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__key".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: true,
                name: "data".to_string(),
                ty: Some(RustType::Named(
                    "std::collections::HashMap<String, i64>".to_string(),
                )),
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "serde_json".to_string(),
                            "from_str".to_string(),
                        ])),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident("__counter_json".to_string())),
                        }],
                    }),
                    method: "unwrap_or_default".to_string(),
                    args: vec![],
                },
            },
            RustStmt::AugAssign {
                target: RustExpr::Deref(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("data".to_string())),
                        method: "entry".to_string(),
                        args: vec![RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__key".to_string())),
                            method: "to_string".to_string(),
                            args: vec![],
                        }],
                    }),
                    method: "or_insert".to_string(),
                    args: vec![RustExpr::Literal(crate::RustLiteral::Int(0))],
                })),
                op: "+".to_string(),
                value: RustExpr::Literal(crate::RustLiteral::Int(1)),
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "serde_json".to_string(),
                    "to_string".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident("data".to_string())),
                }],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        })),
    })
}
