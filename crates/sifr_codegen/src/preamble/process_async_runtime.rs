//! Runtime support for generated async child-process helpers.

use super::process_async_child_runtime::{
    process_async_child_pipe_reader_item, process_async_child_pipe_writer_item,
    process_async_child_table_items, process_async_kill_body, process_async_pipe_close_item,
    process_async_pipe_read_all_item, process_async_pipe_read_item,
    process_async_pipe_reader_close_item, process_async_pipe_write_all_item,
    process_async_spawn_body, process_async_spawn_insert_body, process_async_spawn_params,
    process_async_terminate_body, process_async_wait_body, process_async_wait_params,
    process_handle_wait_body, ProcessAsyncChildTableNeeds,
};
use crate::stdlib_filter::SharedPreludeProcessAsyncNeeds;
use crate::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};

fn string_ty() -> RustType {
    RustType::String_
}

fn process_async_command_setup() -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: true,
            name: "__cmd".to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "tokio".to_string(),
                    "process".to_string(),
                    "Command".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident("program".to_string())),
                }],
            },
        },
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
            method: "args".to_string(),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("args".to_string())),
                method: "iter".to_string(),
                args: vec![],
            }],
        }),
        RustStmt::For {
            var: "__sifr_process_env".to_string(),
            iter: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("env".to_string())),
                method: "iter".to_string(),
                args: vec![],
            },
            body: vec![RustStmt::IfLet {
                pattern: "Some((__sifr_process_env_key, __sifr_process_env_value))".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_process_env".to_string())),
                    method: "split_once".to_string(),
                    args: vec![RustExpr::Literal(crate::RustLiteral::Char('='))],
                },
                then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                    method: "env".to_string(),
                    args: vec![
                        RustExpr::Ident("__sifr_process_env_key".to_string()),
                        RustExpr::Ident("__sifr_process_env_value".to_string()),
                    ],
                })],
                else_body: None,
            }],
        },
        RustStmt::If {
            cond: RustExpr::Ident("has_cwd".to_string()),
            then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__cmd".to_string())),
                method: "current_dir".to_string(),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident("cwd".to_string())),
                }],
            })],
            else_body: None,
        },
    ]
}

fn process_async_ret(name: &str) -> RustType {
    RustType::Named(format!("Result<{name}, ProcessError>"))
}

fn process_status_from_exit_item() -> RustItem {
    RustItem::Fn {
        name: "__sifr_process_status_from_exit".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![
            RustParam::Named {
                name: "code".to_string(),
                ty: RustType::I64,
            },
            RustParam::Named {
                name: "signal".to_string(),
                ty: RustType::Option(Box::new(RustType::I64)),
            },
        ],
        ret: Some(RustType::Named("Status".to_string())),
        body: vec![RustStmt::Expr(RustExpr::Ident(
            "if let Some(__signal) = signal {
                let mut __status = Status::new(code, \"signal\".to_string());
                __status.success = false;
                __status.signal = Some(__signal);
                return __status;
            }
            if code == 0 {
                return Status::new(code, \"success\".to_string());
            }
            return Status::new(code, \"nonzero\".to_string())"
                .to_string(),
        ))],
        is_async: false,
    }
}

pub(crate) fn build_process_async_items(needs: SharedPreludeProcessAsyncNeeds) -> Vec<RustItem> {
    let mut spawn_body = process_async_spawn_body();
    spawn_body.extend(process_async_command_setup());
    spawn_body.push(process_async_spawn_insert_body());

    let mut items = vec![process_status_from_exit_item()];

    items.extend(process_async_child_table_items(
        ProcessAsyncChildTableNeeds {
            spawn: needs.needs_spawn,
            wait: needs.needs_wait,
            kill: needs.needs_kill,
            terminate: needs.needs_terminate,
        },
    ));

    if needs.needs_spawn {
        items.push(process_async_child_pipe_writer_item(
            "__sifr_process_async_child_stdin",
        ));
        items.push(process_async_child_pipe_reader_item(
            "__sifr_process_async_child_stdout",
            "stdout",
        ));
        items.push(process_async_child_pipe_reader_item(
            "__sifr_process_async_child_stderr",
            "stderr",
        ));
        items.push(process_async_pipe_read_all_item());
        items.push(process_async_pipe_read_item());
        items.push(process_async_pipe_reader_close_item());
        items.push(process_async_pipe_write_all_item());
        items.push(process_async_pipe_close_item());
    }
    if needs.needs_spawn_function {
        items.push(RustItem::Fn {
            name: "__sifr_process_async_spawn".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_spawn_params(),
            ret: Some(process_async_ret("AsyncChild")),
            body: spawn_body,
            is_async: true,
        });
    }
    if needs.needs_wait {
        items.push(RustItem::Fn {
            name: "__sifr_process_async_wait".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_wait_params(),
            ret: Some(process_async_ret("Status")),
            body: process_async_wait_body(),
            is_async: true,
        });
    }
    if needs.needs_handle_wait {
        items.push(RustItem::Fn {
            name: "__sifr_process_handle_wait".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_wait_params(),
            ret: Some(process_async_ret("Status")),
            body: process_handle_wait_body(),
            is_async: true,
        });
    }
    if needs.needs_kill {
        items.push(RustItem::Fn {
            name: "__sifr_process_async_kill".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_wait_params(),
            ret: Some(process_async_ret("()")),
            body: process_async_kill_body(),
            is_async: true,
        });
    }
    if needs.needs_terminate {
        items.push(RustItem::Fn {
            name: "__sifr_process_async_terminate".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: process_async_wait_params(),
            ret: Some(process_async_ret("()")),
            body: process_async_terminate_body(),
            is_async: true,
        });
    }

    items
}
