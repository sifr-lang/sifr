use super::*;
use crate::{render_expr, RustExpr};

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
    let args = rendered_args
        .iter()
        .cloned()
        .map(|arg| parse_test_arg(&arg))
        .collect::<Vec<_>>();
    super::lower_intrinsic(name, &args)
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
pub(crate) fn lowers_runtime_diagnostic_intrinsic_with_observability_metadata() {
    let lowered = lower_intrinsic(
        "runtime_emit_diagnostic",
        &[
            "level".to_string(),
            "target".to_string(),
            "name".to_string(),
            "message".to_string(),
        ],
    )
    .expect("runtime_emit_diagnostic should lower");

    assert_eq!(lowered.required_feature, None);
    assert!(lowered
        .additional_required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::Metrics));
    assert!(lowered
        .additional_required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::Tracing));
    let rendered = render_expr(&lowered.expr);
    assert!(rendered.contains("tracing::event!"));
    assert!(rendered.contains("metrics::counter!"));
    assert!(rendered.contains("target: \"sifr.runtime\""));
    assert!(rendered.contains("diagnostic_target = __sifr_diagnostic_target"));
    assert!(rendered.contains("\"sifr.runtime.diagnostic.emitted\""));
    assert!(rendered.contains("\"sifr.runtime.diagnostic.rejected\""));
    assert!(rendered.contains("\"reason\" => \"unsupported_level\""));
    assert!(rendered.contains("\"surface\" => \"runtime\""));
    assert!(rendered.contains("tracing::Level::INFO"));
    assert!(rendered.contains("DiagnosticError::new"));
    assert!(rendered.contains("unsupported diagnostic level"));
}

#[test]
pub(crate) fn runtime_diagnostic_intrinsic_rejects_wrong_arity() {
    assert!(lower_intrinsic(
        "runtime_emit_diagnostic",
        &["level".to_string(), "target".to_string()],
    )
    .is_none());
}

