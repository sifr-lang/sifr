//! Sifr Intrinsic Type Registry
//!
//! Defines type signatures for all `_sifr.*` intrinsic modules.
//! These are compiler-provided primitives that map directly to Rust code.
//! User-facing stdlib modules live in `lib/sifr/*.sifr` files.

use sifr_type_system::{Type, FunctionType};
use std::collections::HashMap;

/// An intrinsic module definition with its functions and constants.
pub struct IntrinsicModule {
    pub functions: HashMap<String, FunctionType>,
    pub constants: HashMap<String, Type>,
}

/// Look up an intrinsic module by its dotted name (e.g., "_sifr.io").
/// Returns None if the module is not a known intrinsic module.
pub fn get_intrinsic_module(module_name: &str) -> Option<IntrinsicModule> {
    match module_name {
        "_sifr.io" => Some(intrinsic_io()),
        "_sifr.json" => Some(intrinsic_json()),
        "_sifr.sys" => Some(intrinsic_sys()),
        "_sifr.fs" => Some(intrinsic_fs()),
        "_sifr.math" => Some(intrinsic_math()),
        "_sifr.test" => Some(intrinsic_test()),
        "_sifr.collections" => Some(intrinsic_collections()),
        "_sifr.bytes" => Some(intrinsic_bytes()),
        "_sifr.time" => Some(intrinsic_time()),
        "_sifr.crypto" => Some(intrinsic_crypto()),
        "_sifr.regex" => Some(intrinsic_regex()),
        "_sifr.uuid" => Some(intrinsic_uuid()),
        "_sifr.platform" => Some(intrinsic_platform()),
        _ => None,
    }
}

/// Check if a module name is an intrinsic module.
pub fn is_intrinsic_module(module_name: &str) -> bool {
    module_name.starts_with("_sifr.")
}

/// Check if a module name is a user-facing stdlib module.
pub fn is_stdlib_module(module_name: &str) -> bool {
    module_name.starts_with("sifr.")
}

/// _sifr.io — File I/O intrinsics
fn intrinsic_io() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // read_text(path: str) -> str
    functions.insert("read_text".to_string(), FunctionType::all_borrow(
        vec![("path".to_string(), Type::Str)],
        Type::Str,
    ));

    // write_text(path: str, content: str) -> None
    functions.insert("write_text".to_string(), FunctionType::all_borrow(vec![
            ("path".to_string(), Type::Str),
            ("content".to_string(), Type::Str),
        ], Type::None));

    // exists(path: str) -> bool
    functions.insert("exists".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool));

    // read_lines(path: str) -> list[str]
    functions.insert("read_lines".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::List(Box::new(Type::Str))));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.json — JSON serialization/deserialization intrinsics
