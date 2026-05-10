//! IR-backed helpers for generating codegen preamble items.

use crate::{RustExpr, RustItem, RustMatchArm, RustParam, RustStmt, RustType, Type, Visibility};

pub fn sifr_type_to_rust_type(ty: &Type) -> RustType {
    match ty {
        Type::Int | Type::LiteralInt(_) => RustType::I64,
        Type::Float => RustType::F64,
        Type::Bool | Type::LiteralBool(_) => RustType::Bool,
        Type::Str | Type::LiteralStr(_) => RustType::String_,
        Type::Bytes => RustType::Vec(Box::new(RustType::Named("u8".to_string()))),
        Type::None => RustType::Unit,
        Type::List(inner) => RustType::Vec(Box::new(sifr_type_to_rust_type(inner))),
        Type::Dict(key, value) => RustType::HashMap(
            Box::new(sifr_type_to_rust_type(key)),
            Box::new(sifr_type_to_rust_type(value)),
        ),
        Type::Set(inner) => RustType::HashSet(Box::new(sifr_type_to_rust_type(inner))),
        Type::Tuple(items) => RustType::Tuple(items.iter().map(sifr_type_to_rust_type).collect()),
        Type::Result(ok, err) => RustType::Result(
            Box::new(sifr_type_to_rust_type(ok)),
            Box::new(sifr_type_to_rust_type(err)),
        ),
        Type::Task(ok, err) => RustType::Generic {
            base: "__SifrTask".to_string(),
            params: vec![
                sifr_type_to_rust_type(ok),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::BlockingTask(ok, err) => RustType::Generic {
            base: "__SifrBlockingTask".to_string(),
            params: vec![
                sifr_type_to_rust_type(ok),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::TaskResult(ok, err) => RustType::Generic {
            base: "__SifrTaskResult".to_string(),
            params: vec![
                sifr_type_to_rust_type(ok),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::Failure(err) => RustType::Generic {
            base: "__SifrFailure".to_string(),
            params: vec![task_error_type_to_rust_type(err)],
        },
        Type::TimeoutResult(err) => RustType::Generic {
            base: "__SifrTimeoutResult".to_string(),
            params: vec![task_error_type_to_rust_type(err)],
        },
        Type::Select2(first, second) => RustType::Generic {
            base: "__SifrSelect2".to_string(),
            params: vec![
                sifr_type_to_rust_type(first),
                sifr_type_to_rust_type(second),
            ],
        },
        Type::AsyncGenerator(item, err) => RustType::Generic {
            base: "AsyncGenerator".to_string(),
            params: vec![
                sifr_type_to_rust_type(item),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::Union(members) => {
            let non_none: Vec<&Type> = members
                .iter()
                .filter(|m| !matches!(m, Type::None))
                .collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                RustType::Option(Box::new(sifr_type_to_rust_type(non_none[0])))
            } else {
                RustType::Named(ty.rust_type())
            }
        }
        _ => RustType::Named(ty.rust_type()),
    }
}

fn task_error_type_to_rust_type(ty: &Type) -> RustType {
    if matches!(ty.resolve_alias(), Type::Never) {
        RustType::Named("std::convert::Infallible".to_string())
    } else {
        sifr_type_to_rust_type(ty)
    }
}

pub fn build_error_type_items(
    name: &str,
    extra_fields: &[(String, RustType)],
    constructor_defaults: &[(String, RustExpr)],
) -> Vec<RustItem> {
    let mut fields = vec![("message".to_string(), RustType::String_)];
    fields.extend(extra_fields.iter().cloned());

    let mut init_fields = vec![(
        "message".to_string(),
        RustExpr::Ident("message".to_string()),
    )];
    init_fields.extend(constructor_defaults.iter().cloned());

    vec![
        RustItem::Struct {
            name: name.to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields,
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "new".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![RustParam::Named {
                    name: "message".to_string(),
                    ty: RustType::String_,
                }],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                    name: "Self".to_string(),
                    fields: init_fields,
                }))],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: Some("std::fmt::Display".to_string()),
            items: vec![RustItem::Fn {
                name: "fmt".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![
                    RustParam::SelfParam { mutable: false },
                    RustParam::Named {
                        name: "f".to_string(),
                        ty: RustType::Ref {
                            mutable: true,
                            inner: Box::new(RustType::Named("std::fmt::Formatter<'_>".to_string())),
                        },
                    },
                ],
                ret: Some(RustType::Named("std::fmt::Result".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fmt".to_string(),
                        "Display".to_string(),
                        "fmt".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "message".to_string(),
                            }),
                        },
                        RustExpr::Ident("f".to_string()),
                    ],
                }))],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: Some("std::error::Error".to_string()),
            items: vec![],
        },
    ]
}

pub fn build_error_into_error_impl(source_name: &str) -> RustItem {
    RustItem::Impl {
        target: "Error".to_string(),
        type_params: vec![],
        trait_: Some(format!("From<{source_name}>")),
        items: vec![RustItem::Fn {
            name: "from".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "err".to_string(),
                ty: RustType::Named(source_name.to_string()),
            }],
            ret: Some(RustType::Named("Self".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Self".to_string(), "new".to_string()])),
                args: vec![RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("err".to_string())),
                    field: "message".to_string(),
                }],
            }))],
            is_async: false,
        }],
    }
}

pub fn build_failure_type_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "__SifrFailure<E>".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string()],
            fields: vec![
                ("primary".to_string(), RustType::Named("E".to_string())),
                (
                    "secondary".to_string(),
                    RustType::Vec(Box::new(RustType::Named("SecondaryError".to_string()))),
                ),
            ],
        },
        RustItem::Impl {
            target: "__SifrFailure<E>".to_string(),
            type_params: vec![crate::RustTypeParam {
                name: "E".to_string(),
                bounds: vec![],
            }],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "new".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::Named {
                        name: "primary".to_string(),
                        ty: RustType::Named("E".to_string()),
                    }],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            (
                                "primary".to_string(),
                                RustExpr::Ident("primary".to_string()),
                            ),
                            (
                                "secondary".to_string(),
                                RustExpr::Ident("Vec::new()".to_string()),
                            ),
                        ],
                    }))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "map_primary".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![crate::RustTypeParam {
                        name: "F".to_string(),
                        bounds: vec![],
                    }],
                    params: vec![
                        RustParam::SelfValue,
                        RustParam::Named {
                            name: "f".to_string(),
                            ty: RustType::Named("impl FnOnce(E) -> F".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named("__SifrFailure<F>".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "__SifrFailure".to_string(),
                        fields: vec![
                            (
                                "primary".to_string(),
                                RustExpr::Ident("f(self.primary)".to_string()),
                            ),
                            (
                                "secondary".to_string(),
                                RustExpr::Ident("self.secondary".to_string()),
                            ),
                        ],
                    }))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "push_secondary_message".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::SelfParam { mutable: true },
                        RustParam::Named {
                            name: "message".to_string(),
                            ty: RustType::String_,
                        },
                    ],
                    ret: Some(RustType::Unit),
                    body: vec![RustStmt::Expr(RustExpr::Ident(
                        "self.secondary.push(SecondaryError::new(message))".to_string(),
                    ))],
                    is_async: false,
                },
            ],
        },
    ]
}

