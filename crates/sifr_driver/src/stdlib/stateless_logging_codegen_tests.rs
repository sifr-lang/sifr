use super::compile_stdlib_uncached;
use sifr_sysroot::sha256_hex;

#[test]
fn logging_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.logging")
        .expect("_sifr.logging should generate private Rust code");

    assert_eq!(private_code.module, "_sifr.logging");
    assert_eq!(private_code.source_path, "stdlib/_sifr/logging.sifr");
    assert_eq!(
        private_code.source_sha256,
        sha256_hex(include_bytes!("../../../../stdlib/_sifr/logging.sifr"))
    );
    assert!(
        private_code
            .rust
            .contains("::sifr_stdlib::logging::set_global_level(")
    );
    assert!(
        private_code
            .rust
            .contains("::sifr_stdlib::logging::get_global_level().into_sifr_int()"),
        "{}",
        private_code.rust
    );
    assert!(
        private_code
            .rust
            .contains("::sifr_runtime::interop::SifrIntBridge::from(level)")
    );
    assert!(
        compiled
            .code
            .transitive_deps
            .get("sifr.logging")
            .is_some_and(|deps| deps.contains("_sifr.logging"))
    );
}
