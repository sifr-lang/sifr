use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.collections — Extended collection intrinsics
pub(super) fn intrinsic_collections() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // --- Set operations (backed by list[int] with dedup) ---

    // new_set() -> list[int]
    functions.insert(
        "new_set".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))),
    );

    // set_from_list(items: list[int]) -> list[int]
    functions.insert(
        "set_from_list".to_string(),
        FunctionType::all_borrow(
            vec![("items".to_string(), Type::List(Box::new(Type::Int)))],
            Type::List(Box::new(Type::Int)),
        ),
    );

    // set_add(s: list[int], item: int) -> list[int]
    functions.insert(
        "set_add".to_string(),
        FunctionType::all_borrow(
            vec![
                ("s".to_string(), Type::List(Box::new(Type::Int))),
                ("item".to_string(), Type::Int),
            ],
            Type::List(Box::new(Type::Int)),
        ),
    );

    // set_contains(s: list[int], item: int) -> bool
    functions.insert(
        "set_contains".to_string(),
        FunctionType::all_borrow(
            vec![
                ("s".to_string(), Type::List(Box::new(Type::Int))),
                ("item".to_string(), Type::Int),
            ],
            Type::Bool,
        ),
    );

    // set_remove(s: list[int], item: int) -> list[int]
    functions.insert(
        "set_remove".to_string(),
        FunctionType::all_borrow(
            vec![
                ("s".to_string(), Type::List(Box::new(Type::Int))),
                ("item".to_string(), Type::Int),
            ],
            Type::List(Box::new(Type::Int)),
        ),
    );

    // set_len(s: list[int]) -> int
    functions.insert(
        "set_len".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::List(Box::new(Type::Int)))],
            Type::Int,
        ),
    );

    // set_union(a: list[int], b: list[int]) -> list[int]
    functions.insert(
        "set_union".to_string(),
        FunctionType::all_borrow(
            vec![
                ("a".to_string(), Type::List(Box::new(Type::Int))),
                ("b".to_string(), Type::List(Box::new(Type::Int))),
            ],
            Type::List(Box::new(Type::Int)),
        ),
    );

    // set_intersection(a: list[int], b: list[int]) -> list[int]
    functions.insert(
        "set_intersection".to_string(),
        FunctionType::all_borrow(
            vec![
                ("a".to_string(), Type::List(Box::new(Type::Int))),
                ("b".to_string(), Type::List(Box::new(Type::Int))),
            ],
            Type::List(Box::new(Type::Int)),
        ),
    );

    // --- Counter (backed by dict[str, int] via HashMap) ---

    // counter_from_list(items: list[str]) -> str (JSON-encoded counts)
    functions.insert(
        "counter_from_list".to_string(),
        FunctionType::all_borrow(
            vec![("items".to_string(), Type::List(Box::new(Type::Str)))],
            Type::Str,
        ),
    );

    // counter_get(counter: str, key: str) -> int
    functions.insert(
        "counter_get".to_string(),
        FunctionType::all_borrow(
            vec![
                ("counter".to_string(), Type::Str),
                ("key".to_string(), Type::Str),
            ],
            Type::Int,
        ),
    );

    // counter_most_common(counter: str, n: int) -> str (JSON-encoded list of pairs)
    functions.insert(
        "counter_most_common".to_string(),
        FunctionType::all_borrow(
            vec![
                ("counter".to_string(), Type::Str),
                ("n".to_string(), Type::Int),
            ],
            Type::Str,
        ),
    );

    // counter_total(counter: str) -> int (sum of all counts)
    functions.insert(
        "counter_total".to_string(),
        FunctionType::all_borrow(vec![("counter".to_string(), Type::Str)], Type::Int),
    );

    // counter_values(counter: str) -> list[int] (all count values)
    functions.insert(
        "counter_values".to_string(),
        FunctionType::all_borrow(
            vec![("counter".to_string(), Type::Str)],
            Type::List(Box::new(Type::Int)),
        ),
    );

    // counter_keys(counter: str) -> list[str] (all keys)
    functions.insert(
        "counter_keys".to_string(),
        FunctionType::all_borrow(
            vec![("counter".to_string(), Type::Str)],
            Type::List(Box::new(Type::Str)),
        ),
    );

    // counter_items(counter: str) -> str (JSON-encoded list of [key, count] pairs)
    functions.insert(
        "counter_items".to_string(),
        FunctionType::all_borrow(vec![("counter".to_string(), Type::Str)], Type::Str),
    );

    // counter_increment(counter: str, key: str) -> str (increment key count by 1, return new JSON)
    functions.insert(
        "counter_increment".to_string(),
        FunctionType::all_borrow(
            vec![
                ("counter".to_string(), Type::Str),
                ("key".to_string(), Type::Str),
            ],
            Type::Str,
        ),
    );

    // --- DefaultDict ---

    // defaultdict_new(default_value: int) -> str (JSON-encoded empty dict with default)
    functions.insert(
        "defaultdict_new".to_string(),
        FunctionType::all_borrow(vec![("default_value".to_string(), Type::Int)], Type::Str),
    );

    // defaultdict_get(dd: str, key: str) -> int
    functions.insert(
        "defaultdict_get".to_string(),
        FunctionType::all_borrow(
            vec![
                ("dd".to_string(), Type::Str),
                ("key".to_string(), Type::Str),
            ],
            Type::Int,
        ),
    );

    // defaultdict_set(dd: str, key: str, value: int) -> str
    functions.insert(
        "defaultdict_set".to_string(),
        FunctionType::all_borrow(
            vec![
                ("dd".to_string(), Type::Str),
                ("key".to_string(), Type::Str),
                ("value".to_string(), Type::Int),
            ],
            Type::Str,
        ),
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

/// _sifr.time — Time intrinsics
pub(super) fn intrinsic_time() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // time_now() -> float (epoch seconds)
    functions.insert(
        "time_now".to_string(),
        FunctionType::all_borrow(vec![], Type::Float),
    );

    // sleep(seconds: float) -> None
    functions.insert(
        "sleep".to_string(),
        FunctionType::all_borrow(vec![("seconds".to_string(), Type::Float)], Type::None),
    );

    // time_format(epoch: float, fmt: str) -> str
    functions.insert(
        "time_format".to_string(),
        FunctionType::all_borrow(
            vec![
                ("epoch".to_string(), Type::Float),
                ("fmt".to_string(), Type::Str),
            ],
            Type::Str,
        ),
    );

    // perf_counter() -> float (high-resolution monotonic clock for benchmarking)
    functions.insert(
        "perf_counter".to_string(),
        FunctionType::all_borrow(vec![], Type::Float),
    );

    // monotonic() -> float (guaranteed non-decreasing clock for timeouts)
    functions.insert(
        "monotonic".to_string(),
        FunctionType::all_borrow(vec![], Type::Float),
    );

    // strptime(s: str, fmt: str) -> Result[str, ValueError] (parse time string, return ISO datetime)
    functions.insert(
        "strptime".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str), ("fmt".to_string(), Type::Str)],
            result_ty(Type::Str, "ValueError"),
        ),
    );

    // gmtime(epoch: float) -> str (UTC time as ISO string)
    functions.insert(
        "gmtime".to_string(),
        FunctionType::all_borrow(vec![("epoch".to_string(), Type::Float)], Type::Str),
    );

    // localtime(epoch: float) -> str (local time as ISO string)
    functions.insert(
        "localtime".to_string(),
        FunctionType::all_borrow(vec![("epoch".to_string(), Type::Float)], Type::Str),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