pub fn build_timeout_result_type_items() -> Vec<RustItem> {
    vec![RustItem::Enum {
        name: "__SifrTimeoutResult<E>".to_string(),
        visibility: Visibility::Private,
        derives: vec!["Debug".to_string()],
        repr: None,
        variants: vec![
            crate::RustEnumVariant {
                name: "Inner".to_string(),
                tuple_fields: vec![RustType::Named("E".to_string())],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "Timeout".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
        ],
    }]
}

pub fn build_async_generator_type_items() -> Vec<RustItem> {
    let type_params = vec![
        crate::RustTypeParam {
            name: "T".to_string(),
            bounds: vec![],
        },
        crate::RustTypeParam {
            name: "E".to_string(),
            bounds: vec![],
        },
    ];

    vec![
        RustItem::Struct {
            name: "AsyncGenerator<T, E>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "items".to_string(),
                    RustType::Named("std::vec::IntoIter<T>".to_string()),
                ),
                ("closed".to_string(), RustType::Bool),
                (
                    "_err".to_string(),
                    RustType::Named("std::marker::PhantomData<E>".to_string()),
                ),
            ],
        },
        RustItem::Impl {
            target: "AsyncGenerator<T, E>".to_string(),
            type_params,
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "new".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::Named {
                        name: "items".to_string(),
                        ty: RustType::Vec(Box::new(RustType::Named("T".to_string()))),
                    }],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            (
                                "items".to_string(),
                                RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("items".to_string())),
                                    method: "into_iter".to_string(),
                                    args: vec![],
                                },
                            ),
                            (
                                "closed".to_string(),
                                RustExpr::Literal(crate::RustLiteral::Bool(false)),
                            ),
                            (
                                "_err".to_string(),
                                RustExpr::Path(vec![
                                    "std".to_string(),
                                    "marker".to_string(),
                                    "PhantomData".to_string(),
                                ]),
                            ),
                        ],
                    }))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "anext".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: true }],
                    ret: Some(RustType::Result(
                        Box::new(RustType::Option(Box::new(RustType::Named("T".to_string())))),
                        Box::new(RustType::Named("E".to_string())),
                    )),
                    body: vec![
                        RustStmt::If {
                            cond: RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "closed".to_string(),
                            },
                            then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Ident("None".to_string())],
                            }))],
                            else_body: None,
                        },
                        RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("self".to_string())),
                                    field: "items".to_string(),
                                }),
                                method: "next".to_string(),
                                args: vec![],
                            }],
                        })),
                    ],
                    is_async: true,
                },
                RustItem::Fn {
                    name: "aclose".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: true }],
                    ret: Some(RustType::Result(
                        Box::new(RustType::Unit),
                        Box::new(RustType::Named("GeneratorCloseError".to_string())),
                    )),
                    body: vec![
                        RustStmt::Assign {
                            target: RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "closed".to_string(),
                            },
                            value: RustExpr::Literal(crate::RustLiteral::Bool(true)),
                        },
                        RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
                        })),
                    ],
                    is_async: true,
                },
            ],
        },
    ]
}

pub fn build_cancellation_error_type_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "CancellationError".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string()],
            fields: vec![],
        },
        RustItem::Impl {
            target: "CancellationError".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "new".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                    name: "Self".to_string(),
                    fields: vec![],
                }))],
                is_async: false,
            }],
        },
    ]
}

pub fn build_async_exit_cause_type_items() -> Vec<RustItem> {
    vec![RustItem::Enum {
        name: "AsyncExitCause".to_string(),
        visibility: Visibility::Private,
        derives: vec!["Clone".to_string(), "Debug".to_string()],
        repr: None,
        variants: vec![
            crate::RustEnumVariant {
                name: "Normal".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "Return".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "OrdinaryError".to_string(),
                tuple_fields: vec![RustType::Named("String".to_string())],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "Timeout".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "Cancellation".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "RuntimeFault".to_string(),
                tuple_fields: vec![RustType::Named("String".to_string())],
                fields: vec![],
                value: None,
            },
        ],
    }]
}

