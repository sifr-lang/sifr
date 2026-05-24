use super::{
    file_handles_lock_expr, RustExpr, RustItem, RustMatchArm, RustParam, RustStmt, RustType,
    Visibility,
};

pub(crate) fn file_handle_read_bytes_method() -> RustItem {
    RustItem::Fn {
        name: "read_bytes".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::Vec(Box::new(RustType::Named("u8".to_string())))),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: file_handles_lock_expr(),
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::BinaryRead(ref mut __r))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "__buf".to_string(),
                                ty: None,
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "Vec::<u8>".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "io".to_string(),
                                        "Read".to_string(),
                                        "read_to_end".to_string(),
                                    ])),
                                    args: vec![
                                        RustExpr::Ident("__r".to_string()),
                                        RustExpr::Ref {
                                            mutable: true,
                                            expr: Box::new(RustExpr::Ident("__buf".to_string())),
                                        },
                                    ],
                                }),
                                method: "map_err".to_string(),
                                args: vec![RustExpr::Ident("__io_err".to_string())],
                            }))),
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Ident("__buf".to_string())],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for binary reading".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

pub(crate) fn file_handle_write_bytes_method() -> RustItem {
    RustItem::Fn {
        name: "write_bytes".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::SelfParam { mutable: false },
            RustParam::Named {
                name: "data".to_string(),
                ty: RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::Vec(Box::new(RustType::Named("u8".to_string())))),
                },
            },
        ],
        ret: Some(RustType::Result(
            Box::new(RustType::Unit),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: file_handles_lock_expr(),
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::BinaryWrite(ref mut __w))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "io".to_string(),
                                        "Write".to_string(),
                                        "write_all".to_string(),
                                    ])),
                                    args: vec![
                                        RustExpr::Ident("__w".to_string()),
                                        RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(RustExpr::Ident("data".to_string())),
                                        },
                                    ],
                                }),
                                method: "map_err".to_string(),
                                args: vec![RustExpr::Ident("__io_err".to_string())],
                            }))),
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for binary writing".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

