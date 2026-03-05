use super::*;

/// _sifr.io — File I/O intrinsics
pub(super) fn intrinsic_io() -> IntrinsicModule {
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

    // exists(path: str) -> bool  (infallible — just checks existence)
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

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.json — JSON serialization/deserialization intrinsics
pub(super) fn intrinsic_json() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // json_loads(s: str) -> Result[str, JSONDecodeError]
    functions.insert(
        "json_loads".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str)],
            result_ty(Type::Str, "JSONDecodeError"),
        ),
    );

    // json_dumps(obj: Any) -> str
    functions.insert(
        "json_dumps".to_string(),
        FunctionType::all_borrow(vec![("obj".to_string(), Type::Any)], Type::Str),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
