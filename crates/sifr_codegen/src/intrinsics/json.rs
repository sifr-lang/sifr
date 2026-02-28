//! JSON intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustStmt, RustType};

pub(super) fn lower_json_loads(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__json_input".to_string(),
            ty: None,
            value: args[0].clone(),
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "serde_json".to_string(),
                        "from_str::<serde_json::Value>".to_string(),
                    ])),
                    args: vec![RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__json_input".to_string())),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "v".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("v".to_string())),
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
                    name: "JSONDecodeError".to_string(),
                    fields: vec![
                        (
                            "message".to_string(),
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                        ),
                        (
                            "line".to_string(),
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("e".to_string())),
                                    method: "line".to_string(),
                                    args: vec![],
                                }),
                                ty: RustType::I64,
                            },
                        ),
                        (
                            "column".to_string(),
                            RustExpr::Cast {
                                expr: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("e".to_string())),
                                    method: "column".to_string(),
                                    args: vec![],
                                }),
                                ty: RustType::I64,
                            },
                        ),
                    ],
                }),
                is_move: false,
            }],
        })),
    })
}

pub(super) fn lower_json_dumps(args: &[RustExpr]) -> Option<RustExpr> {
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
                expr: Box::new(args[0].clone()),
            }],
        }),
        method: "unwrap_or_default".to_string(),
        args: vec![],
    })
}
