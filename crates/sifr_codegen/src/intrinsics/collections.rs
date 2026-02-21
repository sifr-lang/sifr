//! Collections intrinsic lowerers for registry migration.

use crate::{RustExpr, RustParam, RustStmt, RustType};

fn cloned_vec(expr: &str) -> String {
    format!("({expr}).clone()")
}

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_new_set(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "Vec::<i64>".to_string(),
            "new".to_string(),
        ])),
        args: vec![],
    })
}

pub(super) fn lower_set_from_list(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "s".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(format!("({})", args[0])))),
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("s".to_string())),
                method: "sort".to_string(),
                args: vec![],
            }),
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("s".to_string())),
                method: "dedup".to_string(),
                args: vec![],
            }),
        ],
        expr: Some(Box::new(RustExpr::Ident("s".to_string()))),
    })
}

pub(super) fn lower_set_add(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "s".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(format!("({})", args[0])))),
            },
            RustStmt::Let {
                mutable: false,
                name: "v".to_string(),
                ty: None,
                value: RustExpr::Ident(args[1].clone()),
            },
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("s".to_string())),
                        method: "contains".to_string(),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident("v".to_string())),
                        }],
                    }),
                },
                then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("s".to_string())),
                    method: "push".to_string(),
                    args: vec![RustExpr::Ident("v".to_string())],
                })],
                else_body: None,
            },
        ],
        expr: Some(Box::new(RustExpr::Ident("s".to_string()))),
    })
}

pub(super) fn lower_set_contains(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(args[0].clone())),
        method: "contains".to_string(),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Ident(args[1].clone())),
        }],
    })
}

pub(super) fn lower_set_remove(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "s".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(format!("({})", args[0])))),
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("s".to_string())),
                method: "retain".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "x".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                            "x".to_string(),
                        )))),
                        op: "!=".to_string(),
                        right: Box::new(RustExpr::Ident(args[1].clone())),
                    }),
                    is_move: false,
                }],
            }),
        ],
        expr: Some(Box::new(RustExpr::Ident("s".to_string()))),
    })
}

pub(super) fn lower_set_len(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(args[0].clone())),
            method: "len".to_string(),
            args: vec![],
        }),
        ty: crate::RustType::I64,
    })
}

pub(super) fn lower_set_union(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "s".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(format!("({})", args[0])))),
            },
            RustStmt::For {
                var: "v".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(format!("({})", args[1]))),
                    method: "iter".to_string(),
                    args: vec![],
                },
                body: vec![RustStmt::If {
                    cond: RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("s".to_string())),
                            method: "contains".to_string(),
                            args: vec![RustExpr::Ident("v".to_string())],
                        }),
                    },
                    then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("s".to_string())),
                        method: "push".to_string(),
                        args: vec![RustExpr::Deref(Box::new(RustExpr::Ident("v".to_string())))],
                    })],
                    else_body: None,
                }],
            },
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("s".to_string())),
                method: "sort".to_string(),
                args: vec![],
            }),
        ],
        expr: Some(Box::new(RustExpr::Ident("s".to_string()))),
    })
}

pub(super) fn lower_set_intersection(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__a".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(format!("({})", args[0])))),
            },
            RustStmt::Let {
                mutable: false,
                name: "__b".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(format!("({})", args[1])))),
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__a".to_string())),
                        method: "iter".to_string(),
                        args: vec![],
                    }),
                    method: "filter".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "x".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__b".to_string())),
                            method: "contains".to_string(),
                            args: vec![RustExpr::Ident("x".to_string())],
                        }),
                        is_move: false,
                    }],
                }),
                method: "cloned".to_string(),
                args: vec![],
            }),
            method: "collect::<Vec<i64>>".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_counter_from_list(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: "counts".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "collections".to_string(),
                        "HashMap::<String, i64>".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::For {
                var: "item".to_string(),
                iter: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(format!("({})", args[0]))),
                    method: "iter".to_string(),
                    args: vec![],
                },
                body: vec![RustStmt::AugAssign {
                    target: RustExpr::Deref(Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("counts".to_string())),
                            method: "entry".to_string(),
                            args: vec![RustExpr::Clone(Box::new(RustExpr::Ident(
                                "item".to_string(),
                            )))],
                        }),
                        method: "or_insert".to_string(),
                        args: vec![RustExpr::Literal(crate::RustLiteral::Int(0))],
                    })),
                    op: "+".to_string(),
                    value: RustExpr::Literal(crate::RustLiteral::Int(1)),
                }],
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
                    expr: Box::new(RustExpr::Ident("counts".to_string())),
                }],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_counter_get(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); let __key = {}; *data.get(__key.as_str()).unwrap_or(&0) }}",
        borrowed_str(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_counter_most_common(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); let mut pairs: Vec<(String, i64)> = data.into_iter().collect(); pairs.sort_by(|a, b| b.1.cmp(&a.1)); pairs.truncate({} as usize); let items: Vec<String> = pairs.iter().map(|(k, v)| format!(\"[\\\"{{}}\\\",{{}}]\", k, v)).collect(); format!(\"[{{}}]\", items.join(\",\")) }}",
        borrowed_str(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_counter_total(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
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
                        expr: Box::new(RustExpr::Ident(format!("({})", args[0]))),
                    }],
                }),
                method: "unwrap_or_default".to_string(),
                args: vec![],
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("data".to_string())),
                method: "values".to_string(),
                args: vec![],
            }),
            method: "sum::<i64>".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_counter_values(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
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
                        expr: Box::new(RustExpr::Ident(format!("({})", args[0]))),
                    }],
                }),
                method: "unwrap_or_default".to_string(),
                args: vec![],
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("data".to_string())),
                    method: "values".to_string(),
                    args: vec![],
                }),
                method: "cloned".to_string(),
                args: vec![],
            }),
            method: "collect::<Vec<i64>>".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_counter_keys(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); data.keys().cloned().collect::<Vec<String>>() }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_counter_items(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); let mut pairs: Vec<(String, i64)> = data.into_iter().collect(); pairs.sort_by(|a, b| a.0.cmp(&b.0)); let items: Vec<String> = pairs.iter().map(|(k, v)| format!(\"[\\\"{{}}\\\",{{}}]\", k, v)).collect(); format!(\"[{{}}]\", items.join(\",\")) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_counter_increment(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); *data.entry({}.to_string()).or_insert(0) += 1; serde_json::to_string(&data).unwrap_or_default() }}",
        borrowed_str(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_defaultdict_new(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "format!(\"{{\\\"__default__\\\":{{}}}}\", {})",
        args[0]
    )))
}

pub(super) fn lower_defaultdict_get(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); let def = data.get(\"__default__\").cloned().unwrap_or(0); *data.get({}).unwrap_or(&def) }}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_defaultdict_set(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut data: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str({}).unwrap_or_default(); data.insert({}.to_string(), serde_json::json!({})); serde_json::to_string(&data).unwrap_or_default() }}",
        borrowed_str(&args[0]),
        args[1],
        args[2]
    )))
}
