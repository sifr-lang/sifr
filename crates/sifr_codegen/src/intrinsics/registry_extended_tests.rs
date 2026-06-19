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
        Some(sifr_stdlib::StdlibFeature::PythonRuntime)
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
        Some(sifr_stdlib::StdlibFeature::PythonRuntime)
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
        Some(sifr_stdlib::StdlibFeature::PythonRuntime)
    );
    assert!(render_expr(&from_str.expr).contains("sifr_runtime::python::from_str"));

    let to_i32 = lower_intrinsic(
        "py_to_i32",
        &["object_handle".to_string(), "object_token".to_string()],
    )
    .expect("py_to_i32 should lower");
    assert_eq!(
        to_i32.required_feature,
        Some(sifr_stdlib::StdlibFeature::PythonRuntime)
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
}

#[test]
pub(crate) fn lowers_uuid_intrinsic_via_registry() {
    let uuid = lower_intrinsic("uuid4", &[]).expect("uuid4");
    assert_eq!(
        uuid.required_feature,
        Some(sifr_stdlib::StdlibFeature::Rand)
    );
    assert!(render_expr(&uuid.expr).contains("rand::random::<u32>()"));
    assert!(render_expr(&uuid.expr).contains("format!(\"{:08x}-{:04x}-{:04x}-{:04x}-{:012x}\""));
    assert!(render_expr(&uuid.expr).contains("(rand::random::<u16>() & 4095)"));

    let uuid3 =
        lower_intrinsic("uuid3_text", &["ns".to_string(), "name".to_string()]).expect("uuid3");
    assert_eq!(
        uuid3.required_feature,
        Some(sifr_stdlib::StdlibFeature::Uuid)
    );
    assert!(render_expr(&uuid3.expr).contains("uuid::Uuid::parse_str"));
    assert!(render_expr(&uuid3.expr).contains("uuid::Uuid::new_v3"));

    let uuid5 =
        lower_intrinsic("uuid5_text", &["ns".to_string(), "name".to_string()]).expect("uuid5");
    assert_eq!(
        uuid5.required_feature,
        Some(sifr_stdlib::StdlibFeature::Uuid)
    );
    assert!(render_expr(&uuid5.expr).contains("uuid::Uuid::new_v5"));
}

#[test]
pub(crate) fn lowers_toml_intrinsic_with_dependency_metadata() {
    let parsed = lower_intrinsic("toml_parse", &["payload".to_string()]).expect("toml_parse");
    assert_eq!(
        parsed.required_feature,
        Some(sifr_stdlib::StdlibFeature::Toml)
    );
    assert!(render_expr(&parsed.expr).contains("parse::<toml::Table>()"));
    assert!(render_expr(&parsed.expr).contains("TOMLDecodeError"));
}

#[test]
pub(crate) fn lowers_datetime_intrinsics_via_registry() {
    let now = lower_intrinsic("datetime_now", &[]).expect("datetime_now");
    assert_eq!(
        now.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&now.expr).contains("chrono::Local::now()"));

    let now_struct = lower_intrinsic("datetime_now_struct", &[]).expect("datetime_now_struct");
    assert_eq!(
        now_struct.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
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
        Some(sifr_stdlib::StdlibFeature::Chrono)
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
    let esc = lower_intrinsic("html_escape", &["s".to_string()]).expect("html_escape");
    assert!(render_expr(&esc.expr).contains("replace('&', \"&amp;\")"));

    let unesc = lower_intrinsic("html_unescape", &["s".to_string()]).expect("html_unescape");
    assert!(render_expr(&unesc.expr).contains("replace(\"&amp;\", \"&\")"));
}

#[test]
pub(crate) fn lowers_calendar_intrinsics_via_registry() {
    let leap = lower_intrinsic("calendar_isleap", &["year".to_string()]).expect("calendar_isleap");
    let rendered = render_expr(&leap.expr);
    // Structured IR adds parentheses around binop comparisons
    assert!(rendered.contains("((__y % 4) == 0)"));

    let weekday = lower_intrinsic(
        "calendar_weekday",
        &["y".to_string(), "m".to_string(), "d".to_string()],
    )
    .expect("calendar_weekday");
    assert!(render_expr(&weekday.expr).contains("__t = vec![0, 3, 2, 5"));
    assert!(render_expr(&weekday.expr).contains("__t[(__m0 - 1) as usize]"));

    let monthrange = lower_intrinsic("calendar_monthrange", &["y".to_string(), "m".to_string()])
        .expect("calendar_monthrange");
    assert!(render_expr(&monthrange.expr).contains("vec![__wd, __days]"));
}