pub fn build_task_scope_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "__SifrTask<T, E>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "receiver".to_string(),
                    RustType::Option(Box::new(RustType::Named(
                        "tokio::sync::oneshot::Receiver<__SifrTaskResult<T, E>>".to_string(),
                    ))),
                ),
                (
                    "abort_handle".to_string(),
                    RustType::Named("tokio::task::AbortHandle".to_string()),
                ),
                (
                    "observed".to_string(),
                    RustType::Named(
                        "std::sync::Arc<std::sync::atomic::AtomicBool>".to_string(),
                    ),
                ),
                (
                    "_error".to_string(),
                    RustType::Named("std::marker::PhantomData<E>".to_string()),
                ),
            ],
        },
        RustItem::Struct {
            name: "__SifrBlockingTask<T, E>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "handle".to_string(),
                    RustType::Option(Box::new(RustType::Named(
                        "tokio::task::JoinHandle<__SifrTaskResult<T, E>>".to_string(),
                    ))),
                ),
                (
                    "observed".to_string(),
                    RustType::Named(
                        "std::sync::Arc<std::sync::atomic::AtomicBool>".to_string(),
                    ),
                ),
                (
                    "_error".to_string(),
                    RustType::Named("std::marker::PhantomData<E>".to_string()),
                ),
            ],
        },
        RustItem::Struct {
            name: "__SifrScopeChild".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "handle".to_string(),
                    RustType::Named(
                        "tokio::task::JoinHandle<__SifrScopeChildOutcome>".to_string(),
                    ),
                ),
                (
                    "observed".to_string(),
                    RustType::Named(
                        "std::sync::Arc<std::sync::atomic::AtomicBool>".to_string(),
                    ),
                ),
            ],
        },
        RustItem::Enum {
            name: "__SifrScopeChildOutcome".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string()],
            repr: None,
            variants: vec![
                crate::RustEnumVariant {
                    name: "Ok".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "Failed".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "Cancelled".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
            ],
        },
        RustItem::Enum {
            name: "__SifrTaskResult<T, E>".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string()],
            repr: None,
            variants: vec![
                crate::RustEnumVariant {
                    name: "Ok".to_string(),
                    tuple_fields: vec![RustType::Named("T".to_string())],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "Err".to_string(),
                    tuple_fields: vec![RustType::Named("__SifrFailure<E>".to_string())],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "Cancelled".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "__SifrFailure<CancellationError>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
            ],
        },
        RustItem::Impl {
            target: "__SifrTaskResult<T, E>".to_string(),
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec![],
                },
                crate::RustTypeParam {
                    name: "E".to_string(),
                    bounds: vec![],
                },
            ],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "cancelled".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "Self".to_string(),
                        "Cancelled".to_string(),
                    ])),
                    args: vec![RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "__SifrFailure".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "CancellationError".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        }],
                    }],
                }))],
                is_async: false,
            }],
        },
        RustItem::Enum {
            name: "__SifrSelect2<A, B>".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string()],
            repr: None,
            variants: vec![
                crate::RustEnumVariant {
                    name: "First".to_string(),
                    tuple_fields: vec![RustType::Named("A".to_string())],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "Second".to_string(),
                    tuple_fields: vec![RustType::Named("B".to_string())],
                    fields: vec![],
                    value: None,
                },
            ],
        },
        RustItem::Enum {
            name: "__SifrTimeoutResult<E>".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string()],
            repr: None,
            variants: vec![
                crate::RustEnumVariant {
                    name: "Inner".to_string(),
                    tuple_fields: vec![RustType::Named("E".to_string())],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "Timeout".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
            ],
        },
        RustItem::Impl {
            target: "__SifrTask<T, E>".to_string(),
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec![],
                },
                crate::RustTypeParam {
                    name: "E".to_string(),
                    bounds: vec![],
                },
            ],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "join".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfValue],
                    ret: Some(RustType::Named("__SifrTaskResult<T, E>".to_string())),
                    body: vec![RustStmt::Expr(RustExpr::Ident(
                        "let __SifrTask { receiver, observed, .. } = self;\n        observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        if let Some(receiver) = receiver {\n            return match receiver.await {\n                Ok(result) => result,\n                Err(_) => __SifrTaskResult::cancelled(),\n            };\n        }\n        return __SifrTaskResult::cancelled()".to_string(),
                    ))],
                    is_async: true,
                },
                RustItem::Fn {
                    name: "cancel".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::Unit),
                    body: vec![RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field: "abort_handle".to_string(),
                        }),
                        method: "abort".to_string(),
                        args: vec![],
                    })],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "cancel_and_join".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfValue],
                    ret: Some(RustType::Named("__SifrTaskResult<T, E>".to_string())),
                    body: vec![RustStmt::Expr(RustExpr::Ident(
                        "let __SifrTask { receiver, abort_handle, observed, _error } = self;\n        observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        abort_handle.abort();\n        if let Some(receiver) = receiver {\n            let _ = receiver.await;\n        }\n        return __SifrTaskResult::cancelled()".to_string(),
                    ))],
                    is_async: true,
                },
                RustItem::Fn {
                    name: "__sifr_timeout".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::SelfValue,
                        RustParam::Named {
                            name: "duration".to_string(),
                            ty: RustType::Named("std::time::Duration".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named(
                        "__SifrTaskResult<T, __SifrTimeoutResult<E>>".to_string(),
                    )),
                    body: vec![RustStmt::Expr(RustExpr::Ident(
                        "let __SifrTask { receiver, abort_handle, observed, _error } = self;\n        observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        if let Some(mut receiver) = receiver {\n            return tokio::select! {\n                biased;\n                result = &mut receiver => {\n                    match result {\n                        Ok(__SifrTaskResult::Ok(value)) => __SifrTaskResult::Ok(value),\n                        Ok(__SifrTaskResult::Err(failure)) => __SifrTaskResult::Err(failure.map_primary(__SifrTimeoutResult::Inner)),\n                        Ok(__SifrTaskResult::Cancelled(failure)) => __SifrTaskResult::Cancelled(failure),\n                        Err(_) => __SifrTaskResult::cancelled(),\n                    }\n                },\n                _ = tokio::time::sleep(duration) => {\n                    abort_handle.abort();\n                    let _ = receiver.await;\n                    __SifrTaskResult::Err(__SifrFailure::new(__SifrTimeoutResult::Timeout))\n                }\n            };\n        }\n        return __SifrTaskResult::cancelled()".to_string(),
                    ))],
                    is_async: true,
                },
            ],
        },
        RustItem::Impl {
            target: "__SifrBlockingTask<T, E>".to_string(),
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec![],
                },
                crate::RustTypeParam {
                    name: "E".to_string(),
                    bounds: vec![],
                },
            ],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "join".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfValue],
                    ret: Some(RustType::Named("__SifrTaskResult<T, E>".to_string())),
                    body: vec![RustStmt::Expr(RustExpr::Ident(
                        "let __SifrBlockingTask { handle, observed, .. } = self;\n        observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        if let Some(handle) = handle {\n            return match handle.await {\n                Ok(result) => result,\n                Err(_) => __SifrTaskResult::cancelled(),\n            };\n        }\n        return __SifrTaskResult::cancelled()".to_string(),
                    ))],
                    is_async: true,
                },
                RustItem::Fn {
                    name: "cancel".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::Unit),
                    body: vec![RustStmt::Expr(RustExpr::Ident(
                        "if let Some(handle) = &self.handle {\n            handle.abort();\n        }"
                            .to_string(),
                    ))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "cancel_and_join".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfValue],
                    ret: Some(RustType::Named("__SifrTaskResult<T, E>".to_string())),
                    body: vec![RustStmt::Expr(RustExpr::Ident(
                        "if let Some(handle) = &self.handle {\n            handle.abort();\n        }\n        return self.join().await".to_string(),
                    ))],
                    is_async: true,
                },
            ],
        },
        RustItem::Struct {
            name: "__SifrTaskScope".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "children".to_string(),
                    RustType::Vec(Box::new(RustType::Named("__SifrScopeChild".to_string()))),
                ),
                ("fail_fast".to_string(), RustType::Bool),
            ],
        },
        RustItem::Impl {
            target: "__SifrTaskScope".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "new".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            (
                                "children".to_string(),
                                RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "Vec".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            ),
                            (
                                "fail_fast".to_string(),
                                RustExpr::Literal(crate::RustLiteral::Bool(false)),
                            ),
                        ],
                    }))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "new_task_group".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            (
                                "children".to_string(),
                                RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "Vec".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            ),
                            (
                                "fail_fast".to_string(),
                                RustExpr::Literal(crate::RustLiteral::Bool(true)),
                            ),
                        ],
                    }))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__sifr_spawn_infallible".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![
                        crate::RustTypeParam {
                            name: "T".to_string(),
                            bounds: vec!["Send".to_string(), "'static".to_string()],
                        },
                        crate::RustTypeParam {
                            name: "F".to_string(),
                            bounds: vec![
                                "std::future::Future<Output = T>".to_string(),
                                "Send".to_string(),
                                "'static".to_string(),
                            ],
                        },
                    ],
                    params: vec![
                        RustParam::SelfParam { mutable: true },
                        RustParam::Named {
                            name: "future".to_string(),
                            ty: RustType::Named("F".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named(
                        "__SifrTask<T, std::convert::Infallible>".to_string(),
                    )),
                    body: vec![
                        RustStmt::LetPattern {
                            pattern: "(sender, receiver)".to_string(),
                            value: RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "tokio".to_string(),
                                    "sync".to_string(),
                                    "oneshot".to_string(),
                                    "channel".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "observed".to_string(),
                            ty: None,
                            value: RustExpr::Ident(
                                "std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))"
                                    .to_string(),
                            ),
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "child_observed".to_string(),
                            ty: None,
                            value: RustExpr::Ident(
                                "std::sync::Arc::clone(&observed)".to_string(),
                            ),
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "child".to_string(),
                            ty: None,
                            value: RustExpr::Ident(
                                "tokio::spawn(async move { let result = future.await; let _ = sender.send(__SifrTaskResult::Ok(result)); __SifrScopeChildOutcome::Ok })"
                                    .to_string(),
                            ),
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "abort_handle".to_string(),
                            ty: None,
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("child".to_string())),
                                method: "abort_handle".to_string(),
                                args: vec![],
                            },
                        },
                        RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "children".to_string(),
                            }),
                            method: "push".to_string(),
                            args: vec![RustExpr::StructInit {
                                name: "__SifrScopeChild".to_string(),
                                fields: vec![
                                    ("handle".to_string(), RustExpr::Ident("child".to_string())),
                                    (
                                        "observed".to_string(),
                                        RustExpr::Ident("child_observed".to_string()),
                                    ),
                                ],
                            }],
                        }),
                        RustStmt::Return(Some(RustExpr::StructInit {
                            name: "__SifrTask".to_string(),
                            fields: vec![
                                (
                                    "receiver".to_string(),
                                    RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                                        args: vec![RustExpr::Ident("receiver".to_string())],
                                    },
                                ),
                                (
                                    "abort_handle".to_string(),
                                    RustExpr::Ident("abort_handle".to_string()),
                                ),
                                (
                                    "observed".to_string(),
                                    RustExpr::Ident("observed".to_string()),
                                ),
                                (
                                    "_error".to_string(),
                                    RustExpr::Path(vec![
                                        "std".to_string(),
                                        "marker".to_string(),
                                        "PhantomData".to_string(),
                                    ]),
                                ),
                            ],
                        })),
                    ],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__sifr_spawn_result".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![
                        crate::RustTypeParam {
                            name: "T".to_string(),
                            bounds: vec!["Send".to_string(), "'static".to_string()],
                        },
                        crate::RustTypeParam {
                            name: "E".to_string(),
                            bounds: vec!["Send".to_string(), "'static".to_string()],
                        },
                        crate::RustTypeParam {
                            name: "F".to_string(),
                            bounds: vec![
                                "std::future::Future<Output = Result<T, E>>".to_string(),
                                "Send".to_string(),
                                "'static".to_string(),
                            ],
                        },
                    ],
                    params: vec![
                        RustParam::SelfParam { mutable: true },
                        RustParam::Named {
                            name: "future".to_string(),
                            ty: RustType::Named("F".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named("__SifrTask<T, E>".to_string())),
                    body: vec![
                        RustStmt::LetPattern {
                            pattern: "(sender, receiver)".to_string(),
                            value: RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "tokio".to_string(),
                                    "sync".to_string(),
                                    "oneshot".to_string(),
                                    "channel".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "observed".to_string(),
                            ty: None,
                            value: RustExpr::Ident(
                                "std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))"
                                    .to_string(),
                            ),
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "child_observed".to_string(),
                            ty: None,
                            value: RustExpr::Ident(
                                "std::sync::Arc::clone(&observed)".to_string(),
                            ),
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "child".to_string(),
                            ty: None,
                            value: RustExpr::Ident(
                                "tokio::spawn(async move { let result = match future.await { Ok(value) => __SifrTaskResult::Ok(value), Err(err) => __SifrTaskResult::Err(__SifrFailure::new(err)) }; let outcome = match &result { __SifrTaskResult::Ok(_) => __SifrScopeChildOutcome::Ok, __SifrTaskResult::Err(_) => __SifrScopeChildOutcome::Failed, __SifrTaskResult::Cancelled(_) => __SifrScopeChildOutcome::Cancelled }; let _ = sender.send(result); outcome })"
                                    .to_string(),
                            ),
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "abort_handle".to_string(),
                            ty: None,
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("child".to_string())),
                                method: "abort_handle".to_string(),
                                args: vec![],
                            },
                        },
                        RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "children".to_string(),
                            }),
                            method: "push".to_string(),
                            args: vec![RustExpr::StructInit {
                                name: "__SifrScopeChild".to_string(),
                                fields: vec![
                                    ("handle".to_string(), RustExpr::Ident("child".to_string())),
                                    (
                                        "observed".to_string(),
                                        RustExpr::Ident("child_observed".to_string()),
                                    ),
                                ],
                            }],
                        }),
                        RustStmt::Return(Some(RustExpr::StructInit {
                            name: "__SifrTask".to_string(),
                            fields: vec![
                                (
                                    "receiver".to_string(),
                                    RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                                        args: vec![RustExpr::Ident("receiver".to_string())],
                                    },
                                ),
                                (
                                    "abort_handle".to_string(),
                                    RustExpr::Ident("abort_handle".to_string()),
                                ),
                                (
                                    "observed".to_string(),
                                    RustExpr::Ident("observed".to_string()),
                                ),
                                (
                                    "_error".to_string(),
                                    RustExpr::Path(vec![
                                        "std".to_string(),
                                        "marker".to_string(),
                                        "PhantomData".to_string(),
                                    ]),
                                ),
                            ],
                        })),
                    ],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__sifr_join_all".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: true }],
                    ret: Some(RustType::Named("Result<(), ScopeFailure>".to_string())),
                    body: vec![RustStmt::Expr(RustExpr::Ident(
                        r#"if self.fail_fast {
            let mut failure: Option<ScopeFailure> = None;
            let mut policy_cancelling = false;
            let mut abort_handles = Vec::with_capacity(self.children.len());
            let mut join_set = tokio::task::JoinSet::new();
            for child in self.children.drain(..) {
                abort_handles.push(child.handle.abort_handle());
                join_set.spawn(async move {
                    let observed = child.observed.load(std::sync::atomic::Ordering::SeqCst);
                    (observed, child.handle.await)
                });
            }
            while let Some(joined) = join_set.join_next().await {
                let mut group_failure_seen = false;
                match joined {
                    Ok((observed, Ok(__SifrScopeChildOutcome::Ok))) => {}
                    Ok((observed, Ok(__SifrScopeChildOutcome::Failed))) => {
                        group_failure_seen = true;
                        if !observed && failure.is_none() {
                            failure = Some(ScopeFailure::new("unobserved child task failed".to_string()));
                        }
                    }
                    Ok((observed, Ok(__SifrScopeChildOutcome::Cancelled))) => {
                        group_failure_seen = true;
                        if !observed && !policy_cancelling && failure.is_none() {
                            failure = Some(ScopeFailure::new("unobserved child task was cancelled".to_string()));
                        }
                    }
                    Ok((observed, Err(join_error))) => {
                        group_failure_seen = !join_error.is_cancelled();
                        if !observed && !policy_cancelling && failure.is_none() {
                            let message = if join_error.is_cancelled() { "unobserved child task was cancelled" } else { "unobserved child task failed" };
                            failure = Some(ScopeFailure::new(message.to_string()));
                        }
                    }
                    Err(_) => {
                        group_failure_seen = true;
                        if !policy_cancelling && failure.is_none() {
                            failure = Some(ScopeFailure::new("task group child observer failed".to_string()));
                        }
                    }
                }
                if group_failure_seen && !policy_cancelling {
                    policy_cancelling = true;
                    for abort_handle in &abort_handles {
                        abort_handle.abort();
                    }
                }
            }
            if let Some(failure) = failure {
                return Err(failure);
            }
            return Ok(());
        }
        let mut failure: Option<ScopeFailure> = None;
        while let Some(child) = self.children.pop() {
            let observed = child.observed.load(std::sync::atomic::Ordering::SeqCst);
            match child.handle.await {
                Ok(__SifrScopeChildOutcome::Ok) => {}
                Ok(__SifrScopeChildOutcome::Failed) => {
                    if !observed && failure.is_none() {
                        failure = Some(ScopeFailure::new("unobserved child task failed".to_string()));
                    }
                }
                Ok(__SifrScopeChildOutcome::Cancelled) => {
                    if !observed && failure.is_none() {
                        failure = Some(ScopeFailure::new("unobserved child task was cancelled".to_string()));
                    }
                }
                Err(join_error) => {
                    if !observed && failure.is_none() {
                        let message = if join_error.is_cancelled() { "unobserved child task was cancelled" } else { "unobserved child task failed" };
                        failure = Some(ScopeFailure::new(message.to_string()));
                    }
                }
            }
        }
        if let Some(failure) = failure {
            return Err(failure);
        }
        return Ok(())"#.to_string(),
                    ))],
                    is_async: true,
                },
            ],
        },
        RustItem::Fn {
            name: "__sifr_spawn_blocking_infallible".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "F".to_string(),
                    bounds: vec![
                        "FnOnce() -> T".to_string(),
                        "Send".to_string(),
                        "'static".to_string(),
                    ],
                },
            ],
            params: vec![RustParam::Named {
                name: "work".to_string(),
                ty: RustType::Named("F".to_string()),
            }],
            ret: Some(RustType::Named(
                "__SifrBlockingTask<T, std::convert::Infallible>".to_string(),
            )),
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n        let handle = tokio::task::spawn_blocking(move || __SifrTaskResult::Ok(work()));\n        return __SifrBlockingTask { handle: Some(handle), observed, _error: std::marker::PhantomData }".to_string(),
            ))],
            is_async: false,
        },
        RustItem::Fn {
            name: "__sifr_spawn_blocking_result".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "E".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "F".to_string(),
                    bounds: vec![
                        "FnOnce() -> Result<T, E>".to_string(),
                        "Send".to_string(),
                        "'static".to_string(),
                    ],
                },
            ],
            params: vec![RustParam::Named {
                name: "work".to_string(),
                ty: RustType::Named("F".to_string()),
            }],
            ret: Some(RustType::Named("__SifrBlockingTask<T, E>".to_string())),
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n        let handle = tokio::task::spawn_blocking(move || match work() { Ok(value) => __SifrTaskResult::Ok(value), Err(err) => __SifrTaskResult::Err(__SifrFailure::new(err)) });\n        return __SifrBlockingTask { handle: Some(handle), observed, _error: std::marker::PhantomData }".to_string(),
            ))],
            is_async: false,
        },
        RustItem::Fn {
            name: "__sifr_task_gather".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "E".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
            ],
            params: vec![RustParam::Named {
                name: "handles".to_string(),
                ty: RustType::Named("Vec<__SifrTask<T, E>>".to_string()),
            }],
            ret: Some(RustType::Named(
                "__SifrTaskResult<Vec<T>, E>".to_string(),
            )),
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "let input_len = handles.len();\n        let mut values: Vec<Option<T>> = std::iter::repeat_with(|| None).take(input_len).collect();\n        let mut failure_results: Vec<Option<__SifrTaskResult<Vec<T>, E>>> = std::iter::repeat_with(|| None).take(input_len).collect();\n        let mut abort_handles = Vec::with_capacity(input_len);\n        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();\n        let mut observer_count = 0usize;\n        let mut cancelling = false;\n        for (index, handle) in handles.into_iter().enumerate() {\n            let __SifrTask { receiver: task_receiver, abort_handle, observed, _error } = handle;\n            observed.store(true, std::sync::atomic::Ordering::SeqCst);\n            abort_handles.push(abort_handle);\n            observer_count += 1;\n            let sender = sender.clone();\n            if let Some(task_receiver) = task_receiver {\n                tokio::spawn(async move {\n                    let result = match task_receiver.await {\n                        Ok(result) => result,\n                        Err(_) => __SifrTaskResult::cancelled(),\n                    };\n                    let _ = sender.send((index, result));\n                });\n            } else {\n                let _ = sender.send((index, __SifrTaskResult::cancelled()));\n            }\n        }\n        drop(sender);\n        let mut remaining = observer_count;\n        while remaining > 0 {\n            let Some((index, result)) = receiver.recv().await else {\n                break;\n            };\n            remaining -= 1;\n            match result {\n                __SifrTaskResult::Ok(value) => {\n                    if !cancelling {\n                        values[index] = Some(value);\n                    }\n                }\n                __SifrTaskResult::Err(failure) => {\n                    failure_results[index] = Some(__SifrTaskResult::Err(failure));\n                    if !cancelling {\n                        cancelling = true;\n                        for abort_handle in &abort_handles {\n                            abort_handle.abort();\n                        }\n                    }\n                }\n                __SifrTaskResult::Cancelled(failure) => {\n                    failure_results[index] = Some(__SifrTaskResult::Cancelled(failure));\n                    if !cancelling {\n                        cancelling = true;\n                        for abort_handle in &abort_handles {\n                            abort_handle.abort();\n                        }\n                    }\n                }\n            }\n        }\n        let mut primary_failure: Option<__SifrTaskResult<Vec<T>, E>> = None;\n        for result in failure_results.into_iter().flatten() {\n            if let Some(existing) = primary_failure.as_mut() {\n                match (existing, result) {\n                    (__SifrTaskResult::Err(failure), __SifrTaskResult::Err(_)) => {\n                        failure.push_secondary_message(\"sibling task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Err(failure), __SifrTaskResult::Cancelled(_)) => {\n                        failure.push_secondary_message(\"sibling task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), __SifrTaskResult::Err(_)) => {\n                        failure.push_secondary_message(\"sibling task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), __SifrTaskResult::Cancelled(_)) => {\n                        failure.push_secondary_message(\"sibling task was cancelled\".to_string());\n                    }\n                    _ => {}\n                }\n            } else {\n                primary_failure = Some(result);\n            }\n        }\n        if let Some(result) = primary_failure {\n            return result;\n        }\n        let mut ordered_values = Vec::with_capacity(input_len);\n        for value in values {\n            let Some(value) = value else {\n                return __SifrTaskResult::cancelled();\n            };\n            ordered_values.push(value);\n        }\n        return __SifrTaskResult::Ok(ordered_values)".to_string(),
            ))],
            is_async: true,
        },
        RustItem::Fn {
            name: "__sifr_task_race".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "E".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
            ],
            params: vec![RustParam::Named {
                name: "handles".to_string(),
                ty: RustType::Named("Vec<__SifrTask<T, E>>".to_string()),
            }],
            ret: Some(RustType::Named("__SifrTaskResult<T, E>".to_string())),
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "let mut abort_handles = Vec::new();\n        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();\n        let mut observer_count = 0usize;\n        for handle in handles {\n            let __SifrTask { receiver: task_receiver, abort_handle, observed, _error } = handle;\n            observed.store(true, std::sync::atomic::Ordering::SeqCst);\n            abort_handles.push(abort_handle);\n            if let Some(task_receiver) = task_receiver {\n                observer_count += 1;\n                let sender = sender.clone();\n                tokio::spawn(async move {\n                    let result = match task_receiver.await {\n                        Ok(result) => result,\n                        Err(_) => __SifrTaskResult::cancelled(),\n                    };\n                    let _ = sender.send(result);\n                });\n            }\n        }\n        drop(sender);\n        let Some(mut first) = receiver.recv().await else {\n            return __SifrTaskResult::cancelled();\n        };\n        for abort_handle in &abort_handles {\n            abort_handle.abort();\n        }\n        let mut remaining = observer_count.saturating_sub(1);\n        while remaining > 0 {\n            let Some(loser) = receiver.recv().await else {\n                break;\n            };\n            match (&mut first, loser) {\n                (__SifrTaskResult::Err(failure), __SifrTaskResult::Err(_)) => {\n                    failure.push_secondary_message(\"race loser task failed\".to_string());\n                }\n                (__SifrTaskResult::Err(failure), __SifrTaskResult::Cancelled(_)) => {\n                    failure.push_secondary_message(\"race loser task was cancelled\".to_string());\n                }\n                (__SifrTaskResult::Cancelled(failure), __SifrTaskResult::Err(_)) => {\n                    failure.push_secondary_message(\"race loser task failed\".to_string());\n                }\n                (__SifrTaskResult::Cancelled(failure), __SifrTaskResult::Cancelled(_)) => {\n                    failure.push_secondary_message(\"race loser task was cancelled\".to_string());\n                }\n                _ => {}\n            }\n            remaining -= 1;\n        }\n        return first".to_string(),
            ))],
            is_async: true,
        },
        RustItem::Fn {
            name: "__sifr_task_select".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "A".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "EA".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "B".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "EB".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
            ],
            params: vec![
                RustParam::Named {
                    name: "first".to_string(),
                    ty: RustType::Named("__SifrTask<A, EA>".to_string()),
                },
                RustParam::Named {
                    name: "second".to_string(),
                    ty: RustType::Named("__SifrTask<B, EB>".to_string()),
                },
            ],
            ret: Some(RustType::Named(
                "__SifrSelect2<__SifrTaskResult<A, EA>, __SifrTaskResult<B, EB>>".to_string(),
            )),
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "let __SifrTask { receiver: first_receiver, abort_handle: first_abort, observed: first_observed, _error: _first_error } = first;\n        let __SifrTask { receiver: second_receiver, abort_handle: second_abort, observed: second_observed, _error: _second_error } = second;\n        first_observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        second_observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        let (Some(mut first_receiver), Some(mut second_receiver)) = (first_receiver, second_receiver) else {\n            return __SifrSelect2::First(__SifrTaskResult::cancelled());\n        };\n        return tokio::select! {\n            biased;\n            first_result = &mut first_receiver => {\n                second_abort.abort();\n                let mut result = match first_result {\n                    Ok(result) => result,\n                    Err(_) => __SifrTaskResult::cancelled(),\n                };\n                let loser = second_receiver.await;\n                match (&mut result, loser) {\n                    (__SifrTaskResult::Err(failure), Ok(__SifrTaskResult::Err(_))) => {\n                        failure.push_secondary_message(\"select loser task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Err(failure), Ok(__SifrTaskResult::Cancelled(_)) | Err(_)) => {\n                        failure.push_secondary_message(\"select loser task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), Ok(__SifrTaskResult::Err(_))) => {\n                        failure.push_secondary_message(\"select loser task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), Ok(__SifrTaskResult::Cancelled(_)) | Err(_)) => {\n                        failure.push_secondary_message(\"select loser task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Ok(_), Ok(__SifrTaskResult::Err(_)) | Ok(__SifrTaskResult::Cancelled(_))) => {\n                        second_observed.store(false, std::sync::atomic::Ordering::SeqCst);\n                    }\n                    _ => {}\n                }\n                __SifrSelect2::First(result)\n            },\n            second_result = &mut second_receiver => {\n                first_abort.abort();\n                let mut result = match second_result {\n                    Ok(result) => result,\n                    Err(_) => __SifrTaskResult::cancelled(),\n                };\n                let loser = first_receiver.await;\n                match (&mut result, loser) {\n                    (__SifrTaskResult::Err(failure), Ok(__SifrTaskResult::Err(_))) => {\n                        failure.push_secondary_message(\"select loser task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Err(failure), Ok(__SifrTaskResult::Cancelled(_)) | Err(_)) => {\n                        failure.push_secondary_message(\"select loser task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), Ok(__SifrTaskResult::Err(_))) => {\n                        failure.push_secondary_message(\"select loser task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), Ok(__SifrTaskResult::Cancelled(_)) | Err(_)) => {\n                        failure.push_secondary_message(\"select loser task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Ok(_), Ok(__SifrTaskResult::Err(_)) | Ok(__SifrTaskResult::Cancelled(_))) => {\n                        first_observed.store(false, std::sync::atomic::Ordering::SeqCst);\n                    }\n                    _ => {}\n                }\n                __SifrSelect2::Second(result)\n            }\n        }".to_string(),
            ))],
            is_async: true,
        },
    ]
}

