use super::{RustItem, RustParam, RustStmt, RustType, Visibility};
use std::collections::HashMap;

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

pub fn build_task_scope_process_items() -> Vec<RustItem> {
    let command_type = sifr_type_system::stdlib_class_rust_name("sifr.process", "Command");
    let process_handle_type =
        sifr_type_system::stdlib_class_rust_name("sifr.process", "ProcessHandle");
    let process_error_type =
        sifr_type_system::stdlib_class_rust_name("_sifr.process", "ProcessError");
    vec![RustItem::Impl {
        target: "__SifrTaskScope".to_string(),
        type_params: vec![],
        trait_: None,
        items: vec![RustItem::Fn {
            name: "__sifr_scope_spawn_process".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![
                RustParam::SelfParam { mutable: true },
                RustParam::Named {
                    name: "command".to_string(),
                    ty: RustType::Named(command_type.clone()),
                },
            ],
            ret: Some(RustType::Named(format!(
                "Result<{process_handle_type}, {process_error_type}>"
            ))),
            body: scoped_process_body(&command_type, &process_handle_type, &process_error_type),
            is_async: false,
        }],
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
                ret: Some(RustType::Named(
                    "__SifrTask<T, WorkerRuntimeError>".to_string(),
                )),
                body: scoped_task_body(
                    "tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            let result = match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|_| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(value) => __SifrTaskResult::Ok(value),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerRuntimeError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerRuntimeError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            };\n            let outcome = match &result {\n                __SifrTaskResult::Ok(_) => __SifrScopeChildOutcome::Ok,\n                __SifrTaskResult::Err(_) => __SifrScopeChildOutcome::Failed,\n                __SifrTaskResult::Cancelled(_) => __SifrScopeChildOutcome::Cancelled,\n            };\n            let _ = sender.send(result);\n            outcome\n        })",
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
                    "tokio::task::spawn_blocking(move || {\n            let workers = std::thread::available_parallelism().map_or(1usize, std::num::NonZeroUsize::get);\n            let pool = rayon::ThreadPoolBuilder::new().num_threads(workers).build();\n            let result = match pool {\n                Ok(pool) => pool.install(|| {\n                    __sifr_with_silent_worker_panic_hook(|_| {\n                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {\n                            Ok(Ok(value)) => __SifrTaskResult::Ok(value),\n                            Ok(Err(error)) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"{}\", error)))),\n                            Err(_) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(\"cpu worker panicked\".to_string()))),\n                        }\n                    })\n                }),\n                Err(error) => __SifrTaskResult::Err(__SifrFailure::new(WorkerError::new(format!(\"cpu worker pool could not start: {}\", error)))),\n            };\n            let outcome = match &result {\n                __SifrTaskResult::Ok(_) => __SifrScopeChildOutcome::Ok,\n                __SifrTaskResult::Err(_) => __SifrScopeChildOutcome::Failed,\n                __SifrTaskResult::Cancelled(_) => __SifrScopeChildOutcome::Cancelled,\n            };\n            let _ = sender.send(result);\n            outcome\n        })",
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
    vec![RustStmt::compiler_fragment(format!(
        "let (sender, receiver) = tokio::sync::oneshot::channel();\n        let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n        let child_observed = std::sync::Arc::clone(&observed);\n        let child = {child_expr};\n        let cancellation_inner = ::sifr_runtime::cancellation::CancellationCarrier::new();\n        let cancellation = __SifrCancellationCarrier::new(cancellation_inner, child.abort_handle());\n        self.children.push(__SifrScopeChild {{ handle: child, cancellation: Some(cancellation.clone()), observed: child_observed, start_on_join: None, stop_on_fail_fast: None }});\n        return __SifrTask {{ receiver: Some(receiver), cancellation, observed, _error: std::marker::PhantomData }}"
    ))]
}

fn scoped_process_body(
    command_type: &str,
    process_handle_type: &str,
    process_error_type: &str,
) -> Vec<RustStmt> {
    let replacements = HashMap::from([
        ("Command".to_string(), command_type.to_string()),
        ("__SifrTokioCommand".to_string(), "Command".to_string()),
        ("ProcessHandle".to_string(), process_handle_type.to_string()),
        ("ProcessError".to_string(), process_error_type.to_string()),
    ]);
    vec![RustStmt::compiler_fragment(
        crate::stdlib_filter::rewrite_rust_identifiers(
            "if command.has_stdin_data {
            return Err(ProcessError { message: \"scoped process spawn does not consume Command.stdin_bytes; use stdin(Stdio(\\\"pipe\\\")) and ProcessHandle.stdin()\".to_string() });
        }
        fn __sifr_scoped_process_stdio_from_mode(mode: &str) -> Result<std::process::Stdio, ProcessError> {
            match mode {
                \"pipe\" => Ok(std::process::Stdio::piped()),
                \"inherit\" => Ok(std::process::Stdio::inherit()),
                \"null\" => Ok(std::process::Stdio::null()),
                _ => Err(ProcessError { message: format!(\"unsupported scoped process stdio mode: {}\", mode) }),
            }
        }
        let mut __cmd = tokio::process::__SifrTokioCommand::new(&command.program);
        for __arg in &command.arguments {
            __cmd.arg(__arg);
        }
        for __entry in &command.env_vars {
            if let Some((__key, __value)) = __entry.split_once('=') {
                __cmd.env(__key, __value);
            }
        }
        if command.has_working_dir {
            __cmd.current_dir(&command.working_dir);
        }
        __cmd.stdin(__sifr_scoped_process_stdio_from_mode(&command.stdin_mode)?);
        __cmd.stdout(__sifr_scoped_process_stdio_from_mode(&command.stdout_mode)?);
        __cmd.stderr(__sifr_scoped_process_stdio_from_mode(&command.stderr_mode)?);
        let __child = __cmd.spawn().map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
        let (__handle, __observed) = ::sifr_stdlib::process::process_async_register_scoped_child(__child);
        let (__start_sender, __start_receiver) = tokio::sync::oneshot::channel();
        let (__stop_sender, mut __stop_receiver) = tokio::sync::oneshot::channel();
        let __child_observed = std::sync::Arc::clone(&__observed);
        let __observer_observed = std::sync::Arc::clone(&__observed);
        let __observer_handle = __handle;
        let __observer = tokio::spawn(async move {
            let _ = __start_receiver.await;
            let __child = ::sifr_stdlib::process::process_async_take_child(__observer_handle);
            let __outcome = match __child {
                Some(mut __child) => {
                    let __wait_result = tokio::select! {
                        __status = __child.wait() => __status.map(|__status| (false, __status)),
                        _ = &mut __stop_receiver => {
                            let _ = __child.start_kill();
                            __child.wait().await.map(|__status| (true, __status))
                        }
                    };
                    match __wait_result {
                        Ok((true, _)) => __SifrScopeChildOutcome::Cancelled,
                        Ok((false, __status)) if __status.success() => __SifrScopeChildOutcome::Ok,
                        Ok((false, _)) => __SifrScopeChildOutcome::Failed,
                        Err(_) => __SifrScopeChildOutcome::Failed,
                    }
                }
                None if __observer_observed.load(std::sync::atomic::Ordering::SeqCst) => __SifrScopeChildOutcome::Ok,
                None => __SifrScopeChildOutcome::Failed,
            };
            ::sifr_stdlib::process::process_async_remove_observed(__observer_handle);
            __outcome
        });
        let __stop_on_fail_fast: Box<dyn FnOnce() + Send + 'static> = Box::new(move || {
            let _ = __stop_sender.send(());
        });
        self.children.push(__SifrScopeChild { handle: __observer, cancellation: None, observed: __child_observed, start_on_join: Some(__start_sender), stop_on_fail_fast: Some(__stop_on_fail_fast) });
        return Ok(ProcessHandle::new(__handle));",
            &replacements,
        ),
    )]
}
