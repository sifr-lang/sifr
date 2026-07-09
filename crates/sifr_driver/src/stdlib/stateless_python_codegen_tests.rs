use super::compile_stdlib_uncached;
use sha2::{Digest, Sha256};

#[test]
fn python_primitive_constructors_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    assert_eq!(private_code.module, "_sifr.python");
    assert_eq!(private_code.source_path, "stdlib/_sifr/python.sifr");
    assert_eq!(
        private_code.source_sha256,
        sha256_hex(include_str!("../../../../stdlib/_sifr/python.sifr"))
    );
    for name in [
        "py_from_none",
        "py_from_bool",
        "py_from_int",
        "py_from_float",
        "py_from_str",
        "py_from_bytes",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains("PythonError { message: __sifr_bridge_error.message.to_string(), kind: __sifr_bridge_error.kind.to_string(), exception_type: __sifr_bridge_error.exception_type.to_string(), traceback: __sifr_bridge_error.traceback.to_string(), context: __sifr_bridge_error.context.to_string() }"));
    let private_intrinsics = compiled
        .code
        .intrinsic_names
        .get("_sifr.python")
        .expect("_sifr.python intrinsic names should be tracked");
    for name in [
        "py_from_none",
        "py_from_bool",
        "py_from_int",
        "py_from_float",
        "py_from_str",
        "py_from_bytes",
    ] {
        assert!(
            !private_intrinsics.contains(name),
            "{name} should not remain a compiler-retained _sifr.python intrinsic"
        );
    }
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.python_core")
        .is_some_and(|deps| deps.contains("_sifr.python")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.python")
        .is_some_and(|deps| deps.contains("_sifr.python")));
}

fn sha256_hex(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}
