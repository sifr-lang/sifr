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
    let imported = lower_intrinsic("py_import_module", &["module".to_string()])
        .expect("py_import_module should lower");
    assert_eq!(
        imported.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::PythonRuntime)
    );
    let rendered = render_expr(&imported.expr);
    assert!(rendered.contains("sifr_runtime::python::import_module"));
    assert!(rendered.contains("PythonError"));

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
        Some(sifr_stdlib_model::StdlibFeature::PythonRuntime)
    );
    let call_rendered = render_expr(&call.expr);
    assert!(call_rendered.contains("sifr_runtime::python::call_object"));
    assert!(call_rendered.contains("(callable_handle, callable_token)"));
    assert!(call_rendered.contains("__sifr_python_args"));
    assert!(call_rendered.contains("__sifr_python_kwargs"));

    let from_str =
        lower_intrinsic("py_from_str", &["value".to_string()]).expect("py_from_str should lower");
    assert_eq!(
        from_str.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::PythonRuntime)
    );
    assert!(render_expr(&from_str.expr).contains("sifr_runtime::python::from_str"));

    let to_i32 = lower_intrinsic(
        "py_to_i32",
        &["object_handle".to_string(), "object_token".to_string()],
    )
    .expect("py_to_i32 should lower");
    assert_eq!(
        to_i32.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::PythonRuntime)
    );
    let to_i32_rendered = render_expr(&to_i32.expr);
    assert!(to_i32_rendered.contains("sifr_runtime::python::to_i32"));
    assert!(to_i32_rendered.contains("(object_handle, object_token)"));

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

    let diagnostics = lower_intrinsic("py_resource_diagnostics", &[])
        .expect("py_resource_diagnostics should lower");
    assert!(render_expr(&diagnostics.expr).contains("sifr_runtime::python::resource_diagnostics"));

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
pub(crate) fn lowers_datetime_intrinsics_via_registry() {
    let now = lower_intrinsic("datetime_now", &[]).expect("datetime_now");
    assert_eq!(
        now.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Chrono)
    );
    assert!(render_expr(&now.expr).contains("chrono::Local::now()"));

    let now_struct = lower_intrinsic("datetime_now_struct", &[]).expect("datetime_now_struct");
    assert_eq!(
        now_struct.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Chrono)
    );
    assert!(render_expr(&now_struct.expr).contains("chrono::Datelike::year(&__dt) as i64"));
    assert!(render_expr(&now_struct.expr).contains("chrono::Timelike::second(&__dt) as i64"));

    let fmt = lower_intrinsic("datetime_format", &["dt".to_string(), "mask".to_string()])
        .expect("datetime_format");
    assert!(render_expr(&fmt.expr).contains("NaiveDateTime::parse_from_str"));

    let from_ts =
        lower_intrinsic("datetime_from_timestamp", &["ts".to_string()]).expect("from_timestamp");
    assert_eq!(
        from_ts.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Chrono)
    );
    assert!(render_expr(&from_ts.expr).contains("DateTime::from_timestamp"));
    assert!(render_expr(&from_ts.expr).contains("ok_or_else"));
    assert!(render_expr(&from_ts.expr).contains("\"invalid timestamp\".to_string()"));
}

#[test]
pub(crate) fn lowers_sys_intrinsics_via_registry() {
    let exit = lower_intrinsic("sys_exit", &["code".to_string()]).expect("sys_exit");
    assert!(render_expr(&exit.expr).contains("std::process::exit("));
    assert!(render_expr(&exit.expr).contains("as i32"));

    let version = lower_intrinsic("sys_version", &[]).expect("sys_version");
    assert_eq!(render_expr(&version.expr), "\"sifr 0.1.0\".to_string()");

    let platform = lower_intrinsic("sys_platform", &[]).expect("sys_platform");
    assert_eq!(
        render_expr(&platform.expr),
        "std::env::consts::OS.to_string()"
    );

    let maxsize = lower_intrinsic("sys_maxsize", &[]).expect("sys_maxsize");
    assert_eq!(render_expr(&maxsize.expr), "i64::MAX");
}