#[test]
pub(crate) fn runtime_module_dependency_metadata_includes_observability_facades() {
    let deps = sifr_stdlib_manifest::try_generated_cargo_dependencies(
        &std::collections::HashSet::from(["sifr.runtime".to_string()]),
        &std::collections::HashSet::new(),
    )
    .expect("source-tree sysroot dependencies should resolve");

    assert_eq!(deps.len(), 3);
    assert!(deps[0].starts_with("sifr_stdlib = "));
    assert!(deps[0].contains("default-features = false"));
    assert!(deps[0].contains("features = [\"runtime-observability\"]"));
    assert_eq!(
        &deps[1..],
        [
            "metrics = \"0.24.6\"".to_string(),
            "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }"
                .to_string()
        ]
    );
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
pub(crate) fn lowers_env_intrinsics_via_registry() {
    let get = lower_intrinsic("env_get", &["key".to_string()]).expect("env_get should lower");
    assert!(render_expr(&get.expr).contains("std::env::var"));

    let set = lower_intrinsic("env_set", &["k".to_string(), "v".to_string()])
        .expect("env_set should lower");
    assert!(render_expr(&set.expr).contains("std::env::set_var"));

    let keys = lower_intrinsic("env_keys", &[]).expect("env_keys should lower");
    assert!(render_expr(&keys.expr).contains("std::env::vars_os()"));
}

#[test]
pub(crate) fn lowers_os_intrinsics_via_registry() {
    let run =
        lower_intrinsic("run_command", &["cmd".to_string()]).expect("run_command should lower");
    assert!(render_expr(&run.expr).contains("std::process::Command::new(\"sh\".to_string())"));
    assert!(render_expr(&run.expr).contains(".arg(\"-c\".to_string())"));

    let args = lower_intrinsic("get_args", &[]).expect("get_args should lower");
    assert_eq!(
        render_expr(&args.expr),
        "std::env::args().collect::<Vec<String>>()"
    );

    let pid = lower_intrinsic("getpid", &[]).expect("getpid should lower");
    assert_eq!(render_expr(&pid.expr), "std::process::id() as i64");

    let cpus = lower_intrinsic("cpu_count", &[]).expect("cpu_count should lower");
    assert!(render_expr(&cpus.expr).contains("available_parallelism"));

    let which = lower_intrinsic("which", &["tool".to_string()]).expect("which should lower");
    assert!(render_expr(&which.expr).contains("std::env::var(\"PATH\".to_string())"));

    let disk = lower_intrinsic("disk_usage", &["path".to_string()]).expect("disk_usage lowers");
    assert!(render_expr(&disk.expr).contains("std::process::Command::new(\"df\".to_string())"));
    assert!(render_expr(&disk.expr).contains("split_whitespace().collect::<Vec<&str>>()"));

    let sep = lower_intrinsic("os_sep", &[]).expect("os_sep lowers");
    assert_eq!(
        render_expr(&sep.expr),
        "std::path::MAIN_SEPARATOR.to_string()"
    );

    let linesep = lower_intrinsic("os_linesep", &[]).expect("os_linesep lowers");
    assert!(render_expr(&linesep.expr).contains("cfg!(target_os = \"windows\")"));

    let name = lower_intrinsic("os_name", &[]).expect("os_name lowers");
    assert!(render_expr(&name.expr).contains("\"posix\".to_string()"));
}

#[test]
pub(crate) fn lowers_signal_intrinsics_via_registry() {
    let ctrl_c = lower_intrinsic("signal_ctrl_c", &[]).expect("signal_ctrl_c");
    assert_eq!(
        ctrl_c.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Tokio)
    );
    let ctrl_c_rendered = render_expr(&ctrl_c.expr);
    assert!(ctrl_c_rendered.contains("tokio::signal::ctrl_c().await"));
    assert!(ctrl_c_rendered.contains("SIGINT"));

    let terminate = lower_intrinsic("signal_terminate", &[]).expect("signal_terminate");
    assert_eq!(
        terminate.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Tokio)
    );
    let terminate_rendered = render_expr(&terminate.expr);
    assert!(terminate_rendered.contains("#[cfg(unix)]"));
    assert!(terminate_rendered.contains("#[cfg(not(unix))]"));
    assert!(terminate_rendered.contains("SignalKind::terminate"));
    assert!(terminate_rendered.contains("SIGTERM is unsupported"));

    let shutdown = lower_intrinsic("signal_shutdown", &[]).expect("signal_shutdown");
    assert_eq!(
        shutdown.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Tokio)
    );
    let shutdown_rendered = render_expr(&shutdown.expr);
    assert!(shutdown_rendered.contains("#[cfg(unix)]"));
    assert!(shutdown_rendered.contains("#[cfg(not(unix))]"));
    assert!(shutdown_rendered.contains("tokio::select!"));
    assert!(shutdown_rendered.contains("tokio::signal::ctrl_c().await"));
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
pub(crate) fn lowers_collections_counter_intrinsics_via_registry() {
    let from_list =
        lower_intrinsic("counter_from_list", &["vals".to_string()]).expect("counter_from_list");
    assert!(render_expr(&from_list.expr).contains("HashMap::<String, i64>"));
    assert!(render_expr(&from_list.expr).contains("let __items = vals;"));

    let get = lower_intrinsic("counter_get", &["data".to_string(), "k".to_string()])
        .expect("counter_get");
    assert!(render_expr(&get.expr).contains("serde_json::from_str"));
    assert!(render_expr(&get.expr).contains("let __counter_json = data;"));
    assert!(!render_expr(&get.expr).contains("from_str(&(data))"));

    let incr = lower_intrinsic("counter_increment", &["data".to_string(), "k".to_string()])
        .expect("counter_increment");
    assert!(render_expr(&incr.expr).contains("or_insert(0) += 1"));
    assert!(render_expr(&incr.expr).contains("let __key = k;"));
    assert!(render_expr(&incr.expr).contains("__key.to_string()"));

    for retired in [
        "new_set",
        "set_from_list",
        "set_add",
        "set_contains",
        "set_remove",
        "set_len",
        "set_union",
        "set_intersection",
        "defaultdict_new",
        "defaultdict_get",
        "defaultdict_set",
    ] {
        assert!(
            lower_intrinsic(retired, &["value".to_string()]).is_none(),
            "{retired} should lower through _sifr.collections private Rust interop declarations"
        );
    }
}

