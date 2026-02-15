//! Sifr Standard Library Type Registry
//!
//! Defines type signatures for all `sifr.*` stdlib modules.
//! Used by the HIR lowering to type-check stdlib imports.

use sifr_type_system::{Type, FunctionType};
use std::collections::HashMap;

/// A stdlib module definition with its functions and constants.
pub struct StdlibModule {
    pub functions: HashMap<String, FunctionType>,
    pub constants: HashMap<String, Type>,
}

/// Look up a stdlib module by its dotted name (e.g., "sifr.io").
/// Returns None if the module is not a known stdlib module.
pub fn get_stdlib_module(module_name: &str) -> Option<StdlibModule> {
    match module_name {
        "sifr.io" => Some(stdlib_io()),
        "sifr.json" => Some(stdlib_json()),
        "sifr.env" => Some(stdlib_env()),
        "sifr.os" => Some(stdlib_os()),
        "sifr.math" => Some(stdlib_math()),
        "sifr.test" => Some(stdlib_test()),
        "sifr.collections" => Some(stdlib_collections()),
        "sifr.bytes" => Some(stdlib_bytes()),
        "sifr.time" => Some(stdlib_time()),
        "sifr.random" => Some(stdlib_random()),
        "sifr.re" => Some(stdlib_re()),
        "sifr.hash" => Some(stdlib_hash()),
        "sifr.encoding" => Some(stdlib_encoding()),
        _ => None,
    }
}

/// Check if a module name is a stdlib module.
pub fn is_stdlib_module(module_name: &str) -> bool {
    module_name.starts_with("sifr.")
}

