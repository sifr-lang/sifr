use super::compile_stdlib_uncached;
use sha2::{Digest, Sha256};

#[test]
fn platform_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.platform")
        .expect("_sifr.platform should generate private Rust code");

    assert_eq!(private_code.module, "_sifr.platform");
    assert_eq!(private_code.source_path, "stdlib/_sifr/platform.sifr");
    assert_eq!(
        private_code.source_sha256,
        sha256_hex(include_str!("../../../../stdlib/_sifr/platform.sifr"))
    );
    assert!(private_code
        .rust
        .contains("sifr_stdlib::platform::platform_system()"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.platform")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.platform")
        .is_some_and(|deps| deps.contains("_sifr.platform")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.platform")
        .is_some_and(|names| !names.contains("platform_system")));
}

#[test]
fn sys_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.sys")
        .expect("_sifr.sys should generate private Rust code");

    assert_eq!(private_code.module, "_sifr.sys");
    assert_eq!(private_code.source_path, "stdlib/_sifr/sys.sifr");
    assert_eq!(
        private_code.source_sha256,
        sha256_hex(include_str!("../../../../stdlib/_sifr/sys.sifr"))
    );
    for name in [
        "run_command",
        "env_get",
        "env_set",
        "env_unset",
        "env_keys",
        "env_values",
        "env_items",
        "get_args",
        "sys_exit",
        "sys_version",
        "sys_platform",
        "sys_maxsize",
        "getpid",
        "cpu_count",
        "which",
        "os_sep",
        "os_linesep",
        "os_name",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::sys::{name}(")),
            "{name} should lower through _sifr.sys private Rust interop declarations"
        );
    }
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.sys")
        .is_some_and(|names| !names.contains("run_command")
            && !names.contains("env_get")
            && !names.contains("sys_version")
            && !names.contains("which")
            && !names.contains("os_sep")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.env")
        .is_some_and(|deps| deps.contains("_sifr.sys")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.sys")
        .is_some_and(|deps| deps.contains("_sifr.sys")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.os")
        .is_some_and(|deps| deps.contains("_sifr.sys")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.shutil")
        .is_some_and(|deps| deps.contains("_sifr.sys")));
}

fn sha256_hex(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[test]
fn html_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.html")
        .expect("_sifr.html should generate private Rust code");

    assert!(private_code
        .rust
        .contains("sifr_stdlib::html::html_escape(s)"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.html")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.html")
        .is_some_and(|deps| deps.contains("_sifr.html")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.html")
        .is_some_and(|names| !names.contains("html_escape")));
}

#[test]
fn calendar_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.calendar")
        .expect("_sifr.calendar should generate private Rust code");

    assert!(private_code.rust.contains(
        "sifr_stdlib::calendar::calendar_isleap(sifr_runtime::interop::SifrIntBridge::from(year))"
    ));
    assert!(private_code.rust.contains(
        "sifr_stdlib::calendar::calendar_monthrange(sifr_runtime::interop::SifrIntBridge::from(year), sifr_runtime::interop::SifrIntBridge::from(month)).into_iter().map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating()).collect()"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.calendar")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.calendar")
        .is_some_and(|deps| deps.contains("_sifr.calendar")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.calendar")
        .is_some_and(|names| !names.contains("calendar_isleap")));
}

#[test]
fn uuid_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.uuid")
        .expect("_sifr.uuid should generate private Rust code");

    assert!(private_code.rust.contains("sifr_stdlib::uuid::uuid4()"));
    assert!(private_code
        .rust
        .contains("sifr_stdlib::uuid::uuid3_text(namespace, name)"));
    assert!(private_code
        .rust
        .contains("sifr_stdlib::uuid::uuid5_text(namespace, name)"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.uuid")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.uuid")
        .is_some_and(|deps| deps.contains("_sifr.uuid")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.uuid")
        .is_some_and(|names| !names.contains("uuid4")));
}

#[test]
fn bytes_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.bytes")
        .expect("_sifr.bytes should generate private Rust code");

    assert!(private_code
        .rust
        .contains("sifr_stdlib::bytes::encode_utf8(s)"));
    assert!(private_code
        .rust
        .contains("sifr_stdlib::bytes::bytes_to_hex(bytes)"));
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.bytes")
        .is_some_and(|names| !names.contains("encode_utf8") && !names.contains("bytes_to_hex")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.bytes")
        .is_some_and(|deps| deps.contains("_sifr.bytes")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.hashlib")
        .is_some_and(|deps| deps.contains("_sifr.bytes")));
}

#[test]
fn regex_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.regex")
        .expect("_sifr.regex should generate private Rust code");

    for name in [
        "re_match",
        "re_find",
        "re_replace",
        "re_findall",
        "re_split",
        "re_find_start",
        "re_find_end",
        "re_match_flags",
        "re_find_flags",
        "re_replace_flags",
        "re_findall_flags",
        "re_split_flags",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::regex::{name}(")),
            "{name} should lower through _sifr.regex private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| RegexError { message: __sifr_bridge_error.to_string(), detail: __sifr_bridge_error.to_string() })"
    ));
    assert!(private_code
        .rust
        .contains("sifr_runtime::interop::SifrIntBridge::from(flags)"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.regex")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.re")
        .is_some_and(|deps| deps.contains("_sifr.regex")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.re")
        .is_some_and(std::collections::HashSet::is_empty));
    let exports = compiled
        .defs
        .functions
        .get("sifr.re")
        .expect("sifr.re exports should be collected");
    assert!(exports.contains_key("re_match"));
    assert!(exports.contains_key("search"));
    assert!(!exports.contains_key("_re_match_impl"));
}

#[test]
fn url_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.url")
        .expect("_sifr.url should generate private Rust code");

    for name in [
        "url_parse_parts",
        "url_build_parts",
        "url_percent_encode",
        "url_percent_decode",
        "url_percent_encode_bytes",
        "url_percent_decode_bytes",
        "url_normalize_path",
        "url_query_parse_flat",
        "url_query_build_flat",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::url::{name}(")),
            "{name} should lower through _sifr.url private Rust interop declarations"
        );
    }
    assert!(private_code
        .rust
        .contains("port.map(sifr_runtime::interop::SifrIntBridge::from)"));
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.url")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.url")
        .is_some_and(|deps| deps.contains("_sifr.url")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.url")
        .is_some_and(std::collections::HashSet::is_empty));
    let exports = compiled
        .defs
        .functions
        .get("sifr.url")
        .expect("sifr.url exports should be collected");
    assert!(exports.contains_key("parse_url"));
    assert!(exports.contains_key("build_query"));
    assert!(!exports.contains_key("_url_parse_parts"));
    assert!(!exports.contains_key("_url_query_build_flat"));
}

#[test]
fn toml_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.toml")
        .expect("_sifr.toml should generate private Rust code");

    assert!(private_code
        .rust
        .contains("sifr_stdlib::toml::toml_parse_tokens(text)"));
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.toml")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.tomllib")
        .is_some_and(|deps| deps.contains("_sifr.toml")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.tomllib")
        .is_some_and(std::collections::HashSet::is_empty));
    let exports = compiled
        .defs
        .functions
        .get("sifr.tomllib")
        .expect("sifr.tomllib exports should be collected");
    assert!(exports.contains_key("loads"));
    assert!(!exports.contains_key("_decode_toml_tokens"));
    assert!(!exports.contains_key("_decode_toml_value_at"));
}