fn intrinsic_json() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // json_loads(s: str) -> str  (returns JSON as string for now)
    functions.insert("json_loads".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // json_dumps(obj: Any) -> str
    functions.insert("json_dumps".to_string(), FunctionType::all_borrow(vec![("obj".to_string(), Type::Any)], Type::Str));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.math — Math intrinsics
fn intrinsic_math() -> IntrinsicModule {
    let mut functions = HashMap::new();
    let mut constants = HashMap::new();

    // sqrt(x: float) -> float
    functions.insert("sqrt".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // floor(x: float) -> int
    functions.insert("floor".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Int));

    // ceil(x: float) -> int
    functions.insert("ceil".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Int));

    // abs_val(x: float) -> float
    functions.insert("abs_val".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // log(x: float) -> float
    functions.insert("log".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // sin(x: float) -> float
    functions.insert("sin".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // cos(x: float) -> float
    functions.insert("cos".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // tan(x: float) -> float
    functions.insert("tan".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // pow_val(x: float, y: float) -> float
    functions.insert("pow_val".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float), ("y".to_string(), Type::Float)], Type::Float));

    // min_val(a: float, b: float) -> float
    functions.insert("min_val".to_string(), FunctionType::all_borrow(vec![("a".to_string(), Type::Float), ("b".to_string(), Type::Float)], Type::Float));

    // max_val(a: float, b: float) -> float
    functions.insert("max_val".to_string(), FunctionType::all_borrow(vec![("a".to_string(), Type::Float), ("b".to_string(), Type::Float)], Type::Float));

    // round_val(x: float) -> int
    functions.insert("round_val".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Int));

    // asin(x: float) -> float
    functions.insert("asin".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // acos(x: float) -> float
    functions.insert("acos".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // atan(x: float) -> float
    functions.insert("atan".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // atan2(y: float, x: float) -> float
    functions.insert("atan2".to_string(), FunctionType::all_borrow(vec![("y".to_string(), Type::Float), ("x".to_string(), Type::Float)], Type::Float));

    // sinh(x: float) -> float
    functions.insert("sinh".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // cosh(x: float) -> float
    functions.insert("cosh".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // tanh(x: float) -> float
    functions.insert("tanh".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // log10(x: float) -> float
    functions.insert("log10".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // log2(x: float) -> float
    functions.insert("log2".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // degrees(x: float) -> float (radians to degrees)
    functions.insert("degrees".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // radians(x: float) -> float (degrees to radians)
    functions.insert("radians".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // isnan(x: float) -> bool
    functions.insert("isnan".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool));

    // isinf(x: float) -> bool
    functions.insert("isinf".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool));

    // trunc(x: float) -> int
    functions.insert("trunc".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Int));

    // copysign(x: float, y: float) -> float
    functions.insert("copysign".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float), ("y".to_string(), Type::Float)], Type::Float));

    // fmod(x: float, y: float) -> float
    functions.insert("fmod".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float), ("y".to_string(), Type::Float)], Type::Float));

    // hypot(x: float, y: float) -> float
    functions.insert("hypot".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float), ("y".to_string(), Type::Float)], Type::Float));

    // Constants
    constants.insert("pi".to_string(), Type::Float);
    constants.insert("e".to_string(), Type::Float);
    constants.insert("tau".to_string(), Type::Float);
    constants.insert("inf".to_string(), Type::Float);
    constants.insert("nan".to_string(), Type::Float);

    IntrinsicModule {
        functions,
        constants,
    }
}

/// _sifr.test — Test assertion intrinsics
fn intrinsic_test() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // assert_eq(actual: Any, expected: Any) -> None
    functions.insert("assert_eq".to_string(), FunctionType::all_borrow(vec![
            ("actual".to_string(), Type::Any),
            ("expected".to_string(), Type::Any),
        ], Type::None));

    // assert_ne(actual: Any, expected: Any) -> None
    functions.insert("assert_ne".to_string(), FunctionType::all_borrow(vec![
            ("actual".to_string(), Type::Any),
            ("expected".to_string(), Type::Any),
        ], Type::None));

    // assert_true(value: bool) -> None
    functions.insert("assert_true".to_string(), FunctionType::all_borrow(vec![("value".to_string(), Type::Bool)], Type::None));

    // assert_false(value: bool) -> None
    functions.insert("assert_false".to_string(), FunctionType::all_borrow(vec![("value".to_string(), Type::Bool)], Type::None));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.collections — Extended collection intrinsics
