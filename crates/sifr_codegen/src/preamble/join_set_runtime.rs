use super::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};

pub(crate) fn build_join_set_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "JoinItemId".to_string(),
            visibility: Visibility::Private,
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "Copy".to_string(),
                "PartialEq".to_string(),
                "Eq".to_string(),
                "Hash".to_string(),
            ],
            fields: vec![("value".to_string(), RustType::I64)],
        },
        RustItem::Impl {
            target: "JoinItemId".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "new".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![RustParam::Named {
                    name: "value".to_string(),
                    ty: RustType::I64,
                }],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                    name: "Self".to_string(),
                    fields: vec![("value".to_string(), RustExpr::Ident("value".to_string()))],
                }))],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: "JoinItemId".to_string(),
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
                            inner: Box::new(RustType::Named(
                                "std::fmt::Formatter<'_>".to_string(),
                            )),
                        },
                    },
                ],
                ret: Some(RustType::Named("std::fmt::Result".to_string())),
                body: vec![RustStmt::compiler_fragment(
                    "return write!(f, \"{}\", self.value)".to_string(),
                )],
                is_async: false,
            }],
        },
        RustItem::Enum {
            name: "CancelOutcome".to_string(),
            visibility: Visibility::Private,
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
                "Eq".to_string(),
                "Hash".to_string(),
            ],
            repr: None,
            variants: vec![
                crate::RustEnumVariant {
                    name: "Cancelled".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "AlreadyCompleted".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "AlreadyFailed".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "AlreadyStarted".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "CouldNotCancel".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "CancelFailed".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
                crate::RustEnumVariant {
                    name: "TimedOutDuringCancel".to_string(),
                    tuple_fields: vec![],
                    fields: vec![],
                    value: None,
                },
            ],
        },
        RustItem::Impl {
            target: "CancelOutcome".to_string(),
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
                            inner: Box::new(RustType::Named(
                                "std::fmt::Formatter<'_>".to_string(),
                            )),
                        },
                    },
                ],
                ret: Some(RustType::Named("std::fmt::Result".to_string())),
                body: vec![RustStmt::compiler_fragment(
                    "return write!(f, \"{}\", match self {\n            CancelOutcome::Cancelled => \"Cancelled\",\n            CancelOutcome::AlreadyCompleted => \"AlreadyCompleted\",\n            CancelOutcome::AlreadyFailed => \"AlreadyFailed\",\n            CancelOutcome::AlreadyStarted => \"AlreadyStarted\",\n            CancelOutcome::CouldNotCancel => \"CouldNotCancel\",\n            CancelOutcome::CancelFailed => \"CancelFailed\",\n            CancelOutcome::TimedOutDuringCancel => \"TimedOutDuringCancel\",\n        })"
                        .to_string(),
                )],
                is_async: false,
            }],
        },
        RustItem::Struct {
            name: "__SifrJoinEntry<T, E>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                ("id".to_string(), RustType::Named("JoinItemId".to_string())),
                (
                    "handle".to_string(),
                    RustType::Named(
                        "tokio::task::JoinHandle<__SifrTaskResult<T, E>>".to_string(),
                    ),
                ),
                (
                    "cancellation".to_string(),
                    RustType::Named("Option<__SifrCancellationCarrier>".to_string()),
                ),
                (
                    "blocking_abort".to_string(),
                    RustType::Named("Option<tokio::task::AbortHandle>".to_string()),
                ),
            ],
        },
        RustItem::Struct {
            name: "__SifrJoinSet<T, E>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                (
                    "entries".to_string(),
                    RustType::Vec(Box::new(RustType::Named(
                        "__SifrJoinEntry<T, E>".to_string(),
                    ))),
                ),
                ("next_id".to_string(), RustType::I64),
                (
                    "_error".to_string(),
                    RustType::Named("std::marker::PhantomData<E>".to_string()),
                ),
            ],
        },
        RustItem::Fn {
            name: "__sifr_join_set_new".to_string(),
            visibility: Visibility::Private,
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
            params: vec![],
            ret: Some(RustType::Named("__SifrJoinSet<T, E>".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                name: "__SifrJoinSet::<T, E>".to_string(),
                fields: vec![
                    (
                        "entries".to_string(),
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                            args: vec![],
                        },
                    ),
                    ("next_id".to_string(), RustExpr::Literal(crate::RustLiteral::Int(0))),
                    (
                        "_error".to_string(),
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
        RustItem::Impl {
            target: "__SifrJoinSet<T, E>".to_string(),
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
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "__sifr_add_task".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::SelfParam { mutable: true },
                        RustParam::Named {
                            name: "task".to_string(),
                            ty: RustType::Named("__SifrTask<T, E>".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named("JoinItemId".to_string())),
                    body: vec![RustStmt::compiler_fragment(
                        "let id = self.__sifr_next_id();\n        let __SifrTask { receiver, cancellation, observed, _error } = task;\n        observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        let handle = tokio::spawn(async move {\n            let Some(receiver) = receiver else {\n                return __SifrTaskResult::cancelled();\n            };\n            return match receiver.await {\n                Ok(result) => result,\n                Err(_) => __SifrTaskResult::cancelled(),\n            };\n        });\n        self.entries.push(__SifrJoinEntry { id, handle, cancellation: Some(cancellation), blocking_abort: None });\n        return id"
                            .to_string(),
                    )],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__sifr_add_blocking_task".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::SelfParam { mutable: true },
                        RustParam::Named {
                            name: "task".to_string(),
                            ty: RustType::Named("__SifrBlockingTask<T, E>".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named("JoinItemId".to_string())),
                    body: vec![RustStmt::compiler_fragment(
                        "let id = self.__sifr_next_id();\n        let __SifrBlockingTask { handle, observed, _error } = task;\n        observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        let abort_handle = handle.as_ref().map(tokio::task::JoinHandle::abort_handle);\n        let handle = tokio::spawn(async move {\n            let Some(handle) = handle else {\n                return __SifrTaskResult::cancelled();\n            };\n            return match handle.await {\n                Ok(result) => result,\n                Err(_) => __SifrTaskResult::cancelled(),\n            };\n        });\n        self.entries.push(__SifrJoinEntry { id, handle, cancellation: None, blocking_abort: abort_handle });\n        return id"
                            .to_string(),
                    )],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__sifr_spawn_blocking".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![
                        crate::RustTypeParam {
                            name: "F".to_string(),
                            bounds: vec![
                                "FnOnce() -> Result<T, E>".to_string(),
                                "Send".to_string(),
                                "'static".to_string(),
                            ],
                        },
                    ],
                    params: vec![
                        RustParam::SelfParam { mutable: true },
                        RustParam::Named {
                            name: "work".to_string(),
                            ty: RustType::Named("F".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named("JoinItemId".to_string())),
                    body: vec![RustStmt::compiler_fragment(
                        "let id = self.__sifr_next_id();\n        let handle = tokio::task::spawn_blocking(move || {\n            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                Ok(Ok(value)) => __SifrTaskResult::Ok(value),\n                Ok(Err(error)) => __SifrTaskResult::Err(__SifrFailure::new(error)),\n                Err(_) => __SifrTaskResult::cancelled(),\n            }\n        });\n        let abort_handle = Some(handle.abort_handle());\n        self.entries.push(__SifrJoinEntry { id, handle, cancellation: None, blocking_abort: abort_handle });\n        return id"
                            .to_string(),
                    )],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__sifr_join_all".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfValue],
                    ret: Some(RustType::Named("Vec<__SifrTaskResult<T, E>>".to_string())),
                    body: vec![RustStmt::compiler_fragment(
                        "let mut results = Vec::with_capacity(self.entries.len());\n        for entry in self.entries {\n            match entry.handle.await {\n                Ok(result) => results.push(result),\n                Err(join_error) if join_error.is_cancelled() => results.push(__SifrTaskResult::cancelled()),\n                Err(_) => results.push(__SifrTaskResult::cancelled()),\n            }\n        }\n        return results"
                            .to_string(),
                    )],
                    is_async: true,
                },
                RustItem::Fn {
                    name: "__sifr_cancel_all".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfValue],
                    ret: Some(RustType::Named("Vec<CancelOutcome>".to_string())),
                    body: vec![RustStmt::compiler_fragment(
                        "let mut outcomes = Vec::with_capacity(self.entries.len());\n        for entry in self.entries {\n            let was_finished = entry.handle.is_finished();\n            if let Some(cancellation) = entry.cancellation {\n                let _ = cancellation.request_cancel();\n            } else if let Some(abort_handle) = entry.blocking_abort {\n                abort_handle.abort();\n            } else {\n                entry.handle.abort();\n            }\n            match entry.handle.await {\n                Ok(__SifrTaskResult::Ok(_)) => outcomes.push(CancelOutcome::AlreadyCompleted),\n                Ok(__SifrTaskResult::Err(_)) => outcomes.push(CancelOutcome::AlreadyFailed),\n                Ok(__SifrTaskResult::Cancelled(_)) => outcomes.push(CancelOutcome::Cancelled),\n                Err(join_error) if join_error.is_cancelled() => outcomes.push(if was_finished { CancelOutcome::AlreadyStarted } else { CancelOutcome::Cancelled }),\n                Err(_) => outcomes.push(CancelOutcome::CancelFailed),\n            }\n        }\n        return outcomes"
                            .to_string(),
                    )],
                    is_async: true,
                },
                RustItem::Fn {
                    name: "__sifr_next_id".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: true }],
                    ret: Some(RustType::Named("JoinItemId".to_string())),
                    body: vec![RustStmt::compiler_fragment(
                        "let id = JoinItemId::new(self.next_id);\n        self.next_id = self.next_id.saturating_add(1);\n        return id"
                            .to_string(),
                    )],
                    is_async: false,
                },
            ],
        },
    ]
}

