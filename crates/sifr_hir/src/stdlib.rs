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
