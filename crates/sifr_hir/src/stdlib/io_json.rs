use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn json_value_stub() -> Type {
    Type::Class {
        name: "JsonValue".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    }
}

fn json_decode_error_ty() -> Type {
    Type::Class {
        name: "JSONDecodeError".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("line".to_string(), Type::Int),
            ("column".to_string(), Type::Int),
        ],
        methods: vec![],
        parent_class: Some("Error".to_string()),
    }
}

fn json_integer_range_error_ty() -> Type {
    Type::Class {
        name: "JsonIntegerRangeError".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("path".to_string(), Type::Str),
            ("profile".to_string(), Type::Str),
        ],
        methods: vec![],
        parent_class: Some("Error".to_string()),
    }
}

fn json_limit_error_ty() -> Type {
    Type::Class {
        name: "JsonLimitError".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("limit".to_string(), Type::Int),
        ],
        methods: vec![],
        parent_class: Some("Error".to_string()),
    }
}

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

    // json_loads(s: str) -> Result[JsonValue, JSONDecodeError]
    functions.insert(
        "json_loads".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str)],
            Type::Result(
                Box::new(json_value_stub()),
                Box::new(json_decode_error_ty()),
            ),
        ),
    );

    // json_validate_integer_digit_limits(s: str) -> Result[None, JsonLimitError]
    functions.insert(
        "json_validate_integer_digit_limits".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str)],
            Type::Result(Box::new(Type::None), Box::new(json_limit_error_ty())),
        ),
    );

    // json_dumps(obj: Any) -> str
    functions.insert(
        "json_dumps".to_string(),
        FunctionType::all_borrow(vec![("obj".to_string(), Type::Any)], Type::Str),
    );

    // json_dumps_value(obj: JsonValue) -> str
    functions.insert(
        "json_dumps_value".to_string(),
        FunctionType::all_borrow(vec![("obj".to_string(), json_value_stub())], Type::Str),
    );

    // json_dumps_value_exact(obj: JsonValue) -> str
    functions.insert(
        "json_dumps_value_exact".to_string(),
        FunctionType::all_borrow(vec![("obj".to_string(), json_value_stub())], Type::Str),
    );

    // json_dumps_value_web(obj: JsonValue) -> Result[str, JsonIntegerRangeError]
    functions.insert(
        "json_dumps_value_web".to_string(),
        FunctionType::all_borrow(
            vec![("obj".to_string(), json_value_stub())],
            Type::Result(Box::new(Type::Str), Box::new(json_integer_range_error_ty())),
        ),
    );

    // json_dumps_value_string_ints(obj: JsonValue) -> str
    functions.insert(
        "json_dumps_value_string_ints".to_string(),
        FunctionType::all_borrow(vec![("obj".to_string(), json_value_stub())], Type::Str),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
