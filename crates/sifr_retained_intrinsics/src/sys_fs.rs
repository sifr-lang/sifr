use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.sys — retained process-command helper.
pub(super) fn intrinsic_sys() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // run_command(cmd: str) -> Result[str, IOError]
    functions.insert(
        "run_command".to_string(),
        FunctionType::all_borrow(
            vec![("cmd".to_string(), Type::Str)],
            result_ty(Type::Str, "IOError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.fs — File system intrinsics (io + os file ops)
pub(super) fn intrinsic_fs() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // chdir(path: str) -> Result[None, IOError]
    functions.insert(
        "chdir".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::None, "IOError"),
        ),
    );

    // getpid() -> int
    functions.insert(
        "getpid".to_string(),
        FunctionType::all_borrow(vec![], Type::Int),
    );

    // cpu_count() -> int
    functions.insert(
        "cpu_count".to_string(),
        FunctionType::all_borrow(vec![], Type::Int),
    );

    // stat_size(path: str) -> Result[int, IOError] (file size in bytes)
    functions.insert(
        "stat_size".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::Int, "IOError"),
        ),
    );

    // which(name: str) -> str | None (find executable in PATH)
    functions.insert(
        "which".to_string(),
        FunctionType::all_borrow(
            vec![("name".to_string(), Type::Str)],
            Type::Union(vec![Type::Str, Type::None]),
        ),
    );

    // disk_usage(path: str) -> list[int] ([total, used, free] in bytes)
    functions.insert(
        "disk_usage".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            Type::List(Box::new(Type::Int)),
        ),
    );

    // os.sep, os.linesep, os.name as zero-arg functions in _sifr.fs
    functions.insert(
        "os_sep".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "os_linesep".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "os_name".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
