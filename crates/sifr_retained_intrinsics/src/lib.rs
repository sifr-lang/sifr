//! Transitional compiler-retained stdlib intrinsic signatures.
//!
//! This crate hosts fallback signatures that still feed lowering and driver
//! bootstrap while native stdlib declarations continue replacing them.

use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

mod collections_bytes_time;
mod crypto_regex_uuid;
mod http;
mod i18n_core;
mod io_json;
mod math_test;
mod net;
mod platform_misc;
mod process;
mod python;
mod runtime;
mod signal;
mod sys_fs;
mod task;
mod text_encoding;
mod tls;
mod unicode_core;
mod url;

use collections_bytes_time::{intrinsic_bytes, intrinsic_collections, intrinsic_time};
use crypto_regex_uuid::{intrinsic_crypto, intrinsic_regex, intrinsic_uuid};
use http::intrinsic_http;
use i18n_core::intrinsic_i18n;
use io_json::{intrinsic_io, intrinsic_json};
use math_test::{intrinsic_math, intrinsic_test};
use net::intrinsic_net;
use platform_misc::{
    intrinsic_calendar, intrinsic_compress, intrinsic_datetime, intrinsic_html, intrinsic_logging,
    intrinsic_platform, intrinsic_toml,
};
use process::intrinsic_process;
use python::intrinsic_python;
use runtime::intrinsic_runtime;
use signal::intrinsic_signal;
use sys_fs::{intrinsic_fs, intrinsic_sys};
use task::intrinsic_task;
use text_encoding::intrinsic_encoding;
use tls::intrinsic_tls;
use unicode_core::intrinsic_unicode;
use url::intrinsic_url;

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
        "_sifr.test" => Some(intrinsic_test()),
        "_sifr.collections" => Some(intrinsic_collections()),
        "_sifr.bytes" => Some(intrinsic_bytes()),
        "_sifr.encoding" => Some(intrinsic_encoding()),
        "_sifr.unicode" => Some(intrinsic_unicode()),
        "_sifr.i18n" => Some(intrinsic_i18n()),
        "_sifr.time" => Some(intrinsic_time()),
        "_sifr.crypto" => Some(intrinsic_crypto()),
        "_sifr.regex" => Some(intrinsic_regex()),
        // Retained as a stdlib-lowering bootstrap fallback while these leaves
        // migrate to compiled private declarations.
        "_sifr.math" => Some(intrinsic_math()),
        "_sifr.uuid" => Some(intrinsic_uuid()),
        "_sifr.platform" => Some(intrinsic_platform()),
        "_sifr.net" => Some(intrinsic_net()),
        "_sifr.tls" => Some(intrinsic_tls()),
        "_sifr.url" => Some(intrinsic_url()),
        "_sifr.http" => Some(intrinsic_http()),
        "_sifr.process" => Some(intrinsic_process()),
        "_sifr.python" => Some(intrinsic_python()),
        "_sifr.signal" => Some(intrinsic_signal()),
        "_sifr.runtime" => Some(intrinsic_runtime()),
        "_sifr.task" => Some(intrinsic_task()),
        "_sifr.toml" => Some(intrinsic_toml()),
        "_sifr.datetime" => Some(intrinsic_datetime()),
        // Retained as a stdlib-lowering bootstrap fallback while these leaves
        // migrate to compiled private declarations.
        "_sifr.html" => Some(intrinsic_html()),
        // Retained as a stdlib-lowering bootstrap fallback while this leaf
        // migrates to compiled private declarations.
        "_sifr.calendar" => Some(intrinsic_calendar()),
        "_sifr.compress" => Some(intrinsic_compress()),
        "_sifr.logging" => Some(intrinsic_logging()),
        _ => None,
    }
}

/// Check if a module name is an intrinsic module.
#[must_use]
pub fn is_intrinsic_module(module_name: &str) -> bool {
    module_name.starts_with("_sifr.")
}

/// Check if a module name is a user-facing stdlib module.
#[must_use]
pub fn is_stdlib_module(module_name: &str) -> bool {
    module_name.starts_with("sifr.")
}

#[cfg(test)]
mod tests {
    use super::{get_intrinsic_module, is_intrinsic_module, is_stdlib_module};

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
    fn module_classification_keeps_intrinsic_and_user_surfaces_separate() {
        assert!(is_intrinsic_module("_sifr.io"));
        assert!(!is_intrinsic_module("sifr.io"));
        assert!(is_stdlib_module("sifr.io"));
        assert!(!is_stdlib_module("_sifr.io"));
    }

    #[test]
    fn legacy_subprocess_intrinsics_are_not_registered() {
        let sys = get_intrinsic_module("_sifr.sys").expect("_sifr.sys should exist");

        for removed in [
            "subprocess_run",
            "subprocess_run_with_input",
            "subprocess_run_structured",
        ] {
            assert!(
                !sys.functions.contains_key(removed),
                "{removed} must stay removed; use _sifr.process intrinsics instead"
            );
        }
    }
}