#[test]
pub(crate) fn lowers_bytes_intrinsics_via_registry() {
    for retired in ["encode_utf8", "bytes_to_hex"] {
        assert!(
            lower_intrinsic(retired, &["value".to_string()]).is_none(),
            "{retired} should lower through _sifr.bytes private Rust interop declarations"
        );
    }

    let enc_result = lower_intrinsic("str_encode_utf8_result", &["s".to_string()])
        .expect("str_encode_utf8_result");
    assert!(render_expr(&enc_result.expr).contains("sifr_runtime::encoding::encode_bytes"));
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
    assert!(render_expr(&enc_with_codec.expr).contains("sifr_runtime::encoding::encode_bytes"));

    let dec = lower_intrinsic("decode_utf8", &["vals".to_string()]).expect("decode_utf8");
    assert!(render_expr(&dec.expr).contains("sifr_runtime::encoding::decode_text"));

    let dec_with_codec = lower_intrinsic(
        "decode_utf8_with_encoding",
        &["vals".to_string(), "codec".to_string()],
    )
    .expect("decode_utf8_with_encoding");
    assert!(render_expr(&dec_with_codec.expr).contains("sifr_runtime::encoding::decode_text"));

    let process_text = lower_intrinsic(
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
    .expect("process_output_text");
    assert!(render_expr(&process_text.expr).contains("sifr_runtime::encoding::decode_text"));
    assert!(process_text
        .additional_required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::EncodingRs));

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

    let to_hex_strict =
        lower_intrinsic("bytes_to_hex_strict", &["vals".to_string()]).expect("bytes_to_hex_strict");
    assert!(render_expr(&to_hex_strict.expr).contains("{:02x}"));
    assert!(!render_expr(&to_hex_strict.expr).contains("Ok"));

    let from_hex = lower_intrinsic("bytes_from_hex", &["hex".to_string()]).expect("bytes_from_hex");
    let from_hex_rendered = render_expr(&from_hex.expr);
    assert!(from_hex_rendered.contains("invalid hex character"));
    assert!(from_hex_rendered.contains("Ok::<Vec<u8>, ParseError>"));
}

#[test]
pub(crate) fn lowers_time_intrinsics_via_registry() {
    let now = lower_intrinsic("time_now", &[]).expect("time_now");
    assert!(render_expr(&now.expr).contains("SystemTime::now()"));

    let sleep = lower_intrinsic("sleep", &["0.1".to_string()]).expect("sleep");
    assert!(render_expr(&sleep.expr).contains("is_finite()"));
    assert!(render_expr(&sleep.expr).contains("Duration::from_nanos"));

    let fmt = lower_intrinsic("time_format", &["secs".to_string(), "mask".to_string()])
        .expect("time_format");
    assert_eq!(
        fmt.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Chrono)
    );
    assert!(render_expr(&fmt.expr).contains("DateTime::from_timestamp"));

    let perf = lower_intrinsic("perf_counter", &[]).expect("perf_counter");
    assert!(render_expr(&perf.expr).contains("SystemTime::now()"));

    let mono = lower_intrinsic("monotonic", &[]).expect("monotonic");
    assert!(render_expr(&mono.expr).contains("SystemTime::now()"));

    let parse = lower_intrinsic("strptime", &["s".to_string(), "f".to_string()]).expect("strptime");
    assert_eq!(
        parse.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Chrono)
    );
    assert!(render_expr(&parse.expr).contains("NaiveDateTime::parse_from_str"));

    let gmt = lower_intrinsic("gmtime", &["ts".to_string()]).expect("gmtime");
    assert_eq!(
        gmt.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Chrono)
    );
    assert!(render_expr(&gmt.expr).contains("chrono::DateTime::<chrono::Utc>::from_timestamp"));

    let local = lower_intrinsic("localtime", &["ts".to_string()]).expect("localtime");
    assert_eq!(
        local.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Chrono)
    );
    assert!(render_expr(&local.expr).contains("with_timezone(&chrono::Local)"));

    let parse_alias = lower_intrinsic("_strptime_intrinsic", &["s".to_string(), "f".to_string()])
        .expect("_strptime_intrinsic");
    assert_eq!(
        parse_alias.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Chrono)
    );
    assert!(render_expr(&parse_alias.expr).contains("NaiveDateTime::parse_from_str"));

    let parsed_parts = lower_intrinsic("time_strptime", &["s".to_string(), "f".to_string()])
        .expect("time_strptime");
    assert_eq!(
        parsed_parts.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Chrono)
    );
    assert!(render_expr(&parsed_parts.expr).contains("Result<Vec<i64>, ValueError>"));

    let gmtime_parts = lower_intrinsic("time_gmtime", &[]).expect("time_gmtime");
    assert_eq!(
        gmtime_parts.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Chrono)
    );
    assert!(render_expr(&gmtime_parts.expr).contains("Utc::now().naive_utc()"));

    let localtime_parts = lower_intrinsic("time_localtime", &[]).expect("time_localtime");
    assert_eq!(
        localtime_parts.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Chrono)
    );
    assert!(render_expr(&localtime_parts.expr).contains("Local::now().naive_local()"));
}

