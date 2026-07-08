use super::compile_stdlib_uncached;
use sha2::{Digest, Sha256};

#[test]
fn fs_text_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.fs")
        .expect("_sifr.fs should generate private Rust code");

    assert_eq!(private_code.module, "_sifr.fs");
    assert_eq!(private_code.source_path, "stdlib/_sifr/fs.sifr");
    assert_eq!(
        private_code.source_sha256,
        sha256_hex(include_str!("../../../../stdlib/_sifr/fs.sifr"))
    );
    for name in [
        "read_text",
        "write_text",
        "exists",
        "read_lines",
        "append_text",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::fs::{name}(")),
            "{name} should lower through _sifr.fs private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| IOError { message: __sifr_bridge_error.to_string(), kind: __sifr_bridge_error.to_string() })"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.fs")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.io")
        .is_some_and(|deps| deps.contains("_sifr.fs")));

    let public_intrinsics = compiled
        .code
        .intrinsic_names
        .get("sifr.io")
        .expect("sifr.io intrinsic names should be tracked");
    for name in [
        "read_text",
        "write_text",
        "exists",
        "read_lines",
        "append_text",
    ] {
        assert!(
            !public_intrinsics.contains(name),
            "{name} should not remain a public sifr.io compiler intrinsic"
        );
    }
}

fn sha256_hex(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}
