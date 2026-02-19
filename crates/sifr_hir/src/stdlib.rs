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

/// Helper: construct a built-in error class type (e.g., IOError, ParseError).
/// Built-in error classes have a single `message: str` field.
fn error_class(name: &str) -> Type {
    Type::Class {
        name: name.to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    }
}

/// Helper: construct Result[T, E] where E is a built-in error class.
fn result_ty(ok: Type, error_name: &str) -> Type {
    Type::Result(Box::new(ok), Box::new(error_class(error_name)))
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
        "_sifr.toml" => Some(intrinsic_toml()),
        "_sifr.datetime" => Some(intrinsic_datetime()),
        "_sifr.html" => Some(intrinsic_html()),
        "_sifr.calendar" => Some(intrinsic_calendar()),
        "_sifr.compress" => Some(intrinsic_compress()),
        "_sifr.logging" => Some(intrinsic_logging()),
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

    // read_text(path: str) -> Result[str, IOError]
    functions.insert("read_text".to_string(), FunctionType::all_borrow(
        vec![("path".to_string(), Type::Str)],
        result_ty(Type::Str, "IOError"),
    ));

    // write_text(path: str, content: str) -> Result[None, IOError]
    functions.insert("write_text".to_string(), FunctionType::all_borrow(vec![
            ("path".to_string(), Type::Str),
            ("content".to_string(), Type::Str),
        ], result_ty(Type::None, "IOError")));

    // exists(path: str) -> bool  (infallible — just checks existence)
    functions.insert("exists".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool));

    // read_lines(path: str) -> Result[list[str], IOError]
    functions.insert("read_lines".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.json — JSON serialization/deserialization intrinsics
fn intrinsic_json() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // json_loads(s: str) -> Result[str, JSONDecodeError]
    functions.insert("json_loads".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], result_ty(Type::Str, "JSONDecodeError")));

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

    // exp(x: float) -> float
    functions.insert("exp".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // expm1(x: float) -> float
    functions.insert("expm1".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // log1p(x: float) -> float
    functions.insert("log1p".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // fabs(x: float) -> float
    functions.insert("fabs".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // isfinite(x: float) -> bool
    functions.insert("isfinite".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool));

    // acosh(x: float) -> float
    functions.insert("acosh".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // asinh(x: float) -> float
    functions.insert("asinh".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // atanh(x: float) -> float
    functions.insert("atanh".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // isqrt(n: int) -> int
    functions.insert("isqrt".to_string(), FunctionType::all_borrow(vec![("n".to_string(), Type::Int)], Type::Int));

    // dist(p: list[float], q: list[float]) -> float
    functions.insert("dist".to_string(), FunctionType::all_borrow(vec![("p".to_string(), Type::List(Box::new(Type::Float))), ("q".to_string(), Type::List(Box::new(Type::Float)))], Type::Float));

    // fsum(data: list[float]) -> float
    functions.insert("fsum".to_string(), FunctionType::all_borrow(vec![("data".to_string(), Type::List(Box::new(Type::Float)))], Type::Float));

    // erf(x: float) -> float
    functions.insert("erf".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // erfc(x: float) -> float
    functions.insert("erfc".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // gamma(x: float) -> float
    functions.insert("gamma".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // lgamma(x: float) -> float
    functions.insert("lgamma".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

    // frexp(x: float) -> list[float] ([mantissa, exponent_as_float])
    functions.insert("frexp".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::List(Box::new(Type::Float))));

    // ldexp(m: float, e: int) -> float
    functions.insert("ldexp".to_string(), FunctionType::all_borrow(vec![("m".to_string(), Type::Float), ("e".to_string(), Type::Int)], Type::Float));

    // modf(x: float) -> list[float] ([fractional_part, integer_part])
    functions.insert("modf".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::List(Box::new(Type::Float))));

    // nextafter(x: float, y: float) -> float
    functions.insert("nextafter".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float), ("y".to_string(), Type::Float)], Type::Float));

    // ulp(x: float) -> float
    functions.insert("ulp".to_string(), FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float));

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

    // assert_almost_eq(actual: float, expected: float, tolerance: float) -> None
    functions.insert("assert_almost_eq".to_string(), FunctionType::all_borrow(vec![
        ("actual".to_string(), Type::Float),
        ("expected".to_string(), Type::Float),
        ("tolerance".to_string(), Type::Float),
    ], Type::None));

    // assert_gt(a: int, b: int) -> None
    functions.insert("assert_gt".to_string(), FunctionType::all_borrow(vec![
        ("a".to_string(), Type::Int),
        ("b".to_string(), Type::Int),
    ], Type::None));

    // assert_lt(a: int, b: int) -> None
    functions.insert("assert_lt".to_string(), FunctionType::all_borrow(vec![
        ("a".to_string(), Type::Int),
        ("b".to_string(), Type::Int),
    ], Type::None));

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

    // counter_total(counter: str) -> int (sum of all counts)
    functions.insert("counter_total".to_string(), FunctionType::all_borrow(vec![("counter".to_string(), Type::Str)], Type::Int));

    // counter_values(counter: str) -> list[int] (all count values)
    functions.insert("counter_values".to_string(), FunctionType::all_borrow(vec![("counter".to_string(), Type::Str)], Type::List(Box::new(Type::Int))));

    // counter_keys(counter: str) -> list[str] (all keys)
    functions.insert("counter_keys".to_string(), FunctionType::all_borrow(vec![("counter".to_string(), Type::Str)], Type::List(Box::new(Type::Str))));

    // counter_items(counter: str) -> str (JSON-encoded list of [key, count] pairs)
    functions.insert("counter_items".to_string(), FunctionType::all_borrow(vec![("counter".to_string(), Type::Str)], Type::Str));

    // counter_increment(counter: str, key: str) -> str (increment key count by 1, return new JSON)
    functions.insert("counter_increment".to_string(), FunctionType::all_borrow(vec![
            ("counter".to_string(), Type::Str),
            ("key".to_string(), Type::Str),
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

    // decode_utf8(bytes: list[int]) -> Result[str, ParseError]
    functions.insert("decode_utf8".to_string(), FunctionType::all_borrow(vec![("bytes".to_string(), Type::List(Box::new(Type::Int)))], result_ty(Type::Str, "ParseError")));

    // bytes_to_hex(bytes: list[int]) -> str
    functions.insert("bytes_to_hex".to_string(), FunctionType::all_borrow(vec![("bytes".to_string(), Type::List(Box::new(Type::Int)))], Type::Str));

    // bytes_from_hex(s: str) -> Result[list[int], ParseError]
    functions.insert("bytes_from_hex".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], result_ty(Type::List(Box::new(Type::Int)), "ParseError")));

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

    // perf_counter() -> float (high-resolution monotonic clock for benchmarking)
    functions.insert("perf_counter".to_string(), FunctionType::all_borrow(vec![], Type::Float));

    // monotonic() -> float (guaranteed non-decreasing clock for timeouts)
    functions.insert("monotonic".to_string(), FunctionType::all_borrow(vec![], Type::Float));

    // strptime(s: str, fmt: str) -> Result[str, ValueError] (parse time string, return ISO datetime)
    functions.insert("strptime".to_string(), FunctionType::all_borrow(vec![
        ("s".to_string(), Type::Str),
        ("fmt".to_string(), Type::Str),
    ], result_ty(Type::Str, "ValueError")));

    // gmtime(epoch: float) -> str (UTC time as ISO string)
    functions.insert("gmtime".to_string(), FunctionType::all_borrow(vec![("epoch".to_string(), Type::Float)], Type::Str));

    // localtime(epoch: float) -> str (local time as ISO string)
    functions.insert("localtime".to_string(), FunctionType::all_borrow(vec![("epoch".to_string(), Type::Float)], Type::Str));

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

    // run_command(cmd: str) -> Result[str, IOError]
    functions.insert("run_command".to_string(), FunctionType::all_borrow(vec![("cmd".to_string(), Type::Str)], result_ty(Type::Str, "IOError")));

    // get_args() -> list[str]
    functions.insert("get_args".to_string(), FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Str))));

    // sys_exit(code: int) -> None (terminates the process)
    functions.insert("sys_exit".to_string(), FunctionType::all_borrow(vec![("code".to_string(), Type::Int)], Type::None));

    // sys_version() -> str (Sifr version string)
    functions.insert("sys_version".to_string(), FunctionType::all_borrow(vec![], Type::Str));

    // sys_platform() -> str (platform identifier: "linux", "macos", "windows")
    functions.insert("sys_platform".to_string(), FunctionType::all_borrow(vec![], Type::Str));

    // sys_maxsize() -> int (maximum int size)
    functions.insert("sys_maxsize".to_string(), FunctionType::all_borrow(vec![], Type::Int));

    // subprocess_run(cmd: str) -> Result[str, IOError]
    functions.insert("subprocess_run".to_string(), FunctionType::all_borrow(vec![("cmd".to_string(), Type::Str)], result_ty(Type::Str, "IOError")));

    // subprocess_run_with_input(cmd: str, stdin: str) -> Result[str, IOError]
    functions.insert("subprocess_run_with_input".to_string(), FunctionType::all_borrow(vec![
        ("cmd".to_string(), Type::Str),
        ("stdin_data".to_string(), Type::Str),
    ], result_ty(Type::Str, "IOError")));

    // subprocess_run_structured(cmd: str) -> Result[list[str], IOError]
    // Returns [stdout, stderr, returncode_str] as a list[str].
    functions.insert("subprocess_run_structured".to_string(), FunctionType::all_borrow(vec![("cmd".to_string(), Type::Str)], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.fs — File system intrinsics (io + os file ops)
fn intrinsic_fs() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // read_text(path: str) -> Result[str, IOError]
    functions.insert("read_text".to_string(), FunctionType::all_borrow(
        vec![("path".to_string(), Type::Str)],
        result_ty(Type::Str, "IOError"),
    ));

    // write_text(path: str, content: str) -> Result[None, IOError]
    functions.insert("write_text".to_string(), FunctionType::all_borrow(vec![
            ("path".to_string(), Type::Str),
            ("content".to_string(), Type::Str),
        ], result_ty(Type::None, "IOError")));

    // exists(path: str) -> bool  (infallible)
    functions.insert("exists".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool));

    // read_lines(path: str) -> Result[list[str], IOError]
    functions.insert("read_lines".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    // append_text(path: str, content: str) -> Result[None, IOError]
    functions.insert("append_text".to_string(), FunctionType::all_borrow(vec![
            ("path".to_string(), Type::Str),
            ("content".to_string(), Type::Str),
        ], result_ty(Type::None, "IOError")));

    // getcwd() -> Result[str, IOError]
    functions.insert("getcwd".to_string(), FunctionType::all_borrow(vec![], result_ty(Type::Str, "IOError")));

    // listdir(path: str) -> Result[list[str], IOError]
    functions.insert("listdir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    // mkdir(path: str) -> Result[None, IOError]
    functions.insert("mkdir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::None, "IOError")));

    // rmdir(path: str) -> Result[None, IOError]
    functions.insert("rmdir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::None, "IOError")));

    // remove_file(path: str) -> Result[None, IOError]
    functions.insert("remove_file".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::None, "IOError")));

    // rename(src: str, dst: str) -> Result[None, IOError]
    functions.insert("rename".to_string(), FunctionType::all_borrow(vec![
            ("src".to_string(), Type::Str),
            ("dst".to_string(), Type::Str),
        ], result_ty(Type::None, "IOError")));

    // is_file(path: str) -> bool  (infallible)
    functions.insert("is_file".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool));

    // is_dir(path: str) -> bool  (infallible)
    functions.insert("is_dir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::Bool));

    // copy_file(src: str, dst: str) -> Result[None, IOError]
    functions.insert("copy_file".to_string(), FunctionType::all_borrow(vec![
            ("src".to_string(), Type::Str),
            ("dst".to_string(), Type::Str),
        ], result_ty(Type::None, "IOError")));

    // walk_dir(path: str) -> Result[list[str], IOError]
    functions.insert("walk_dir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    // rmdir_all(path: str) -> Result[None, IOError]
    functions.insert("rmdir_all".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::None, "IOError")));

    // gettempdir() -> str  (infallible — reads env/system temp)
    functions.insert("gettempdir".to_string(), FunctionType::all_borrow(vec![], Type::Str));

    // makedirs(path: str) -> Result[None, IOError]
    functions.insert("makedirs".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::None, "IOError")));

    // touch(path: str) -> Result[None, IOError] (create file if not exists)
    functions.insert("touch".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::None, "IOError")));

    // resolve_path(path: str) -> Result[str, IOError] (canonicalize path)
    functions.insert("resolve_path".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::Str, "IOError")));

    // iterdir(path: str) -> Result[list[str], IOError] (list directory entries as full paths)
    functions.insert("iterdir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    // chdir(path: str) -> Result[None, IOError]
    functions.insert("chdir".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::None, "IOError")));

    // getpid() -> int
    functions.insert("getpid".to_string(), FunctionType::all_borrow(vec![], Type::Int));

    // cpu_count() -> int
    functions.insert("cpu_count".to_string(), FunctionType::all_borrow(vec![], Type::Int));

    // stat_size(path: str) -> Result[int, IOError] (file size in bytes)
    functions.insert("stat_size".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::Int, "IOError")));

    // which(name: str) -> str | None (find executable in PATH)
    functions.insert("which".to_string(), FunctionType::all_borrow(vec![("name".to_string(), Type::Str)], Type::Union(vec![Type::Str, Type::None])));

    // disk_usage(path: str) -> list[int] ([total, used, free] in bytes)
    functions.insert("disk_usage".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], Type::List(Box::new(Type::Int))));

    // open_file(path: str, mode: str) -> Result[int, IOError]
    // Returns an opaque file handle ID (i64) for use with file_* intrinsics.
    functions.insert("open_file".to_string(), FunctionType::all_borrow(vec![
            ("path".to_string(), Type::Str),
            ("mode".to_string(), Type::Str),
        ], result_ty(Type::Int, "IOError")));

    // file_read(handle: int) -> Result[str, IOError]
    functions.insert("file_read".to_string(), FunctionType::all_borrow(vec![("handle".to_string(), Type::Int)], result_ty(Type::Str, "IOError")));

    // file_write(handle: int, data: str) -> Result[None, IOError]
    functions.insert("file_write".to_string(), FunctionType::all_borrow(vec![
            ("handle".to_string(), Type::Int),
            ("data".to_string(), Type::Str),
        ], result_ty(Type::None, "IOError")));

    // file_readline(handle: int) -> Result[str | None, IOError]
    functions.insert("file_readline".to_string(), FunctionType::all_borrow(vec![("handle".to_string(), Type::Int)], result_ty(Type::Union(vec![Type::Str, Type::None]), "IOError")));

    // file_readlines(handle: int) -> Result[list[str], IOError]
    functions.insert("file_readlines".to_string(), FunctionType::all_borrow(vec![("handle".to_string(), Type::Int)], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    // file_close(handle: int) -> None
    functions.insert("file_close".to_string(), FunctionType::all_borrow(vec![("handle".to_string(), Type::Int)], Type::None));

    // file_read_bytes(handle: int) -> Result[list[int], IOError]
    functions.insert("file_read_bytes".to_string(), FunctionType::all_borrow(vec![("handle".to_string(), Type::Int)], result_ty(Type::List(Box::new(Type::Int)), "IOError")));

    // file_write_bytes(handle: int, data: list[int]) -> Result[None, IOError]
    functions.insert("file_write_bytes".to_string(), FunctionType::all_borrow(vec![
            ("handle".to_string(), Type::Int),
            ("data".to_string(), Type::List(Box::new(Type::Int))),
        ], result_ty(Type::None, "IOError")));

    // glob_pattern(dir: str, pattern: str) -> Result[list[str], IOError]
    functions.insert("glob_pattern".to_string(), FunctionType::all_borrow(vec![
            ("dir".to_string(), Type::Str),
            ("pattern".to_string(), Type::Str),
        ], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    // rglob_pattern(dir: str, pattern: str) -> Result[list[str], IOError]
    functions.insert("rglob_pattern".to_string(), FunctionType::all_borrow(vec![
            ("dir".to_string(), Type::Str),
            ("pattern".to_string(), Type::Str),
        ], result_ty(Type::List(Box::new(Type::Str)), "IOError")));

    // os.sep, os.linesep, os.name as zero-arg functions in _sifr.fs
    functions.insert("os_sep".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    functions.insert("os_linesep".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    functions.insert("os_name".to_string(), FunctionType::all_borrow(vec![], Type::Str));

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

    // base64_decode(s: str) -> Result[str, ParseError]
    functions.insert("base64_decode".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], result_ty(Type::Str, "ParseError")));

    // random_uniform(min: float, max: float) -> float
    functions.insert("random_uniform".to_string(), FunctionType::all_borrow(vec![
            ("min".to_string(), Type::Float),
            ("max".to_string(), Type::Float),
        ], Type::Float));

    // sha1(s: str) -> str (hex digest)
    functions.insert("sha1".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // sha512(s: str) -> str (hex digest)
    functions.insert("sha512".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // urlsafe_b64encode(s: str) -> str
    functions.insert("urlsafe_b64encode".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // urlsafe_b64decode(s: str) -> Result[str, ParseError]
    functions.insert("urlsafe_b64decode".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], result_ty(Type::Str, "ParseError")));

    // random_shuffle(items: list[Any]) -> list[Any]
    functions.insert("random_shuffle".to_string(), FunctionType::all_borrow(vec![("items".to_string(), Type::List(Box::new(Type::Any)))], Type::List(Box::new(Type::Any))));

    // random_sample(items: list[Any], k: int) -> Result[list[Any], ValueError]
    functions.insert("random_sample".to_string(), FunctionType::all_borrow(vec![
            ("items".to_string(), Type::List(Box::new(Type::Any))),
            ("k".to_string(), Type::Int),
        ], result_ty(Type::List(Box::new(Type::Any)), "ValueError")));

    // random_randrange(start: int, stop: int, step: int) -> Result[int, ValueError]
    functions.insert("random_randrange".to_string(), FunctionType::all_borrow(vec![
            ("start".to_string(), Type::Int),
            ("stop".to_string(), Type::Int),
            ("step".to_string(), Type::Int),
        ], result_ty(Type::Int, "ValueError")));

    // random_gauss(mu: float, sigma: float) -> float
    functions.insert("random_gauss".to_string(), FunctionType::all_borrow(vec![
            ("mu".to_string(), Type::Float),
            ("sigma".to_string(), Type::Float),
        ], Type::Float));

    // sha224(s: str) -> str (hex digest)
    functions.insert("sha224".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // sha384(s: str) -> str (hex digest)
    functions.insert("sha384".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // blake2b(s: str) -> str (hex digest)
    functions.insert("blake2b".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // blake2s(s: str) -> str (hex digest)
    functions.insert("blake2s".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // b32encode(s: str) -> str
    functions.insert("b32encode".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));

    // b32decode(s: str) -> Result[str, ParseError]
    functions.insert("b32decode".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], result_ty(Type::Str, "ParseError")));

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.regex — Combined regex intrinsics
fn intrinsic_regex() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // re_match(pattern: str, text: str) -> Result[bool, RegexError]
    functions.insert("re_match".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], result_ty(Type::Bool, "RegexError")));

    // re_find(pattern: str, text: str) -> Result[str | None, RegexError]
    functions.insert("re_find".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], result_ty(Type::Union(vec![Type::Str, Type::None]), "RegexError")));

    // re_replace(pattern: str, replacement: str, text: str) -> Result[str, RegexError]
    functions.insert("re_replace".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("replacement".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], result_ty(Type::Str, "RegexError")));

    // re_findall(pattern: str, text: str) -> Result[list[str], RegexError]
    functions.insert("re_findall".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], result_ty(Type::List(Box::new(Type::Str)), "RegexError")));

    // re_split(pattern: str, text: str) -> Result[list[str], RegexError]
    functions.insert("re_split".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], result_ty(Type::List(Box::new(Type::Str)), "RegexError")));

    // re_find_start(pattern: str, text: str) -> Result[int, RegexError]
    // Returns the start index of the first match, or -1 if no match
    functions.insert("re_find_start".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], result_ty(Type::Int, "RegexError")));

    // re_find_end(pattern: str, text: str) -> Result[int, RegexError]
    // Returns the end index of the first match, or -1 if no match
    functions.insert("re_find_end".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
        ], result_ty(Type::Int, "RegexError")));

    // re_match_flags(pattern: str, text: str, flags: int) -> Result[bool, RegexError]
    functions.insert("re_match_flags".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
            ("flags".to_string(), Type::Int),
        ], result_ty(Type::Bool, "RegexError")));

    // re_find_flags(pattern: str, text: str, flags: int) -> Result[str | None, RegexError]
    functions.insert("re_find_flags".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
            ("flags".to_string(), Type::Int),
        ], result_ty(Type::Union(vec![Type::Str, Type::None]), "RegexError")));

    // re_replace_flags(pattern: str, replacement: str, text: str, flags: int) -> Result[str, RegexError]
    functions.insert("re_replace_flags".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("replacement".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
            ("flags".to_string(), Type::Int),
        ], result_ty(Type::Str, "RegexError")));

    // re_findall_flags(pattern: str, text: str, flags: int) -> Result[list[str], RegexError]
    functions.insert("re_findall_flags".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
            ("flags".to_string(), Type::Int),
        ], result_ty(Type::List(Box::new(Type::Str)), "RegexError")));

    // re_split_flags(pattern: str, text: str, flags: int) -> Result[list[str], RegexError]
    functions.insert("re_split_flags".to_string(), FunctionType::all_borrow(vec![
            ("pattern".to_string(), Type::Str),
            ("text".to_string(), Type::Str),
            ("flags".to_string(), Type::Int),
        ], result_ty(Type::List(Box::new(Type::Str)), "RegexError")));

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
    // platform_node() -> str (hostname)
    functions.insert("platform_node".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    // platform_release() -> str (OS release version)
    functions.insert("platform_release".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    // platform_version() -> str (OS version string)
    functions.insert("platform_version".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    // platform_processor() -> str (processor type)
    functions.insert("platform_processor".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    IntrinsicModule { functions, constants: HashMap::new() }
}

/// _sifr.toml — TOML parsing intrinsics
fn intrinsic_toml() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // toml_parse(text: str) -> Result[str, TOMLDecodeError]
    functions.insert("toml_parse".to_string(), FunctionType::all_borrow(vec![("text".to_string(), Type::Str)], result_ty(Type::Str, "TOMLDecodeError")));
    IntrinsicModule { functions, constants: HashMap::new() }
}

/// _sifr.datetime — Date/time intrinsics
fn intrinsic_datetime() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // datetime_now() -> str (ISO 8601 formatted current datetime)
    functions.insert("datetime_now".to_string(), FunctionType::all_borrow(vec![], Type::Str));
    // datetime_now_struct() -> list[int] ([year, month, day, hour, minute, second])
    functions.insert("datetime_now_struct".to_string(), FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))));
    // datetime_format(dt: str, fmt: str) -> str
    functions.insert("datetime_format".to_string(), FunctionType::all_borrow(vec![
        ("dt".to_string(), Type::Str),
        ("fmt".to_string(), Type::Str),
    ], Type::Str));
    // datetime_from_timestamp(ts: float) -> Result[str, ValueError]
    functions.insert("datetime_from_timestamp".to_string(), FunctionType::all_borrow(vec![
        ("ts".to_string(), Type::Float),
    ], result_ty(Type::Str, "ValueError")));
    // time_strptime(s: str, fmt: str) -> list[int] ([year, month, day, hour, minute, second, weekday, yearday])
    functions.insert("time_strptime".to_string(), FunctionType::all_borrow(vec![
        ("s".to_string(), Type::Str),
        ("fmt".to_string(), Type::Str),
    ], result_ty(Type::List(Box::new(Type::Int)), "ValueError")));
    // time_gmtime() -> list[int] ([year, month, day, hour, minute, second, weekday, yearday])
    functions.insert("time_gmtime".to_string(), FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))));
    // time_localtime() -> list[int] ([year, month, day, hour, minute, second, weekday, yearday])
    functions.insert("time_localtime".to_string(), FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))));
    IntrinsicModule { functions, constants: HashMap::new() }
}