pub fn build_io_error_items() -> Vec<RustItem> {
    let mut items = build_error_type_items(
        "IOError",
        &[("kind".to_string(), RustType::String_)],
        &[(
            "kind".to_string(),
            RustExpr::Literal(crate::RustLiteral::Str("Other".to_string())),
        )],
    );

    items.push(RustItem::Fn {
        name: "__io_err".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "e".to_string(),
            ty: RustType::Named("std::io::Error".to_string()),
        }],
        ret: Some(RustType::Named("IOError".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "msg".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("e".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "kind".to_string(),
                ty: None,
                value: RustExpr::If {
                    cond: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("e".to_string())),
                            method: "kind".to_string(),
                            args: vec![],
                        }),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "io".to_string(),
                            "ErrorKind".to_string(),
                            "NotFound".to_string(),
                        ])),
                    }),
                    then_expr: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                        "FileNotFound".to_string(),
                    ))),
                    else_expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "kind".to_string(),
                                args: vec![],
                            }),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "io".to_string(),
                                "ErrorKind".to_string(),
                                "PermissionDenied".to_string(),
                            ])),
                        }),
                        then_expr: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                            "PermissionDenied".to_string(),
                        ))),
                        else_expr: Some(Box::new(RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("e".to_string())),
                                    method: "kind".to_string(),
                                    args: vec![],
                                }),
                                op: "==".to_string(),
                                right: Box::new(RustExpr::Path(vec![
                                    "std".to_string(),
                                    "io".to_string(),
                                    "ErrorKind".to_string(),
                                    "AlreadyExists".to_string(),
                                ])),
                            }),
                            then_expr: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                                "FileExists".to_string(),
                            ))),
                            else_expr: Some(Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                                "Other".to_string(),
                            )))),
                        })),
                    })),
                },
            },
            RustStmt::Return(Some(RustExpr::StructInit {
                name: "IOError".to_string(),
                fields: vec![
                    ("message".to_string(), RustExpr::Ident("msg".to_string())),
                    ("kind".to_string(), RustExpr::Ident("kind".to_string())),
                ],
            })),
        ],
        is_async: false,
    });

    items
}

