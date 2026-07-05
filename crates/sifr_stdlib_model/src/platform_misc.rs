use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.platform — Platform information bootstrap signatures.
pub(super) fn intrinsic_platform() -> IntrinsicModule {
    let mut functions = HashMap::new();
    functions.insert(
        "platform_system".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "platform_arch".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "platform_node".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "platform_release".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "platform_version".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "platform_processor".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.toml — TOML parsing intrinsics
pub(super) fn intrinsic_toml() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // toml_parse_tokens(text: str) -> Result[list[str], ParseError>
    functions.insert(
        "toml_parse_tokens".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            Type::Result(
                Box::new(Type::List(Box::new(Type::Str))),
                Box::new(Type::Class {
                    name: "ParseError".to_string(),
                    fields: vec![("message".to_string(), Type::Str)],
                    methods: vec![],
                    parent_class: Some("Error".to_string()),
                }),
            ),
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.datetime — Date/time intrinsics
pub(super) fn intrinsic_datetime() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // datetime_now() -> str (ISO 8601 formatted current datetime)
    functions.insert(
        "datetime_now".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    // datetime_now_struct() -> list[int] ([year, month, day, hour, minute, second])
    functions.insert(
        "datetime_now_struct".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))),
    );
    // datetime_format(dt: str, fmt: str) -> str
    functions.insert(
        "datetime_format".to_string(),
        FunctionType::all_borrow(
            vec![
                ("dt".to_string(), Type::Str),
                ("fmt".to_string(), Type::Str),
            ],
            Type::Str,
        ),
    );
    // datetime_from_timestamp(ts: float) -> Result[str, ValueError]
    functions.insert(
        "datetime_from_timestamp".to_string(),
        FunctionType::all_borrow(
            vec![("ts".to_string(), Type::Float)],
            result_ty(Type::Str, "ValueError"),
        ),
    );
    // time_strptime(s: str, fmt: str) -> list[int] ([year, month, day, hour, minute, second, weekday, yearday])
    functions.insert(
        "time_strptime".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str), ("fmt".to_string(), Type::Str)],
            result_ty(Type::List(Box::new(Type::Int)), "ValueError"),
        ),
    );
    // time_gmtime() -> list[int] ([year, month, day, hour, minute, second, weekday, yearday])
    functions.insert(
        "time_gmtime".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))),
    );
    // time_localtime() -> list[int] ([year, month, day, hour, minute, second, weekday, yearday])
    functions.insert(
        "time_localtime".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.html — HTML escaping bootstrap signatures.
pub(super) fn intrinsic_html() -> IntrinsicModule {
    let mut functions = HashMap::new();
    functions.insert(
        "html_escape".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    functions.insert(
        "html_unescape".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.calendar — Calendar/date calculation intrinsics
pub(super) fn intrinsic_calendar() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // calendar_isleap(year: int) -> bool
    functions.insert(
        "calendar_isleap".to_string(),
        FunctionType::all_borrow(vec![("year".to_string(), Type::Int)], Type::Bool),
    );
    // calendar_weekday(year: int, month: int, day: int) -> int (0=Monday..6=Sunday)
    functions.insert(
        "calendar_weekday".to_string(),
        FunctionType::all_borrow(
            vec![
                ("year".to_string(), Type::Int),
                ("month".to_string(), Type::Int),
                ("day".to_string(), Type::Int),
            ],
            Type::Int,
        ),
    );
    // calendar_monthrange(year: int, month: int) -> list[int] ([weekday_of_first, days_in_month])
    functions.insert(
        "calendar_monthrange".to_string(),
        FunctionType::all_borrow(
            vec![
                ("year".to_string(), Type::Int),
                ("month".to_string(), Type::Int),
            ],
            Type::List(Box::new(Type::Int)),
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.compress — Compression intrinsics (gzip + zip)
pub(super) fn intrinsic_compress() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // _gzip_compress_bytes_impl(data: str) -> bytes (compressed bytes)
    functions.insert(
        "_gzip_compress_bytes_impl".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Str)], Type::Bytes),
    );
    // _gzip_decompress_bytes_impl(data: bytes) -> Result[str, IOError]
    functions.insert(
        "_gzip_decompress_bytes_impl".to_string(),
        FunctionType::all_borrow(
            vec![("data".to_string(), Type::Bytes)],
            result_ty(Type::Str, "IOError"),
        ),
    );
    // zip_create(path: str) -> Result[None, IOError]
    functions.insert(
        "zip_create".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::None, "IOError"),
        ),
    );
    // zip_add_file(zip_path: str, name: str, content: str) -> Result[None, IOError]
    functions.insert(
        "zip_add_file".to_string(),
        FunctionType::all_borrow(
            vec![
                ("zip_path".to_string(), Type::Str),
                ("name".to_string(), Type::Str),
                ("content".to_string(), Type::Str),
            ],
            result_ty(Type::None, "IOError"),
        ),
    );
    // zip_add_file_bytes(zip_path: str, name: str, content: bytes) -> Result[None, IOError]
    functions.insert(
        "zip_add_file_bytes".to_string(),
        FunctionType::all_borrow(
            vec![
                ("zip_path".to_string(), Type::Str),
                ("name".to_string(), Type::Str),
                ("content".to_string(), Type::Bytes),
            ],
            result_ty(Type::None, "IOError"),
        ),
    );
    // zip_read_file(zip_path: str, name: str) -> Result[str, IOError]
    functions.insert(
        "zip_read_file".to_string(),
        FunctionType::all_borrow(
            vec![
                ("zip_path".to_string(), Type::Str),
                ("name".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "IOError"),
        ),
    );
    // zip_read_file_bytes(zip_path: str, name: str) -> Result[bytes, IOError]
    functions.insert(
        "zip_read_file_bytes".to_string(),
        FunctionType::all_borrow(
            vec![
                ("zip_path".to_string(), Type::Str),
                ("name".to_string(), Type::Str),
            ],
            result_ty(Type::Bytes, "IOError"),
        ),
    );
    // zip_namelist(zip_path: str) -> Result[list[str], IOError]
    functions.insert(
        "zip_namelist".to_string(),
        FunctionType::all_borrow(
            vec![("zip_path".to_string(), Type::Str)],
            result_ty(Type::List(Box::new(Type::Str)), "IOError"),
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.logging — Logging intrinsics for global state management
pub(super) fn intrinsic_logging() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // set_global_level(level: int) -> None
    functions.insert(
        "set_global_level".to_string(),
        FunctionType::all_borrow(vec![("level".to_string(), Type::Int)], Type::None),
    );

    // get_global_level() -> int
    functions.insert(
        "get_global_level".to_string(),
        FunctionType::all_borrow(vec![], Type::Int),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
