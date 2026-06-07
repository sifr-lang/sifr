use super::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};

pub fn build_task_scope_offload_items() -> Vec<RustItem> {
    vec![RustItem::Impl {
        target: "__SifrTaskScope".to_string(),
        type_params: vec![],
        trait_: None,
        items: vec![
            RustItem::Fn {
                name: "__sifr_scope_spawn_blocking_infallible".to_string(),
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
                params: scoped_worker_params(),
                ret: Some(RustType::Named(
                    "__SifrTask<T, std::convert::Infallible>".to_string(),
                )),
                body: scoped_task_body(
                    "tokio::task::spawn_blocking(move || {\n            let result = __SifrTaskResult::Ok(work());\n            let _ = sender.send(result);\n            __SifrScopeChildOutcome::Ok\n        })",
                ),
                is_async: false,
            },
            RustItem::Fn {
                name: "__sifr_scope_spawn_blocking_result".to_string(),
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
                params: scoped_worker_params(),
                ret: Some(RustType::Named("__SifrTask<T, E>".to_string())),
                body: scoped_task_body(
                    "tokio::task::spawn_blocking(move || {\n            let result = match work() {\n                Ok(value) => __SifrTaskResult::Ok(value),\n                Err(err) => __SifrTaskResult::Err(__SifrFailure::new(err)),\n            };\n            let outcome = match &result {\n                __SifrTaskResult::Ok(_) => __SifrScopeChildOutcome::Ok,\n                __SifrTaskResult::Err(_) => __SifrScopeChildOutcome::Failed,\n                __SifrTaskResult::Cancelled(_) => __SifrScopeChildOutcome::Cancelled,\n            };\n            let _ = sender.send(result);\n            outcome\n        })",
                ),
                is_async: false,
            },
        ],
    }]
}

pub fn build_task_scope_cpu_offload_items() -> Vec<RustItem> {
    vec![RustItem::Impl {
            target: "__SifrTaskScope".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "__sifr_scope_spawn_cpu_infallible".to_string(),
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
                    params: scoped_worker_params(),
                    ret: Some(RustType::Named("__SifrTask<T, WorkerRuntimeError>".to_string())),
                    body: scoped_task_body(
                        "tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            let result = match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(value) => __SifrTaskResult::Ok(value),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerRuntimeError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerRuntimeError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            };\n            let outcome = match &result {\n                __SifrTaskResult::Ok(_) => __SifrScopeChildOutcome::Ok,\n                __SifrTaskResult::Err(_) => __SifrScopeChildOutcome::Failed,\n                __SifrTaskResult::Cancelled(_) => __SifrScopeChildOutcome::Cancelled,\n            };\n            let _ = sender.send(result);\n            outcome\n        })",
                    ),
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__sifr_scope_spawn_cpu_result".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![
                        crate::RustTypeParam {
                            name: "T".to_string(),
                            bounds: vec!["Send".to_string(), "'static".to_string()],
                        },
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
                    params: scoped_worker_params(),
                    ret: Some(RustType::Named("__SifrTask<T, WorkerError>".to_string())),
                    body: scoped_task_body(
                        "tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            let result = match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(Ok(value)) => __SifrTaskResult::Ok(value),\n                            Ok(Err(error)) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"{}\", error)))),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            };\n            let outcome = match &result {\n                __SifrTaskResult::Ok(_) => __SifrScopeChildOutcome::Ok,\n                __SifrTaskResult::Err(_) => __SifrScopeChildOutcome::Failed,\n                __SifrTaskResult::Cancelled(_) => __SifrScopeChildOutcome::Cancelled,\n            };\n            let _ = sender.send(result);\n            outcome\n        })",
                    ),
                    is_async: false,
                },
            ],
        }]
}

fn scoped_worker_params() -> Vec<RustParam> {
    vec![
        RustParam::SelfParam { mutable: true },
        RustParam::Named {
            name: "work".to_string(),
            ty: RustType::Named("F".to_string()),
        },
    ]
}

fn scoped_task_body(child_expr: &str) -> Vec<RustStmt> {
    vec![RustStmt::Expr(RustExpr::Ident(format!(
        "let (sender, receiver) = tokio::sync::oneshot::channel();\n        let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n        let child_observed = std::sync::Arc::clone(&observed);\n        let child = {child_expr};\n        let abort_handle = child.abort_handle();\n        self.children.push(__SifrScopeChild {{ handle: child, observed: child_observed }});\n        return __SifrTask {{ receiver: Some(receiver), abort_handle, observed, _error: std::marker::PhantomData }}"
    )))]
}