fn intrinsic_collections() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // --- Set operations (backed by list[int] with dedup) ---

    // new_set() -> list[int]
    functions.insert("new_set".to_string(), FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))));

    // set_from_list(items: list[int]) -> list[int]
    functions.insert("set_from_list".to_string(), FunctionType::all_borrow(vec![("items".to_string(), Type::List(Box::new(Type::Int)))], Type::List(Box::new(Type::Int))));

    // set_add(s: list[int], item: int) -> list[int]
    functions.insert("set_add".to_string(), FunctionType::all_borrow(vec![
            ("s".to_string(), Type::List(Box::new(Type::Int))),
            ("item".to_string(), Type::Int),
        ], Type::List(Box::new(Type::Int))));

    // set_contains(s: list[int], item: int) -> bool
    functions.insert("set_contains".to_string(), FunctionType::all_borrow(vec![
            ("s".to_string(), Type::List(Box::new(Type::Int))),
            ("item".to_string(), Type::Int),
        ], Type::Bool));

    // set_remove(s: list[int], item: int) -> list[int]
    functions.insert("set_remove".to_string(), FunctionType::all_borrow(vec![
            ("s".to_string(), Type::List(Box::new(Type::Int))),
            ("item".to_string(), Type::Int),
        ], Type::List(Box::new(Type::Int))));

    // set_len(s: list[int]) -> int
    functions.insert("set_len".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::List(Box::new(Type::Int)))], Type::Int));

    // set_union(a: list[int], b: list[int]) -> list[int]
    functions.insert("set_union".to_string(), FunctionType::all_borrow(vec![
            ("a".to_string(), Type::List(Box::new(Type::Int))),
            ("b".to_string(), Type::List(Box::new(Type::Int))),
        ], Type::List(Box::new(Type::Int))));

    // set_intersection(a: list[int], b: list[int]) -> list[int]
    functions.insert("set_intersection".to_string(), FunctionType::all_borrow(vec![
            ("a".to_string(), Type::List(Box::new(Type::Int))),
            ("b".to_string(), Type::List(Box::new(Type::Int))),
        ], Type::List(Box::new(Type::Int))));

    // --- Counter (backed by dict[str, int] via HashMap) ---

    // counter_from_list(items: list[str]) -> str (JSON-encoded counts)
    functions.insert("counter_from_list".to_string(), FunctionType::all_borrow(vec![("items".to_string(), Type::List(Box::new(Type::Str)))], Type::Str));

    // counter_get(counter: str, key: str) -> int
    functions.insert("counter_get".to_string(), FunctionType::all_borrow(vec![
            ("counter".to_string(), Type::Str),
            ("key".to_string(), Type::Str),
        ], Type::Int));

    // counter_most_common(counter: str, n: int) -> str (JSON-encoded list of pairs)
    functions.insert("counter_most_common".to_string(), FunctionType::all_borrow(vec![
            ("counter".to_string(), Type::Str),
            ("n".to_string(), Type::Int),
        ], Type::Str));

    // --- DefaultDict ---

    // defaultdict_new(default_value: int) -> str (JSON-encoded empty dict with default)
    functions.insert("defaultdict_new".to_string(), FunctionType::all_borrow(vec![("default_value".to_string(), Type::Int)], Type::Str));

    // defaultdict_get(dd: str, key: str) -> int
    functions.insert("defaultdict_get".to_string(), FunctionType::all_borrow(vec![
            ("dd".to_string(), Type::Str),
            ("key".to_string(), Type::Str),
        ], Type::Int));

    // defaultdict_set(dd: str, key: str, value: int) -> str
    functions.insert("defaultdict_set".to_string(), FunctionType::all_borrow(vec![
            ("dd".to_string(), Type::Str),
            ("key".to_string(), Type::Str),
            ("value".to_string(), Type::Int),
        ], Type::Str));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.bytes — Binary data intrinsics
fn intrinsic_bytes() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // encode_utf8(s: str) -> list[int]
    functions.insert("encode_utf8".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::List(Box::new(Type::Int))));

    // decode_utf8(bytes: list[int]) -> str
    functions.insert("decode_utf8".to_string(), FunctionType::all_borrow(vec![("bytes".to_string(), Type::List(Box::new(Type::Int)))], Type::Str));

    // bytes_to_hex(bytes: list[int]) -> str
    functions.insert("bytes_to_hex".to_string(), FunctionType::all_borrow(vec![("bytes".to_string(), Type::List(Box::new(Type::Int)))], Type::Str));

    // bytes_from_hex(s: str) -> list[int]
    functions.insert("bytes_from_hex".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::List(Box::new(Type::Int))));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.time — Time intrinsics
fn intrinsic_time() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // time_now() -> float (epoch seconds)
    functions.insert("time_now".to_string(), FunctionType::all_borrow(vec![], Type::Float));

    // sleep(seconds: float) -> None
    functions.insert("sleep".to_string(), FunctionType::all_borrow(vec![("seconds".to_string(), Type::Float)], Type::None));

    // time_format(epoch: float, fmt: str) -> str
    functions.insert("time_format".to_string(), FunctionType::all_borrow(vec![
            ("epoch".to_string(), Type::Float),
            ("fmt".to_string(), Type::Str),
        ], Type::Str));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.sys — Combined system intrinsics (env + os)
fn intrinsic_sys() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // env_get(key: str) -> str | None
    functions.insert("env_get".to_string(), FunctionType::all_borrow(vec![("key".to_string(), Type::Str)], Type::Union(vec![Type::Str, Type::None])));

    // env_set(key: str, value: str) -> None
    functions.insert("env_set".to_string(), FunctionType::all_borrow(vec![
            ("key".to_string(), Type::Str),
            ("value".to_string(), Type::Str),
        ], Type::None));

    // run_command(cmd: str) -> str
    functions.insert("run_command".to_string(), FunctionType::all_borrow(vec![("cmd".to_string(), Type::Str)], Type::Str));

    // get_args() -> list[str]
    functions.insert("get_args".to_string(), FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Str))));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.fs — File system intrinsics (io + os file ops)