/// sifr.io — File I/O operations
fn stdlib_io() -> StdlibModule {
    let mut functions = HashMap::new();

    // read_file(path: str) -> str
    functions.insert("read_file".to_string(), FunctionType {
        params: vec![("path".to_string(), Type::Str)],
        return_type: Box::new(Type::Str),
    });

    // write_file(path: str, content: str) -> None
    functions.insert("write_file".to_string(), FunctionType {
        params: vec![
            ("path".to_string(), Type::Str),
            ("content".to_string(), Type::Str),
        ],
        return_type: Box::new(Type::None),
    });

    // file_exists(path: str) -> bool
    functions.insert("file_exists".to_string(), FunctionType {
        params: vec![("path".to_string(), Type::Str)],
        return_type: Box::new(Type::Bool),
    });

    // read_lines(path: str) -> list[str]
    functions.insert("read_lines".to_string(), FunctionType {
        params: vec![("path".to_string(), Type::Str)],
        return_type: Box::new(Type::List(Box::new(Type::Str))),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.json — JSON serialization/deserialization
fn stdlib_json() -> StdlibModule {
    let mut functions = HashMap::new();

    // json_loads(s: str) -> str  (returns JSON as string for now)
    functions.insert("json_loads".to_string(), FunctionType {
        params: vec![("s".to_string(), Type::Str)],
        return_type: Box::new(Type::Str),
    });

    // json_dumps(obj: str) -> str
    functions.insert("json_dumps".to_string(), FunctionType {
        params: vec![("obj".to_string(), Type::Str)],
        return_type: Box::new(Type::Str),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.env — Environment variables
fn stdlib_env() -> StdlibModule {
    let mut functions = HashMap::new();

    // get_env(key: str) -> str | None
    functions.insert("get_env".to_string(), FunctionType {
        params: vec![("key".to_string(), Type::Str)],
        return_type: Box::new(Type::Union(vec![Type::Str, Type::None])),
    });

    // set_env(key: str, value: str) -> None
    functions.insert("set_env".to_string(), FunctionType {
        params: vec![
            ("key".to_string(), Type::Str),
            ("value".to_string(), Type::Str),
        ],
        return_type: Box::new(Type::None),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.os — OS operations
fn stdlib_os() -> StdlibModule {
    let mut functions = HashMap::new();

    // run_command(cmd: str) -> str
    functions.insert("run_command".to_string(), FunctionType {
        params: vec![("cmd".to_string(), Type::Str)],
        return_type: Box::new(Type::Str),
    });

    // get_args() -> list[str]
    functions.insert("get_args".to_string(), FunctionType {
        params: vec![],
        return_type: Box::new(Type::List(Box::new(Type::Str))),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.math — Math functions and constants
fn stdlib_math() -> StdlibModule {
    let mut functions = HashMap::new();
    let mut constants = HashMap::new();

    // sqrt(x: float) -> float
    functions.insert("sqrt".to_string(), FunctionType {
        params: vec![("x".to_string(), Type::Float)],
        return_type: Box::new(Type::Float),
    });

    // floor(x: float) -> int
    functions.insert("floor".to_string(), FunctionType {
        params: vec![("x".to_string(), Type::Float)],
        return_type: Box::new(Type::Int),
    });

    // ceil(x: float) -> int
    functions.insert("ceil".to_string(), FunctionType {
        params: vec![("x".to_string(), Type::Float)],
        return_type: Box::new(Type::Int),
    });

    // abs_float(x: float) -> float  (to avoid conflict with built-in abs for int)
    functions.insert("fabs".to_string(), FunctionType {
        params: vec![("x".to_string(), Type::Float)],
        return_type: Box::new(Type::Float),
    });

    // Constants
    constants.insert("pi".to_string(), Type::Float);
    constants.insert("e".to_string(), Type::Float);

    StdlibModule {
        functions,
        constants,
    }
}

/// sifr.test — Test assertions
fn stdlib_test() -> StdlibModule {
    let mut functions = HashMap::new();

    // assert_eq(actual: Any, expected: Any) -> None
    functions.insert("assert_eq".to_string(), FunctionType {
        params: vec![
            ("actual".to_string(), Type::Any),
            ("expected".to_string(), Type::Any),
        ],
        return_type: Box::new(Type::None),
    });

    // assert_ne(actual: Any, expected: Any) -> None
    functions.insert("assert_ne".to_string(), FunctionType {
        params: vec![
            ("actual".to_string(), Type::Any),
            ("expected".to_string(), Type::Any),
        ],
        return_type: Box::new(Type::None),
    });

    // assert_true(value: bool) -> None
    functions.insert("assert_true".to_string(), FunctionType {
        params: vec![("value".to_string(), Type::Bool)],
        return_type: Box::new(Type::None),
    });

    // assert_false(value: bool) -> None
    functions.insert("assert_false".to_string(), FunctionType {
        params: vec![("value".to_string(), Type::Bool)],
        return_type: Box::new(Type::None),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.collections — Extended collection types
/// Since we can't add new types without generics, we use functions that
/// operate on existing types (list[int] for sets, dict for counters/defaultdicts)
fn stdlib_collections() -> StdlibModule {
    let mut functions = HashMap::new();

    // --- Set operations (backed by list[int] with dedup) ---

    // new_set() -> list[int]
    functions.insert("new_set".to_string(), FunctionType {
        params: vec![],
        return_type: Box::new(Type::List(Box::new(Type::Int))),
    });

    // set_from_list(items: list[int]) -> list[int]
    functions.insert("set_from_list".to_string(), FunctionType {
        params: vec![("items".to_string(), Type::List(Box::new(Type::Int)))],
        return_type: Box::new(Type::List(Box::new(Type::Int))),
    });

    // set_add(s: list[int], item: int) -> list[int]
    functions.insert("set_add".to_string(), FunctionType {
        params: vec![
            ("s".to_string(), Type::List(Box::new(Type::Int))),
            ("item".to_string(), Type::Int),
        ],
        return_type: Box::new(Type::List(Box::new(Type::Int))),
    });

    // set_contains(s: list[int], item: int) -> bool
    functions.insert("set_contains".to_string(), FunctionType {
        params: vec![
            ("s".to_string(), Type::List(Box::new(Type::Int))),
            ("item".to_string(), Type::Int),
        ],
        return_type: Box::new(Type::Bool),
    });

    // set_remove(s: list[int], item: int) -> list[int]
    functions.insert("set_remove".to_string(), FunctionType {
        params: vec![
            ("s".to_string(), Type::List(Box::new(Type::Int))),
            ("item".to_string(), Type::Int),
        ],
        return_type: Box::new(Type::List(Box::new(Type::Int))),
    });

    // set_len(s: list[int]) -> int
    functions.insert("set_len".to_string(), FunctionType {
        params: vec![("s".to_string(), Type::List(Box::new(Type::Int)))],
        return_type: Box::new(Type::Int),
    });

    // set_union(a: list[int], b: list[int]) -> list[int]
    functions.insert("set_union".to_string(), FunctionType {
        params: vec![
            ("a".to_string(), Type::List(Box::new(Type::Int))),
            ("b".to_string(), Type::List(Box::new(Type::Int))),
        ],
        return_type: Box::new(Type::List(Box::new(Type::Int))),
    });

    // set_intersection(a: list[int], b: list[int]) -> list[int]
    functions.insert("set_intersection".to_string(), FunctionType {
        params: vec![
            ("a".to_string(), Type::List(Box::new(Type::Int))),
            ("b".to_string(), Type::List(Box::new(Type::Int))),
        ],
        return_type: Box::new(Type::List(Box::new(Type::Int))),
    });

    // --- Counter (backed by dict[str, int] via HashMap) ---

    // counter_from_list(items: list[str]) -> str (JSON-encoded counts)
    functions.insert("counter_from_list".to_string(), FunctionType {
        params: vec![("items".to_string(), Type::List(Box::new(Type::Str)))],
        return_type: Box::new(Type::Str),
    });

    // counter_get(counter: str, key: str) -> int
    functions.insert("counter_get".to_string(), FunctionType {
        params: vec![
            ("counter".to_string(), Type::Str),
            ("key".to_string(), Type::Str),
        ],
        return_type: Box::new(Type::Int),
    });

    // counter_most_common(counter: str, n: int) -> str (JSON-encoded list of pairs)
    functions.insert("counter_most_common".to_string(), FunctionType {
        params: vec![
            ("counter".to_string(), Type::Str),
            ("n".to_string(), Type::Int),
        ],
        return_type: Box::new(Type::Str),
    });

    // --- DefaultDict ---

    // defaultdict_new(default_value: int) -> str (JSON-encoded empty dict with default)
    functions.insert("defaultdict_new".to_string(), FunctionType {
        params: vec![("default_value".to_string(), Type::Int)],
        return_type: Box::new(Type::Str),
    });

    // defaultdict_get(dd: str, key: str) -> int
    functions.insert("defaultdict_get".to_string(), FunctionType {
        params: vec![
            ("dd".to_string(), Type::Str),
            ("key".to_string(), Type::Str),
        ],
        return_type: Box::new(Type::Int),
    });

    // defaultdict_set(dd: str, key: str, value: int) -> str
    functions.insert("defaultdict_set".to_string(), FunctionType {
        params: vec![
            ("dd".to_string(), Type::Str),
            ("key".to_string(), Type::Str),
            ("value".to_string(), Type::Int),
        ],
        return_type: Box::new(Type::Str),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.bytes — Binary data operations
fn stdlib_bytes() -> StdlibModule {
    let mut functions = HashMap::new();

    // encode_utf8(s: str) -> list[int]
    functions.insert("encode_utf8".to_string(), FunctionType {
        params: vec![("s".to_string(), Type::Str)],
        return_type: Box::new(Type::List(Box::new(Type::Int))),
    });

    // decode_utf8(bytes: list[int]) -> str
    functions.insert("decode_utf8".to_string(), FunctionType {
        params: vec![("bytes".to_string(), Type::List(Box::new(Type::Int)))],
        return_type: Box::new(Type::Str),
    });

    // bytes_to_hex(bytes: list[int]) -> str
    functions.insert("bytes_to_hex".to_string(), FunctionType {
        params: vec![("bytes".to_string(), Type::List(Box::new(Type::Int)))],
        return_type: Box::new(Type::Str),
    });

    // bytes_from_hex(s: str) -> list[int]
    functions.insert("bytes_from_hex".to_string(), FunctionType {
        params: vec![("s".to_string(), Type::Str)],
        return_type: Box::new(Type::List(Box::new(Type::Int))),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.time — Time operations
fn stdlib_time() -> StdlibModule {
    let mut functions = HashMap::new();

    // time_now() -> float (epoch seconds)
    functions.insert("time_now".to_string(), FunctionType {
        params: vec![],
        return_type: Box::new(Type::Float),
    });

    // sleep(seconds: float) -> None
    functions.insert("sleep".to_string(), FunctionType {
        params: vec![("seconds".to_string(), Type::Float)],
        return_type: Box::new(Type::None),
    });

    // time_format(epoch: float, fmt: str) -> str
    functions.insert("time_format".to_string(), FunctionType {
        params: vec![
            ("epoch".to_string(), Type::Float),
            ("fmt".to_string(), Type::Str),
        ],
        return_type: Box::new(Type::Str),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.random — Random number generation
fn stdlib_random() -> StdlibModule {
    let mut functions = HashMap::new();

    // random_int(min: int, max: int) -> int
    functions.insert("random_int".to_string(), FunctionType {
        params: vec![
            ("min".to_string(), Type::Int),
            ("max".to_string(), Type::Int),
        ],
        return_type: Box::new(Type::Int),
    });

    // random_float() -> float
    functions.insert("random_float".to_string(), FunctionType {
        params: vec![],
        return_type: Box::new(Type::Float),
    });

    // random_choice(items: list[int]) -> int
    functions.insert("random_choice".to_string(), FunctionType {
        params: vec![("items".to_string(), Type::List(Box::new(Type::Int)))],
        return_type: Box::new(Type::Int),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.re — Regular expressions
fn stdlib_re() -> StdlibModule {
    let mut functions = HashMap::new();

    // re_match(pattern: str, text: str) -> bool
    functions.insert("re_match".to_string(), FunctionType {
        params: vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ],
        return_type: Box::new(Type::Bool),
    });

    // re_find(pattern: str, text: str) -> str | None
    functions.insert("re_find".to_string(), FunctionType {
        params: vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ],
        return_type: Box::new(Type::Union(vec![Type::Str, Type::None])),
    });

    // re_replace(pattern: str, replacement: str, text: str) -> str
    functions.insert("re_replace".to_string(), FunctionType {
        params: vec![
            ("pattern".to_string(), Type::Str),
            ("replacement".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ],
        return_type: Box::new(Type::Str),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.hash — Hashing functions
fn stdlib_hash() -> StdlibModule {
    let mut functions = HashMap::new();

    // sha256(s: str) -> str (hex digest)
    functions.insert("sha256".to_string(), FunctionType {
        params: vec![("s".to_string(), Type::Str)],
        return_type: Box::new(Type::Str),
    });

    // md5_hash(s: str) -> str (hex digest)
    functions.insert("md5_hash".to_string(), FunctionType {
        params: vec![("s".to_string(), Type::Str)],
        return_type: Box::new(Type::Str),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}

/// sifr.encoding — Encoding/decoding utilities
fn stdlib_encoding() -> StdlibModule {
    let mut functions = HashMap::new();

    // base64_encode(s: str) -> str
    functions.insert("base64_encode".to_string(), FunctionType {
        params: vec![("s".to_string(), Type::Str)],
        return_type: Box::new(Type::Str),
    });

    // base64_decode(s: str) -> str
    functions.insert("base64_decode".to_string(), FunctionType {
        params: vec![("s".to_string(), Type::Str)],
        return_type: Box::new(Type::Str),
    });

    StdlibModule {
        functions,
        constants: HashMap::new(),
    }
}