#[test]
pub(crate) fn lowers_gzip_intrinsics_with_dependency_metadata() {
    let compress = lower_intrinsic("gzip_compress", &["data".to_string()]).expect("gzip_compress");
    assert_eq!(
        compress.required_feature,
        Some(sifr_stdlib::StdlibFeature::Flate2)
    );
    assert!(render_expr(&compress.expr).contains("GzEncoder"));

    let decompress =
        lower_intrinsic("gzip_decompress", &["bytes".to_string()]).expect("gzip_decompress");
    assert_eq!(
        decompress.required_feature,
        Some(sifr_stdlib::StdlibFeature::Flate2)
    );
    assert!(render_expr(&decompress.expr).contains("GzDecoder"));
}

#[test]
pub(crate) fn lowers_zip_intrinsics_with_dependency_metadata() {
    let create = lower_intrinsic("zip_create", &["path".to_string()]).expect("zip_create");
    assert_eq!(
        create.required_feature,
        Some(sifr_stdlib::StdlibFeature::Zip)
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
    assert_eq!(add.required_feature, Some(sifr_stdlib::StdlibFeature::Zip));
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
        Some(sifr_stdlib::StdlibFeature::Zip)
    );
    assert!(render_expr(&add_bytes.expr).contains("write_all"));

    let read = lower_intrinsic("zip_read_file", &["path".to_string(), "name".to_string()])
        .expect("zip_read_file");
    assert_eq!(read.required_feature, Some(sifr_stdlib::StdlibFeature::Zip));
    assert!(render_expr(&read.expr).contains("ZipArchive::new"));

    let read_bytes = lower_intrinsic(
        "zip_read_file_bytes",
        &["path".to_string(), "name".to_string()],
    )
    .expect("zip_read_file_bytes");
    assert_eq!(
        read_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Zip)
    );
    assert!(render_expr(&read_bytes.expr).contains("read_to_end"));

    let names = lower_intrinsic("zip_namelist", &["path".to_string()]).expect("zip_namelist");
    assert_eq!(
        names.required_feature,
        Some(sifr_stdlib::StdlibFeature::Zip)
    );
    assert!(render_expr(&names.expr).contains("__zip.by_index"));
}