pub fn build_file_handle_infra_items() -> Vec<RustItem> {
    vec![
        RustItem::Enum {
            name: "SifrFileHandle".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            repr: None,
            variants: vec![
                crate::RustEnumVariant {
                    name: "TextRead".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "std::io::BufReader<std::fs::File>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "TextWrite".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "std::io::BufWriter<std::fs::File>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "BinaryRead".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "std::io::BufReader<std::fs::File>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "BinaryWrite".to_string(),
                    tuple_fields: vec![RustType::Named(
                        "std::io::BufWriter<std::fs::File>".to_string(),
                    )],
                    fields: vec![],
                    value: None,
                },
            ],
        },
        RustItem::Static {
            name: "__SIFR_FILE_HANDLES".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, SifrFileHandle>>>"
                    .to_string(),
            ),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "LazyLock".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Closure {
                    params: vec![],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "sync".to_string(),
                            "Mutex".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "collections".to_string(),
                                "HashMap".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        }],
                    }),
                    is_move: false,
                }],
            },
        },
        RustItem::Static {
            name: "__SIFR_NEXT_FILE_HANDLE_ID".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named("std::sync::atomic::AtomicI64".to_string()),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "atomic".to_string(),
                    "AtomicI64".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Literal(crate::RustLiteral::Int(1))],
            },
        },
        RustItem::Fn {
            name: "__sifr_next_file_handle_id".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::I64),
            body: vec![RustStmt::Return(Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__SIFR_NEXT_FILE_HANDLE_ID".to_string())),
                method: "fetch_add".to_string(),
                args: vec![
                    RustExpr::Literal(crate::RustLiteral::Int(1)),
                    RustExpr::Path(vec![
                        "std".to_string(),
                        "sync".to_string(),
                        "atomic".to_string(),
                        "Ordering".to_string(),
                        "SeqCst".to_string(),
                    ]),
                ],
            }))],
            is_async: false,
        },
    ]
}