pub(crate) fn file_handle_readlines_method() -> RustItem {
    RustItem::Fn {
        name: "readlines".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::Vec(Box::new(RustType::String_))),
            Box::new(RustType::Named("IOError".to_string())),
        )),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__hid".to_string(),
                ty: None,
                value: RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "_handle".to_string(),
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__handles".to_string(),
                ty: None,
                value: file_handles_lock_expr(),
            },
            RustStmt::Match {
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__handles".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__hid".to_string())),
                    }],
                },
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(SifrFileHandle::TextRead(ref mut __r))".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "__lines".to_string(),
                                ty: Some(RustType::Vec(Box::new(RustType::String_))),
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "Vec::<String>".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            RustStmt::Let {
                                mutable: true,
                                name: "__line".to_string(),
                                ty: None,
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "String".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            RustStmt::Loop {
                                body: vec![
                                    RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__line".to_string())),
                                        method: "clear".to_string(),
                                        args: vec![],
                                    }),
                                    RustStmt::Let {
                                        mutable: false,
                                        name: "__n".to_string(),
                                        ty: None,
                                        value: RustExpr::Try(Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::FnCall {
                                                func: Box::new(RustExpr::Path(vec![
                                                    "std".to_string(),
                                                    "io".to_string(),
                                                    "BufRead".to_string(),
                                                    "read_line".to_string(),
                                                ])),
                                                args: vec![
                                                    RustExpr::Ident("__r".to_string()),
                                                    RustExpr::Ref {
                                                        mutable: true,
                                                        expr: Box::new(RustExpr::Ident(
                                                            "__line".to_string(),
                                                        )),
                                                    },
                                                ],
                                            }),
                                            method: "map_err".to_string(),
                                            args: vec![RustExpr::Ident("__io_err".to_string())],
                                        })),
                                    },
                                    RustStmt::If {
                                        cond: RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__n".to_string())),
                                            op: "==".to_string(),
                                            right: Box::new(RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        },
                                        then_body: vec![RustStmt::Break],
                                        else_body: None,
                                    },
                                    RustStmt::Let {
                                        mutable: true,
                                        name: "__l".to_string(),
                                        ty: None,
                                        value: RustExpr::Clone(Box::new(RustExpr::Ident(
                                            "__line".to_string(),
                                        ))),
                                    },
                                    RustStmt::If {
                                        cond: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__l".to_string())),
                                            method: "ends_with".to_string(),
                                            args: vec![RustExpr::Literal(
                                                crate::RustLiteral::Char('\n'),
                                            )],
                                        },
                                        then_body: vec![
                                            RustStmt::Expr(RustExpr::MethodCall {
                                                receiver: Box::new(RustExpr::Ident(
                                                    "__l".to_string(),
                                                )),
                                                method: "pop".to_string(),
                                                args: vec![],
                                            }),
                                            RustStmt::If {
                                                cond: RustExpr::MethodCall {
                                                    receiver: Box::new(RustExpr::Ident(
                                                        "__l".to_string(),
                                                    )),
                                                    method: "ends_with".to_string(),
                                                    args: vec![RustExpr::Literal(
                                                        crate::RustLiteral::Char('\r'),
                                                    )],
                                                },
                                                then_body: vec![RustStmt::Expr(
                                                    RustExpr::MethodCall {
                                                        receiver: Box::new(RustExpr::Ident(
                                                            "__l".to_string(),
                                                        )),
                                                        method: "pop".to_string(),
                                                        args: vec![],
                                                    },
                                                )],
                                                else_body: None,
                                            },
                                        ],
                                        else_body: None,
                                    },
                                    RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__lines".to_string())),
                                        method: "push".to_string(),
                                        args: vec![RustExpr::Ident("__l".to_string())],
                                    }),
                                ],
                            },
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Ident("__lines".to_string())],
                            })),
                        ],
                    },
                    RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "IOError".to_string(),
                                fields: vec![
                                    (
                                        "message".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "file not open for reading".to_string(),
                                        )),
                                    ),
                                    (
                                        "kind".to_string(),
                                        RustExpr::Literal(crate::RustLiteral::Str(
                                            "Other".to_string(),
                                        )),
                                    ),
                                ],
                            }],
                        }))],
                    },
                ],
            },
        ],
        is_async: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_error_type_items, build_file_handle_infra_items, build_file_handle_struct_items,
        build_io_error_items, build_logging_items, build_random_module_state_items, render_items,
        sifr_type_to_rust_type, Type,
    };

    fn count_raw_in_type(ty: &RustType) -> usize {
        match ty {
            RustType::Vec(inner)
            | RustType::HashSet(inner)
            | RustType::VecDeque(inner)
            | RustType::Option(inner) => count_raw_in_type(inner),
            RustType::HashMap(k, v) | RustType::Result(k, v) => {
                count_raw_in_type(k) + count_raw_in_type(v)
            }
            RustType::Tuple(items) => items.iter().map(count_raw_in_type).sum(),
            RustType::Ref { inner, .. } => count_raw_in_type(inner),
            RustType::Generic { params, .. } | RustType::Fn { params, .. } => {
                params.iter().map(count_raw_in_type).sum()
            }
            _ => 0,
        }
    }

    fn count_raw_in_expr(expr: &RustExpr) -> usize {
        match expr {
            RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) => 0,
            RustExpr::MethodCall { receiver, args, .. }
            | RustExpr::FnCall {
                func: receiver,
                args,
            } => count_raw_in_expr(receiver) + args.iter().map(count_raw_in_expr).sum::<usize>(),
            RustExpr::MacroCall { args, .. }
            | RustExpr::Vec(args)
            | RustExpr::Tuple(args)
            | RustExpr::Array(args) => args.iter().map(count_raw_in_expr).sum(),
            RustExpr::TimeoutAwait { duration, future } => {
                count_raw_in_expr(duration) + count_raw_in_expr(future)
            }
            RustExpr::FormatMacro { args, .. } => args.iter().map(count_raw_in_expr).sum(),
            RustExpr::BinOp { left, right, .. } => {
                count_raw_in_expr(left) + count_raw_in_expr(right)
            }
            RustExpr::UnaryOp { operand, .. }
            | RustExpr::Deref(operand)
            | RustExpr::Clone(operand)
            | RustExpr::Try(operand)
            | RustExpr::Paren(operand)
            | RustExpr::Await(operand) => count_raw_in_expr(operand),
            RustExpr::Field { expr, .. } => count_raw_in_expr(expr),
            RustExpr::Index { expr, index } => count_raw_in_expr(expr) + count_raw_in_expr(index),
            RustExpr::Slice { expr, start, stop } => {
                count_raw_in_expr(expr)
                    + start.as_ref().map(|s| count_raw_in_expr(s)).unwrap_or(0)
                    + stop.as_ref().map(|s| count_raw_in_expr(s)).unwrap_or(0)
            }
            RustExpr::Ref { expr, .. } => count_raw_in_expr(expr),
            RustExpr::Cast { expr, ty } => count_raw_in_expr(expr) + count_raw_in_type(ty),
            RustExpr::Block { stmts, expr } => {
                stmts.iter().map(count_raw_in_stmt).sum::<usize>()
                    + expr.as_ref().map(|e| count_raw_in_expr(e)).unwrap_or(0)
            }
            RustExpr::If {
                cond,
                then_expr,
                else_expr,
            } => {
                count_raw_in_expr(cond)
                    + count_raw_in_expr(then_expr)
                    + else_expr
                        .as_ref()
                        .map(|e| count_raw_in_expr(e))
                        .unwrap_or(0)
            }
            RustExpr::Match { expr, arms } => {
                count_raw_in_expr(expr)
                    + arms
                        .iter()
                        .map(|a| a.body.iter().map(count_raw_in_stmt).sum::<usize>())
                        .sum::<usize>()
            }
            RustExpr::Closure { body, .. } => count_raw_in_expr(body),
            RustExpr::ClosureBlock { body, .. } => body.iter().map(count_raw_in_stmt).sum(),
            RustExpr::StructInit { fields, .. } => {
                fields.iter().map(|(_, v)| count_raw_in_expr(v)).sum()
            }
            RustExpr::Range { start, end } => count_raw_in_expr(start) + count_raw_in_expr(end),
        }
    }

    fn count_raw_in_stmt(stmt: &RustStmt) -> usize {
        match stmt {
            RustStmt::Let { ty, value, .. } => {
                ty.as_ref().map(count_raw_in_type).unwrap_or(0) + count_raw_in_expr(value)
            }
            RustStmt::LetPattern { value, .. } => count_raw_in_expr(value),
            RustStmt::LetElse {
                value, else_body, ..
            } => count_raw_in_expr(value) + else_body.iter().map(count_raw_in_stmt).sum::<usize>(),
            RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
                count_raw_in_expr(target) + count_raw_in_expr(value)
            }
            RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => count_raw_in_expr(expr),
            RustStmt::Assert { cond, msg } => {
                count_raw_in_expr(cond) + msg.as_ref().map(count_raw_in_expr).unwrap_or(0)
            }
            RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => 0,
            RustStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                count_raw_in_expr(cond)
                    + then_body.iter().map(count_raw_in_stmt).sum::<usize>()
                    + else_body
                        .as_ref()
                        .map(|b| b.iter().map(count_raw_in_stmt).sum::<usize>())
                        .unwrap_or(0)
            }
            RustStmt::IfLet {
                expr,
                then_body,
                else_body,
                ..
            } => {
                count_raw_in_expr(expr)
                    + then_body.iter().map(count_raw_in_stmt).sum::<usize>()
                    + else_body
                        .as_ref()
                        .map(|b| b.iter().map(count_raw_in_stmt).sum::<usize>())
                        .unwrap_or(0)
            }
            RustStmt::Match { expr, arms } => {
                count_raw_in_expr(expr)
                    + arms
                        .iter()
                        .map(|a| a.body.iter().map(count_raw_in_stmt).sum::<usize>())
                        .sum::<usize>()
            }
            RustStmt::For { iter, body, .. } | RustStmt::While { cond: iter, body } => {
                count_raw_in_expr(iter) + body.iter().map(count_raw_in_stmt).sum::<usize>()
            }
            RustStmt::With { items, body } => {
                items
                    .iter()
                    .map(|item| count_raw_in_expr(&item.value))
                    .sum::<usize>()
                    + body.iter().map(count_raw_in_stmt).sum::<usize>()
            }
            RustStmt::Loop { body } | RustStmt::Block(body) => {
                body.iter().map(count_raw_in_stmt).sum()
            }
            RustStmt::LocalFn {
                params, ret, body, ..
            } => {
                params
                    .iter()
                    .map(|p| match p {
                        RustParam::Named { ty, .. } | RustParam::NamedMut { ty, .. } => {
                            count_raw_in_type(ty)
                        }
                        RustParam::SelfParam { .. } | RustParam::SelfValue => 0,
                    })
                    .sum::<usize>()
                    + ret.as_ref().map(count_raw_in_type).unwrap_or(0)
                    + body.iter().map(count_raw_in_stmt).sum::<usize>()
            }
        }
    }

    fn count_raw_in_item(item: &RustItem) -> usize {
        match item {
            RustItem::Struct { fields, .. } => {
                fields.iter().map(|(_, t)| count_raw_in_type(t)).sum()
            }
            RustItem::TupleStruct { inner, .. } => count_raw_in_type(inner),
            RustItem::Enum { variants, .. } => variants
                .iter()
                .map(|v| {
                    v.tuple_fields.iter().map(count_raw_in_type).sum::<usize>()
                        + v.fields
                            .iter()
                            .map(|(_, t)| count_raw_in_type(t))
                            .sum::<usize>()
                        + v.value.as_ref().map(count_raw_in_expr).unwrap_or(0)
                })
                .sum(),
            RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
                methods.iter().map(count_raw_in_item).sum()
            }
            RustItem::Fn {
                params, ret, body, ..
            } => {
                params
                    .iter()
                    .map(|p| match p {
                        RustParam::SelfParam { .. } | RustParam::SelfValue => 0,
                        RustParam::Named { ty, .. } | RustParam::NamedMut { ty, .. } => {
                            count_raw_in_type(ty)
                        }
                    })
                    .sum::<usize>()
                    + ret.as_ref().map(count_raw_in_type).unwrap_or(0)
                    + body.iter().map(count_raw_in_stmt).sum::<usize>()
            }
            RustItem::TraitMethodSig { params, ret, .. } => {
                params
                    .iter()
                    .map(|p| match p {
                        RustParam::SelfParam { .. } | RustParam::SelfValue => 0,
                        RustParam::Named { ty, .. } | RustParam::NamedMut { ty, .. } => {
                            count_raw_in_type(ty)
                        }
                    })
                    .sum::<usize>()
                    + ret.as_ref().map(count_raw_in_type).unwrap_or(0)
            }
            RustItem::TypeAlias { ty, .. } => count_raw_in_type(ty),
            RustItem::Const { ty, value, .. } | RustItem::Static { ty, value, .. } => {
                count_raw_in_type(ty) + count_raw_in_expr(value)
            }
            RustItem::Use(_) | RustItem::UseAlias { .. } | RustItem::Attr(_) => 0,
        }
    }

    #[test]
    fn maps_types_to_structured_rust_types() {
        assert_eq!(sifr_type_to_rust_type(&Type::Int), RustType::I64);
        assert_eq!(
            sifr_type_to_rust_type(&Type::List(Box::new(Type::Str))),
            RustType::Vec(Box::new(RustType::String_))
        );
        assert_eq!(
            sifr_type_to_rust_type(&Type::Union(vec![Type::Int, Type::None])),
            RustType::Option(Box::new(RustType::I64))
        );
    }

    #[test]
    fn error_items_render_expected_shapes() {
        let items = build_error_type_items(
            "RegexError",
            &[("detail".to_string(), RustType::String_)],
            &[(
                "detail".to_string(),
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            )],
        );
        let rendered = render_items(&items);
        assert!(rendered.contains("struct RegexError"));
        assert!(rendered.contains("fn new(message: String) -> Self"));
        assert!(rendered.contains("impl std::error::Error for RegexError"));
    }

    #[test]
    fn file_handle_items_render_core_symbols() {
        let mut items = build_file_handle_infra_items();
        items.extend(build_file_handle_struct_items());
        let rendered = render_items(&items);
        assert!(rendered.contains("enum SifrFileHandle"));
        assert!(rendered.contains("static __SIFR_FILE_HANDLES"));
        assert!(rendered.contains("static __SIFR_NEXT_FILE_HANDLE_ID"));
        assert!(rendered.contains("fn __sifr_next_file_handle_id() -> i64"));
        assert!(rendered.contains("impl FileHandle"));
        assert!(rendered.contains("fn read(&self) -> Result<String, IOError>"));
    }

    #[test]
    fn random_module_state_items_render_core_symbols() {
        let items = build_random_module_state_items();
        let rendered = render_items(&items);
        assert!(rendered.contains("struct __SifrRandomModuleState"));
        assert!(rendered.contains("static __SIFR_RANDOM_MODULE_STATE"));
        assert!(rendered.contains("LazyLock"));
        assert!(rendered.contains("Mutex"));
    }

    #[test]
    fn preamble_structural_count_is_zero() {
        let mut all = build_io_error_items();
        all.extend(build_file_handle_infra_items());
        all.extend(build_file_handle_struct_items());
        all.extend(build_logging_items());
        all.extend(build_random_module_state_items());
        let total_structural_violations: usize = all.iter().map(count_raw_in_item).sum();
        assert_eq!(total_structural_violations, 0);
    }
}
