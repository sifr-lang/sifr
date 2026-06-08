use super::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};
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
                ("abort_handle".to_string(), RustType::Named("tokio::task::AbortHandle".to_string())),
                ("observed".to_string(), RustType::Named("std::sync::Arc<std::sync::atomic::AtomicBool>".to_string())),
                ("_error".to_string(), RustType::Named("std::marker::PhantomData<E>".to_string())),
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
                ("observed".to_string(), RustType::Named("std::sync::Arc<std::sync::atomic::AtomicBool>".to_string())),
                ("_error".to_string(), RustType::Named("std::marker::PhantomData<E>".to_string())),
            ],
        },
        RustItem::Struct {
            name: "__SifrScopeChild".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                ("handle".to_string(), RustType::Named("tokio::task::JoinHandle<__SifrScopeChildOutcome>".to_string())),
                ("observed".to_string(), RustType::Named("std::sync::Arc<std::sync::atomic::AtomicBool>".to_string())),
                ("start_on_join".to_string(), RustType::Named("Option<tokio::sync::oneshot::Sender<()>>".to_string())),
                ("stop_on_fail_fast".to_string(), RustType::Named("Option<Box<dyn FnOnce() + Send + 'static>>".to_string())),
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
                                    ("start_on_join".to_string(), RustExpr::Ident("None".to_string())),
                                    ("stop_on_fail_fast".to_string(), RustExpr::Ident("None".to_string())),
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
                                    ("start_on_join".to_string(), RustExpr::Ident("None".to_string())),
                                    ("stop_on_fail_fast".to_string(), RustExpr::Ident("None".to_string())),
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
            let mut stop_on_fail_fast = Vec::new();
            for mut child in self.children.drain(..) {
                if let Some(start) = child.start_on_join.take() {
                    let _ = start.send(());
                }
                if let Some(stop_child) = child.stop_on_fail_fast.take() {
                    stop_on_fail_fast.push(stop_child);
                } else {
                    abort_handles.push(child.handle.abort_handle());
                }
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
                    while let Some(stop_child) = stop_on_fail_fast.pop() {
                        stop_child();
                    }
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
        while let Some(mut child) = self.children.pop() {
            if let Some(start) = child.start_on_join.take() {
                let _ = start.send(());
            }
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