#[test]
fn json_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.json")
        .expect("_sifr.json should generate private Rust code");

    for name in [
        "json_load_tokens",
        "json_validate_integer_digit_limits",
        "json_dump_tokens",
        "json_dump_tokens_exact",
        "json_dump_tokens_string_ints",
        "json_dump_tokens_web",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::json::{name}(")),
            "{name} should lower through _sifr.json private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains("JSONDecodeError { message: __sifr_bridge_error.message().to_string(), line: __sifr_bridge_error.line() as i64, column: __sifr_bridge_error.column() as i64 }"));
    assert!(private_code.rust.contains("JsonLimitError { message: __sifr_bridge_error.message().to_string(), limit: __sifr_bridge_error.limit() as i64 }"));
    assert!(private_code.rust.contains("JsonIntegerRangeError { message: __sifr_bridge_error.message().to_string(), path: __sifr_bridge_error.path().to_string(), profile: __sifr_bridge_error.profile().to_string() }"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.json")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.json")
        .is_some_and(|deps| deps.contains("_sifr.json")));
    let public_intrinsics = compiled
        .code
        .intrinsic_names
        .get("sifr.json")
        .expect("sifr.json intrinsic names should be tracked");
    for name in [
        "json_loads",
        "json_validate_integer_digit_limits",
        "json_dumps",
        "json_dumps_value",
        "json_dumps_value_exact",
        "json_dumps_value_web",
        "json_dumps_value_string_ints",
    ] {
        assert!(
            !public_intrinsics.contains(name),
            "{name} should not remain a public sifr.json compiler intrinsic"
        );
    }
    let exports = compiled
        .defs
        .functions
        .get("sifr.json")
        .expect("sifr.json exports should be collected");
    assert!(exports.contains_key("loads"));
    assert!(exports.contains_key("json_loads"));
    assert!(exports.contains_key("json_dumps"));
    assert!(!exports.contains_key("_decode_json_tokens"));
    assert!(!exports.contains_key("_decode_json_value_at"));
}

#[test]
fn encoding_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.encoding")
        .expect("_sifr.encoding should generate private Rust code");

    for name in [
        "encoding_is_supported",
        "encoding_canonical_label",
        "encoding_decode_text",
        "encoding_decode_recoveries",
        "encoding_decode_incremental_text",
        "encoding_decode_incremental_recoveries",
        "encoding_decode_incremental_pending",
        "encoding_encode_bytes",
        "encoding_encode_recoveries",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::encoding::{name}(")),
            "{name} should lower through _sifr.encoding private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.encoding")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.encoding")
        .is_some_and(|deps| deps.contains("_sifr.encoding")));
    let public_intrinsics = compiled
        .code
        .intrinsic_names
        .get("sifr.encoding")
        .expect("sifr.encoding intrinsic names should be tracked");
    for name in [
        "encoding_decode_text",
        "encoding_decode_recoveries",
        "encoding_decode_outcome",
        "encoding_decode_incremental_outcome",
        "encoding_decode_incremental_pending",
        "encoding_encode_bytes",
        "encoding_encode_recoveries",
        "encoding_encode_outcome",
    ] {
        assert!(
            !public_intrinsics.contains(name),
            "{name} should not remain a public sifr.encoding compiler intrinsic"
        );
    }
    let exports = compiled
        .defs
        .functions
        .get("sifr.encoding")
        .expect("sifr.encoding exports should be collected");
    assert!(exports.contains_key("encoding_decode_outcome"));
    assert!(exports.contains_key("encoding_encode_outcome"));
    assert!(!exports.contains_key("_encoding_decode_text_impl"));
    assert!(!exports.contains_key("_encoding_encode_bytes_impl"));
}

