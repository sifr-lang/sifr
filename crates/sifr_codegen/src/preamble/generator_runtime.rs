use crate::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};

fn type_param(name: &str, bounds: &[&str]) -> crate::RustTypeParam {
    crate::RustTypeParam {
        name: name.to_string(),
        bounds: bounds.iter().map(|bound| (*bound).to_string()).collect(),
    }
}

fn common_generator_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "__SifrYielder<T>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![(
                "slot".to_string(),
                RustType::Named("std::sync::Arc<std::sync::Mutex<Option<T>>>".to_string()),
            )],
        },
        RustItem::Struct {
            name: "__SifrYieldFuture<T>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "slot".to_string(),
                    RustType::Named("std::sync::Arc<std::sync::Mutex<Option<T>>>".to_string()),
                ),
                ("value".to_string(), RustType::Option(Box::new(RustType::Named("T".to_string())))),
            ],
        },
        RustItem::Impl {
            target: "__SifrYieldFuture<T>".to_string(),
            type_params: vec![type_param("T", &[])],
            trait_: Some("Unpin".to_string()),
            items: vec![],
        },
        RustItem::Impl {
            target: "__SifrYieldFuture<T>".to_string(),
            type_params: vec![type_param("T", &[])],
            trait_: Some("std::future::Future".to_string()),
            items: vec![
                RustItem::TypeAlias {
                    name: "Output".to_string(),
                    ty: RustType::Unit,
                },
                RustItem::Fn {
                    name: "poll".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::Named {
                            name: "self".to_string(),
                            ty: RustType::Named("std::pin::Pin<&mut Self>".to_string()),
                        },
                        RustParam::Named {
                            name: "_cx".to_string(),
                            ty: RustType::Named("&mut std::task::Context<'_>".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named("std::task::Poll<()>".to_string())),
                    body: vec![RustStmt::Verbatim(
                        "let state = self.get_mut();\nlet Some(value) = state.value.take() else {\n    return std::task::Poll::Ready(());\n};\n__sifr_store_suspended(&state.slot, value);\nstd::task::Poll::Pending"
                            .to_string(),
                    )],
                    is_async: false,
                },
            ],
        },
        RustItem::Impl {
            target: "__SifrYielder<T>".to_string(),
            type_params: vec![type_param("T", &[])],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "suspend".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![
                    RustParam::SelfParam { mutable: false },
                    RustParam::Named {
                        name: "value".to_string(),
                        ty: RustType::Named("T".to_string()),
                    },
                ],
                ret: Some(RustType::Named("__SifrYieldFuture<T>".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                    name: "__SifrYieldFuture".to_string(),
                    fields: vec![
                        (
                            "slot".to_string(),
                            RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "std".to_string(),
                                    "sync".to_string(),
                                    "Arc".to_string(),
                                    "clone".to_string(),
                                ])),
                                args: vec![RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("self".to_string())),
                                        field: "slot".to_string(),
                                    }),
                                }],
                            },
                        ),
                        (
                            "value".to_string(),
                            RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![RustExpr::Ident("value".to_string())],
                            },
                        ),
                    ],
                }))],
                is_async: false,
            }],
        },
        RustItem::Fn {
            name: "__sifr_store_suspended".to_string(),
            visibility: Visibility::Private,
            type_params: vec![type_param("T", &[])],
            params: vec![
                RustParam::Named {
                    name: "slot".to_string(),
                    ty: RustType::Named(
                        "&std::sync::Arc<std::sync::Mutex<Option<T>>>".to_string(),
                    ),
                },
                RustParam::Named {
                    name: "value".to_string(),
                    ty: RustType::Named("T".to_string()),
                },
            ],
            ret: None,
            body: vec![RustStmt::Verbatim(
                "match slot.lock() {\n    Ok(mut state) => *state = Some(value),\n    Err(poisoned) => *poisoned.into_inner() = Some(value),\n}"
                    .to_string(),
            )],
            is_async: false,
        },
        RustItem::Fn {
            name: "__sifr_take_suspended".to_string(),
            visibility: Visibility::Private,
            type_params: vec![type_param("T", &[])],
            params: vec![RustParam::Named {
                name: "slot".to_string(),
                ty: RustType::Named(
                    "&std::sync::Arc<std::sync::Mutex<Option<T>>>".to_string(),
                ),
            }],
            ret: Some(RustType::Option(Box::new(RustType::Named("T".to_string())))),
            body: vec![RustStmt::Verbatim(
                "match slot.lock() {\n    Ok(mut state) => state.take(),\n    Err(poisoned) => poisoned.into_inner().take(),\n}"
                    .to_string(),
            )],
            is_async: false,
        },
    ]
}

