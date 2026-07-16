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

    assert!(private_code.rust.contains("::sifr_stdlib::hash::sha256(s)"));
    assert!(private_code
        .rust
        .contains("::sifr_stdlib::hash::sha256_bytes(data)"));
    assert!(private_code
        .rust
        .contains("::sifr_stdlib::hash::blake2s_bytes(data)"));
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
