use super::compile_stdlib_uncached;

const COMPLETED_MIGRATED_PRIVATE_DECLARATIONS: &[(&str, &str)] = &[
    (
        "_sifr.platform",
        include_str!("../../../../stdlib/_sifr/platform.sifr"),
    ),
    (
        "_sifr.html",
        include_str!("../../../../stdlib/_sifr/html.sifr"),
    ),
    (
        "_sifr.calendar",
        include_str!("../../../../stdlib/_sifr/calendar.sifr"),
    ),
    (
        "_sifr.uuid",
        include_str!("../../../../stdlib/_sifr/uuid.sifr"),
    ),
    (
        "_sifr.math",
        include_str!("../../../../stdlib/_sifr/math.sifr"),
    ),
    (
        "_sifr.crypto",
        include_str!("../../../../stdlib/_sifr/crypto.sifr"),
    ),
    (
        "_sifr.regex",
        include_str!("../../../../stdlib/_sifr/regex.sifr"),
    ),
    (
        "_sifr.url",
        include_str!("../../../../stdlib/_sifr/url.sifr"),
    ),
    (
        "_sifr.toml",
        include_str!("../../../../stdlib/_sifr/toml.sifr"),
    ),
    (
        "_sifr.json",
        include_str!("../../../../stdlib/_sifr/json.sifr"),
    ),
    (
        "_sifr.encoding",
        include_str!("../../../../stdlib/_sifr/encoding.sifr"),
    ),
];

#[test]
fn completed_private_declarations_follow_adapter_policy_syntax() {
    for (module, source) in COMPLETED_MIGRATED_PRIVATE_DECLARATIONS {
        assert!(
            !source.contains("@rust.via"),
            "{module} must not use callee-injection syntax"
        );
        assert!(
            !source.contains("bridge."),
            "{module} must not route through bridge.* sysroot adapters"
        );
        assert!(
            !source.contains("converter") && !source.contains("pipeline"),
            "{module} must not declare converter-pipeline metadata"
        );
        for line in source
            .lines()
            .filter(|line| line.trim_start().starts_with("@rust("))
        {
            assert!(
                line.contains("@rust(sifr_stdlib.") && line.contains("panic=trusted_no_panic"),
                "{module} declaration must bind directly to sifr_stdlib with sysroot trust: {line}"
            );
        }
    }
}