pub fn build_file_handle_struct_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "FileHandle".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields: vec![
                ("_handle".to_string(), RustType::I64),
                ("_mode".to_string(), RustType::String_),
            ],
        },
        RustItem::Impl {
            target: "FileHandle".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "new".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::Named {
                            name: "_handle".to_string(),
                            ty: RustType::I64,
                        },
                        RustParam::Named {
                            name: "_mode".to_string(),
                            ty: RustType::String_,
                        },
                    ],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            (
                                "_handle".to_string(),
                                RustExpr::Ident("_handle".to_string()),
                            ),
                            ("_mode".to_string(), RustExpr::Ident("_mode".to_string())),
                        ],
                    }))],
                    is_async: false,
                },
                file_handle_read_method(),
                file_handle_write_method(),
                file_handle_readline_method(),
                file_handle_readlines_method(),
                RustItem::Fn {
                    name: "close".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: None,
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
                        RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(file_handles_lock_expr()),
                            method: "remove".to_string(),
                            args: vec![RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(RustExpr::Ident("__hid".to_string())),
                            }],
                        }),
                    ],
                    is_async: false,
                },
                file_handle_read_bytes_method(),
                file_handle_write_bytes_method(),
                RustItem::Fn {
                    name: "__enter__".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::Ref {
                        mutable: false,
                        inner: Box::new(RustType::Named("Self".to_string())),
                    }),
                    body: vec![RustStmt::Return(Some(RustExpr::Ident("self".to_string())))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__exit__".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: None,
                    body: vec![RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("self".to_string())),
                        method: "close".to_string(),
                        args: vec![],
                    })],
                    is_async: false,
                },
            ],
        },
    ]
}

