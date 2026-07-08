use super::IntrinsicModule;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

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
    IntrinsicModule {
        functions: HashMap::new(),
        constants: HashMap::new(),
    }
}

/// _sifr.json — JSON serialization/deserialization intrinsics
pub(super) fn intrinsic_json() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // json_load_tokens(text: str) -> Result[list[str], JSONDecodeError]
    functions.insert(
        "json_load_tokens".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            Type::Result(
                Box::new(Type::List(Box::new(Type::Str))),
                Box::new(json_decode_error_ty()),
            ),
        ),
    );

    // json_validate_integer_digit_limits(text: str) -> Result[None, JsonLimitError]
    functions.insert(
        "json_validate_integer_digit_limits".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            Type::Result(Box::new(Type::None), Box::new(json_limit_error_ty())),
        ),
    );

    // json_dump_tokens(tokens: list[str]) -> str
    functions.insert(
        "json_dump_tokens".to_string(),
        FunctionType::all_borrow(
            vec![("tokens".to_string(), Type::List(Box::new(Type::Str)))],
            Type::Str,
        ),
    );

    // json_dump_tokens_exact(tokens: list[str]) -> str
    functions.insert(
        "json_dump_tokens_exact".to_string(),
        FunctionType::all_borrow(
            vec![("tokens".to_string(), Type::List(Box::new(Type::Str)))],
            Type::Str,
        ),
    );

    // json_dump_tokens_string_ints(tokens: list[str]) -> str
    functions.insert(
        "json_dump_tokens_string_ints".to_string(),
        FunctionType::all_borrow(
            vec![("tokens".to_string(), Type::List(Box::new(Type::Str)))],
            Type::Str,
        ),
    );

    // json_dump_tokens_web(tokens: list[str]) -> Result[str, JsonIntegerRangeError]
    functions.insert(
        "json_dump_tokens_web".to_string(),
        FunctionType::all_borrow(
            vec![("tokens".to_string(), Type::List(Box::new(Type::Str)))],
            Type::Result(Box::new(Type::Str), Box::new(json_integer_range_error_ty())),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