#[test]
fn platform_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.platform")
        .expect("_sifr.platform should generate private Rust code");

    assert!(private_code.contains("sifr_stdlib::platform::platform_system()"));
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
fn html_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.html")
        .expect("_sifr.html should generate private Rust code");

    assert!(private_code.contains("sifr_stdlib::html::html_escape(s)"));
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

    assert!(private_code.contains(
        "sifr_stdlib::calendar::calendar_isleap(sifr_runtime::interop::SifrIntBridge::from(year))"
    ));
    assert!(private_code.contains(
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

    assert!(private_code.contains("sifr_stdlib::uuid::uuid4()"));
    assert!(private_code.contains("sifr_stdlib::uuid::uuid3_text(namespace, name)"));
    assert!(private_code.contains("sifr_stdlib::uuid::uuid5_text(namespace, name)"));
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
fn math_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.math")
        .expect("_sifr.math should generate private Rust code");

    assert!(private_code.contains("sifr_stdlib::math::sqrt(x)"));
    assert!(private_code.contains("sifr_stdlib::math::pow_val(x, y)"));
    assert!(private_code.contains("sifr_stdlib::math::floor(x)"));
    assert!(private_code.contains("sifr_stdlib::math::frexp(x)"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.math")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.math")
        .is_some_and(|deps| deps.contains("_sifr.math")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.math")
        .is_some_and(|names| !names.contains("sqrt")));
    let public_constants = compiled
        .defs
        .constants
        .get("sifr.math")
        .expect("sifr.math should export public constants");
    assert!(public_constants.contains_key("pi"));
    assert!(public_constants.contains_key("tau"));
    assert!(public_constants.contains_key("inf"));
    assert!(public_constants.contains_key("nan"));
    let public_functions = compiled
        .defs
        .functions
        .get("sifr.math")
        .expect("sifr.math should export public functions");
    for name in ["dist", "fsum", "sumprod"] {
        let function = public_functions
            .get(name)
            .unwrap_or_else(|| panic!("sifr.math should export {name}"));
        assert!(
            function.params.iter().all(
                |(_, _, convention)| *convention == sifr_type_system::ParamConvention::borrow()
            ),
            "{name} should keep read-only public list parameters"
        );
    }
    for name in ["dist_impl", "fsum_impl", "sumprod_impl"] {
        assert!(
            !public_functions.contains_key(name),
            "{name} should stay an internal aggregate bridge helper"
        );
    }
}

#[test]
fn crypto_hash_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.crypto")
        .expect("_sifr.crypto should generate private Rust code");

    assert!(private_code.contains("sifr_stdlib::hash::sha256(s)"));
    assert!(private_code.contains("sifr_stdlib::hash::sha256_bytes(data)"));
    assert!(private_code.contains("sifr_stdlib::hash::blake2s_bytes(data)"));
    assert!(private_code.contains("sifr_stdlib::base64::base64_encode(s)"));
    assert!(private_code.contains("sifr_stdlib::base64::base64_encode_bytes(data)"));
    assert!(private_code.contains("sifr_stdlib::base64::urlsafe_b64encode(s)"));
    assert!(private_code.contains("sifr_stdlib::base64::urlsafe_b64encode_bytes(data)"));
    assert!(private_code.contains("sifr_stdlib::base64::b32encode(s)"));
    assert!(private_code.contains("sifr_stdlib::base64::b32hexencode(s)"));
    for fallible_name in [
        "base64_decode",
        "base64_decode_bytes",
        "base64_encode_opts",
        "base64_decode_opts",
        "urlsafe_b64decode",
        "urlsafe_b64decode_bytes",
        "b32decode",
        "b32hexdecode",
    ] {
        assert!(
            private_code.contains(&format!("sifr_stdlib::base64::{fallible_name}(")),
            "{fallible_name} should lower through _sifr.crypto private Rust interop declarations"
        );
    }
    assert!(private_code.contains(
        "map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.crypto")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.hashlib")
        .is_some_and(|deps| deps.contains("_sifr.crypto")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.hashlib")
        .is_some_and(|names| !names.contains("sha256") && !names.contains("_sha256_impl")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.base64")
        .is_some_and(|deps| deps.contains("_sifr.crypto")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.base64")
        .is_some_and(std::collections::HashSet::is_empty));
    let hashlib_exports = compiled
        .defs
        .functions
        .get("sifr.hashlib")
        .expect("sifr.hashlib exports should be collected");
    assert!(hashlib_exports.contains_key("sha256"));
    assert!(!hashlib_exports.contains_key("_sha256_impl"));
    let base64_exports = compiled
        .defs
        .functions
        .get("sifr.base64")
        .expect("sifr.base64 exports should be collected");
    assert!(base64_exports.contains_key("base64_encode"));
    assert!(base64_exports.contains_key("b32decode"));
    assert!(!base64_exports.contains_key("_base64_encode_impl"));
    assert!(!base64_exports.contains_key("_b32encode_impl"));
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
            private_code.contains(&format!("sifr_stdlib::regex::{name}(")),
            "{name} should lower through _sifr.regex private Rust interop declarations"
        );
    }
    assert!(private_code.contains(
        "map_err(|__sifr_bridge_error| RegexError { message: __sifr_bridge_error.to_string(), detail: __sifr_bridge_error.to_string() })"
    ));
    assert!(private_code.contains("sifr_runtime::interop::SifrIntBridge::from(flags)"));
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
            private_code.contains(&format!("sifr_stdlib::url::{name}(")),
            "{name} should lower through _sifr.url private Rust interop declarations"
        );
    }
    assert!(private_code.contains("port.map(sifr_runtime::interop::SifrIntBridge::from)"));
    assert!(private_code.contains(
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

    assert!(private_code.contains("sifr_stdlib::toml::toml_parse_tokens(text)"));
    assert!(private_code.contains(
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
            private_code.contains(&format!("sifr_stdlib::json::{name}(")),
            "{name} should lower through _sifr.json private Rust interop declarations"
        );
    }
    assert!(private_code.contains("JSONDecodeError { message: __sifr_bridge_error.message().to_string(), line: __sifr_bridge_error.line() as i64, column: __sifr_bridge_error.column() as i64 }"));
    assert!(private_code.contains("JsonLimitError { message: __sifr_bridge_error.message().to_string(), limit: __sifr_bridge_error.limit() as i64 }"));
    assert!(private_code.contains("JsonIntegerRangeError { message: __sifr_bridge_error.message().to_string(), path: __sifr_bridge_error.path().to_string(), profile: __sifr_bridge_error.profile().to_string() }"));
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
            private_code.contains(&format!("sifr_stdlib::encoding::{name}(")),
            "{name} should lower through _sifr.encoding private Rust interop declarations"
        );
    }
    assert!(private_code.contains(
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
