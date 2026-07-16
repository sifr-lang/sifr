use super::compile_stdlib_uncached;
use sha2::{Digest, Sha256};

#[test]
fn fs_private_declarations_codegen_through_sifr_stdlib() {
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
        "open_file",
        "file_read",
        "file_write",
        "file_readline",
        "file_readlines",
        "file_close",
        "file_read_bytes",
        "file_write_bytes",
        "getcwd",
        "listdir",
        "mkdir",
        "rmdir",
        "remove_file",
        "rename",
        "chdir",
        "stat_size",
        "disk_usage",
        "is_file",
        "is_dir",
        "copy_file",
        "walk_dir",
        "rmdir_all",
        "gettempdir",
        "makedirs",
        "touch",
        "resolve_path",
        "iterdir",
        "glob_pattern",
        "rglob_pattern",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::fs::{name}(")),
            "{name} should lower through _sifr.fs private Rust interop declarations"
        );
    }
    assert!(private_code
        .rust
        .contains("fn __io_err<E: ::std::fmt::Display + 'static>"));
    assert!(private_code
        .rust
        .contains(".map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))"));
    assert!(!private_code
        .rust
        .contains("kind: __sifr_bridge_error.to_string()"));
    assert!(private_code.rust.contains(
        "::sifr_stdlib::fs::stat_size(path).map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())"
    ));
    assert!(private_code.rust.contains(
        "::sifr_stdlib::fs::disk_usage(path).into_iter().map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating()).collect()"
    ));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.io")
        .is_some_and(|deps| deps.contains("_sifr.fs")));
}

fn sha256_hex(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}
