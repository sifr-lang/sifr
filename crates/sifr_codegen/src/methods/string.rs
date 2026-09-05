use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

use super::common::exact_int_to_bound_expr;

mod search;

fn replacement_or_split_limit(object: &RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "len".to_string(),
            args: vec![],
        }),
        method: "saturating_add".to_string(),
        args: vec![RustExpr::Verbatim("1usize".to_string())],
    }
}

fn bind_nontrivial_string_receiver_once(
    object: &RustExpr,
    args: &[RustExpr],
    expected_arg_count: usize,
    lower: fn(&RustExpr, &[RustExpr]) -> Option<RustExpr>,
) -> Option<RustExpr> {
    if args.len() != expected_arg_count || matches!(object, RustExpr::Ident(_)) {
        return None;
    }
    let binding = "__sifr_string_receiver".to_string();
    let lowered = lower(&RustExpr::Ident(binding.clone()), args)?;
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: binding,
            ty: None,
            value: RustExpr::Ref {
                mutable: false,
                expr: Box::new(object.clone()),
            },
        }],
        expr: Some(Box::new(lowered)),
    })
}

fn lower_zero_arg_method(object: &RustExpr, args: &[RustExpr], method: &str) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: method.to_string(),
        args: vec![],
    })
}

fn lower_trim_to_string(
    object: &RustExpr,
    args: &[RustExpr],
    trim_method: &str,
) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: trim_method.to_string(),
            args: vec![],
        }),
        method: "to_string".to_string(),
        args: vec![],
    })
}

fn is_already_borrowed_rendered_expr(arg: &RustExpr) -> bool {
    match arg {
        RustExpr::Ref { .. } => true,
        RustExpr::MethodCall { method, .. } => method == "as_str",
        RustExpr::Paren(inner)
        | RustExpr::Try(inner)
        | RustExpr::Await(inner)
        | RustExpr::Clone(inner) => is_already_borrowed_rendered_expr(inner),
        _ => false,
    }
}

fn render_borrowed_arg_expr(arg: &RustExpr) -> RustExpr {
    match arg {
        RustExpr::Literal(RustLiteral::Str(value)) => {
            RustExpr::Verbatim(format!("\"{}\"", value.escape_default()))
        }
        RustExpr::Ref { .. } => arg.clone(),
        _ if is_already_borrowed_rendered_expr(arg) => arg.clone(),
        _ => RustExpr::Ref {
            mutable: false,
            expr: Box::new(arg.clone()),
        },
    }
}

fn literal_single_char(arg: &RustExpr) -> Option<char> {
    let RustExpr::Literal(RustLiteral::Str(value)) = arg else {
        return None;
    };
    let mut chars = value.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(ch)
}

fn string_pattern_arg(arg: &RustExpr) -> RustExpr {
    literal_single_char(arg).map_or_else(
        || render_borrowed_arg_expr(arg),
        |ch| RustExpr::Literal(RustLiteral::Char(ch)),
    )
}

fn is_none_expr(expr: &RustExpr) -> bool {
    matches!(expr, RustExpr::Path(path) if path.len() == 1 && path[0] == "None")
}

fn lower_non_empty_char_all(
    object: &RustExpr,
    args: &[RustExpr],
    char_predicate_method: &str,
) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "is_empty".to_string(),
                args: vec![],
            }),
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "all".to_string(),
            args: vec![char_predicate(char_predicate_method)],
        }),
    })
}

fn char_predicate(method: &str) -> RustExpr {
    if method == "is_ascii_digit" {
        return char_predicate_closure(method);
    }
    RustExpr::Path(vec!["char".to_string(), method.to_string()])
}

fn char_predicate_closure(method: &str) -> RustExpr {
    RustExpr::Closure {
        params: vec![RustParam::Named {
            name: "c".to_string(),
            ty: RustType::Named("_".to_string()),
        }],
        body: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("c".to_string())),
            method: method.to_string(),
            args: vec![],
        }),
        is_move: false,
    }
}

fn lower_has_alpha_and_filtered_all(
    object: &RustExpr,
    args: &[RustExpr],
    alpha_case_method: &str,
) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "any".to_string(),
            args: vec![char_predicate("is_alphabetic")],
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "filter".to_string(),
                args: vec![char_predicate_closure("is_alphabetic")],
            }),
            method: "all".to_string(),
            args: vec![char_predicate(alpha_case_method)],
        }),
    })
}

pub(super) fn lower_upper(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_zero_arg_method(object, args, "to_uppercase")
}

pub(super) fn lower_lower(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_zero_arg_method(object, args, "to_lowercase")
}

pub(super) fn lower_strip(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim")
}