#[test]
pub(crate) fn lowers_base64_intrinsics_with_dependency_metadata() {
    let enc = lower_intrinsic("base64_encode", &["text".to_string()]).expect("base64_encode");
    assert_eq!(
        enc.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&enc.expr).contains("base64::Engine::encode"));
    assert!(render_expr(&enc.expr).contains("general_purpose::STANDARD"));

    let dec = lower_intrinsic("base64_decode", &["s".to_string()]).expect("base64_decode");
    assert_eq!(
        dec.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&dec.expr).contains("base64::Engine::decode"));
    assert!(render_expr(&dec.expr).contains("general_purpose::STANDARD"));

    let enc_bytes =
        lower_intrinsic("base64_encode_bytes", &["b".to_string()]).expect("base64_encode_bytes");
    assert_eq!(
        enc_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&enc_bytes.expr).contains("into_bytes"));

    let dec_bytes =
        lower_intrinsic("base64_decode_bytes", &["b".to_string()]).expect("base64_decode_bytes");
    assert_eq!(
        dec_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&dec_bytes.expr).contains("base64::Engine::decode"));

    let enc_opts = lower_intrinsic(
        "base64_encode_opts",
        &["s".to_string(), "alt".to_string(), "wrap".to_string()],
    )
    .expect("base64_encode_opts");
    assert_eq!(
        enc_opts.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&enc_opts.expr).contains("wrapcol must be >= 0"));

    let dec_opts = lower_intrinsic(
        "base64_decode_opts",
        &[
            "s".to_string(),
            "alt".to_string(),
            "validate".to_string(),
            "ignore".to_string(),
        ],
    )
    .expect("base64_decode_opts");
    assert_eq!(
        dec_opts.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&dec_opts.expr).contains("invalid base64 character"));

    let url_enc =
        lower_intrinsic("urlsafe_b64encode", &["s".to_string()]).expect("urlsafe_b64encode");
    assert_eq!(
        url_enc.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&url_enc.expr).contains("base64::Engine::encode"));
    assert!(render_expr(&url_enc.expr).contains("general_purpose::URL_SAFE"));

    let url_dec =
        lower_intrinsic("urlsafe_b64decode", &["s".to_string()]).expect("urlsafe_b64decode");
    assert_eq!(
        url_dec.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&url_dec.expr).contains("base64::Engine::decode"));
    assert!(render_expr(&url_dec.expr).contains("general_purpose::URL_SAFE"));

    let url_enc_bytes = lower_intrinsic("urlsafe_b64encode_bytes", &["b".to_string()])
        .expect("urlsafe_b64encode_bytes");
    assert_eq!(
        url_enc_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&url_enc_bytes.expr).contains("into_bytes"));

    let url_dec_bytes = lower_intrinsic("urlsafe_b64decode_bytes", &["b".to_string()])
        .expect("urlsafe_b64decode_bytes");
    assert_eq!(
        url_dec_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Base64)
    );
    assert!(render_expr(&url_dec_bytes.expr).contains("base64::Engine::decode"));
}

#[test]
pub(crate) fn lowers_base32_intrinsics_via_registry() {
    let b32e = lower_intrinsic("b32encode", &["s".to_string()]).expect("b32encode");
    assert!(render_expr(&b32e.expr).contains("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"));

    let b32d = lower_intrinsic("b32decode", &["s".to_string()]).expect("b32decode");
    assert!(render_expr(&b32d.expr).contains("invalid base32 char"));

    let b32he = lower_intrinsic("b32hexencode", &["s".to_string()]).expect("b32hexencode");
    assert!(render_expr(&b32he.expr).contains("0123456789ABCDEFGHIJKLMNOPQRSTUV"));

    let b32hd = lower_intrinsic("b32hexdecode", &["s".to_string()]).expect("b32hexdecode");
    assert!(render_expr(&b32hd.expr).contains("invalid base32hex char"));
}

#[test]
pub(crate) fn lowers_hashlib_intrinsics_with_dependency_metadata() {
    let sha1 = lower_intrinsic("sha1", &["s".to_string()]).expect("sha1");
    assert_eq!(
        sha1.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha1)
    );
    assert!(render_expr(&sha1.expr).contains("<sha1::Sha1 as sha1::Digest>::digest"));

    let sha1_bytes = lower_intrinsic("sha1_bytes", &["b".to_string()]).expect("sha1_bytes");
    assert_eq!(
        sha1_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha1)
    );
    assert!(render_expr(&sha1_bytes.expr).contains("to_vec"));

    let sha512 = lower_intrinsic("sha512", &["s".to_string()]).expect("sha512");
    assert_eq!(
        sha512.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha2)
    );
    assert!(render_expr(&sha512.expr).contains("<sha2::Sha512 as sha2::Digest>::digest"));

    let sha512_bytes = lower_intrinsic("sha512_bytes", &["b".to_string()]).expect("sha512_bytes");
    assert_eq!(
        sha512_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha2)
    );
    assert!(render_expr(&sha512_bytes.expr).contains("to_vec"));

    let sha224 = lower_intrinsic("sha224", &["s".to_string()]).expect("sha224");
    assert_eq!(
        sha224.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha2)
    );
    assert!(render_expr(&sha224.expr).contains("<sha2::Sha224 as sha2::Digest>::digest"));

    let sha224_bytes = lower_intrinsic("sha224_bytes", &["b".to_string()]).expect("sha224_bytes");
    assert_eq!(
        sha224_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha2)
    );
    assert!(render_expr(&sha224_bytes.expr).contains("to_vec"));

    let sha384 = lower_intrinsic("sha384", &["s".to_string()]).expect("sha384");
    assert_eq!(
        sha384.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha2)
    );
    assert!(render_expr(&sha384.expr).contains("<sha2::Sha384 as sha2::Digest>::digest"));

    let sha384_bytes = lower_intrinsic("sha384_bytes", &["b".to_string()]).expect("sha384_bytes");
    assert_eq!(
        sha384_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha2)
    );
    assert!(render_expr(&sha384_bytes.expr).contains("to_vec"));

    let blake2b = lower_intrinsic("blake2b", &["s".to_string()]).expect("blake2b");
    assert_eq!(
        blake2b.required_feature,
        Some(sifr_stdlib::StdlibFeature::Blake2)
    );
    assert!(render_expr(&blake2b.expr).contains("Blake2b512"));

    let blake2b_bytes =
        lower_intrinsic("blake2b_bytes", &["b".to_string()]).expect("blake2b_bytes");
    assert_eq!(
        blake2b_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Blake2)
    );
    assert!(render_expr(&blake2b_bytes.expr).contains("to_vec"));

    let blake2s = lower_intrinsic("blake2s", &["s".to_string()]).expect("blake2s");
    assert_eq!(
        blake2s.required_feature,
        Some(sifr_stdlib::StdlibFeature::Blake2)
    );
    assert!(render_expr(&blake2s.expr).contains("Blake2s256"));

    let blake2s_bytes =
        lower_intrinsic("blake2s_bytes", &["b".to_string()]).expect("blake2s_bytes");
    assert_eq!(
        blake2s_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Blake2)
    );
    assert!(render_expr(&blake2s_bytes.expr).contains("to_vec"));
}