#[test]
fn unicode_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.unicode")
        .expect("_sifr.unicode should generate private Rust code");

    for name in [
        "data_version",
        "normalize",
        "is_normalized",
        "name",
        "lookup",
        "category",
        "bidirectional",
        "combining",
        "east_asian_width",
        "mirrored",
        "decomposition",
        "decimal",
        "digit",
        "numeric_value",
        "case_fold",
        "graphemes",
        "grapheme_indices_flat",
        "words",
        "word_boundaries_flat",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::unicode::{name}(")),
            "{name} should lower through _sifr.unicode private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.unicode")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.unicode")
        .is_some_and(|deps| deps.contains("_sifr.unicode")));
    let public_intrinsics = compiled
        .code
        .intrinsic_names
        .get("sifr.unicode")
        .expect("sifr.unicode intrinsic names should be tracked");
    for name in [
        "unicode_normalize",
        "unicode_is_normalized",
        "unicode_name",
        "unicode_lookup",
        "unicode_case_fold",
        "unicode_graphemes",
    ] {
        assert!(
            !public_intrinsics.contains(name),
            "{name} should not remain a public sifr.unicode compiler intrinsic"
        );
    }
    let exports = compiled
        .defs
        .functions
        .get("sifr.unicode")
        .expect("sifr.unicode exports should be collected");
    assert!(exports.contains_key("normalize"));
    assert!(exports.contains_key("grapheme_indices"));
    assert!(!exports.contains_key("_unicode_normalize_impl"));
    assert!(!exports.contains_key("_unicode_graphemes_impl"));
    assert!(!exports.contains_key("_unicode_grapheme_indices_flat_impl"));
}

