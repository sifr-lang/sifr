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

    // subprocess_run(cmd: str) -> Result[str, IOError]
    functions.insert(
        "subprocess_run".to_string(),
        FunctionType::all_borrow(
            vec![("cmd".to_string(), Type::Str)],
            result_ty(Type::Str, "IOError"),
        ),
    );

    // subprocess_run_with_input(cmd: str, stdin: str) -> Result[str, IOError]
    functions.insert(
        "subprocess_run_with_input".to_string(),
        FunctionType::all_borrow(
            vec![
                ("cmd".to_string(), Type::Str),
                ("stdin_data".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "IOError"),
        ),
    );

    // subprocess_run_structured(cmd: str) -> Result[list[str], IOError]
    // Returns [stdout, stderr, returncode_str] as a list[str].
    functions.insert(
        "subprocess_run_structured".to_string(),
        FunctionType::all_borrow(
            vec![("cmd".to_string(), Type::Str)],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
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

    // read_text(path: str) -> Result[str, IOError]
    functions.insert(
        "read_text".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::Str, "IOError"),
        ),
    );

    // write_text(path: str, content: str) -> Result[None, IOError]
    functions.insert(
        "write_text".to_string(),
        FunctionType::all_borrow(
            vec![
                ("path".to_string(), Type::Str),
                ("content".to_string(), Type::Str),
            ],
            result_ty(Type::None, "IOError"),
        ),
    );

    // exists(path: str) -> bool  (infallible)
    functions.insert(
        "exists".to_string(),
        FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool),
    );

    // read_lines(path: str) -> Result[list[str], IOError]
    functions.insert(
        "read_lines".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
        ),
    );

    // append_text(path: str, content: str) -> Result[None, IOError]
    functions.insert(
        "append_text".to_string(),
        FunctionType::all_borrow(
            vec![
                ("path".to_string(), Type::Str),
                ("content".to_string(), Type::Str),
            ],
            result_ty(Type::None, "IOError"),
        ),
    );

    // getcwd() -> Result[str, IOError]
    functions.insert(
        "getcwd".to_string(),
        FunctionType::all_borrow(vec![], result_ty(Type::Str, "IOError")),
    );

    // listdir(path: str) -> Result[list[str], IOError]
    functions.insert(
        "listdir".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
        ),
    );

    // mkdir(path: str) -> Result[None, IOError]
    functions.insert(
        "mkdir".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::None, "IOError"),
        ),
    );

    // rmdir(path: str) -> Result[None, IOError]
    functions.insert(
        "rmdir".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::None, "IOError"),
        ),
    );

    // remove_file(path: str) -> Result[None, IOError]
    functions.insert(
        "remove_file".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::None, "IOError"),
        ),
    );

    // rename(src: str, dst: str) -> Result[None, IOError]
    functions.insert(
        "rename".to_string(),
        FunctionType::all_borrow(
            vec![
                ("src".to_string(), Type::Str),
                ("dst".to_string(), Type::Str),
            ],
            result_ty(Type::None, "IOError"),
        ),
    );

    // is_file(path: str) -> bool  (infallible)
    functions.insert(
        "is_file".to_string(),
        FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool),
    );

    // is_dir(path: str) -> bool  (infallible)
    functions.insert(
        "is_dir".to_string(),
        FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool),
    );

    // copy_file(src: str, dst: str) -> Result[None, IOError]
    functions.insert(
        "copy_file".to_string(),
        FunctionType::all_borrow(
            vec![
                ("src".to_string(), Type::Str),
                ("dst".to_string(), Type::Str),
            ],
            result_ty(Type::None, "IOError"),
        ),
    );

    // walk_dir(path: str) -> Result[list[str], IOError]
    functions.insert(
        "walk_dir".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
        ),
    );

    // rmdir_all(path: str) -> Result[None, IOError]
    functions.insert(
        "rmdir_all".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::None, "IOError"),
        ),
    );

    // gettempdir() -> str  (infallible — reads env/system temp)
    functions.insert(
        "gettempdir".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );

    // makedirs(path: str) -> Result[None, IOError]
    functions.insert(
        "makedirs".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::None, "IOError"),
        ),
    );

    // touch(path: str) -> Result[None, IOError] (create file if not exists)
    functions.insert(
        "touch".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::None, "IOError"),
        ),
    );

    // resolve_path(path: str) -> Result[str, IOError] (canonicalize path)
    functions.insert(
        "resolve_path".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::Str, "IOError"),
        ),
    );

    // iterdir(path: str) -> Result[list[str], IOError] (list directory entries as full paths)
    functions.insert(
        "iterdir".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
        ),
    );

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

    // glob_pattern(dir: str, pattern: str) -> Result[list[str], IOError]
    functions.insert(
        "glob_pattern".to_string(),
        FunctionType::all_borrow(
            vec![
                ("dir".to_string(), Type::Str),
                ("pattern".to_string(), Type::Str),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
        ),
    );

    // rglob_pattern(dir: str, pattern: str) -> Result[list[str], IOError]
    functions.insert(
        "rglob_pattern".to_string(),
        FunctionType::all_borrow(
            vec![
                ("dir".to_string(), Type::Str),
                ("pattern".to_string(), Type::Str),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
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
