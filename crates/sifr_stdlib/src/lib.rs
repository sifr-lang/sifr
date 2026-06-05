//! Sifr Intrinsic Type Registry
//!
//! Defines type signatures for all `_sifr.*` intrinsic modules.
//! These are compiler-provided primitives that map directly to Rust code.
//! User-facing stdlib modules live in `lib/sifr/*.sifr` files.

use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

mod collections_bytes_time;
mod crypto_regex_uuid;
mod io_json;
mod math_test;
mod platform_misc;
mod sources;
mod sys_fs;

use collections_bytes_time::{intrinsic_bytes, intrinsic_collections, intrinsic_time};
use crypto_regex_uuid::{intrinsic_crypto, intrinsic_regex, intrinsic_uuid};
use io_json::{intrinsic_io, intrinsic_json};
use math_test::{intrinsic_math, intrinsic_test};
use platform_misc::{
    intrinsic_calendar, intrinsic_compress, intrinsic_datetime, intrinsic_html, intrinsic_logging,
    intrinsic_platform, intrinsic_toml,
};
pub use sources::{StdlibSource, STDLIB_SOURCES};
use sys_fs::{intrinsic_fs, intrinsic_sys};

/// An intrinsic module definition with its functions and constants.
pub struct IntrinsicModule {
    pub functions: HashMap<String, FunctionType>,
    pub constants: HashMap<String, Type>,
}

/// Helper: construct a built-in error class type (e.g., `IOError`, `ParseError`).
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

#[cfg(test)]
mod tests {
    use super::{get_intrinsic_module, is_intrinsic_module, is_stdlib_module, STDLIB_SOURCES};

    #[test]
    fn known_intrinsic_module_has_signatures() {
        let module = get_intrinsic_module("_sifr.io").expect("_sifr.io should be registered");

        assert!(module.functions.contains_key("read_text"));
        assert!(module.constants.is_empty());
    }

    #[test]
    fn unknown_intrinsic_module_is_not_registered() {
        assert!(get_intrinsic_module("_sifr.not_real").is_none());
    }

    #[test]
    fn stdlib_source_inventory_contains_user_modules() {
        let json = STDLIB_SOURCES
            .iter()
            .find(|source| source.module == "sifr.json")
            .expect("sifr.json should be embedded in the stdlib inventory");

        assert!(json.source.contains("from _sifr.json import"));
    }

    #[test]
    fn module_classification_keeps_intrinsic_and_user_surfaces_separate() {
        assert!(is_intrinsic_module("_sifr.io"));
        assert!(!is_intrinsic_module("sifr.io"));
        assert!(is_stdlib_module("sifr.io"));
        assert!(!is_stdlib_module("_sifr.io"));
    }
}