pub(super) fn lower_startswith(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "starts_with".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_endswith(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "ends_with".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_split(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if let Some(lowered) = bind_nontrivial_string_receiver_once(object, args, 2, lower_split) {
        return Some(lowered);
    }
    match args.len() {
        0 => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "split_whitespace".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![to_string_method_path()],
            }),
            method: "collect::<Vec<String>>".to_string(),
            args: vec![],
        }),
        1 => Some(if is_none_expr(&args[0]) {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(object.clone()),
                        method: "split_whitespace".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![to_string_method_path()],
                }),
                method: "collect::<Vec<String>>".to_string(),
                args: vec![],
            }
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(object.clone()),
                        method: "split".to_string(),
                        args: vec![string_pattern_arg(&args[0])],
                    }),
                    method: "map".to_string(),
                    args: vec![to_string_method_path()],
                }),
                method: "collect::<Vec<String>>".to_string(),
                args: vec![],
            }
        }),
        2 => {
            let maxsplit = args[1].clone();
            Some(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(maxsplit.clone()),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                }),
                then_expr: Box::new(lower_split(object, &args[..1])?),
                else_expr: Some(Box::new(if is_none_expr(&args[0]) {
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(object.clone()),
                                    method: "splitn".to_string(),
                                    args: vec![
                                        exact_int_to_bound_expr(
                                            RustExpr::Paren(Box::new(RustExpr::BinOp {
                                                left: Box::new(maxsplit),
                                                op: "+".to_string(),
                                                right: Box::new(RustExpr::FnCall {
                                                    func: Box::new(RustExpr::Path(vec![
                                                        "SifrInt".to_string(),
                                                        "from_i64".to_string(),
                                                    ])),
                                                    args: vec![RustExpr::Literal(
                                                        RustLiteral::Int(1),
                                                    )],
                                                }),
                                            })),
                                            replacement_or_split_limit(object),
                                        ),
                                        RustExpr::Closure {
                                            params: vec![RustParam::Named {
                                                name: "c".to_string(),
                                                ty: RustType::Named("_".to_string()),
                                            }],
                                            body: Box::new(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "c".to_string(),
                                                )),
                                                method: "is_whitespace".to_string(),
                                                args: vec![],
                                            }),
                                            is_move: false,
                                        },
                                    ],
                                }),
                                method: "filter".to_string(),
                                args: vec![RustExpr::Closure {
                                    params: vec![RustParam::Named {
                                        name: "s".to_string(),
                                        ty: RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(RustExpr::UnaryOp {
                                        op: "!".to_string(),
                                        operand: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("s".to_string())),
                                            method: "is_empty".to_string(),
                                            args: vec![],
                                        }),
                                    }),
                                    is_move: false,
                                }],
                            }),
                            method: "map".to_string(),
                            args: vec![to_string_method_path()],
                        }),
                        method: "collect::<Vec<String>>".to_string(),
                        args: vec![],
                    }
                } else {
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(object.clone()),
                                method: "splitn".to_string(),
                                args: vec![
                                    exact_int_to_bound_expr(
                                        RustExpr::Paren(Box::new(RustExpr::BinOp {
                                            left: Box::new(args[1].clone()),
                                            op: "+".to_string(),
                                            right: Box::new(RustExpr::FnCall {
                                                func: Box::new(RustExpr::Path(vec![
                                                    "SifrInt".to_string(),
                                                    "from_i64".to_string(),
                                                ])),
                                                args: vec![RustExpr::Literal(RustLiteral::Int(1))],
                                            }),
                                        })),
                                        replacement_or_split_limit(object),
                                    ),
                                    string_pattern_arg(&args[0]),
                                ],
                            }),
                            method: "map".to_string(),
                            args: vec![to_string_method_path()],
                        }),
                        method: "collect::<Vec<String>>".to_string(),
                        args: vec![],
                    }
                })),
            })
        }
        _ => None,
    }
}

fn to_string_method_path() -> RustExpr {
    RustExpr::Path(
        ["std", "string", "ToString", "to_string"]
            .into_iter()
            .map(str::to_string)
            .collect(),
    )
}

pub(super) fn lower_replace(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if let Some(lowered) = bind_nontrivial_string_receiver_once(object, args, 3, lower_replace) {
        return Some(lowered);
    }
    match args {
        [old, new] => Some(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "replace".to_string(),
            args: vec![string_pattern_arg(old), render_borrowed_arg_expr(new)],
        }),
        [old, new, count] => Some(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(count.clone()),
                op: "<".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
            }),
            then_expr: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "replace".to_string(),
                args: vec![string_pattern_arg(old), render_borrowed_arg_expr(new)],
            }),
            else_expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "replacen".to_string(),
                args: vec![
                    string_pattern_arg(old),
                    render_borrowed_arg_expr(new),
                    exact_int_to_bound_expr(count.clone(), replacement_or_split_limit(object)),
                ],
            })),
        }),
        _ => None,
    }
}

