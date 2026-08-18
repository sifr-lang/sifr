use super::*;
use crate::{render_expr, RustExpr};
use sifr_ir::CompilerIntrinsicId;

pub(crate) fn parse_test_arg(rendered: &str) -> RustExpr {
    if let Ok(v) = rendered.parse::<i64>() {
        return RustExpr::Literal(crate::RustLiteral::Int(v));
    }
    if let Ok(v) = rendered.parse::<f64>() {
        return RustExpr::Literal(crate::RustLiteral::Float(v));
    }
    if rendered.contains("::")
        && rendered.split("::").all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c == '_' || c.is_ascii_alphanumeric())
        })
    {
        return RustExpr::Path(rendered.split("::").map(str::to_string).collect());
    }
    RustExpr::Ident(rendered.to_string())
}

pub(crate) fn lower_intrinsic(name: &str, rendered_args: &[String]) -> Option<LoweredIntrinsic> {
    let intrinsic = match name {
        "builtin_open" => CompilerIntrinsicId::OpenBinary,
        "builtin_open_text" => CompilerIntrinsicId::OpenText,
        "assert_eq" => CompilerIntrinsicId::TestAssertEqual,
        "assert_ne" => CompilerIntrinsicId::TestAssertNotEqual,
        "assert_true" => CompilerIntrinsicId::TestAssertTrue,
        "assert_false" => CompilerIntrinsicId::TestAssertFalse,
        "assert_almost_eq" => CompilerIntrinsicId::TestAssertAlmostEqual,
        "assert_gt" => CompilerIntrinsicId::TestAssertGreaterThan,
        "assert_lt" => CompilerIntrinsicId::TestAssertLessThan,
        "str_encode_utf8_result" => CompilerIntrinsicId::StringEncode,
        "str_encode_utf8_result_with_encoding" => CompilerIntrinsicId::StringEncodeWithEncoding,
        "decode_utf8" => CompilerIntrinsicId::BytesDecode,
        "decode_utf8_with_encoding" => CompilerIntrinsicId::BytesDecodeWithEncoding,
        "bytes_from_hex" => CompilerIntrinsicId::BytesFromHex,
        "bytes_with_size" => CompilerIntrinsicId::BytesWithSize,
        "bytes_from_ints" => CompilerIntrinsicId::BytesFromIntegers,
        "task_current_context" => CompilerIntrinsicId::TaskCurrentContext,
        _ => return None,
    };
    let args = rendered_args
        .iter()
        .cloned()
        .map(|arg| parse_test_arg(&arg))
        .collect::<Vec<_>>();
    super::lower_intrinsic(intrinsic, &args)
}

