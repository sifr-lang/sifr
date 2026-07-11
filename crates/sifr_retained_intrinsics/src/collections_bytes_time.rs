use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::HashMap;

/// _sifr.collections — Extended collection intrinsics
pub(super) fn intrinsic_collections() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // --- Private set adapter operations (backed by list[int] with dedup) ---
    functions.insert(
        "_new_set_impl".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))),
    );

    functions.insert(
        "_set_from_list_impl".to_string(),
        FunctionType {
            params: vec![(
                "items".to_string(),
                Type::List(Box::new(Type::Int)),
                ParamConvention::own(),
            )],
            return_type: Box::new(Type::List(Box::new(Type::Int))),
        },
    );

    functions.insert(
        "_set_add_impl".to_string(),
        FunctionType {
            params: vec![
                (
                    "s".to_string(),
                    Type::List(Box::new(Type::Int)),
                    ParamConvention::own(),
                ),
                ("item".to_string(), Type::Int, ParamConvention::own()),
            ],
            return_type: Box::new(Type::List(Box::new(Type::Int))),
        },
    );

    functions.insert(
        "_set_contains_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("s".to_string(), Type::List(Box::new(Type::Int))),
                ("item".to_string(), Type::Int),
            ],
            Type::Bool,
        ),
    );

    functions.insert(
        "_set_remove_impl".to_string(),
        FunctionType {
            params: vec![
                (
                    "s".to_string(),
                    Type::List(Box::new(Type::Int)),
                    ParamConvention::own(),
                ),
                ("item".to_string(), Type::Int, ParamConvention::own()),
            ],
            return_type: Box::new(Type::List(Box::new(Type::Int))),
        },
    );

    functions.insert(
        "_set_len_impl".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::List(Box::new(Type::Int)))],
            Type::Int,
        ),
    );

    functions.insert(
        "_set_union_impl".to_string(),
        FunctionType {
            params: vec![
                (
                    "a".to_string(),
                    Type::List(Box::new(Type::Int)),
                    ParamConvention::own(),
                ),
                (
                    "b".to_string(),
                    Type::List(Box::new(Type::Int)),
                    ParamConvention::own(),
                ),
            ],
            return_type: Box::new(Type::List(Box::new(Type::Int))),
        },
    );

    functions.insert(
        "_set_intersection_impl".to_string(),
        FunctionType {
            params: vec![
                (
                    "a".to_string(),
                    Type::List(Box::new(Type::Int)),
                    ParamConvention::own(),
                ),
                (
                    "b".to_string(),
                    Type::List(Box::new(Type::Int)),
                    ParamConvention::own(),
                ),
            ],
            return_type: Box::new(Type::List(Box::new(Type::Int))),
        },
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.bytes — Binary data intrinsics
pub(super) fn intrinsic_bytes() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // encode_utf8(s: str) -> bytes
    functions.insert(
        "encode_utf8".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Bytes),
    );

    // decode_utf8(bytes: bytes) -> Result[str, ParseError]
    functions.insert(
        "decode_utf8".to_string(),
        FunctionType::all_borrow(
            vec![("bytes".to_string(), Type::Bytes)],
            result_ty(Type::Str, "ParseError"),
        ),
    );

    // bytes_to_hex(bytes: bytes) -> Result[str, ParseError]
    functions.insert(
        "bytes_to_hex".to_string(),
        FunctionType::all_borrow(
            vec![("bytes".to_string(), Type::Bytes)],
            result_ty(Type::Str, "ParseError"),
        ),
    );

    // bytes_to_hex_strict(bytes: bytes) -> str (infallible fast path)
    functions.insert(
        "bytes_to_hex_strict".to_string(),
        FunctionType::all_borrow(vec![("bytes".to_string(), Type::Bytes)], Type::Str),
    );

    // bytes_from_hex(s: str) -> Result[bytes, ParseError]
    functions.insert(
        "bytes_from_hex".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str)],
            result_ty(Type::Bytes, "ParseError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