#[test]
pub(crate) fn lowers_extended_math_intrinsics_via_registry() {
    let remainder =
        lower_intrinsic("remainder", &["x".to_string(), "y".to_string()]).expect("remainder");
    assert!(render_expr(&remainder.expr).contains("__abs_frac < 0.5"));

    let dist = lower_intrinsic("dist", &["p".to_string(), "q".to_string()]).expect("dist");
    assert!(render_expr(&dist.expr).contains("__p.len() != __q.len()"));

    let fsum = lower_intrinsic("fsum", &["vals".to_string()]).expect("fsum");
    assert!(render_expr(&fsum.expr).contains("__sum + __comp"));

    let sumprod = lower_intrinsic("sumprod", &["a".to_string(), "b".to_string()]).expect("sumprod");
    assert!(render_expr(&sumprod.expr).contains("__p.len().min(__q.len())"));

    let ldexp = lower_intrinsic("ldexp", &["m".to_string(), "e".to_string()]).expect("ldexp");
    assert!(render_expr(&ldexp.expr).contains("(2.0_f64).powi"));

    let modf = lower_intrinsic("modf", &["x".to_string()]).expect("modf");
    assert!(render_expr(&modf.expr).contains("__x.is_nan()"));

    let ulp = lower_intrinsic("ulp", &["x".to_string()]).expect("ulp");
    assert!(render_expr(&ulp.expr).contains("__x.is_infinite()"));

    let nextafter =
        lower_intrinsic("nextafter", &["x".to_string(), "y".to_string()]).expect("nextafter");
    assert!(render_expr(&nextafter.expr).contains("__x == __y"));

    let erf = lower_intrinsic("erf", &["x".to_string()]).expect("erf");
    assert!(render_expr(&erf.expr).contains("__x >= 0.0"));

    let erfc = lower_intrinsic("erfc", &["x".to_string()]).expect("erfc");
    assert!(render_expr(&erfc.expr).contains("2.0 - __r"));

    let frexp = lower_intrinsic("frexp", &["x".to_string()]).expect("frexp");
    assert!(render_expr(&frexp.expr).contains("__x == 0.0"));

    let gamma = lower_intrinsic("gamma", &["x".to_string()]).expect("gamma");
    assert!(render_expr(&gamma.expr).contains("__x <= 0.0"));

    let lgamma = lower_intrinsic("lgamma", &["x".to_string()]).expect("lgamma");
    assert!(render_expr(&lgamma.expr).contains("__r.exp()"));
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
