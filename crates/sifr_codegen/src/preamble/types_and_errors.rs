use crate::{
    homogeneous_large_tuple_backing_array, RustExpr, RustItem, RustParam, RustStmt, RustType, Type,
    Visibility,
};

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
        Type::Tuple(items) => {
            if let Some((elem, len)) = homogeneous_large_tuple_backing_array(ty) {
                RustType::Named(format!(
                    "[{}; {}]",
                    crate::Renderer::render_type_string(&sifr_type_to_rust_type(elem)),
                    len
                ))
            } else {
                RustType::Tuple(items.iter().map(sifr_type_to_rust_type).collect())
            }
        }
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

pub(crate) fn task_error_type_to_rust_type(ty: &Type) -> RustType {
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
                (
                    "factory".to_string(),
                    RustType::Named(
                        "Option<Box<dyn FnOnce() -> Vec<T> + Send + 'static>>".to_string(),
                    ),
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
                            ("factory".to_string(), RustExpr::Ident("None".to_string())),
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
                    name: "new_lazy".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![crate::RustTypeParam {
                        name: "F".to_string(),
                        bounds: vec![
                            "FnOnce() -> Vec<T>".to_string(),
                            "Send".to_string(),
                            "'static".to_string(),
                        ],
                    }],
                    params: vec![RustParam::Named {
                        name: "factory".to_string(),
                        ty: RustType::Named("F".to_string()),
                    }],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            (
                                "items".to_string(),
                                RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Vec(vec![])),
                                    method: "into_iter".to_string(),
                                    args: vec![],
                                },
                            ),
                            (
                                "factory".to_string(),
                                RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                                    args: vec![RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "Box".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![RustExpr::Ident("factory".to_string())],
                                    }],
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
                        RustStmt::IfLet {
                            pattern: "Some(__sifr_async_generator_factory)".to_string(),
                            expr: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("self".to_string())),
                                    field: "factory".to_string(),
                                }),
                                method: "take".to_string(),
                                args: vec![],
                            },
                            then_body: vec![RustStmt::Assign {
                                target: RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("self".to_string())),
                                    field: "items".to_string(),
                                },
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::FnCall {
                                        func: Box::new(RustExpr::Ident(
                                            "__sifr_async_generator_factory".to_string(),
                                        )),
                                        args: vec![],
                                    }),
                                    method: "into_iter".to_string(),
                                    args: vec![],
                                },
                            }],
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
