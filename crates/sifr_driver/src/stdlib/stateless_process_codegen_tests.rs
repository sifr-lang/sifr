use super::compile_stdlib_uncached;

#[test]
fn process_sync_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.process")
        .expect("_sifr.process should generate private Rust code");

    for name in [
        "process_run",
        "process_output",
        "process_output_text",
        "process_output_timeout",
        "process_shell_run",
        "process_shell_output",
        "process_shell_output_text",
        "process_shell_output_timeout",
        "process_spawn",
        "process_child_stdin",
        "process_child_stdout",
        "process_child_stderr",
        "process_pipe_read_all",
        "process_pipe_read",
        "process_pipe_reader_close",
        "process_pipe_write_all",
        "process_pipe_close",
        "process_wait",
        "process_kill",
        "process_terminate",
        "process_async_run",
        "process_async_run_timeout",
        "process_async_output",
        "process_async_output_timeout",
        "process_async_shell_run",
        "process_async_shell_output",
        "process_async_shell_output_timeout",
    ] {
        assert!(
            private_code.rust.contains(&format!("fn {name}(")),
            "{name} should be emitted by _sifr.process private declarations"
        );
        assert!(
            private_code
                .rust
                .contains(&format!("sifr_stdlib::process::{name}(")),
            "{name} should lower through _sifr.process private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains(
        "map_err(|__sifr_bridge_error| ProcessError { message: __sifr_bridge_error.to_string() })"
    ));

    let private_intrinsics = compiled
        .code
        .intrinsic_names
        .get("_sifr.process")
        .expect("_sifr.process intrinsic names should be tracked");
    for name in [
        "process_run",
        "process_output",
        "process_output_text",
        "process_output_timeout",
        "process_shell_run",
        "process_shell_output",
        "process_shell_output_text",
        "process_shell_output_timeout",
        "process_spawn",
        "process_child_stdin",
        "process_child_stdout",
        "process_child_stderr",
        "process_pipe_read_all",
        "process_pipe_read",
        "process_pipe_reader_close",
        "process_pipe_write_all",
        "process_pipe_close",
        "process_wait",
        "process_kill",
        "process_terminate",
        "process_async_run",
        "process_async_run_timeout",
        "process_async_output",
        "process_async_output_timeout",
        "process_async_shell_run",
        "process_async_shell_output",
        "process_async_shell_output_timeout",
    ] {
        assert!(
            !private_intrinsics.contains(name),
            "{name} should not remain an active _sifr.process compiler intrinsic"
        );
    }

    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.process")
        .is_some_and(|deps| deps.contains("_sifr.process")));
}