#[test]
fn compression_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.compress")
        .expect("_sifr.compress should generate private Rust code");

    for (module, name) in [
        ("gzip", "gzip_compress_bytes"),
        ("gzip", "gzip_decompress_bytes"),
        ("zipfile", "zip_create"),
        ("zipfile", "zip_add_file"),
        ("zipfile", "zip_add_file_bytes"),
        ("zipfile", "zip_read_file"),
        ("zipfile", "zip_read_file_bytes"),
        ("zipfile", "zip_namelist"),
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::{module}::{name}(")),
            "{name} should lower through _sifr.compress private Rust interop declarations"
        );
    }
    assert!(private_code
        .rust
        .contains("sifr_stdlib::gzip::gzip_compress_bytes(data)"));
    assert!(private_code
        .rust
        .contains("fn __io_err<E: std::fmt::Display + 'static>"));
    assert!(private_code
        .rust
        .contains(".map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))"));
    assert!(!private_code
        .rust
        .contains("kind: __sifr_bridge_error.to_string()"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.compress")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.gzip")
        .is_some_and(|deps| deps.contains("_sifr.compress")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.zipfile")
        .is_some_and(|deps| deps.contains("_sifr.compress")));

    for module in ["sifr.gzip", "sifr.zipfile"] {
        let public_intrinsics = compiled
            .code
            .intrinsic_names
            .get(module)
            .expect("public compression intrinsic names should be tracked");
        for name in [
            "gzip_compress",
            "gzip_decompress",
            "zip_create",
            "zip_add_file",
            "zip_add_file_bytes",
            "zip_read_file",
            "zip_read_file_bytes",
            "zip_namelist",
        ] {
            assert!(
                !public_intrinsics.contains(name),
                "{name} should not remain a public {module} compiler intrinsic"
            );
        }
    }

    let gzip_exports = compiled
        .defs
        .functions
        .get("sifr.gzip")
        .expect("sifr.gzip exports should be collected");
    assert!(gzip_exports.contains_key("compress"));
    assert!(gzip_exports.contains_key("decompress"));
    assert!(!gzip_exports.contains_key("_gzip_compress_bytes_impl"));
    assert!(!gzip_exports.contains_key("_gzip_decompress_bytes_impl"));
    let zip_exports = compiled
        .defs
        .functions
        .get("sifr.zipfile")
        .expect("sifr.zipfile exports should be collected");
    assert!(zip_exports.contains_key("is_zipfile"));
    assert!(zip_exports.contains_key("zip_namelist"));
}

#[test]
fn datetime_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.datetime")
        .expect("_sifr.datetime should generate private Rust code");

    for name in [
        "datetime_now",
        "datetime_now_struct",
        "datetime_format",
        "datetime_from_timestamp",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::time::{name}(")),
            "{name} should lower through _sifr.datetime private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ValueError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.datetime")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.datetime")
        .is_some_and(|deps| deps.contains("_sifr.datetime")));

    let public_intrinsics = compiled
        .code
        .intrinsic_names
        .get("sifr.datetime")
        .expect("sifr.datetime intrinsic names should be tracked");
    for name in [
        "datetime_now",
        "datetime_now_struct",
        "datetime_format",
        "datetime_from_timestamp",
    ] {
        assert!(
            !public_intrinsics.contains(name),
            "{name} should not remain a public sifr.datetime compiler intrinsic"
        );
    }
    let exports = compiled
        .defs
        .functions
        .get("sifr.datetime")
        .expect("sifr.datetime exports should be collected");
    assert!(exports.contains_key("now"));
    assert!(exports.contains_key("from_timestamp"));
    assert!(exports.contains_key("datetime_now_struct"));
}

#[test]
fn i18n_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.i18n")
        .expect("_sifr.i18n should generate private Rust code");

    for name in [
        "i18n_locale_canonicalize",
        "i18n_locale_maximize",
        "i18n_locale_minimize",
        "i18n_host_locale",
        "i18n_format_number",
        "i18n_format_datetime",
        "i18n_plural_category",
        "i18n_collate",
        "i18n_mo_validate",
        "i18n_mo_load_file",
        "i18n_mo_lookup",
        "i18n_mo_lookup_context",
        "i18n_mo_lookup_plural",
        "i18n_mo_lookup_context_plural",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::i18n::{name}(")),
            "{name} should lower through _sifr.i18n private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains(
        "sifr_stdlib::i18n::i18n_format_datetime(locale, style, sifr_runtime::interop::SifrIntBridge::from(year)"
    ));
    assert!(private_code.rust.contains(
        "sifr_stdlib::i18n::i18n_collate(locale, strength, left, right).map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())"
    ));
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.i18n")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.i18n")
        .is_some_and(|deps| deps.contains("_sifr.i18n")));
    let public_intrinsics = compiled
        .code
        .intrinsic_names
        .get("sifr.i18n")
        .expect("sifr.i18n intrinsic names should be tracked");
    for name in [
        "i18n_locale_canonicalize",
        "i18n_format_number",
        "i18n_format_datetime",
        "i18n_plural_category",
        "i18n_mo_lookup_context_plural",
    ] {
        assert!(
            !public_intrinsics.contains(name),
            "{name} should not remain a public sifr.i18n compiler intrinsic"
        );
    }
    let exports = compiled
        .defs
        .functions
        .get("sifr.i18n")
        .expect("sifr.i18n exports should be collected");
    assert!(exports.contains_key("canonicalize_locale"));
    assert!(exports.contains_key("i18n_format_number"));
    assert!(!exports.contains_key("_i18n_format_number_impl"));
    assert!(!exports.contains_key("_i18n_mo_lookup_context_plural_impl"));
}
