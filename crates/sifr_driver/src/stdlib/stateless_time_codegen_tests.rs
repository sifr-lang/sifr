use super::compile_stdlib_uncached;

#[test]
fn time_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.time")
        .expect("_sifr.time should generate private Rust code");

    for (decl_name, target_name) in [
        ("time_now", "time_now"),
        ("time_format", "time_format"),
        ("perf_counter", "perf_counter"),
        ("sleep", "sleep"),
        ("monotonic", "monotonic"),
        ("strptime", "strptime"),
        ("_strptime_intrinsic", "strptime"),
        ("gmtime", "gmtime"),
        ("_gmtime_intrinsic", "gmtime"),
        ("localtime", "localtime"),
        ("_localtime_intrinsic", "localtime"),
        ("time_strptime", "time_strptime"),
        ("time_gmtime", "time_gmtime"),
        ("time_localtime", "time_localtime"),
    ] {
        assert!(
            private_code.rust.contains(&format!("fn {decl_name}(")),
            "{decl_name} should be emitted by _sifr.time private declarations"
        );
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::time::{target_name}(")),
            "{decl_name} should lower through _sifr.time private Rust interop declarations"
        );
    }
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.time")
        .is_some_and(|deps| deps.contains("_sifr.time")));
}