#[test]
pub(crate) fn lowers_random_intrinsics_via_registry() {
    let rint =
        lower_intrinsic("random_int", &["1".to_string(), "9".to_string()]).expect("random_int");
    assert_eq!(
        rint.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Rand)
    );
    assert!(render_expr(&rint.expr).contains("rand::RngExt::random_range"));

    let rfloat = lower_intrinsic("random_float", &[]).expect("random_float");
    assert_eq!(
        rfloat.required_feature,
        Some(sifr_stdlib_manifest::StdlibFeature::Rand)
    );
    assert!(render_expr(&rfloat.expr).contains("rand::random::<f64>()"));

    let choice = lower_intrinsic("random_choice", &["items".to_string()]).expect("random_choice");
    assert!(render_expr(&choice.expr).contains("items.len()"));

    let uniform = lower_intrinsic("random_uniform", &["0.0".to_string(), "1.0".to_string()])
        .expect("random_uniform");
    assert!(render_expr(&uniform.expr).contains("rand::random::<f64>()"));

    let shuffle = lower_intrinsic("random_shuffle", &["vals".to_string()]).expect("random_shuffle");
    assert!(render_expr(&shuffle.expr).contains("SliceRandom::shuffle"));

    let sample = lower_intrinsic("random_sample", &["vals".to_string(), "3".to_string()])
        .expect("random_sample");
    assert!(render_expr(&sample.expr).contains("IndexedRandom::sample"));

    let randrange = lower_intrinsic(
        "random_randrange",
        &["0".to_string(), "10".to_string(), "1".to_string()],
    )
    .expect("random_randrange");
    assert!(render_expr(&randrange.expr).contains("randrange: step must not be zero"));

    let gauss = lower_intrinsic("random_gauss", &["0.0".to_string(), "1.0".to_string()])
        .expect("random_gauss");
    assert!(gauss
        .additional_required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::RandDistr));
    assert!(render_expr(&gauss.expr).contains("rand_distr"));

    let state_words =
        lower_intrinsic("random_module_state_words", &[]).expect("random_module_state_words");
    assert_eq!(state_words.required_feature, None);
    assert!(render_expr(&state_words.expr).contains("__SIFR_RANDOM_MODULE_STATE"));
    assert!(render_expr(&state_words.expr).contains(".words"));

    let state_index =
        lower_intrinsic("random_module_state_index", &[]).expect("random_module_state_index");
    assert_eq!(state_index.required_feature, None);
    assert!(render_expr(&state_index.expr).contains(".index"));

    let state_gauss = lower_intrinsic("random_module_state_gauss_next", &[])
        .expect("random_module_state_gauss_next");
    assert_eq!(state_gauss.required_feature, None);
    assert!(render_expr(&state_gauss.expr).contains(".gauss_next"));

    let set_state = lower_intrinsic(
        "random_module_set_state",
        &[
            "words".to_string(),
            "index".to_string(),
            "gauss".to_string(),
        ],
    )
    .expect("random_module_set_state");
    assert_eq!(set_state.required_feature, None);
    assert!(render_expr(&set_state.expr).contains("length 624"));
    assert!(render_expr(&set_state.expr).contains("random module state index"));
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