fn sync_generator_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "__SifrGenerator<T>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "producer".to_string(),
                    RustType::Named(
                        "Option<std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'static>>>"
                            .to_string(),
                    ),
                ),
                (
                    "yielded".to_string(),
                    RustType::Named("std::sync::Arc<std::sync::Mutex<Option<T>>>".to_string()),
                ),
                ("complete".to_string(), RustType::Bool),
            ],
        },
        RustItem::Impl {
            target: "__SifrGenerator<T>".to_string(),
            type_params: vec![type_param("T", &[])],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "new".to_string(),
                visibility: Visibility::Private,
                type_params: vec![
                    type_param("F", &["FnOnce(__SifrYielder<T>) -> Fut", "'static"]),
                    type_param("Fut", &["std::future::Future<Output = ()>", "'static"]),
                ],
                params: vec![RustParam::Named {
                    name: "factory".to_string(),
                    ty: RustType::Named("F".to_string()),
                }],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Verbatim(
                    "let yielded = std::sync::Arc::new(std::sync::Mutex::new(None));\nlet producer = factory(__SifrYielder {\n    slot: std::sync::Arc::clone(&yielded),\n});\nSelf {\n    producer: Some(Box::pin(producer)),\n    yielded,\n    complete: false,\n}"
                        .to_string(),
                )],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: "__SifrGenerator<T>".to_string(),
            type_params: vec![type_param("T", &[])],
            trait_: Some("Iterator".to_string()),
            items: vec![
                RustItem::TypeAlias {
                    name: "Item".to_string(),
                    ty: RustType::Named("T".to_string()),
                },
                RustItem::Fn {
                    name: "next".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: true }],
                    ret: Some(RustType::Option(Box::new(RustType::Named("T".to_string())))),
                    body: vec![RustStmt::Verbatim(
                        "if self.complete {\n    return None;\n}\nlet completed = {\n    let Some(producer) = self.producer.as_mut() else {\n        self.complete = true;\n        return None;\n    };\n    let mut context = std::task::Context::from_waker(std::task::Waker::noop());\n    std::future::Future::poll(producer.as_mut(), &mut context).is_ready()\n};\nlet yielded = __sifr_take_suspended(&self.yielded);\nif completed {\n    self.complete = true;\n    self.producer = None;\n}\nyielded"
                            .to_string(),
                    )],
                    is_async: false,
                },
            ],
        },
    ]
}

fn async_generator_items() -> Vec<RustItem> {
    vec![
        RustItem::TypeAlias {
            name: "__SifrAsyncProducer<E>".to_string(),
            ty: RustType::Named("std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), E>> + Send + 'static>>".to_string()),
        },
        RustItem::Struct {
            name: "AsyncGenerator<T, E>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "producer".to_string(),
                    RustType::Named(
                        "Option<__SifrAsyncProducer<E>>"
                            .to_string(),
                    ),
                ),
                (
                    "yielded".to_string(),
                    RustType::Named("std::sync::Arc<std::sync::Mutex<Option<T>>>".to_string()),
                ),
                ("closed".to_string(), RustType::Bool),
            ],
        },
        RustItem::Impl {
            target: "AsyncGenerator<T, E>".to_string(),
            type_params: vec![type_param("T", &[]), type_param("E", &[])],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "new_lazy".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![
                        type_param("F", &["FnOnce(__SifrYielder<T>) -> Fut", "Send", "'static"]),
                        type_param(
                            "Fut",
                            &["std::future::Future<Output = Result<(), E>>", "Send", "'static"],
                        ),
                    ],
                    params: vec![RustParam::Named {
                        name: "factory".to_string(),
                        ty: RustType::Named("F".to_string()),
                    }],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Verbatim(
                        "let yielded = std::sync::Arc::new(std::sync::Mutex::new(None));\nlet producer = factory(__SifrYielder {\n    slot: std::sync::Arc::clone(&yielded),\n});\nSelf {\n    producer: Some(Box::pin(producer)),\n    yielded,\n    closed: false,\n}"
                            .to_string(),
                    )],
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
                    body: vec![RustStmt::Verbatim(
                        "if self.closed {\n    return Ok(None);\n}\nlet yielded = std::sync::Arc::clone(&self.yielded);\nlet outcome = {\n    let Some(producer) = self.producer.as_mut() else {\n        self.closed = true;\n        return Ok(None);\n    };\n    std::future::poll_fn(|context| {\n        match std::future::Future::poll(producer.as_mut(), context) {\n            std::task::Poll::Ready(Ok(())) => std::task::Poll::Ready(Ok(None)),\n            std::task::Poll::Ready(Err(error)) => std::task::Poll::Ready(Err(error)),\n            std::task::Poll::Pending => match __sifr_take_suspended(&yielded) {\n                Some(value) => std::task::Poll::Ready(Ok(Some(value))),\n                None => std::task::Poll::Pending,\n            },\n        }\n    })\n    .await\n};\nif !outcome.as_ref().is_ok_and(Option::is_some) {\n    self.closed = true;\n    self.producer = None;\n}\noutcome"
                            .to_string(),
                    )],
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
                    body: vec![RustStmt::Verbatim(
                        "self.closed = true;\nself.producer = None;\nlet _ = __sifr_take_suspended(&self.yielded);\nOk(())"
                            .to_string(),
                    )],
                    is_async: true,
                },
            ],
        },
    ]
}

pub fn build_generator_runtime_items(
    include_common: bool,
    uses_sync_generator: bool,
    uses_async_generator: bool,
) -> Vec<RustItem> {
    let mut items = if include_common {
        common_generator_items()
    } else {
        Vec::new()
    };
    if uses_sync_generator {
        items.extend(sync_generator_items());
    }
    if uses_async_generator {
        items.extend(async_generator_items());
    }
    items
}