pub fn build_logging_items() -> Vec<RustItem> {
    vec![RustItem::Static {
        name: "__SIFR_GLOBAL_LOG_LEVEL".to_string(),
        visibility: Visibility::Private,
        ty: RustType::Named("std::sync::LazyLock<std::sync::Mutex<i64>>".to_string()),
        value: RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "sync".to_string(),
                "LazyLock".to_string(),
                "new".to_string(),
            ])),
            args: vec![RustExpr::Closure {
                params: vec![],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "sync".to_string(),
                        "Mutex".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpr::Literal(crate::RustLiteral::Int(20))],
                }),
                is_move: false,
            }],
        },
    }]
}

pub fn build_random_module_state_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "__SifrRandomModuleState".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields: vec![
                (
                    "words".to_string(),
                    RustType::Vec(Box::new(RustType::Named("i64".to_string()))),
                ),
                ("index".to_string(), RustType::I64),
                (
                    "gauss_next".to_string(),
                    RustType::Option(Box::new(RustType::F64)),
                ),
            ],
        },
        RustItem::Static {
            name: "__SIFR_RANDOM_MODULE_STATE".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<__SifrRandomModuleState>>".to_string(),
            ),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "LazyLock".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Closure {
                    params: vec![],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "sync".to_string(),
                            "Mutex".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![RustExpr::StructInit {
                            name: "__SifrRandomModuleState".to_string(),
                            fields: vec![
                                (
                                    "words".to_string(),
                                    RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "Vec".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![],
                                    },
                                ),
                                (
                                    "index".to_string(),
                                    RustExpr::Literal(crate::RustLiteral::Int(0)),
                                ),
                                (
                                    "gauss_next".to_string(),
                                    RustExpr::Literal(crate::RustLiteral::None),
                                ),
                            ],
                        }],
                    }),
                    is_move: false,
                }],
            },
        },
    ]
}