pub(super) fn lower_find(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    search::lower_find_like(object, args, "find")
}

pub(super) fn lower_rfind(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    search::lower_find_like(object, args, "rfind")
}

pub(super) fn lower_lstrip(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim_start")
}

pub(super) fn lower_rstrip(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim_end")
}

pub(super) fn lower_count(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "SifrInt".to_string(),
            "from".to_string(),
        ])),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "matches".to_string(),
                args: vec![render_borrowed_arg_expr(&args[0])],
            }),
            method: "count".to_string(),
            args: vec![],
        }],
    })
}

pub(super) fn lower_join(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(args[0].clone()),
        method: "join".to_string(),
        args: vec![render_borrowed_arg_expr(object)],
    })
}

pub(super) fn lower_title(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "split_whitespace".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "w".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Block {
                        stmts: vec![RustStmt::Let {
                            mutable: true,
                            name: "c".to_string(),
                            ty: None,
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("w".to_string())),
                                method: "chars".to_string(),
                                args: vec![],
                            },
                        }],
                        expr: Some(Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("c".to_string())),
                                    method: "next".to_string(),
                                    args: vec![],
                                }),
                                method: "map".to_string(),
                                args: vec![RustExpr::Closure {
                                    params: vec![RustParam::Named {
                                        name: "f".to_string(),
                                        ty: RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "f".to_string(),
                                                )),
                                                method: "to_uppercase".to_string(),
                                                args: vec![],
                                            }),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        }),
                                        op: "+".to_string(),
                                        right: Box::new(RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::MethodCall {
                                                    receiver: Box::new(RustExpr::Ident(
                                                        "c".to_string(),
                                                    )),
                                                    method: "as_str".to_string(),
                                                    args: vec![],
                                                }),
                                                method: "to_lowercase".to_string(),
                                                args: vec![],
                                            }),
                                        }),
                                    }),
                                    is_move: false,
                                }],
                            }),
                            method: "unwrap_or_default".to_string(),
                            args: vec![],
                        })),
                    }),
                    is_move: false,
                }],
            }),
            method: "collect::<Vec<_>>".to_string(),
            args: vec![],
        }),
        method: "join".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::StaticStr(" ".to_string()))],
    })
}

pub(super) fn lower_capitalize(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "_s".to_string(),
                ty: None,
                value: RustExpr::Clone(Box::new(object.clone())),
            },
            RustStmt::Let {
                mutable: true,
                name: "_c".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("_s".to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("_c".to_string())),
                    method: "next".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "f".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("f".to_string())),
                                method: "to_uppercase".to_string(),
                                args: vec![],
                            }),
                            method: "to_string".to_string(),
                            args: vec![],
                        }),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("_c".to_string())),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }),
                                method: "to_lowercase".to_string(),
                                args: vec![],
                            }),
                        }),
                    }),
                    is_move: false,
                }],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_swapcase(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "c".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::If {
                    cond: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("c".to_string())),
                        method: "is_uppercase".to_string(),
                        args: vec![],
                    }),
                    then_expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("c".to_string())),
                            method: "to_lowercase".to_string(),
                            args: vec![],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    else_expr: Some(Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("c".to_string())),
                            method: "to_uppercase".to_string(),
                            args: vec![],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    })),
                }),
                is_move: false,
            }],
        }),
        method: "collect::<String>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_isdigit(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_non_empty_char_all(object, args, "is_ascii_digit")
}

pub(super) fn lower_isalpha(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_non_empty_char_all(object, args, "is_alphabetic")
}

pub(super) fn lower_isalnum(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_non_empty_char_all(object, args, "is_alphanumeric")
}

pub(super) fn lower_isspace(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_non_empty_char_all(object, args, "is_whitespace")
}

pub(super) fn lower_isupper(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_has_alpha_and_filtered_all(object, args, "is_uppercase")
}

pub(super) fn lower_islower(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_has_alpha_and_filtered_all(object, args, "is_lowercase")
}

pub(super) fn lower_center(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_checked_padding(object, args, "checked_center")
}

pub(super) fn lower_ljust(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_checked_padding(object, args, "checked_ljust")
}

pub(super) fn lower_rjust(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_checked_padding(object, args, "checked_rjust")
}

pub(super) fn lower_zfill(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_checked_padding(object, args, "checked_zfill")
}

fn lower_checked_padding(object: &RustExpr, args: &[RustExpr], helper: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_runtime".to_string(),
                helper.to_string(),
            ])),
            args: vec![
                render_borrowed_arg_expr(object),
                render_borrowed_arg_expr(&args[0]),
            ],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__padding_error".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "OverflowError".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Ident("__padding_error".to_string())],
            }),
            is_move: false,
        }],
    })
}