#[test]
pub(crate) fn math_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    let retired = [
        "sqrt",
        "pow_val",
        "atan2",
        "round_val",
        "floor",
        "ceil",
        "isfinite",
        "isqrt",
    ];
    for name in retired {
        assert!(
            lower_intrinsic(name, &["x".to_string(), "y".to_string(), "z".to_string()]).is_none(),
            "{name} should lower through _sifr.math private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn json_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    let retired = [
        "json_loads",
        "json_validate_integer_digit_limits",
        "json_dumps",
        "json_dumps_value",
        "json_dumps_value_exact",
        "json_dumps_value_web",
        "json_dumps_value_string_ints",
        "json_load_tokens",
        "json_dump_tokens",
        "json_dump_tokens_web",
    ];
    for name in retired {
        assert!(
            lower_intrinsic(name, &["payload".to_string()]).is_none(),
            "{name} should lower through _sifr.json private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn encoding_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    let retired = [
        "encoding_is_supported",
        "encoding_canonical_label",
        "encoding_decode_text",
        "encoding_decode_recoveries",
        "encoding_decode_outcome",
        "encoding_decode_incremental_outcome",
        "encoding_decode_incremental_pending",
        "encoding_encode_bytes",
        "encoding_encode_recoveries",
        "encoding_encode_outcome",
    ];
    for name in retired {
        assert!(
            lower_intrinsic(
                name,
                &[
                    "payload".to_string(),
                    "codec".to_string(),
                    "errors".to_string()
                ]
            )
            .is_none(),
            "{name} should lower through _sifr.encoding private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn runtime_module_dependency_metadata_includes_observability_facades() {
    let deps = sifr_stdlib_manifest::try_generated_cargo_dependencies(
        &std::collections::HashSet::from(["sifr.runtime".to_string()]),
        &std::collections::HashSet::new(),
    )
    .expect("source-tree sysroot dependencies should resolve");

    assert_eq!(deps.len(), 1);
    assert!(deps[0].starts_with("sifr_stdlib = "));
    assert!(deps[0].contains("default-features = false"));
    assert!(deps[0].contains("features = [\"runtime-observability\"]"));
    assert!(!deps.iter().any(|dep| dep.starts_with("metrics = ")));
    assert!(!deps.iter().any(|dep| dep.starts_with("tracing = ")));
}

#[test]
pub(crate) fn lowers_task_current_context_as_language_runtime_glue() {
    let lowered =
        lower_intrinsic("task_current_context", &[]).expect("task_current_context should lower");

    assert_eq!(
        lowered.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Tokio)
    );
    assert_eq!(render_expr(&lowered.expr), "__sifr_task_current_context()");
}

#[test]
pub(crate) fn task_current_context_intrinsic_rejects_wrong_arity() {
    assert!(lower_intrinsic("task_current_context", &["unexpected".to_string()]).is_none());
}

#[test]
pub(crate) fn unicode_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for name in [
        "unicode_data_version",
        "unicode_normalize",
        "unicode_is_normalized",
        "unicode_name",
        "unicode_lookup",
        "unicode_category",
        "unicode_bidirectional",
        "unicode_combining",
        "unicode_east_asian_width",
        "unicode_mirrored",
        "unicode_decomposition",
        "unicode_decimal",
        "unicode_digit",
        "unicode_numeric_value",
        "unicode_case_fold",
        "unicode_graphemes",
        "unicode_grapheme_indices",
        "unicode_words",
        "unicode_word_boundaries",
    ] {
        assert!(
            lower_intrinsic(name, &["text".to_string()]).is_none(),
            "{name} should lower through _sifr.unicode private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn i18n_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for name in [
        "i18n_locale_canonicalize",
        "i18n_locale_maximize",
        "i18n_locale_minimize",
        "i18n_host_locale",
        "i18n_format_number",
        "i18n_format_datetime",
        "i18n_plural_category",
        "i18n_collate",
        "i18n_mo_validate",
        "i18n_mo_load_file",
        "i18n_mo_lookup",
        "i18n_mo_lookup_context",
        "i18n_mo_lookup_plural",
        "i18n_mo_lookup_context_plural",
    ] {
        assert!(
            lower_intrinsic(name, &["value".to_string()]).is_none(),
            "{name} should lower through _sifr.i18n private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn env_and_sys_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for name in [
        "env_get",
        "env_set",
        "env_unset",
        "env_keys",
        "env_values",
        "env_items",
        "get_args",
        "sys_exit",
        "sys_version",
        "sys_platform",
        "sys_maxsize",
        "getpid",
        "cpu_count",
        "which",
        "os_sep",
        "os_linesep",
        "os_name",
        "platform_system",
        "platform_arch",
        "platform_node",
        "platform_release",
        "platform_version",
        "platform_processor",
    ] {
        assert!(
            lower_intrinsic(name, &["value".to_string()]).is_none(),
            "{name} should lower through private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn os_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for name in ["run_command", "chdir", "stat_size", "disk_usage"] {
        assert!(
            lower_intrinsic(name, &["value".to_string()]).is_none(),
            "{name} should lower through private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn signal_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for name in ["signal_ctrl_c", "signal_terminate", "signal_shutdown"] {
        assert!(
            lower_intrinsic(name, &[]).is_none(),
            "{name} should lower through _sifr.signal private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn fs_text_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for name in [
        "read_text",
        "write_text",
        "exists",
        "read_lines",
        "append_text",
    ] {
        assert!(
            lower_intrinsic(name, &["path".to_string(), "content".to_string()]).is_none(),
            "{name} should lower through _sifr.fs private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn fs_path_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for (name, args) in [
        ("getcwd", &[][..]),
        ("listdir", &["path"][..]),
        ("mkdir", &["path"][..]),
        ("rmdir", &["path"][..]),
        ("remove_file", &["path"][..]),
        ("rename", &["src", "dst"][..]),
        ("is_file", &["path"][..]),
        ("is_dir", &["path"][..]),
        ("copy_file", &["src", "dst"][..]),
        ("walk_dir", &["path"][..]),
        ("rmdir_all", &["path"][..]),
        ("gettempdir", &[][..]),
        ("makedirs", &["path"][..]),
        ("touch", &["path"][..]),
        ("resolve_path", &["path"][..]),
        ("iterdir", &["path"][..]),
        ("glob_pattern", &["dir", "pattern"][..]),
        ("rglob_pattern", &["dir", "pattern"][..]),
    ] {
        let args = args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        assert!(
            lower_intrinsic(name, &args).is_none(),
            "{name} should lower through _sifr.fs private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn lowers_test_intrinsics_via_registry() {
    let eq = lower_intrinsic("assert_eq", &["a".to_string(), "b".to_string()])
        .expect("assert_eq lowers");
    assert_eq!(render_expr(&eq.expr), "assert_eq!(a, b)");

    let almost = lower_intrinsic(
        "assert_almost_eq",
        &["x".to_string(), "y".to_string(), "tol".to_string()],
    )
    .expect("assert_almost_eq lowers");
    assert!(render_expr(&almost.expr).contains("assert_almost_eq failed"));

    let gt = lower_intrinsic("assert_gt", &["l".to_string(), "r".to_string()])
        .expect("assert_gt lowers");
    assert!(render_expr(&gt.expr).contains("assert_gt failed"));
}

#[test]
pub(crate) fn collections_bridge_helpers_are_not_intrinsics() {
    for retired in [
        "new_set",
        "set_from_list",
        "set_add",
        "set_contains",
        "set_remove",
        "set_len",
        "set_union",
        "set_intersection",
    ] {
        assert!(
            lower_intrinsic(retired, &["value".to_string()]).is_none(),
            "{retired} should lower through _sifr.collections private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn lowers_encoding_intrinsics_via_registry() {
    let enc_result = lower_intrinsic("str_encode_utf8_result", &["s".to_string()])
        .expect("str_encode_utf8_result");
    assert!(render_expr(&enc_result.expr).contains("::sifr_runtime::encoding::encode_bytes"));
    assert_eq!(
        enc_result.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::SifrRuntime)
    );
    assert!(enc_result
        .additional_required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::EncodingRs));

    let enc_with_codec = lower_intrinsic(
        "str_encode_utf8_result_with_encoding",
        &["s".to_string(), "codec".to_string()],
    )
    .expect("str_encode_utf8_result_with_encoding");
    assert!(render_expr(&enc_with_codec.expr).contains("::sifr_runtime::encoding::encode_bytes"));

    let dec = lower_intrinsic("decode_utf8", &["vals".to_string()]).expect("decode_utf8");
    assert!(render_expr(&dec.expr).contains("::sifr_runtime::encoding::decode_text"));

    let dec_with_codec = lower_intrinsic(
        "decode_utf8_with_encoding",
        &["vals".to_string(), "codec".to_string()],
    )
    .expect("decode_utf8_with_encoding");
    assert!(render_expr(&dec_with_codec.expr).contains("::sifr_runtime::encoding::decode_text"));

    assert!(
        lower_intrinsic(
            "process_output_text",
            &[
                "program".to_string(),
                "args".to_string(),
                "env".to_string(),
                "cwd".to_string(),
                "has_cwd".to_string(),
                "stdin".to_string(),
                "has_stdin".to_string(),
                "encoding".to_string(),
            ],
        )
        .is_none(),
        "process_output_text should lower through _sifr.process private declarations"
    );

    let with_size =
        lower_intrinsic("bytes_with_size", &["n".to_string()]).expect("bytes_with_size");
    let with_size_rendered = render_expr(&with_size.expr);
    assert!(with_size_rendered.contains("non-negative size"));
    assert!(with_size_rendered.contains("Ok::<Vec<u8>, ValueError>"));

    let from_ints =
        lower_intrinsic("bytes_from_ints", &["vals".to_string()]).expect("bytes_from_ints");
    let from_ints_rendered = render_expr(&from_ints.expr);
    assert!(from_ints_rendered.contains("byte out of range at index"));
    assert!(from_ints_rendered.contains("Ok::<Vec<u8>, ValueError>"));

    assert!(lower_intrinsic("bytes_to_hex_strict", &["vals".to_string()]).is_none());

    let from_hex = lower_intrinsic("bytes_from_hex", &["hex".to_string()]).expect("bytes_from_hex");
    let from_hex_rendered = render_expr(&from_hex.expr);
    assert!(from_hex_rendered.contains("invalid hex character"));
    assert!(from_hex_rendered.contains("Ok::<Vec<u8>, ParseError>"));
}

#[test]
pub(crate) fn time_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for (name, args) in [
        ("time_now", &[][..]),
        ("time_format", &["secs", "mask"][..]),
        ("perf_counter", &[][..]),
        ("sleep", &["secs"][..]),
        ("monotonic", &[][..]),
        ("strptime", &["s", "f"][..]),
        ("gmtime", &["ts"][..]),
        ("localtime", &["ts"][..]),
        ("_strptime_intrinsic", &["s", "f"][..]),
        ("_gmtime_intrinsic", &["ts"][..]),
        ("_localtime_intrinsic", &["ts"][..]),
        ("time_strptime", &["s", "f"][..]),
        ("time_gmtime", &[][..]),
        ("time_localtime", &[][..]),
    ] {
        let args = args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        assert!(
            lower_intrinsic(name, &args).is_none(),
            "{name} should lower through _sifr.time private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn random_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for (name, args) in [
        ("random_int", &["1", "9"][..]),
        ("random_float", &[][..]),
        ("random_choice", &["items"][..]),
        ("random_uniform", &["0.0", "1.0"][..]),
        ("random_shuffle", &["vals"][..]),
        ("random_sample", &["vals", "3"][..]),
        ("random_randrange", &["0", "10", "1"][..]),
        ("random_gauss", &["0.0", "1.0"][..]),
        ("random_module_state_words", &[][..]),
        ("random_module_state_index", &[][..]),
        ("random_module_state_gauss_next", &[][..]),
        ("random_module_set_state", &["words", "index", "gauss"][..]),
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
pub(crate) fn re_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for (name, args) in [
        ("re_match", &["pat", "txt"][..]),
        ("re_find", &["pat", "txt"][..]),
        ("re_replace", &["pat", "repl", "txt"][..]),
        ("re_findall", &["pat", "txt"][..]),
        ("re_split", &["pat", "txt"][..]),
        ("re_find_start", &["pat", "txt"][..]),
        ("re_find_end", &["pat", "txt"][..]),
        ("re_match_flags", &["pat", "txt", "flags"][..]),
        ("re_find_flags", &["pat", "txt", "flags"][..]),
        ("re_replace_flags", &["pat", "repl", "txt", "flags"][..]),
        ("re_findall_flags", &["pat", "txt", "flags"][..]),
        ("re_split_flags", &["pat", "txt", "flags"][..]),
    ] {
        let args = args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        assert!(
            lower_intrinsic(name, &args).is_none(),
            "{name} should lower through _sifr.regex private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn url_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for (name, args) in [
        ("url_parse", &["value"][..]),
        (
            "url_build",
            &["scheme", "host", "path", "query", "port"][..],
        ),
        ("url_percent_encode", &["value"][..]),
        ("url_percent_decode", &["value"][..]),
        ("url_percent_encode_bytes", &["value"][..]),
        ("url_percent_decode_bytes", &["value"][..]),
        ("url_normalize_path", &["path"][..]),
        ("url_query_parse", &["query"][..]),
        ("url_query_build", &["pairs"][..]),
    ] {
        let args = args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        assert!(
            lower_intrinsic(name, &args).is_none(),
            "{name} should lower through _sifr.url private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn core_hash_intrinsics_are_owned_by_compiled_stdlib_declarations() {
    for name in ["sha256", "sha256_bytes", "md5", "md5_bytes"] {
        assert!(
            lower_intrinsic(name, &["payload".to_string()]).is_none(),
            "{name} should lower through _sifr.crypto private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn lowers_platform_intrinsics_via_registry() {
    assert!(lower_intrinsic("platform_system", &[]).is_none());
    assert!(lower_intrinsic("platform_arch", &[]).is_none());
    assert!(lower_intrinsic("platform_node", &[]).is_none());
    assert!(lower_intrinsic("platform_release", &[]).is_none());
    assert!(lower_intrinsic("platform_version", &[]).is_none());
    assert!(lower_intrinsic("platform_processor", &[]).is_none());
}