fn file_handles_lock_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__SIFR_FILE_HANDLES".to_string())),
            method: "lock".to_string(),
            args: vec![],
        }),
        method: "unwrap_or_else".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__err".to_string(),
                ty: RustType::Named("_".to_string()),
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

fn file_handle_read_method() -> RustItem {
    RustItem::Fn {
        name: "read".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::String_),
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
                                name: "__s".to_string(),
                                ty: None,
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "String".to_string(),
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
                                        "read_to_string".to_string(),
                                    ])),
                                    args: vec![
                                        RustExpr::Ident("__r".to_string()),
                                        RustExpr::Ref {
                                            mutable: true,
                                            expr: Box::new(RustExpr::Ident("__s".to_string())),
                                        },
                                    ],
                                }),
                                method: "map_err".to_string(),
                                args: vec![RustExpr::Ident("__io_err".to_string())],
                            }))),
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::Ident("__s".to_string())],
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

fn file_handle_write_method() -> RustItem {
    RustItem::Fn {
        name: "write".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::SelfParam { mutable: false },
            RustParam::Named {
                name: "data".to_string(),
                ty: RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::String_),
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
                        pattern: "Some(SifrFileHandle::TextWrite(ref mut __w))".to_string(),
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
                                        RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("data".to_string())),
                                            method: "as_bytes".to_string(),
                                            args: vec![],
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
                                            "file not open for writing".to_string(),
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

fn file_handle_readline_method() -> RustItem {
    RustItem::Fn {
        name: "readline".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::Result(
            Box::new(RustType::Option(Box::new(RustType::String_))),
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
                                    right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                                },
                                then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                    args: vec![RustExpr::Literal(crate::RustLiteral::None)],
                                }))],
                                else_body: None,
                            },
                            RustStmt::If {
                                cond: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__line".to_string())),
                                    method: "ends_with".to_string(),
                                    args: vec![RustExpr::Literal(crate::RustLiteral::Char('\n'))],
                                },
                                then_body: vec![
                                    RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("__line".to_string())),
                                        method: "pop".to_string(),
                                        args: vec![],
                                    }),
                                    RustStmt::If {
                                        cond: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__line".to_string(),
                                            )),
                                            method: "ends_with".to_string(),
                                            args: vec![RustExpr::Literal(
                                                crate::RustLiteral::Char('\r'),
                                            )],
                                        },
                                        then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__line".to_string(),
                                            )),
                                            method: "pop".to_string(),
                                            args: vec![],
                                        })],
                                        else_body: None,
                                    },
                                ],
                                else_body: None,
                            },
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                                    args: vec![RustExpr::Ident("__line".to_string())],
                                }],
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

fn file_handle_read_bytes_method() -> RustItem {
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

fn file_handle_write_bytes_method() -> RustItem {
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

fn file_handle_readlines_method() -> RustItem {
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
    use crate::render_items;

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
