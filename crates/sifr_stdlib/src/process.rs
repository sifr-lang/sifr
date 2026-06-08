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
            result_ty(process_status_tuple(), "ProcessError"),
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
            ],
            result_ty(Type::Int, "ProcessError"),
        ),
    );
    functions.insert(
        "process_wait".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(process_status_tuple(), "ProcessError"),
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
            result_ty(process_bytes_output_tuple(), "ProcessError"),
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
        "process_shell_run".to_string(),
        FunctionType::all_borrow(
            vec![("script".to_string(), Type::Str)],
            result_ty(process_status_tuple(), "ProcessError"),
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
