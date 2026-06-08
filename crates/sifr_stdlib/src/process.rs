use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn process_bytes_output_tuple() -> Type {
    Type::Tuple(vec![Type::Bytes, Type::Bytes, process_status_tuple()])
}

fn process_bytes_timeout_output_tuple() -> Type {
    Type::Tuple(vec![
        Type::Bytes,
        Type::Bytes,
        process_status_tuple(),
        Type::Bool,
    ])
}

fn process_text_output_tuple() -> Type {
    Type::Tuple(vec![Type::Str, Type::Str, process_status_tuple()])
}

fn process_status_tuple() -> Type {
    Type::Tuple(vec![Type::Int, Type::Union(vec![Type::Int, Type::None])])
}

fn process_status_result() -> Type {
    result_ty(process_status_tuple(), "ProcessError")
}

fn process_bytes_output_result() -> Type {
    result_ty(process_bytes_output_tuple(), "ProcessError")
}

fn process_status_class() -> Type {
    Type::Class {
        name: "Status".to_string(),
        fields: vec![
            ("code".to_string(), Type::Int),
            ("success".to_string(), Type::Bool),
            (
                "signal".to_string(),
                Type::Union(vec![Type::Int, Type::None]),
            ),
            ("timed_out".to_string(), Type::Bool),
            ("cancelled".to_string(), Type::Bool),
            ("kind".to_string(), Type::Str),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn process_output_class() -> Type {
    Type::Class {
        name: "Output".to_string(),
        fields: vec![
            ("stdout".to_string(), Type::Bytes),
            ("stderr".to_string(), Type::Bytes),
            ("status".to_string(), process_status_class()),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn process_async_child_class() -> Type {
    Type::Class {
        name: "AsyncChild".to_string(),
        fields: vec![("_handle".to_string(), Type::Int)],
        methods: vec![],
        parent_class: None,
    }
}

fn process_status_object_result() -> Type {
    result_ty(process_status_class(), "ProcessError")
}

fn process_output_object_result() -> Type {
    result_ty(process_output_class(), "ProcessError")
}

fn process_async_child_object_result() -> Type {
    result_ty(process_async_child_class(), "ProcessError")
}

fn process_async_pipe_read_result() -> Type {
    result_ty(Type::Bytes, "ProcessError")
}

/// _sifr.process — Native process execution intrinsics.
pub(super) fn intrinsic_process() -> IntrinsicModule {
    let mut functions = HashMap::new();
    let args_ty = Type::List(Box::new(Type::Str));
    let env_ty = Type::List(Box::new(Type::Str));

    functions.insert(
        "process_run".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), args_ty.clone()),
                ("env".to_string(), env_ty.clone()),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
            ],
            process_status_result(),
        ),
    );
    functions.insert(
        "process_spawn".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), args_ty.clone()),
                ("env".to_string(), env_ty.clone()),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin_mode".to_string(), Type::Str),
                ("stdout_mode".to_string(), Type::Str),
                ("stderr_mode".to_string(), Type::Str),
            ],
            result_ty(Type::Int, "ProcessError"),
        ),
    );
    functions.insert(
        "process_child_stdin".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Int, "ProcessError"),
        ),
    );
    functions.insert(
        "process_child_stdout".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Int, "ProcessError"),
        ),
    );
    functions.insert(
        "process_child_stderr".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Int, "ProcessError"),
        ),
    );
    functions.insert(
        "process_pipe_read_all".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Bytes, "ProcessError"),
        ),
    );
    functions.insert(
        "process_pipe_read".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("max_bytes".to_string(), Type::Int),
            ],
            result_ty(Type::Bytes, "ProcessError"),
        ),
    );
    functions.insert(
        "process_pipe_reader_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::None, "ProcessError"),
        ),
    );
    functions.insert(
        "process_pipe_write_all".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            result_ty(Type::None, "ProcessError"),
        ),
    );
    functions.insert(
        "process_pipe_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::None, "ProcessError"),
        ),
    );
    functions.insert(
        "process_wait".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            process_status_result(),
        ),
    );
    functions.insert(
        "process_kill".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::None, "ProcessError"),
        ),
    );
    functions.insert(
        "process_terminate".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::None, "ProcessError"),
        ),
    );
    functions.insert(
        "process_output".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), args_ty.clone()),
                ("env".to_string(), env_ty.clone()),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
            ],
            process_bytes_output_result(),
        ),
    );
    functions.insert(
        "process_output_text".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), args_ty.clone()),
                ("env".to_string(), env_ty.clone()),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
                ("encoding".to_string(), Type::Str),
            ],
            result_ty(process_text_output_tuple(), "ProcessError"),
        ),
    );
    functions.insert(
        "process_output_timeout".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), args_ty.clone()),
                ("env".to_string(), env_ty.clone()),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
                ("timeout_seconds".to_string(), Type::Float),
            ],
            result_ty(process_bytes_timeout_output_tuple(), "ProcessError"),
        ),
    );
    functions.insert(
        "process_async_run".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), Type::List(Box::new(Type::Str))),
                ("env".to_string(), Type::List(Box::new(Type::Str))),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin_mode".to_string(), Type::Str),
            ],
            Type::Awaitable(Box::new(process_status_object_result())),
        ),
    );
    functions.insert(
        "process_async_output".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), Type::List(Box::new(Type::Str))),
                ("env".to_string(), Type::List(Box::new(Type::Str))),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin_mode".to_string(), Type::Str),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
            ],
            Type::Awaitable(Box::new(process_output_object_result())),
        ),
    );
    functions.insert(
        "process_async_output_timeout".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), Type::List(Box::new(Type::Str))),
                ("env".to_string(), Type::List(Box::new(Type::Str))),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin_mode".to_string(), Type::Str),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
                ("timeout_seconds".to_string(), Type::Float),
            ],
            Type::Awaitable(Box::new(process_output_object_result())),
        ),
    );
    functions.insert(
        "process_async_run_timeout".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), Type::List(Box::new(Type::Str))),
                ("env".to_string(), Type::List(Box::new(Type::Str))),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin_mode".to_string(), Type::Str),
                ("timeout_seconds".to_string(), Type::Float),
            ],
            Type::Awaitable(Box::new(process_status_object_result())),
        ),
    );
    functions.insert(
        "process_async_spawn".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), Type::List(Box::new(Type::Str))),
                ("env".to_string(), Type::List(Box::new(Type::Str))),
                ("cwd".to_string(), Type::Str),
                ("has_cwd".to_string(), Type::Bool),
                ("stdin_mode".to_string(), Type::Str),
                ("stdout_mode".to_string(), Type::Str),
                ("stderr_mode".to_string(), Type::Str),
                ("has_stdin".to_string(), Type::Bool),
            ],
            Type::Awaitable(Box::new(process_async_child_object_result())),
        ),
    );
    functions.insert(
        "process_async_wait".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            Type::Awaitable(Box::new(process_status_object_result())),
        ),
    );
    functions.insert(
        "process_async_kill".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            Type::Awaitable(Box::new(result_ty(Type::None, "ProcessError"))),
        ),
    );
    functions.insert(
        "process_async_terminate".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            Type::Awaitable(Box::new(result_ty(Type::None, "ProcessError"))),
        ),
    );
    functions.insert(
        "process_async_child_stdin".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Int, "ProcessError"),
        ),
    );
    functions.insert(
        "process_async_child_stdout".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Int, "ProcessError"),
        ),
    );
    functions.insert(
        "process_async_child_stderr".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Int, "ProcessError"),
        ),
    );
    functions.insert(
        "process_async_pipe_read_all".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            Type::Awaitable(Box::new(process_async_pipe_read_result())),
        ),
    );
    functions.insert(
        "process_async_pipe_read".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("max_bytes".to_string(), Type::Int),
            ],
            Type::Awaitable(Box::new(process_async_pipe_read_result())),
        ),
    );
    functions.insert(
        "process_async_pipe_reader_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::None, "ProcessError"),
        ),
    );
    functions.insert(
        "process_async_pipe_write_all".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            Type::Awaitable(Box::new(result_ty(Type::None, "ProcessError"))),
        ),
    );
    functions.insert(
        "process_async_pipe_close".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::None, "ProcessError"),
        ),
    );
    functions.insert(
        "process_async_shell_run".to_string(),
        FunctionType::all_borrow(
            vec![("script".to_string(), Type::Str)],
            Type::Awaitable(Box::new(process_status_object_result())),
        ),
    );
    functions.insert(
        "process_async_shell_output".to_string(),
        FunctionType::all_borrow(
            vec![
                ("script".to_string(), Type::Str),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
            ],
            Type::Awaitable(Box::new(process_output_object_result())),
        ),
    );
    functions.insert(
        "process_async_shell_output_timeout".to_string(),
        FunctionType::all_borrow(
            vec![
                ("script".to_string(), Type::Str),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
                ("timeout_seconds".to_string(), Type::Float),
            ],
            Type::Awaitable(Box::new(process_output_object_result())),
        ),
    );
    functions.insert(
        "process_shell_run".to_string(),
        FunctionType::all_borrow(
            vec![("script".to_string(), Type::Str)],
            process_status_result(),
        ),
    );
    functions.insert(
        "process_shell_output".to_string(),
        FunctionType::all_borrow(
            vec![
                ("script".to_string(), Type::Str),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
            ],
            result_ty(process_bytes_output_tuple(), "ProcessError"),
        ),
    );
    functions.insert(
        "process_shell_output_text".to_string(),
        FunctionType::all_borrow(
            vec![
                ("script".to_string(), Type::Str),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
                ("encoding".to_string(), Type::Str),
            ],
            result_ty(process_text_output_tuple(), "ProcessError"),
        ),
    );
    functions.insert(
        "process_shell_output_timeout".to_string(),
        FunctionType::all_borrow(
            vec![
                ("script".to_string(), Type::Str),
                ("stdin".to_string(), Type::Bytes),
                ("has_stdin".to_string(), Type::Bool),
                ("timeout_seconds".to_string(), Type::Float),
            ],
            result_ty(process_bytes_timeout_output_tuple(), "ProcessError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
