use super::compile_stdlib_uncached;

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
    for fallback_name in [
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
            !private_code.contains(&format!("sifr_stdlib::base64::{fallback_name}")),
            "{fallback_name} should stay on intrinsic fallback until typed error bridge work"
        );
    }
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
        .is_some_and(|names| names.contains("base64_decode")
            && names.contains("base64_encode_opts")
            && names.contains("b32decode")
            && !names.contains("base64_encode")
            && !names.contains("_base64_encode_impl")
            && !names.contains("b32encode")
            && !names.contains("_b32encode_impl")));
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
