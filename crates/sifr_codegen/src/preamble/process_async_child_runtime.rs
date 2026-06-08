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

pub(super) fn process_async_child_table_items(
    needs_spawn: bool,
    needs_wait: bool,
    needs_kill: bool,
    needs_terminate: bool,
) -> Vec<RustItem> {
    if !needs_spawn && !needs_wait && !needs_kill && !needs_terminate {
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
    }];

    if needs_spawn {
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
        }
        if stdin_mode != \"inherit\" || stdout_mode != \"inherit\" || stderr_mode != \"inherit\" {
            return Err(ProcessError { message: \"async process spawn stdio modes require async owned pipe support\".to_string() });
        }"
        .to_string(),
    ))]
}

pub(super) fn process_async_spawn_insert_body() -> RustStmt {
    RustStmt::Expr(RustExpr::Ident(
        "let __child = __cmd.spawn().map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
        let __handle = __sifr_next_process_async_child_id();
        {
            let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
            __children.insert(__handle, __child);
        }
        return Ok(AsyncChild::new(__handle));"
            .to_string(),
    ))
}

pub(super) fn process_async_wait_body() -> Vec<RustStmt> {
    vec![RustStmt::Expr(RustExpr::Ident(
        "let mut __child = {
            let mut __children = __SIFR_PROCESS_ASYNC_CHILDREN.lock().unwrap_or_else(|__err| __err.into_inner());
            __children.remove(&handle).ok_or_else(|| ProcessError {
                message: format!(\"async process child handle {} is closed or unknown\", handle),
            })?
        };
        let __status = __child.wait().await.map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?;
        return Ok(__sifr_process_status_from_exit(
            __status.code().unwrap_or(-1) as i64,
            __sifr_process_exit_signal(&__status),
        ));"
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
