//! Runtime support builders for generated async child-process helpers.

use crate::{RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, Visibility};

fn string_ty() -> RustType {
    RustType::String_
}

pub(super) fn process_async_spawn_params() -> Vec<RustParam> {
    vec![
        RustParam::Named {
            name: "program".to_string(),
            ty: string_ty(),
        },
        RustParam::Named {
            name: "args".to_string(),
            ty: RustType::Vec(Box::new(string_ty())),
        },
        RustParam::Named {
            name: "env".to_string(),
            ty: RustType::Vec(Box::new(string_ty())),
        },
        RustParam::Named {
            name: "cwd".to_string(),
            ty: string_ty(),
        },
        RustParam::Named {
            name: "has_cwd".to_string(),
            ty: RustType::Bool,
        },
        RustParam::Named {
            name: "stdin_mode".to_string(),
            ty: string_ty(),
        },
        RustParam::Named {
            name: "stdout_mode".to_string(),
            ty: string_ty(),
        },
        RustParam::Named {
            name: "stderr_mode".to_string(),
            ty: string_ty(),
        },
        RustParam::Named {
            name: "has_stdin".to_string(),
            ty: RustType::Bool,
        },
    ]
}

pub(super) fn process_async_wait_params() -> Vec<RustParam> {
    vec![RustParam::Named {
        name: "handle".to_string(),
        ty: RustType::I64,
    }]
}

#[derive(Clone, Copy)]
pub(super) struct ProcessAsyncChildTableNeeds {
    pub(super) spawn: bool,
    pub(super) wait: bool,
    pub(super) kill: bool,
    pub(super) terminate: bool,
}

pub(super) fn process_async_child_table_items(needs: ProcessAsyncChildTableNeeds) -> Vec<RustItem> {
    if !needs.spawn && !needs.wait && !needs.kill && !needs.terminate {
        return Vec::new();
    }
    let mut items = vec![RustItem::Static {
        name: "__SIFR_PROCESS_ASYNC_CHILDREN".to_string(),
        visibility: Visibility::Private,
        ty: RustType::Named(
            "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, tokio::process::Child>>>"
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
        name: "__SIFR_PROCESS_ASYNC_CHILD_OBSERVED".to_string(),
        visibility: Visibility::Private,
        ty: RustType::Named(
            "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, std::sync::Arc<std::sync::atomic::AtomicBool>>>>"
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
    }];

    if needs.spawn {
        items.push(RustItem::Static {
            name: "__SIFR_PROCESS_ASYNC_PIPE_READERS".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, Box<dyn tokio::io::AsyncRead + Unpin + Send>>>>"
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
        });
        items.push(RustItem::Static {
            name: "__SIFR_PROCESS_ASYNC_PIPE_WRITERS".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, Box<dyn tokio::io::AsyncWrite + Unpin + Send>>>>"
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
        });
        items.push(RustItem::Static {
            name: "__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID".to_string(),
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
                args: vec![RustExpr::Literal(RustLiteral::Int(1))],
            },
        });
        items.push(RustItem::Fn {
            name: "__sifr_next_process_async_child_id".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::I64),
            body: vec![RustStmt::Return(Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(
                    "__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID".to_string(),
                )),
                method: "fetch_add".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Int(1)),
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
        });
    }

    items
}

