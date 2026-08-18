use super::compile_stdlib_uncached;

#[test]
fn crypto_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.crypto")
        .expect("_sifr.crypto should generate private Rust code");

    for random_name in [
        "random_int",
        "random_float",
        "random_uniform",
        "random_randrange",
        "random_gauss",
        "random_module_state_words",
        "random_module_state_index",
        "random_module_state_gauss_next",
        "random_module_set_state",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::random::{random_name}(")),
            "{random_name} should lower through _sifr.crypto private Rust interop declarations"
        );
    }

    for hash_name in [
        "md5_bytes",
        "sha1_bytes",
        "sha224_bytes",
        "sha256_bytes",
        "sha384_bytes",
        "sha512_bytes",
        "blake2b_bytes",
        "blake2s_bytes",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::hash::{hash_name}(data)")),
            "{hash_name} should lower through the live _sifr.crypto bytes boundary"
        );
    }
    assert!(!private_code.rust.contains("::sifr_stdlib::hash::sha256(s)"));

    let hashlib_code = compiled
        .code
        .module_rust_code
        .get("sifr.hashlib")
        .expect("sifr.hashlib should generate public Rust code");
    for private_call in [
        "md5_bytes(data)",
        "sha1_bytes(data)",
        "sha224_bytes(data)",
        "sha256_bytes(data)",
        "sha384_bytes(data)",
        "sha512_bytes(data)",
        "blake2b_bytes(data)",
        "blake2s_bytes(data)",
    ] {
        assert!(
            hashlib_code.rust.contains(private_call),
            "{private_call} should be a live canonical hashlib consumer"
        );
    }
    assert!(private_code
        .rust
        .contains("::sifr_stdlib::base64::base64_encode(s)"));
    assert!(private_code
        .rust
        .contains("::sifr_stdlib::base64::base64_encode_bytes(data)"));
    assert!(private_code
        .rust
        .contains("::sifr_stdlib::base64::urlsafe_b64encode(s)"));
    assert!(private_code
        .rust
        .contains("::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)"));
    assert!(private_code
        .rust
        .contains("::sifr_stdlib::base64::b32encode(s)"));
    assert!(private_code
        .rust
        .contains("::sifr_stdlib::base64::b32hexencode(s)"));
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.hashlib")
        .is_some_and(|deps| deps.contains("_sifr.crypto")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.random")
        .is_some_and(|deps| deps.contains("_sifr.crypto")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.base64")
        .is_some_and(|deps| deps.contains("_sifr.crypto")));
}
