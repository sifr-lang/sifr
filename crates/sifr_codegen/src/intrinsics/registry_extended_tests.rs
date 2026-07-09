use super::registry_core_tests::lower_intrinsic;
use crate::{render_expr, RustExpr};

#[test]
pub(crate) fn legacy_subprocess_intrinsics_are_not_lowered() {
    for removed in [
        "subprocess_run",
        "subprocess_run_with_input",
        "subprocess_run_structured",
    ] {
        assert!(
            lower_intrinsic(removed, &["cmd".to_string()]).is_none(),
            "{removed} must stay removed; use process_* intrinsics instead"
        );
    }
}

#[test]
pub(crate) fn lowers_python_intrinsics_with_runtime_feature_metadata() {
    let call = lower_intrinsic(
        "py_call",
        &[
            "callable_handle".to_string(),
            "callable_token".to_string(),
            "args".to_string(),
            "kwargs".to_string(),
        ],
    )
    .expect("py_call should lower");
    assert_eq!(
        call.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::PythonRuntime)
    );
    let call_rendered = render_expr(&call.expr);
    assert!(call_rendered.contains("sifr_runtime::python::call_object"));
    assert!(call_rendered.contains("(callable_handle, callable_token)"));
    assert!(call_rendered.contains("__sifr_python_args"));
    assert!(call_rendered.contains("__sifr_python_kwargs"));
    assert!(call_rendered.contains("PythonError"));

    let from_list = lower_intrinsic("py_from_list", &["values".to_string()])
        .expect("py_from_list should lower");
    let from_list_rendered = render_expr(&from_list.expr);
    assert!(from_list_rendered.contains("sifr_runtime::python::from_list"));
    assert!(from_list_rendered.contains("__sifr_python_value.0"));

    let copy_list_u8 = lower_intrinsic(
        "py_copy_list_u8",
        &["object_handle".to_string(), "object_token".to_string()],
    )
    .expect("py_copy_list_u8 should lower");
    assert!(render_expr(&copy_list_u8.expr).contains("sifr_runtime::python::copy_list_u8"));

    let record_fields = lower_intrinsic(
        "py_copy_record_fields",
        &[
            "object_handle".to_string(),
            "object_token".to_string(),
            "fields".to_string(),
        ],
    )
    .expect("py_copy_record_fields should lower");
    let record_fields_rendered = render_expr(&record_fields.expr);
    assert!(record_fields_rendered.contains("sifr_runtime::python::copy_record_fields"));
    assert!(record_fields_rendered.contains("__sifr_python_fields"));

    let coroutine = lower_intrinsic(
        "py_run_coroutine_blocking",
        &["object_handle".to_string(), "object_token".to_string()],
    )
    .expect("py_run_coroutine_blocking should lower");
    let coroutine_rendered = render_expr(&coroutine.expr);
    assert!(coroutine_rendered.contains("sifr_runtime::python::run_coroutine_blocking"));
    assert!(coroutine_rendered.contains("(object_handle, object_token)"));

    let exit_with_error = lower_intrinsic(
        "py_exit_context_with_error",
        &[
            "object_handle".to_string(),
            "object_token".to_string(),
            "kind".to_string(),
            "exception_type".to_string(),
            "message".to_string(),
            "traceback".to_string(),
            "context".to_string(),
        ],
    )
    .expect("py_exit_context_with_error should lower");
    let exit_with_error_rendered = render_expr(&exit_with_error.expr);
    assert!(exit_with_error_rendered.contains("sifr_runtime::python::exit_context_with_error"));
    assert!(exit_with_error_rendered.contains("(object_handle, object_token)"));

    let buffer = lower_intrinsic(
        "py_buffer_u8",
        &[
            "object_handle".to_string(),
            "object_token".to_string(),
            "false".to_string(),
        ],
    )
    .expect("py_buffer_u8 should lower");
    let buffer_rendered = render_expr(&buffer.expr);
    assert!(buffer_rendered.contains("sifr_runtime::python::buffer_u8"));
    assert!(buffer_rendered.contains("__sifr_python_buffer.shape"));

    let release = lower_intrinsic(
        "py_release_buffer",
        &["buffer_handle".to_string(), "buffer_token".to_string()],
    )
    .expect("py_release_buffer should lower");
    assert!(render_expr(&release.expr).contains("sifr_runtime::python::release_buffer"));

    let arrow = lower_intrinsic(
        "py_arrow_stream",
        &["object_handle".to_string(), "object_token".to_string()],
    )
    .expect("py_arrow_stream should lower");
    let arrow_rendered = render_expr(&arrow.expr);
    assert!(arrow_rendered.contains("sifr_runtime::python::arrow_stream"));
    assert!(arrow_rendered.contains("__sifr_python_arrow.copy_possible"));

    let release_arrow = lower_intrinsic(
        "py_release_arrow",
        &["arrow_handle".to_string(), "arrow_token".to_string()],
    )
    .expect("py_release_arrow should lower");
    assert!(render_expr(&release_arrow.expr).contains("sifr_runtime::python::release_arrow"));

    let dlpack = lower_intrinsic(
        "py_dlpack_tensor",
        &["object_handle".to_string(), "object_token".to_string()],
    )
    .expect("py_dlpack_tensor should lower");
    let dlpack_rendered = render_expr(&dlpack.expr);
    assert!(dlpack_rendered.contains("sifr_runtime::python::dlpack_tensor"));
    assert!(dlpack_rendered.contains("__sifr_python_dlpack.dtype"));

    let release_dlpack = lower_intrinsic(
        "py_release_dlpack",
        &["dlpack_handle".to_string(), "dlpack_token".to_string()],
    )
    .expect("py_release_dlpack should lower");
    assert!(render_expr(&release_dlpack.expr).contains("sifr_runtime::python::release_dlpack"));

    let callback =
        lower_intrinsic("py_threadsafe_callback_echo", &[]).expect("callback should lower");
    let callback_rendered = render_expr(&callback.expr);
    assert!(callback_rendered.contains("sifr_runtime::python::threadsafe_callback_echo"));
    assert!(callback_rendered.contains("__sifr_python_callback.object_handle"));

    let registered_callback = lower_intrinsic("local_callback", &["handle_python".to_string()])
        .expect("local_callback should lower");
    let registered_rendered = render_expr(&registered_callback.expr);
    assert!(registered_rendered.contains("sifr_runtime::python::local_callback"));
    assert!(registered_rendered.contains("handle_python(&__sifr_callback_object)"));
    assert!(registered_rendered.contains("LocalCallback::new"));

    let close_callback = lower_intrinsic(
        "py_close_callback",
        &["callback_handle".to_string(), "callback_token".to_string()],
    )
    .expect("py_close_callback should lower");
    assert!(render_expr(&close_callback.expr).contains("sifr_runtime::python::close_callback"));
}