fn intrinsic_fs() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // read_text(path: str) -> str
    functions.insert("read_text".to_string(), FunctionType::all_borrow(
        vec![("path".to_string(), Type::Str)],
        Type::Str,
    ));

    // write_text(path: str, content: str) -> None
    functions.insert("write_text".to_string(), FunctionType::all_borrow(vec![
            ("path".to_string(), Type::Str),
            ("content".to_string(), Type::Str),
        ], Type::None));

    // exists(path: str) -> bool
    functions.insert("exists".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool));

    // read_lines(path: str) -> list[str]
    functions.insert("read_lines".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::List(Box::new(Type::Str))));

    // append_text(path: str, content: str) -> None
    functions.insert("append_text".to_string(), FunctionType::all_borrow(vec![
            ("path".to_string(), Type::Str),
            ("content".to_string(), Type::Str),
        ], Type::None));

    // getcwd() -> str
    functions.insert("getcwd".to_string(), FunctionType::all_borrow(vec![], Type::Str));

    // listdir(path: str) -> list[str]
    functions.insert("listdir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::List(Box::new(Type::Str))));

    // mkdir(path: str) -> None
    functions.insert("mkdir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::None));

    // rmdir(path: str) -> None
    functions.insert("rmdir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::None));

    // remove_file(path: str) -> None
    functions.insert("remove_file".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::None));

    // rename(src: str, dst: str) -> None
    functions.insert("rename".to_string(), FunctionType::all_borrow(vec![
            ("src".to_string(), Type::Str),
            ("dst".to_string(), Type::Str),
        ], Type::None));

    // is_file(path: str) -> bool
    functions.insert("is_file".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool));

    // is_dir(path: str) -> bool
    functions.insert("is_dir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.crypto — Combined crypto intrinsics (random + hash + encoding)
fn intrinsic_crypto() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // random_int(min: int, max: int) -> int
    functions.insert("random_int".to_string(), FunctionType::all_borrow(vec![
            ("min".to_string(), Type::Int),
            ("max".to_string(), Type::Int),
        ], Type::Int));

    // random_float() -> float
    functions.insert("random_float".to_string(), FunctionType::all_borrow(vec![], Type::Float));

    // random_choice(items: list[Any]) -> Any
    functions.insert("random_choice".to_string(), FunctionType::all_borrow(vec![("items".to_string(), Type::List(Box::new(Type::Any)))], Type::Any));

    // sha256(s: str) -> str (hex digest)
    functions.insert("sha256".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // md5(s: str) -> str (hex digest)
    functions.insert("md5".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // base64_encode(s: str) -> str
    functions.insert("base64_encode".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // base64_decode(s: str) -> str
    functions.insert("base64_decode".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // random_uniform(min: float, max: float) -> float
    functions.insert("random_uniform".to_string(), FunctionType::all_borrow(vec![
            ("min".to_string(), Type::Float),
            ("max".to_string(), Type::Float),
        ], Type::Float));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.regex — Combined regex intrinsics
fn intrinsic_regex() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // re_match(pattern: str, text: str) -> bool
    functions.insert("re_match".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], Type::Bool));

    // re_find(pattern: str, text: str) -> str | None
    functions.insert("re_find".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], Type::Union(vec![Type::Str, Type::None])));

    // re_replace(pattern: str, replacement: str, text: str) -> str
    functions.insert("re_replace".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("replacement".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], Type::Str));

    // re_findall(pattern: str, text: str) -> list[str]
    functions.insert("re_findall".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], Type::List(Box::new(Type::Str))));

    // re_split(pattern: str, text: str) -> list[str]
    functions.insert("re_split".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], Type::List(Box::new(Type::Str))));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.uuid — UUID generation intrinsics
fn intrinsic_uuid() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // uuid4() -> str (random UUID v4)
    functions.insert("uuid4".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    IntrinsicModule { functions, constants: HashMap::new() }
}

/// _sifr.platform — Platform information intrinsics
fn intrinsic_platform() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // platform_system() -> str (e.g., "Linux", "Darwin", "Windows")
    functions.insert("platform_system".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    // platform_arch() -> str (e.g., "x86_64", "aarch64")
    functions.insert("platform_arch".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    IntrinsicModule { functions, constants: HashMap::new() }
}
