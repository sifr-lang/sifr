use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.sys — Combined system intrinsics (env + os)
pub(super) fn intrinsic_sys() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // env_get(key: str) -> str | None
    functions.insert(
        "env_get".to_string(),
        FunctionType::all_borrow(
            vec![("key".to_string(), Type::Str)],
            Type::Union(vec![Type::Str, Type::None]),
        ),
    );

    // env_set(key: str, value: str) -> None
    functions.insert(
        "env_set".to_string(),
        FunctionType::all_borrow(
            vec![
                ("key".to_string(), Type::Str),
                ("value".to_string(), Type::Str),
            ],
            Type::None,
        ),
    );

    // env_unset(key: str) -> None
    functions.insert(
        "env_unset".to_string(),
        FunctionType::all_borrow(vec![("key".to_string(), Type::Str)], Type::None),
    );

    // env_keys() -> list[str]
    functions.insert(
        "env_keys".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Str))),
    );

    // env_values() -> list[str]
    functions.insert(
        "env_values".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Str))),
    );

    // env_items() -> list[str]  (formatted as "key=value")
    functions.insert(
        "env_items".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Str))),
    );

    // run_command(cmd: str) -> Result[str, IOError]
    functions.insert(
        "run_command".to_string(),
        FunctionType::all_borrow(
            vec![("cmd".to_string(), Type::Str)],
            result_ty(Type::Str, "IOError"),
        ),
    );

    // get_args() -> list[str]
    functions.insert(
        "get_args".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Str))),
    );

    // sys_exit(code: int) -> None (terminates the process)
    functions.insert(
        "sys_exit".to_string(),
        FunctionType::all_borrow(vec![("code".to_string(), Type::Int)], Type::None),
    );

    // sys_version() -> str (Sifr version string)
    functions.insert(
        "sys_version".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );

    // sys_platform() -> str (platform identifier: "linux", "macos", "windows")
    functions.insert(
        "sys_platform".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );

    // sys_maxsize() -> int (maximum int size)
    functions.insert(
        "sys_maxsize".to_string(),
        FunctionType::all_borrow(vec![], Type::Int),
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

    // open_file(path: str, mode: str) -> Result[int, IOError]
    // Returns an opaque file handle ID (i64) for use with file_* intrinsics.
    functions.insert(
        "open_file".to_string(),
        FunctionType::all_borrow(
            vec![
                ("path".to_string(), Type::Str),
                ("mode".to_string(), Type::Str),
            ],
            result_ty(Type::Int, "IOError"),
        ),
    );

    // file_read(handle: int) -> Result[str, IOError]
    functions.insert(
        "file_read".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Str, "IOError"),
        ),
    );

    // file_write(handle: int, data: str) -> Result[None, IOError]
    functions.insert(
        "file_write".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Str),
            ],
            result_ty(Type::None, "IOError"),
        ),
    );

    // file_readline(handle: int) -> Result[str | None, IOError]
    functions.insert(
        "file_readline".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Union(vec![Type::Str, Type::None]), "IOError"),
        ),
    );

    // file_readlines(handle: int) -> Result[list[str], IOError]
    functions.insert(
        "file_readlines".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
        ),
    );

    // file_close(handle: int) -> None
    functions.insert(
        "file_close".to_string(),
        FunctionType::all_borrow(vec![("handle".to_string(), Type::Int)], Type::None),
    );

    // file_read_bytes(handle: int) -> Result[bytes, IOError]
    functions.insert(
        "file_read_bytes".to_string(),
        FunctionType::all_borrow(
            vec![("handle".to_string(), Type::Int)],
            result_ty(Type::Bytes, "IOError"),
        ),
    );

    // file_write_bytes(handle: int, data: bytes) -> Result[None, IOError]
    functions.insert(
        "file_write_bytes".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("data".to_string(), Type::Bytes),
            ],
            result_ty(Type::None, "IOError"),
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