/// _sifr.html — HTML escaping intrinsics
fn intrinsic_html() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // html_escape(s: str) -> str
    functions.insert("html_escape".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));
    // html_unescape(s: str) -> str
    functions.insert("html_unescape".to_string(), FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str));
    IntrinsicModule { functions, constants: HashMap::new() }
}

/// _sifr.calendar — Calendar/date calculation intrinsics
fn intrinsic_calendar() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // calendar_isleap(year: int) -> bool
    functions.insert("calendar_isleap".to_string(), FunctionType::all_borrow(vec![("year".to_string(), Type::Int)], Type::Bool));
    // calendar_weekday(year: int, month: int, day: int) -> int (0=Monday..6=Sunday)
    functions.insert("calendar_weekday".to_string(), FunctionType::all_borrow(vec![
        ("year".to_string(), Type::Int),
        ("month".to_string(), Type::Int),
        ("day".to_string(), Type::Int),
    ], Type::Int));
    // calendar_monthrange(year: int, month: int) -> list[int] ([weekday_of_first, days_in_month])
    functions.insert("calendar_monthrange".to_string(), FunctionType::all_borrow(vec![
        ("year".to_string(), Type::Int),
        ("month".to_string(), Type::Int),
    ], Type::List(Box::new(Type::Int))));
    IntrinsicModule { functions, constants: HashMap::new() }
}

