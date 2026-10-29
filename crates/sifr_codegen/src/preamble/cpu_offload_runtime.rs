use super::{RustItem, RustParam, RustStmt, RustType, Visibility};

pub fn build_worker_panic_hook_items() -> Vec<RustItem> {
    vec![
        RustItem::Use(vec![
            "sifr_runtime".to_string(),
            "interop".to_string(),
            "SilentPanicBoundary".to_string(),
        ]),
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
                    bounds: vec!["FnOnce(&SilentPanicBoundary) -> T".to_string()],
                },
            ],
            params: vec![RustParam::Named {
                name: "body".to_string(),
                ty: RustType::Named("F".to_string()),
            }],
            ret: Some(RustType::Named("T".to_string())),
            body: vec![RustStmt::Verbatim(
                "let __sifr_panic_boundary = SilentPanicBoundary::enter();\n        return match __sifr_panic_boundary.catch_unwind(|| body(&__sifr_panic_boundary)) {\n            Ok(value) => value,\n            Err(payload) => std::panic::resume_unwind(payload),\n        }"
                    .to_string(),
            )],
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
            body: vec![RustStmt::Verbatim(
                "let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n        let handle = tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|_| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(value) => __SifrTaskResult::Ok(value),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerRuntimeError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerRuntimeError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            }\n        });\n        return __SifrBlockingTask { handle: Some(handle), observed, _error: std::marker::PhantomData }"
                    .to_string(),
            )],
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
            body: vec![RustStmt::Verbatim(
                "let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n        let handle = tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|_| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(Ok(value)) => __SifrTaskResult::Ok(value),\n                            Ok(Err(error)) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"{}\", error)))),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            }\n        });\n        return __SifrBlockingTask { handle: Some(handle), observed, _error: std::marker::PhantomData }"
                    .to_string(),
            )],
            is_async: false,
        },
    ]
}

#[cfg(test)]
#[path = "../../tests/unit/preamble/cpu_offload_runtime/tests.rs"]
mod tests;