#[test]
pub(crate) fn lowers_process_timeout_intrinsics_via_registry() {
    let output_timeout = lower_intrinsic(
        "process_output_timeout",
        &[
            "program".to_string(),
            "args".to_string(),
            "env".to_string(),
            "cwd".to_string(),
            "has_cwd".to_string(),
            "stdin".to_string(),
            "has_stdin".to_string(),
            "timeout".to_string(),
        ],
    )
    .expect("process_output_timeout");
    let rendered = render_expr(&output_timeout.expr);
    assert!(rendered.contains("std::process::Command::new(&program)"));
    assert!(rendered.contains("__timeout_seconds.is_finite()"));
    assert!(rendered.contains("std::time::Instant::now()"));
    assert!(rendered.contains("std::time::Duration::try_from_secs_f64(__timeout_seconds)"));
    assert!(rendered.contains(".checked_add("));
    assert!(rendered.contains("process timeout is too large for this host clock"));
    assert!(rendered.contains("CommandExt::process_group(&mut __cmd, 0)"));
    assert!(rendered.contains("__child.try_wait()"));
    assert!(rendered.contains("__child.kill()"));
    assert!(rendered.contains(".arg(\"-TERM\")"));
    assert!(rendered.contains(".arg(\"-KILL\")"));
    assert!(rendered.contains("__output.stdout"));
    assert!(rendered.contains("__output.stderr"));
    assert!(rendered.contains("__timed_out"));

    let shell_timeout = lower_intrinsic(
        "process_shell_output_timeout",
        &[
            "script".to_string(),
            "stdin".to_string(),
            "has_stdin".to_string(),
            "timeout".to_string(),
        ],
    )
    .expect("process_shell_output_timeout");
    let shell_rendered = render_expr(&shell_timeout.expr);
    assert!(shell_rendered.contains("std::process::Command::new(\"sh\".to_string())"));
    assert!(shell_rendered.contains("__cmd.arg(\"-c\".to_string())"));
    assert!(shell_rendered.contains("__child.kill()"));

    let async_shell_timeout = lower_intrinsic(
        "process_async_shell_output_timeout",
        &[
            "script".to_string(),
            "stdin".to_string(),
            "has_stdin".to_string(),
            "timeout".to_string(),
        ],
    )
    .expect("process_async_shell_output_timeout");
    let async_shell_rendered = render_expr(&async_shell_timeout.expr);
    assert!(async_shell_rendered.contains("Box::pin(__sifr_process_async_output_timeout("));
    assert!(async_shell_rendered.contains("\"sh\".to_string()"));
    assert!(async_shell_rendered.contains("vec![\"-c\".to_string(), script.clone()]"));
    assert!(async_shell_rendered.contains("stdin.clone()"));
    assert!(async_shell_rendered.contains("has_stdin"));
    assert!(async_shell_rendered.contains("timeout"));
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
pub(crate) fn lowers_gzip_intrinsics_with_dependency_metadata() {
    let compress = lower_intrinsic("gzip_compress", &["data".to_string()]).expect("gzip_compress");
    assert_eq!(
        compress.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Flate2)
    );
    assert!(render_expr(&compress.expr).contains("GzEncoder"));

    let decompress =
        lower_intrinsic("gzip_decompress", &["bytes".to_string()]).expect("gzip_decompress");
    assert_eq!(
        decompress.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Flate2)
    );
    assert!(render_expr(&decompress.expr).contains("GzDecoder"));
}

#[test]
pub(crate) fn lowers_zip_intrinsics_with_dependency_metadata() {
    let create = lower_intrinsic("zip_create", &["path".to_string()]).expect("zip_create");
    assert_eq!(
        create.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Zip)
    );
    assert!(render_expr(&create.expr).contains("ZipWriter::new"));

    let add = lower_intrinsic(
        "zip_add_file",
        &[
            "path".to_string(),
            "name".to_string(),
            "content".to_string(),
        ],
    )
    .expect("zip_add_file");
    assert_eq!(
        add.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Zip)
    );
    assert!(render_expr(&add.expr).contains("start_file"));

    let add_bytes = lower_intrinsic(
        "zip_add_file_bytes",
        &[
            "path".to_string(),
            "name".to_string(),
            "content_bytes".to_string(),
        ],
    )
    .expect("zip_add_file_bytes");
    assert_eq!(
        add_bytes.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Zip)
    );
    assert!(render_expr(&add_bytes.expr).contains("write_all"));

    let read = lower_intrinsic("zip_read_file", &["path".to_string(), "name".to_string()])
        .expect("zip_read_file");
    assert_eq!(
        read.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Zip)
    );
    assert!(render_expr(&read.expr).contains("ZipArchive::new"));

    let read_bytes = lower_intrinsic(
        "zip_read_file_bytes",
        &["path".to_string(), "name".to_string()],
    )
    .expect("zip_read_file_bytes");
    assert_eq!(
        read_bytes.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Zip)
    );
    assert!(render_expr(&read_bytes.expr).contains("read_to_end"));

    let names = lower_intrinsic("zip_namelist", &["path".to_string()]).expect("zip_namelist");
    assert_eq!(
        names.required_feature,
        Some(sifr_stdlib_model::StdlibFeature::Zip)
    );
    assert!(render_expr(&names.expr).contains("__zip.by_index"));
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
pub(crate) fn lowers_file_handle_and_logging_intrinsics_via_registry() {
    let open =
        lower_intrinsic("open_file", &["path".to_string(), "mode".to_string()]).expect("open_file");
    assert!(render_expr(&open.expr).contains("__SIFR_FILE_HANDLES"));
    assert!(render_expr(&open.expr).contains("__sifr_next_file_handle_id()"));

    let read = lower_intrinsic("file_read", &["hid".to_string()]).expect("file_read");
    assert!(render_expr(&read.expr).contains("TextRead"));

    let write = lower_intrinsic("file_write", &["hid".to_string(), "text".to_string()])
        .expect("file_write");
    assert!(render_expr(&write.expr).contains("TextWrite"));

    let close = lower_intrinsic("file_close", &["hid".to_string()]).expect("file_close");
    assert!(render_expr(&close.expr).contains("__SIFR_FILE_HANDLES"));

    let builtin_open = lower_intrinsic("builtin_open", &["path".to_string(), "mode".to_string()])
        .expect("builtin_open");
    assert!(render_expr(&builtin_open.expr).contains("FileHandle"));
    assert!(render_expr(&builtin_open.expr).contains("__sifr_next_file_handle_id()"));

    let set_level =
        lower_intrinsic("set_global_level", &["n".to_string()]).expect("set_global_level");
    assert!(render_expr(&set_level.expr).contains("__SIFR_GLOBAL_LOG_LEVEL"));

    let get_level = lower_intrinsic("get_global_level", &[]).expect("get_global_level");
    assert!(render_expr(&get_level.expr).contains("__SIFR_GLOBAL_LOG_LEVEL"));
}

#[test]
pub(crate) fn lower_intrinsic_accepts_ir_inputs() {
    let ir = super::lower_intrinsic(
        "file_write",
        &[
            RustExpr::Ident("hid".to_string()),
            RustExpr::Ident("text".to_string()),
        ],
    )
    .expect("ir file_write");

    assert!(render_expr(&ir.expr).contains("TextWrite"));
    assert_eq!(ir.required_feature, None);
}