pub(crate) fn build_join_set_cpu_items() -> Vec<RustItem> {
    vec![
        RustItem::Impl {
            target: "__SifrJoinSet<T, WorkerError>".to_string(),
            type_params: vec![crate::RustTypeParam {
                name: "T".to_string(),
                bounds: vec!["Send".to_string(), "'static".to_string()],
            }],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "__sifr_spawn_cpu".to_string(),
                visibility: Visibility::Private,
                type_params: vec![
                    crate::RustTypeParam {
                        name: "E".to_string(),
                        bounds: vec![
                            "Send".to_string(),
                            "std::fmt::Display".to_string(),
                            "'static".to_string(),
                        ],
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
                params: vec![
                    RustParam::SelfParam { mutable: true },
                    RustParam::Named {
                        name: "work".to_string(),
                        ty: RustType::Named("F".to_string()),
                    },
                ],
                ret: Some(RustType::Named("JoinItemId".to_string())),
                body: vec![RustStmt::compiler_fragment(
                    "let id = self.__sifr_next_id();\n        let handle = tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|_| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(Ok(value)) => __SifrTaskResult::Ok(value),\n                            Ok(Err(error)) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"{}\", error)))),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            }\n        });\n        let abort_handle = Some(handle.abort_handle());\n        self.entries.push(__SifrJoinEntry { id, handle, cancellation: None, blocking_abort: abort_handle });\n        return id"
                        .to_string(),
                )],
                is_async: false,
            }],
        },
    ]
}