#[test]
pub(crate) fn python_primitive_constructors_are_owned_by_compiled_stdlib_declarations() {
    for removed in [
        "py_from_none",
        "py_from_bool",
        "py_from_int",
        "py_from_float",
        "py_from_str",
        "py_from_bytes",
    ] {
        assert!(
            lower_intrinsic(removed, &["value".to_string()]).is_none(),
            "{removed} must lower through _sifr.python private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn python_primitive_extractors_are_owned_by_compiled_stdlib_declarations() {
    for removed in [
        "py_to_none",
        "py_to_bool",
        "py_to_int",
        "py_to_i8",
        "py_to_i16",
        "py_to_i32",
        "py_to_i64",
        "py_to_u8",
        "py_to_u16",
        "py_to_u32",
        "py_to_u64",
        "py_to_isize",
        "py_to_usize",
        "py_to_float",
        "py_to_str",
        "py_to_bytes",
    ] {
        assert!(
            lower_intrinsic(
                removed,
                &["object_handle".to_string(), "object_token".to_string()]
            )
            .is_none(),
            "{removed} must lower through _sifr.python private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn python_object_core_is_owned_by_compiled_stdlib_declarations() {
    for (removed, args) in [
        ("py_import_module", vec!["name".to_string()]),
        (
            "py_get_attr",
            vec![
                "object_handle".to_string(),
                "object_token".to_string(),
                "name".to_string(),
            ],
        ),
        (
            "py_get_item_str",
            vec![
                "object_handle".to_string(),
                "object_token".to_string(),
                "key".to_string(),
            ],
        ),
        (
            "py_close",
            vec!["object_handle".to_string(), "object_token".to_string()],
        ),
        ("py_resource_diagnostics", vec![]),
    ] {
        assert!(
            lower_intrinsic(removed, &args).is_none(),
            "{removed} must lower through _sifr.python private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn lowers_uuid_intrinsic_via_registry() {
    for removed in ["uuid4", "uuid3_text", "uuid5_text"] {
        assert!(
            lower_intrinsic(removed, &["ns".to_string(), "name".to_string()]).is_none(),
            "{removed} must lower through private stdlib Rust interop, not active intrinsics"
        );
    }
}

#[test]
pub(crate) fn toml_intrinsic_is_owned_by_compiled_stdlib_declaration() {
    assert!(
        lower_intrinsic("toml_parse_tokens", &["payload".to_string()]).is_none(),
        "TOML should lower through _sifr.toml private Rust interop declarations"
    );
    assert!(
        lower_intrinsic("toml_parse", &["payload".to_string()]).is_none(),
        "legacy TOML parse intrinsic must stay retired"
    );
}

#[test]
pub(crate) fn datetime_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for removed in [
        "datetime_now",
        "datetime_now_struct",
        "datetime_format",
        "datetime_from_timestamp",
    ] {
        assert!(
            lower_intrinsic(removed, &["dt".to_string(), "fmt".to_string()]).is_none(),
            "{removed} must lower through private stdlib Rust interop, not active intrinsics"
        );
    }
}

#[test]
pub(crate) fn process_sync_timeout_intrinsics_lower_through_private_stdlib() {
    for (name, args) in [
        (
            "process_output_timeout",
            &[
                "program",
                "args",
                "env",
                "cwd",
                "has_cwd",
                "stdin",
                "has_stdin",
                "timeout",
            ][..],
        ),
        (
            "process_shell_output_timeout",
            &["script", "stdin", "has_stdin", "timeout"][..],
        ),
        (
            "process_spawn",
            &[
                "program",
                "args",
                "env",
                "cwd",
                "has_cwd",
                "stdin_mode",
                "stdout_mode",
                "stderr_mode",
            ][..],
        ),
        ("process_child_stdin", &["handle"][..]),
        ("process_child_stdout", &["handle"][..]),
        ("process_child_stderr", &["handle"][..]),
        ("process_pipe_read_all", &["handle"][..]),
        ("process_pipe_read", &["handle", "max_bytes"][..]),
        ("process_pipe_reader_close", &["handle"][..]),
        ("process_pipe_write_all", &["handle", "data"][..]),
        ("process_pipe_close", &["handle"][..]),
        ("process_wait", &["handle"][..]),
        ("process_kill", &["handle"][..]),
        ("process_terminate", &["handle"][..]),
        (
            "process_async_run",
            &["program", "args", "env", "cwd", "has_cwd", "stdin_mode"][..],
        ),
        (
            "process_async_run_timeout",
            &[
                "program",
                "args",
                "env",
                "cwd",
                "has_cwd",
                "stdin_mode",
                "timeout",
            ][..],
        ),
        (
            "process_async_output",
            &[
                "program",
                "args",
                "env",
                "cwd",
                "has_cwd",
                "stdin_mode",
                "stdin",
                "has_stdin",
            ][..],
        ),
        (
            "process_async_output_timeout",
            &[
                "program",
                "args",
                "env",
                "cwd",
                "has_cwd",
                "stdin_mode",
                "stdin",
                "has_stdin",
                "timeout",
            ][..],
        ),
        ("process_async_shell_run", &["script"][..]),
        (
            "process_async_shell_output",
            &["script", "stdin", "has_stdin"][..],
        ),
        (
            "process_async_shell_output_timeout",
            &["script", "stdin", "has_stdin", "timeout"][..],
        ),
        (
            "process_async_spawn",
            &[
                "program",
                "args",
                "env",
                "cwd",
                "has_cwd",
                "stdin_mode",
                "stdout_mode",
                "stderr_mode",
                "has_stdin",
            ][..],
        ),
        ("process_async_wait", &["handle"][..]),
        ("process_handle_wait", &["handle"][..]),
        ("process_async_kill", &["handle"][..]),
        ("process_async_terminate", &["handle"][..]),
        ("process_async_child_stdin", &["handle"][..]),
        ("process_async_child_stdout", &["handle"][..]),
        ("process_async_child_stderr", &["handle"][..]),
        ("process_async_pipe_read_all", &["handle"][..]),
        ("process_async_pipe_read", &["handle", "max_bytes"][..]),
        ("process_async_pipe_reader_close", &["handle"][..]),
        ("process_async_pipe_write_all", &["handle", "data"][..]),
        ("process_async_pipe_close", &["handle"][..]),
    ] {
        assert!(
            lower_intrinsic(
                name,
                &args
                    .iter()
                    .map(|arg| (*arg).to_string())
                    .collect::<Vec<_>>()
            )
            .is_none(),
            "{name} should lower through _sifr.process private declarations"
        );
    }
}

#[test]
pub(crate) fn lowers_html_intrinsics_via_registry() {
    assert!(lower_intrinsic("html_escape", &["s".to_string()]).is_none());
    assert!(lower_intrinsic("html_unescape", &["s".to_string()]).is_none());
}

#[test]
pub(crate) fn lowers_calendar_intrinsics_via_registry() {
    assert!(lower_intrinsic("calendar_isleap", &["year".to_string()]).is_none());
    assert!(lower_intrinsic(
        "calendar_weekday",
        &["y".to_string(), "m".to_string(), "d".to_string()],
    )
    .is_none());
    assert!(lower_intrinsic("calendar_monthrange", &["y".to_string(), "m".to_string()]).is_none());
}

#[test]
pub(crate) fn compression_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for (name, args) in [
        ("_gzip_compress_bytes_impl", &["data"][..]),
        ("_gzip_decompress_bytes_impl", &["bytes"][..]),
        ("zip_create", &["path"][..]),
        ("zip_add_file", &["path", "name", "content"][..]),
        ("zip_add_file_bytes", &["path", "name", "content_bytes"][..]),
        ("zip_read_file", &["path", "name"][..]),
        ("zip_read_file_bytes", &["path", "name"][..]),
        ("zip_namelist", &["path"][..]),
    ] {
        assert!(
            lower_intrinsic(
                name,
                &args
                    .iter()
                    .map(|arg| (*arg).to_string())
                    .collect::<Vec<_>>()
            )
            .is_none(),
            "{name} should be provided by _sifr.compress private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn base_encoding_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for (name, args) in [
        ("base64_encode", &["payload"][..]),
        ("base64_encode_bytes", &["payload"][..]),
        ("base64_decode", &["payload"][..]),
        ("base64_decode_bytes", &["payload"][..]),
        ("base64_encode_opts", &["payload", "alt", "wrap"][..]),
        (
            "base64_decode_opts",
            &["payload", "alt", "validate", "ignore"][..],
        ),
        ("urlsafe_b64encode", &["payload"][..]),
        ("urlsafe_b64encode_bytes", &["payload"][..]),
        ("urlsafe_b64decode", &["payload"][..]),
        ("urlsafe_b64decode_bytes", &["payload"][..]),
        ("b32encode", &["payload"][..]),
        ("b32decode", &["payload"][..]),
        ("b32hexencode", &["payload"][..]),
        ("b32hexdecode", &["payload"][..]),
    ] {
        let args = args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        assert!(
            lower_intrinsic(name, &args).is_none(),
            "{name} should lower through _sifr.crypto private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn extended_hash_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for name in [
        "sha1",
        "sha1_bytes",
        "sha224",
        "sha224_bytes",
        "sha384",
        "sha384_bytes",
        "sha512",
        "sha512_bytes",
        "blake2b",
        "blake2b_bytes",
        "blake2s",
        "blake2s_bytes",
    ] {
        assert!(
            lower_intrinsic(name, &["payload".to_string()]).is_none(),
            "{name} should lower through _sifr.crypto private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn lowers_extended_math_intrinsics_via_registry() {
    for name in [
        "remainder",
        "dist",
        "fsum",
        "sumprod",
        "ldexp",
        "modf",
        "ulp",
        "nextafter",
        "erf",
        "erfc",
        "frexp",
        "gamma",
        "lgamma",
    ] {
        assert!(
            lower_intrinsic(name, &["x".to_string(), "y".to_string()]).is_none(),
            "{name} must lower through private stdlib Rust interop, not active intrinsics"
        );
    }
}

#[test]
pub(crate) fn lowers_file_handle_builtin_bridge_and_migrated_intrinsics() {
    for name in [
        "open_file",
        "file_read",
        "file_write",
        "file_readline",
        "file_readlines",
        "file_close",
        "file_read_bytes",
        "file_write_bytes",
        "set_global_level",
        "get_global_level",
    ] {
        assert!(
            lower_intrinsic(name, &["hid".to_string(), "payload".to_string()]).is_none(),
            "{name} should lower through private Rust interop declarations"
        );
    }

    let builtin_open = lower_intrinsic("builtin_open", &["path".to_string(), "mode".to_string()])
        .expect("builtin_open");
    assert!(render_expr(&builtin_open.expr).contains("FileHandle"));
    assert!(render_expr(&builtin_open.expr).contains("sifr_stdlib::fs::open_file"));
    assert!(render_expr(&builtin_open.expr).contains("NativeFileHandle"));
}

#[test]
pub(crate) fn lower_intrinsic_accepts_ir_inputs() {
    let ir = super::lower_intrinsic(
        "builtin_open",
        &[
            RustExpr::Ident("path".to_string()),
            RustExpr::Ident("mode".to_string()),
        ],
    )
    .expect("ir builtin_open");

    assert!(render_expr(&ir.expr).contains("sifr_stdlib::fs::open_file"));
    assert_eq!(ir.required_feature, None);
}