pub(super) fn process_async_spawn_body() -> Vec<RustStmt> {
    vec![RustStmt::Expr(RustExpr::Ident(
        "if has_stdin {
            return Err(ProcessError { message: \"async process spawn does not consume Command.stdin_bytes\".to_string() });
        }"
        .to_string(),
    ))]
}

pub(super) fn process_async_spawn_insert_body() -> RustStmt {
    RustStmt::Expr(RustExpr::Ident(
        "fn __sifr_process_async_stdio_from_mode(mode: &str) -> Result<std::process::Stdio, ProcessError> {
            match mode {
                \"pipe\" => Ok(std::process::Stdio::piped()),
                \"inherit\" => Ok(std::process::Stdio::inherit()),
                \"null\" => Ok(std::process::Stdio::null()),
                _ => Err(ProcessError { message: format!(\"unsupported async process stdio mode: {}\", mode) }),
            }
        }
        __cmd.stdin(__sifr_process_async_stdio_from_mode(&stdin_mode)?);
        __cmd.stdout(__sifr_process_async_stdio_from_mode(&stdout_mode)?);
        __cmd.stderr(__sifr_process_async_stdio_from_mode(&stderr_mode)?);
        let __child = __cmd.spawn().map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
        let __handle = __sifr_next_process_async_child_id();
        {
            let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
            __children.insert(__handle, __child);
        }
        return Ok(AsyncChild::new(__handle));"
            .to_string(),
    ))
}

pub(super) fn process_async_child_pipe_reader_item(
    function_name: &str,
    field_name: &str,
) -> RustItem {
    RustItem::Fn {
        name: function_name.to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: process_async_wait_params(),
        ret: Some(RustType::Named("Result<i64, ProcessError>".to_string())),
        body: vec![RustStmt::Expr(RustExpr::Ident(format!(
            "let __handle = handle;
        let __pipe = {{
            let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
            let __child = __children.get_mut(&__handle).ok_or_else(|| ProcessError {{
                message: format!(\"async process child handle {{}} is closed or unknown\", __handle),
            }})?;
            __child.{field_name}.take().ok_or_else(|| ProcessError {{
                message: format!(\"async process {field_name} pipe is not available or already taken for child handle: {{}}\", __handle),
            }})?
        }};
        let __pipe_handle = __sifr_next_process_async_child_id();
        {{
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_READERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.insert(__pipe_handle, Box::new(__pipe));
        }}
        return Ok(__pipe_handle);"
        )))],
        is_async: false,
    }
}

pub(super) fn process_async_child_pipe_writer_item(function_name: &str) -> RustItem {
    RustItem::Fn {
        name: function_name.to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: process_async_wait_params(),
        ret: Some(RustType::Named("Result<i64, ProcessError>".to_string())),
        body: vec![RustStmt::Expr(RustExpr::Ident(
            "let __handle = handle;
        let __pipe = {
            let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
            let __child = __children.get_mut(&__handle).ok_or_else(|| ProcessError {
                message: format!(\"async process child handle {} is closed or unknown\", __handle),
            })?;
            __child.stdin.take().ok_or_else(|| ProcessError {
                message: format!(\"async process stdin pipe is not available or already taken for child handle: {}\", __handle),
            })?
        };
        let __pipe_handle = __sifr_next_process_async_child_id();
        {
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_WRITERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.insert(__pipe_handle, Box::new(__pipe));
        }
        return Ok(__pipe_handle);"
                .to_string(),
        ))],
        is_async: false,
    }
}

pub(super) fn process_async_pipe_read_all_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_async_pipe_read_all".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: process_async_wait_params(),
        ret: Some(RustType::Named("Result<Vec<u8>, ProcessError>".to_string())),
        body: vec![RustStmt::Expr(RustExpr::Ident(
            "use tokio::io::AsyncReadExt;
        let __handle = handle;
        let mut __pipe = {
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_READERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.remove(&__handle).ok_or_else(|| ProcessError {
                message: format!(\"async process pipe reader handle is closed or unknown: {}\", __handle),
            })?
        };
        let mut __buffer = Vec::new();
        __pipe.read_to_end(&mut __buffer).await.map_err(|__sifr_process_error| ProcessError {
            message: __sifr_process_error.to_string(),
        })?;
        return Ok(__buffer);"
                .to_string(),
        ))],
        is_async: true,
    }
}

pub(super) fn process_async_pipe_read_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_async_pipe_read".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::Named {
                name: "handle".to_string(),
                ty: RustType::I64,
            },
            RustParam::Named {
                name: "max_bytes".to_string(),
                ty: RustType::I64,
            },
        ],
        ret: Some(RustType::Named("Result<Vec<u8>, ProcessError>".to_string())),
        body: vec![RustStmt::Expr(RustExpr::Ident(
            "use tokio::io::AsyncReadExt;
        if max_bytes <= 0 {
            return Err(ProcessError { message: \"async process pipe read size must be positive\".to_string() });
        }
        if max_bytes > 1048576 {
            return Err(ProcessError { message: \"async process pipe read size exceeds 1048576 bytes\".to_string() });
        }
        let __handle = handle;
        let mut __pipe = {
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_READERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.remove(&__handle).ok_or_else(|| ProcessError {
                message: format!(\"async process pipe reader handle is closed or unknown: {}\", __handle),
            })?
        };
        let mut __buffer = vec![0u8; max_bytes as usize];
        let __read = __pipe.read(__buffer.as_mut_slice()).await.map_err(|__sifr_process_error| ProcessError {
            message: __sifr_process_error.to_string(),
        })?;
        __buffer.truncate(__read);
        if __read > 0 {
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_READERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.insert(__handle, __pipe);
        }
        return Ok(__buffer);"
                .to_string(),
        ))],
        is_async: true,
    }
}

pub(super) fn process_async_pipe_reader_close_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_async_pipe_reader_close".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: process_async_wait_params(),
        ret: Some(RustType::Named("Result<(), ProcessError>".to_string())),
        body: vec![RustStmt::Expr(RustExpr::Ident(
            "let __handle = handle;
        let __removed = {
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_READERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.remove(&__handle)
        };
        __removed.ok_or_else(|| ProcessError {
            message: format!(\"async process pipe reader handle is closed or unknown: {}\", __handle),
        })?;
        return Ok(());"
                .to_string(),
        ))],
        is_async: false,
    }
}

pub(super) fn process_async_pipe_write_all_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_async_pipe_write_all".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::Named {
                name: "handle".to_string(),
                ty: RustType::I64,
            },
            RustParam::Named {
                name: "data".to_string(),
                ty: RustType::Vec(Box::new(RustType::Named("u8".to_string()))),
            },
        ],
        ret: Some(RustType::Named("Result<(), ProcessError>".to_string())),
        body: vec![RustStmt::Expr(RustExpr::Ident(
            "use tokio::io::AsyncWriteExt;
        let __handle = handle;
        let mut __pipe = {
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_WRITERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.remove(&__handle).ok_or_else(|| ProcessError {
                message: format!(\"async process pipe writer handle is closed or unknown: {}\", __handle),
            })?
        };
        let __result = __pipe.write_all(data.as_slice()).await.map_err(|__sifr_process_error| ProcessError {
            message: __sifr_process_error.to_string(),
        });
        {
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_WRITERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.insert(__handle, __pipe);
        }
        __result?;
        return Ok(());"
                .to_string(),
        ))],
        is_async: true,
    }
}

pub(super) fn process_async_pipe_close_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_async_pipe_close".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: process_async_wait_params(),
        ret: Some(RustType::Named("Result<(), ProcessError>".to_string())),
        body: vec![RustStmt::Expr(RustExpr::Ident(
            "let __handle = handle;
        let __removed = {
            let mut __pipes = __SIFR_PROCESS_ASYNC_PIPE_WRITERS.lock().unwrap_or_else(|__err| __err.into_inner());
            __pipes.remove(&__handle)
        };
        __removed.ok_or_else(|| ProcessError {
            message: format!(\"async process pipe writer handle is closed or unknown: {}\", __handle),
        })?;
        return Ok(());"
                .to_string(),
        ))],
        is_async: false,
    }
}

pub(super) fn process_async_wait_body() -> Vec<RustStmt> {
    vec![RustStmt::Expr(RustExpr::Ident(
        "struct __SifrAsyncChildWaitGuard {
            handle: i64,
            child: Option<tokio::process::Child>,
        }
        impl Drop for __SifrAsyncChildWaitGuard {
            fn drop(&mut self) {
                if let Some(__child) = self.child.take() {
                    let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
                    __children.insert(self.handle, __child);
                }
            }
        }
        let __child = {
            let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
            __children.remove(&handle).ok_or_else(|| ProcessError {
                message: format!(\"async process child handle {} is closed or unknown\", handle),
            })?
        };
        let mut __guard = __SifrAsyncChildWaitGuard {
            handle,
            child: Some(__child),
        };
        let __status = match __guard.child.as_mut() {
            Some(__child) => __child.wait().await.map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?,
            None => {
                return Err(ProcessError {
                    message: format!(\"async process child handle {} is closed or unknown\", handle),
                });
            }
        };
        let _ = __guard.child.take();
        return Ok(__sifr_process_status_from_exit(
            __status.code().unwrap_or(-1) as i64,
            __sifr_process_exit_signal(&__status),
        ));"
            .to_string(),
    ))]
}

pub(super) fn process_handle_wait_body() -> Vec<RustStmt> {
    vec![RustStmt::Expr(RustExpr::Ident(
        "{
            let __observed = {
                let __observed_children = __SIFR_PROCESS_ASYNC_CHILD_OBSERVED.lock().unwrap_or_else(|__err| __err.into_inner());
                __observed_children.get(&handle).cloned()
            };
            if let Some(__observed) = __observed {
                __observed.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
        let __result = __sifr_process_async_wait(handle).await;
        {
            let mut __observed_children = __SIFR_PROCESS_ASYNC_CHILD_OBSERVED.lock().unwrap_or_else(|__err| __err.into_inner());
            __observed_children.remove(&handle);
        }
        return __result;"
            .to_string(),
    ))]
}

pub(super) fn process_async_kill_body() -> Vec<RustStmt> {
    vec![RustStmt::Expr(RustExpr::Ident(
        "{
            let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
            let __child = __children.get_mut(&handle).ok_or_else(|| ProcessError {
                message: format!(\"async process child handle {} is closed or unknown\", handle),
            })?;
            __child.start_kill().map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
        }
        return Ok(());"
            .to_string(),
    ))]
}

pub(super) fn process_async_terminate_body() -> Vec<RustStmt> {
    vec![RustStmt::Expr(RustExpr::Ident(
        "#[cfg(unix)]
        {
            let __pid = {
                let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
                let __child = __children.get_mut(&handle).ok_or_else(|| ProcessError {
                    message: format!(\"async process child handle {} is closed or unknown\", handle),
                })?;
                __child.id().ok_or_else(|| ProcessError {
                    message: format!(\"async process child handle {} has no running process id\", handle),
                })?.to_string()
            };
            let __status = tokio::process::Command::new(\"kill\")
                .arg(\"-TERM\")
                .arg(&__pid)
                .status()
                .await
                .map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
            if !__status.success() {
                return Err(ProcessError { message: format!(\"async process terminate failed with status: {}\", __status) });
            }
            return Ok(());
        }
        #[cfg(not(unix))]
        {
            let _ = handle;
            return Err(ProcessError { message: \"async process terminate is unsupported on this host; use async_kill for forceful termination\".to_string() });
        }"
            .to_string(),
    ))]
}
