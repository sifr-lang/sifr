use super::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};

pub fn build_worker_panic_hook_items() -> Vec<RustItem> {
    vec![
        RustItem::Static {
            name: "__SIFR_WORKER_PANIC_HOOK_LOCK".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named("std::sync::Mutex<()>".to_string()),
            value: RustExpr::Ident("std::sync::Mutex::new(())".to_string()),
        },
        RustItem::Fn {
            name: "__sifr_with_silent_worker_panic_hook".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec![],
                },
                crate::RustTypeParam {
                    name: "F".to_string(),
                    bounds: vec!["FnOnce() -> T".to_string()],
                },
            ],
            params: vec![RustParam::Named {
                name: "body".to_string(),
                ty: RustType::Named("F".to_string()),
            }],
            ret: Some(RustType::Named("T".to_string())),
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "let _hook_guard = match __SIFR_WORKER_PANIC_HOOK_LOCK.lock() {\n            Ok(guard) => guard,\n            Err(poisoned) => poisoned.into_inner(),\n        };\n        let previous_hook = std::panic::take_hook();\n        std::panic::set_hook(Box::new(|_| {}));\n        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));\n        std::panic::set_hook(previous_hook);\n        return match result {\n            Ok(value) => value,\n            Err(payload) => std::panic::resume_unwind(payload),\n        }"
                    .to_string(),
            ))],
            is_async: false,
        },
    ]
}

pub fn build_cpu_offload_items() -> Vec<RustItem> {
    vec![
        RustItem::Fn {
            name: "__sifr_spawn_cpu_infallible".to_string(),
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
                "__SifrBlockingTask<T, WorkerRuntimeError>".to_string(),
            )),
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n        let handle = tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(value) => __SifrTaskResult::Ok(value),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerRuntimeError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerRuntimeError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            }\n        });\n        return __SifrBlockingTask { handle: Some(handle), observed, _error: std::marker::PhantomData }"
                    .to_string(),
            ))],
            is_async: false,
        },
        RustItem::Fn {
            name: "__sifr_spawn_cpu_result".to_string(),
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
            params: vec![RustParam::Named {
                name: "work".to_string(),
                ty: RustType::Named("F".to_string()),
            }],
            ret: Some(RustType::Named("__SifrBlockingTask<T, WorkerError>".to_string())),
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n        let handle = tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(Ok(value)) => __SifrTaskResult::Ok(value),\n                            Ok(Err(error)) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"{}\", error)))),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            }\n        });\n        return __SifrBlockingTask { handle: Some(handle), observed, _error: std::marker::PhantomData }"
                    .to_string(),
            ))],
            is_async: false,
        },
    ]
}
