use super::leaves_and_plain_calls::{is_allowed_plain_call, try_lower_leaf_or_name_expr};
use super::task_calls::try_lower_task_sleep_call_expr;
use super::{is_reserved_builtin_call_func, resolve_alias_type, try_lower_simple_divmod_call_expr};
use crate::{RustExpr, RustLiteral, RustStmt};
use sifr_ir::HirExpr;
use sifr_type_system::Type;

pub(super) fn try_lower_simple_call_expr(func: &str, args: &[HirExpr]) -> Option<RustExpr> {
    if func == "__sifr_python_omitted_argument" && args.is_empty() {
        return Some(RustExpr::Literal(RustLiteral::None));
    }
    if func == "__sifr_python_present_argument" {
        let [value] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        });
    }
    if func == "__sifr_task_sleep" {
        return try_lower_task_sleep_call_expr(args);
    }
    if args.iter().any(|arg| {
        matches!(
            resolve_alias_type(arg.ty()),
            Type::Class {
                parent_class: Some(_),
                ..
            }
        )
    }) {
        return None;
    }
    if func == "__sifr_task_gather" {
        let [handles] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![try_lower_leaf_or_name_expr(handles)?],
        });
    }
    if func == "__sifr_task_race" {
        let [handles] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![try_lower_leaf_or_name_expr(handles)?],
        });
    }
    if func == "__sifr_task_select" {
        let [first, second] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![
                try_lower_leaf_or_name_expr(first)?,
                try_lower_leaf_or_name_expr(second)?,
            ],
        });
    }
    if func == "__sifr_join_set_new" {
        if !args.is_empty() {
            return None;
        }
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![],
        });
    }
    if func == "__sifr_spawn_blocking_infallible"
        || func == "__sifr_spawn_blocking_result"
        || func == "__sifr_spawn_cpu_infallible"
        || func == "__sifr_spawn_cpu_result"
    {
        let [worker] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![try_lower_leaf_or_name_expr(worker)?],
        });
    }
    if func == "__sifr_parallel_map" || func == "__sifr_parallel_try_map" {
        let [items, worker] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![
                try_lower_leaf_or_name_expr(items)?,
                try_lower_leaf_or_name_expr(worker)?,
            ],
        });
    }
    if func == "__sifr_pool_map" || func == "__sifr_pool_try_map" {
        let [pool, items, worker] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(try_lower_leaf_or_name_expr(pool)?),
                },
                try_lower_leaf_or_name_expr(items)?,
                try_lower_leaf_or_name_expr(worker)?,
            ],
        });
    }
    if func == "anext" {
        let [iterator] = args else {
            return None;
        };
        return Some(RustExpr::MethodCall {
            receiver: Box::new(try_lower_leaf_or_name_expr(iterator)?),
            method: "anext".to_string(),
            args: vec![],
        });
    }
    if func == "hash" {
        return try_lower_simple_hash_call_expr(args);
    }
    if func == "divmod" {
        return try_lower_simple_divmod_call_expr(args);
    }
    if is_reserved_builtin_call_func(func) {
        return None;
    }
    if func.contains("::") || !is_allowed_plain_call(func) {
        return None;
    }
    if args
        .iter()
        .any(|arg| matches!(resolve_alias_type(arg.ty()), Type::Result(_, _)))
    {
        return None;
    }

    let lowered_args = args
        .iter()
        .map(|arg| {
            let lowered = try_lower_leaf_or_name_expr(arg)?;
            Some(crate::RustEmitter::clone_non_copy_name_expr_for_ir(
                arg, lowered,
            ))
        })
        .collect::<Option<Vec<_>>>()?;

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Ident(func.to_string())),
        args: lowered_args,
    })
}

fn try_lower_simple_hash_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [arg] = args else {
        return None;
    };
    let lowered_arg = try_lower_leaf_or_name_expr(arg)?;
    let hash_value = match resolve_alias_type(arg.ty()) {
        Type::Int | Type::LiteralInt(_) => RustExpr::MethodCall {
            receiver: Box::new(lowered_arg),
            method: "normalized_hash_key".to_string(),
            args: vec![],
        },
        Type::FixedInt(_) => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "SifrInt".to_string(),
                    "from".to_string(),
                ])),
                args: vec![lowered_arg],
            }),
            method: "normalized_hash_key".to_string(),
            args: vec![],
        },
        _ => lowered_arg,
    };
    let hasher_ident = "__sifr_hash".to_string();

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: hasher_ident.clone(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "collections".to_string(),
                        "hash_map".to_string(),
                        "DefaultHasher".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "hash".to_string(),
                    "Hash".to_string(),
                    "hash".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(hash_value),
                    },
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident(hasher_ident.clone())),
                    },
                ],
            }),
        ],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "SifrInt".to_string(),
                "from".to_string(),
            ])),
            args: vec![RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "hash".to_string(),
                    "Hasher".to_string(),
                    "finish".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident(hasher_ident)),
                }],
            }],
        })),
    })
}