/// _sifr.compress — Compression intrinsics (gzip + zip)
fn intrinsic_compress() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // gzip_compress(data: str) -> list[int] (compressed bytes)
    functions.insert("gzip_compress".to_string(), FunctionType::all_borrow(vec![("data".to_string(), Type::Str)], Type::List(Box::new(Type::Int))));
    // gzip_decompress(data: list[int]) -> Result[str, IOError]
    functions.insert("gzip_decompress".to_string(), FunctionType::all_borrow(vec![("data".to_string(), Type::List(Box::new(Type::Int)))], result_ty(Type::Str, "IOError")));
    // zip_create(path: str) -> Result[None, IOError]
    functions.insert("zip_create".to_string(), FunctionType::all_borrow(vec![("path".to_string(), Type::Str)], result_ty(Type::None, "IOError")));
    // zip_add_file(zip_path: str, name: str, content: str) -> Result[None, IOError]
    functions.insert("zip_add_file".to_string(), FunctionType::all_borrow(vec![
        ("zip_path".to_string(), Type::Str),
        ("name".to_string(), Type::Str),
        ("content".to_string(), Type::Str),
    ], result_ty(Type::None, "IOError")));
    // zip_read_file(zip_path: str, name: str) -> Result[str, IOError]
    functions.insert("zip_read_file".to_string(), FunctionType::all_borrow(vec![
        ("zip_path".to_string(), Type::Str),
        ("name".to_string(), Type::Str),
    ], result_ty(Type::Str, "IOError")));
    // zip_namelist(zip_path: str) -> Result[list[str], IOError]
    functions.insert("zip_namelist".to_string(), FunctionType::all_borrow(vec![("zip_path".to_string(), Type::Str)], result_ty(Type::List(Box::new(Type::Str)), "IOError")));
    IntrinsicModule { functions, constants: HashMap::new() }
}

/// _sifr.logging — Logging intrinsics for global state management
fn intrinsic_logging() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // set_global_level(level: int) -> None
    functions.insert("set_global_level".to_string(), FunctionType::all_borrow(vec![("level".to_string(), Type::Int)], Type::None));

    // get_global_level() -> int
    functions.insert("get_global_level".to_string(), FunctionType::all_borrow(vec![], Type::Int));

    IntrinsicModule { functions, constants: HashMap::new() }
}
