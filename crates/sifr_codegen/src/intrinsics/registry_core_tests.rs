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
pub(crate) fn lowers_math_intrinsics_via_registry() {
    let lowered = lower_intrinsic("sqrt", &["x".to_string()]).expect("sqrt should lower");
    assert_eq!(render_expr(&lowered.expr), "(x).sqrt()");

    let lowered = lower_intrinsic("pow_val", &["a".to_string(), "b".to_string()])
        .expect("pow_val should lower");
    assert_eq!(render_expr(&lowered.expr), "(a).powf(b)");

    let lowered =
        lower_intrinsic("atan2", &["y".to_string(), "x".to_string()]).expect("atan2 should lower");
    assert_eq!(render_expr(&lowered.expr), "(y).atan2(x)");

    let lowered = lower_intrinsic("round_val", &["n".to_string()]).expect("round_val should lower");
    assert_eq!(render_expr(&lowered.expr), "(n).round() as i64");

    let lowered = lower_intrinsic("floor", &["n".to_string()]).expect("floor should lower");
    assert_eq!(render_expr(&lowered.expr), "(n).floor() as i64");

    let lowered = lower_intrinsic("ceil", &["n".to_string()]).expect("ceil should lower");
    assert_eq!(render_expr(&lowered.expr), "(n).ceil() as i64");

    let lowered = lower_intrinsic("isfinite", &["f".to_string()]).expect("isfinite should lower");
    assert_eq!(render_expr(&lowered.expr), "(f).is_finite()");

    let lowered = lower_intrinsic("isqrt", &["v".to_string()]).expect("isqrt should lower");
    assert_eq!(render_expr(&lowered.expr), "((v) as f64).sqrt() as i64");
}

#[test]
pub(crate) fn lowers_json_intrinsics_with_dependency_metadata() {
    let loads =
        lower_intrinsic("json_loads", &["payload".to_string()]).expect("json_loads should lower");
    assert_eq!(
        loads.required_feature,
        Some(sifr_stdlib::StdlibFeature::SerdeJson)
    );
    assert!(loads
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::SifrRuntime));
    let loads_rendered = render_expr(&loads.expr);
    assert!(loads_rendered.contains("serde_json::from_str"));
    assert!(loads_rendered.contains("validate_json_integer_digit_limits"));

    let validate = lower_intrinsic(
        "json_validate_integer_digit_limits",
        &["payload".to_string()],
    )
    .expect("json_validate_integer_digit_limits should lower");
    assert_eq!(validate.required_feature, None);
    assert!(validate
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::SifrRuntime));
    assert!(render_expr(&validate.expr).contains("JsonLimitError"));

    let dumps =
        lower_intrinsic("json_dumps", &["value".to_string()]).expect("json_dumps should lower");
    assert_eq!(
        dumps.required_feature,
        Some(sifr_stdlib::StdlibFeature::SerdeJson)
    );
    assert_eq!(
        render_expr(&dumps.expr),
        "serde_json::to_string(&value).unwrap_or_default()"
    );
}

#[test]
pub(crate) fn lowers_runtime_diagnostic_intrinsic_with_tracing_metadata() {
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
        .contains(&sifr_stdlib::StdlibFeature::Tracing));
    let rendered = render_expr(&lowered.expr);
    assert!(rendered.contains("tracing::event!"));
    assert!(rendered.contains("target: \"sifr.runtime\""));
    assert!(rendered.contains("diagnostic_target = __sifr_diagnostic_target"));
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
pub(crate) fn runtime_module_dependency_metadata_includes_tracing_only() {
    let deps = sifr_stdlib::generated_cargo_dependencies(
        &std::collections::HashSet::from(["sifr.runtime".to_string()]),
        &std::collections::HashSet::new(),
    );

    assert_eq!(
        deps,
        vec![
            "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }"
                .to_string()
        ]
    );
}

#[test]
pub(crate) fn lowers_unicode_intrinsics_with_dependency_metadata() {
    let normalized = lower_intrinsic(
        "unicode_normalize",
        &["form".to_string(), "text".to_string()],
    )
    .expect("unicode_normalize should lower");
    assert_eq!(
        normalized.required_feature,
        Some(sifr_stdlib::StdlibFeature::SifrRuntime)
    );
    assert!(normalized
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::UnicodeNames));
    assert!(normalized
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::UnicodeNormalization));
    assert!(normalized
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::UnicodeSegmentation));
    let rendered = render_expr(&normalized.expr);
    assert!(rendered.contains("sifr_runtime::unicode::normalize"));
    assert!(rendered.contains("UnicodeDataError"));

    let folded = lower_intrinsic("unicode_case_fold", &["text".to_string()]).expect("case fold");
    assert_eq!(
        render_expr(&folded.expr),
        "sifr_runtime::unicode::case_fold(&text)"
    );

    let graphemes =
        lower_intrinsic("unicode_graphemes", &["text".to_string()]).expect("graphemes lower");
    assert_eq!(
        graphemes.required_feature,
        Some(sifr_stdlib::StdlibFeature::SifrRuntime)
    );
    assert!(graphemes
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::UnicodeSegmentation));
    assert!(render_expr(&graphemes.expr).contains("sifr_runtime::unicode::graphemes"));
}

#[test]
pub(crate) fn lowers_i18n_intrinsics_with_dependency_metadata() {
    let canonical = lower_intrinsic("i18n_locale_canonicalize", &["locale".to_string()])
        .expect("locale canonicalize should lower");
    assert_eq!(
        canonical.required_feature,
        Some(sifr_stdlib::StdlibFeature::SifrRuntime)
    );
    assert!(canonical
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::IcuLocale));
    assert!(canonical
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::IcuDatetime));
    let canonical_rendered = render_expr(&canonical.expr);
    assert!(canonical_rendered.contains("sifr_runtime::i18n::canonicalize_locale"));
    assert!(canonical_rendered.contains("LocaleIdError"));

    let formatted = lower_intrinsic(
        "i18n_format_datetime",
        &[
            "locale".to_string(),
            "style".to_string(),
            "2025".to_string(),
            "1".to_string(),
            "15".to_string(),
            "16".to_string(),
            "9".to_string(),
            "35".to_string(),
        ],
    )
    .expect("date/time formatter should lower");
    let formatted_rendered = render_expr(&formatted.expr);
    assert!(formatted_rendered.contains("sifr_runtime::i18n::format_datetime"));
    assert!(formatted_rendered.contains("FormatError"));

    let plural = lower_intrinsic(
        "i18n_plural_category",
        &[
            "locale".to_string(),
            "rule_type".to_string(),
            "value".to_string(),
        ],
    )
    .expect("plural category should lower");
    assert!(render_expr(&plural.expr).contains("PluralRulesError"));

    let host = lower_intrinsic("i18n_host_locale", &[]).expect("host locale should lower");
    assert_eq!(render_expr(&host.expr), "sifr_runtime::i18n::host_locale()");

    let lookup = lower_intrinsic(
        "i18n_mo_lookup_context_plural",
        &[
            "catalog".to_string(),
            "context".to_string(),
            "singular".to_string(),
            "plural".to_string(),
            "2".to_string(),
        ],
    )
    .expect("catalog plural lookup should lower");
    assert_eq!(
        lookup.required_feature,
        Some(sifr_stdlib::StdlibFeature::SifrRuntime)
    );
    assert!(lookup
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::IcuPlurals));
    let lookup_rendered = render_expr(&lookup.expr);
    assert!(lookup_rendered.contains("sifr_runtime::i18n::mo_lookup_context_plural"));
    assert!(lookup_rendered.contains("CatalogError"));

    let load_file =
        lower_intrinsic("i18n_mo_load_file", &["path".to_string()]).expect("load should lower");
    assert!(render_expr(&load_file.expr).contains("read_mo_catalog_file"));
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
        Some(sifr_stdlib::StdlibFeature::Tokio)
    );
    let ctrl_c_rendered = render_expr(&ctrl_c.expr);
    assert!(ctrl_c_rendered.contains("tokio::signal::ctrl_c().await"));
    assert!(ctrl_c_rendered.contains("SIGINT"));

    let terminate = lower_intrinsic("signal_terminate", &[]).expect("signal_terminate");
    assert_eq!(
        terminate.required_feature,
        Some(sifr_stdlib::StdlibFeature::Tokio)
    );
    let terminate_rendered = render_expr(&terminate.expr);
    assert!(terminate_rendered.contains("SignalKind::terminate"));
    assert!(terminate_rendered.contains("SIGTERM is unsupported"));

    let shutdown = lower_intrinsic("signal_shutdown", &[]).expect("signal_shutdown");
    assert_eq!(
        shutdown.required_feature,
        Some(sifr_stdlib::StdlibFeature::Tokio)
    );
    let shutdown_rendered = render_expr(&shutdown.expr);
    assert!(shutdown_rendered.contains("tokio::select!"));
    assert!(shutdown_rendered.contains("tokio::signal::ctrl_c().await"));
}

#[test]
pub(crate) fn lowers_io_intrinsics_via_registry() {
    let read = lower_intrinsic("read_text", &["path".to_string()]).expect("read_text lowers");
    assert!(render_expr(&read.expr).contains("std::fs::read_to_string"));

    let write = lower_intrinsic("write_text", &["p".to_string(), "c".to_string()])
        .expect("write_text lowers");
    assert!(render_expr(&write.expr).contains("std::fs::write"));

    let exists = lower_intrinsic("exists", &["p".to_string()]).expect("exists lowers");
    assert!(render_expr(&exists.expr).contains("Path::new"));

    let gettempdir = lower_intrinsic("gettempdir", &[]).expect("gettempdir lowers");
    assert_eq!(
        render_expr(&gettempdir.expr),
        "std::env::temp_dir().display().to_string()"
    );

    let append = lower_intrinsic("append_text", &["p".to_string(), "c".to_string()])
        .expect("append_text lowers");
    assert!(render_expr(&append.expr).contains("OpenOptions::new().append(true)"));

    let walk = lower_intrinsic("walk_dir", &["root".to_string()]).expect("walk_dir lowers");
    assert!(render_expr(&walk.expr).contains("__stack.pop()"));
}

#[test]
pub(crate) fn lowers_pathlib_intrinsics_via_registry() {
    let touch = lower_intrinsic("touch", &["p".to_string()]).expect("touch lowers");
    assert!(render_expr(&touch.expr).contains("OpenOptions::new().create(true)"));

    let resolve = lower_intrinsic("resolve_path", &["p".to_string()]).expect("resolve_path lowers");
    assert!(render_expr(&resolve.expr).contains("std::fs::canonicalize"));

    let iterdir = lower_intrinsic("iterdir", &["p".to_string()]).expect("iterdir lowers");
    assert!(render_expr(&iterdir.expr).contains("std::fs::read_dir"));

    let glob = lower_intrinsic("glob_pattern", &["dir".to_string(), "pat".to_string()])
        .expect("glob_pattern lowers");
    assert_eq!(
        glob.required_feature,
        Some(sifr_stdlib::StdlibFeature::Regex)
    );
    assert!(render_expr(&glob.expr).contains("regex::Regex::new"));
    assert!(render_expr(&glob.expr).contains("__re.is_match(&__name)"));

    let rglob = lower_intrinsic("rglob_pattern", &["dir".to_string(), "pat".to_string()])
        .expect("rglob_pattern lowers");
    assert_eq!(
        rglob.required_feature,
        Some(sifr_stdlib::StdlibFeature::Regex)
    );
    assert!(render_expr(&rglob.expr).contains("__stack.pop()"));
    assert!(render_expr(&rglob.expr).contains("__re.is_match(&__name)"));
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
pub(crate) fn lowers_collections_set_intrinsics_via_registry() {
    let new_set = lower_intrinsic("new_set", &[]).expect("new_set lowers");
    assert_eq!(render_expr(&new_set.expr), "Vec::<i64>::new()");

    let add =
        lower_intrinsic("set_add", &["s".to_string(), "v".to_string()]).expect("set_add lowers");
    assert!(render_expr(&add.expr).contains("s.push(v)"));

    let inter = lower_intrinsic("set_intersection", &["a".to_string(), "b".to_string()])
        .expect("set_intersection lowers");
    assert!(render_expr(&inter.expr).contains("collect::<Vec<i64>>()"));
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

    let dd_set = lower_intrinsic(
        "defaultdict_set",
        &["dd".to_string(), "key".to_string(), "v".to_string()],
    )
    .expect("defaultdict_set");
    assert!(render_expr(&dd_set.expr).contains("serde_json::json!"));
    assert!(render_expr(&dd_set.expr).contains("let __defaultdict_json = dd;"));
    assert!(render_expr(&dd_set.expr).contains("let __key = key;"));
    assert!(render_expr(&dd_set.expr).contains("__key.to_string()"));

    let dd_get = lower_intrinsic("defaultdict_get", &["dd".to_string(), "key".to_string()])
        .expect("defaultdict_get");
    assert!(render_expr(&dd_get.expr).contains("let __defaultdict_json = dd;"));
    assert!(render_expr(&dd_get.expr).contains("let __key = key;"));
    assert!(!render_expr(&dd_get.expr).contains("from_str(&(dd))"));
}

#[test]
pub(crate) fn lowers_bytes_intrinsics_via_registry() {
    let enc = lower_intrinsic("encode_utf8", &["s".to_string()]).expect("encode_utf8");
    assert!(render_expr(&enc.expr).contains("as_bytes()"));

    let enc_result = lower_intrinsic("str_encode_utf8_result", &["s".to_string()])
        .expect("str_encode_utf8_result");
    assert!(render_expr(&enc_result.expr).contains("sifr_runtime::encoding::encode_bytes"));
    assert_eq!(
        enc_result.required_feature,
        Some(sifr_stdlib::StdlibFeature::SifrRuntime)
    );
    assert!(enc_result
        .additional_required_features
        .contains(&sifr_stdlib::StdlibFeature::EncodingRs));

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
        .contains(&sifr_stdlib::StdlibFeature::EncodingRs));

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

    let to_hex = lower_intrinsic("bytes_to_hex", &["vals".to_string()]).expect("bytes_to_hex");
    assert!(render_expr(&to_hex.expr).contains("{:02x}"));
    assert!(render_expr(&to_hex.expr).contains("Ok"));

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
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&fmt.expr).contains("DateTime::from_timestamp"));

    let perf = lower_intrinsic("perf_counter", &[]).expect("perf_counter");
    assert!(render_expr(&perf.expr).contains("SystemTime::now()"));

    let mono = lower_intrinsic("monotonic", &[]).expect("monotonic");
    assert!(render_expr(&mono.expr).contains("SystemTime::now()"));

    let parse = lower_intrinsic("strptime", &["s".to_string(), "f".to_string()]).expect("strptime");
    assert_eq!(
        parse.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&parse.expr).contains("NaiveDateTime::parse_from_str"));

    let gmt = lower_intrinsic("gmtime", &["ts".to_string()]).expect("gmtime");
    assert_eq!(
        gmt.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&gmt.expr).contains("chrono::DateTime::<chrono::Utc>::from_timestamp"));

    let local = lower_intrinsic("localtime", &["ts".to_string()]).expect("localtime");
    assert_eq!(
        local.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&local.expr).contains("with_timezone(&chrono::Local)"));

    let parse_alias = lower_intrinsic("_strptime_intrinsic", &["s".to_string(), "f".to_string()])
        .expect("_strptime_intrinsic");
    assert_eq!(
        parse_alias.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&parse_alias.expr).contains("NaiveDateTime::parse_from_str"));

    let parsed_parts = lower_intrinsic("time_strptime", &["s".to_string(), "f".to_string()])
        .expect("time_strptime");
    assert_eq!(
        parsed_parts.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&parsed_parts.expr).contains("Result<Vec<i64>, ValueError>"));

    let gmtime_parts = lower_intrinsic("time_gmtime", &[]).expect("time_gmtime");
    assert_eq!(
        gmtime_parts.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&gmtime_parts.expr).contains("Utc::now().naive_utc()"));

    let localtime_parts = lower_intrinsic("time_localtime", &[]).expect("time_localtime");
    assert_eq!(
        localtime_parts.required_feature,
        Some(sifr_stdlib::StdlibFeature::Chrono)
    );
    assert!(render_expr(&localtime_parts.expr).contains("Local::now().naive_local()"));
}

#[test]
pub(crate) fn lowers_random_intrinsics_via_registry() {
    let rint =
        lower_intrinsic("random_int", &["1".to_string(), "9".to_string()]).expect("random_int");
    assert_eq!(
        rint.required_feature,
        Some(sifr_stdlib::StdlibFeature::Rand)
    );
    assert!(render_expr(&rint.expr).contains("rand::RngExt::random_range"));

    let rfloat = lower_intrinsic("random_float", &[]).expect("random_float");
    assert_eq!(
        rfloat.required_feature,
        Some(sifr_stdlib::StdlibFeature::Rand)
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
        .contains(&sifr_stdlib::StdlibFeature::RandDistr));
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
pub(crate) fn lowers_re_intrinsics_via_registry() {
    let m = lower_intrinsic("re_match", &["pat".to_string(), "txt".to_string()]).expect("re_match");
    assert_eq!(m.required_feature, Some(sifr_stdlib::StdlibFeature::Regex));
    assert!(render_expr(&m.expr).contains("is_match"));

    let f = lower_intrinsic("re_find", &["pat".to_string(), "txt".to_string()]).expect("re_find");
    assert!(render_expr(&f.expr).contains("re.find"));

    let rep = lower_intrinsic(
        "re_replace",
        &["pat".to_string(), "repl".to_string(), "txt".to_string()],
    )
    .expect("re_replace");
    assert!(render_expr(&rep.expr).contains("replace_all"));

    let all =
        lower_intrinsic("re_findall", &["pat".to_string(), "txt".to_string()]).expect("re_findall");
    assert!(render_expr(&all.expr).contains("find_iter"));

    let split =
        lower_intrinsic("re_split", &["pat".to_string(), "txt".to_string()]).expect("re_split");
    assert!(render_expr(&split.expr).contains("re.split"));

    let s = lower_intrinsic("re_find_start", &["pat".to_string(), "txt".to_string()])
        .expect("re_find_start");
    assert!(render_expr(&s.expr).contains("m.start()"));

    let e = lower_intrinsic("re_find_end", &["pat".to_string(), "txt".to_string()])
        .expect("re_find_end");
    assert!(render_expr(&e.expr).contains("m.end()"));

    let mf = lower_intrinsic(
        "re_match_flags",
        &["pat".to_string(), "txt".to_string(), "flags".to_string()],
    )
    .expect("re_match_flags");
    assert!(render_expr(&mf.expr).contains("__flags_val"));

    let rf = lower_intrinsic(
        "re_replace_flags",
        &[
            "pat".to_string(),
            "repl".to_string(),
            "txt".to_string(),
            "flags".to_string(),
        ],
    )
    .expect("re_replace_flags");
    assert_eq!(rf.required_feature, Some(sifr_stdlib::StdlibFeature::Regex));
    assert!(render_expr(&rf.expr).contains("replace_all"));
}

#[test]
pub(crate) fn lowers_hash_intrinsics_via_registry() {
    let sha = lower_intrinsic("sha256", &["payload".to_string()]).expect("sha256");
    assert_eq!(sha.required_feature, Some(sifr_stdlib::StdlibFeature::Sha2));
    assert!(render_expr(&sha.expr).contains("<sha2::Sha256 as sha2::Digest>::digest"));
    assert!(render_expr(&sha.expr).contains(".as_bytes()"));

    let md5 = lower_intrinsic("md5", &["payload".to_string()]).expect("md5");
    assert_eq!(md5.required_feature, Some(sifr_stdlib::StdlibFeature::Md5));
    assert!(render_expr(&md5.expr).contains("md5::compute"));
    assert!(render_expr(&md5.expr).contains(".as_bytes()"));

    let sha_bytes =
        lower_intrinsic("sha256_bytes", &["payload".to_string()]).expect("sha256_bytes");
    assert_eq!(
        sha_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Sha2)
    );
    assert!(render_expr(&sha_bytes.expr).contains("to_vec"));

    let md5_bytes = lower_intrinsic("md5_bytes", &["payload".to_string()]).expect("md5_bytes");
    assert_eq!(
        md5_bytes.required_feature,
        Some(sifr_stdlib::StdlibFeature::Md5)
    );
    assert!(render_expr(&md5_bytes.expr).contains("md5::compute"));
    assert!(render_expr(&md5_bytes.expr).contains(".0"));
    assert!(render_expr(&md5_bytes.expr).contains("to_vec"));
}

#[test]
pub(crate) fn lowers_platform_intrinsics_via_registry() {
    let system = lower_intrinsic("platform_system", &[]).expect("platform_system");
    assert!(render_expr(&system.expr).contains("target_os = \"windows\""));
    assert!(render_expr(&system.expr).contains("\"Windows\""));
    assert!(render_expr(&system.expr).contains("\"Darwin\""));
    assert!(render_expr(&system.expr).contains("\"Linux\""));

    let arch = lower_intrinsic("platform_arch", &[]).expect("platform_arch");
    assert_eq!(
        render_expr(&arch.expr),
        "std::env::consts::ARCH.to_string()"
    );

    let node = lower_intrinsic("platform_node", &[]).expect("platform_node");
    assert!(render_expr(&node.expr).contains("std::env::var(\"HOSTNAME\")"));
    assert!(render_expr(&node.expr).contains("COMPUTERNAME"));
    assert!(render_expr(&node.expr).contains("localhost"));

    let rel = lower_intrinsic("platform_release", &[]).expect("platform_release");
    assert!(render_expr(&rel.expr).contains("Command::new(\"uname\").arg(\"-r\")"));
    assert!(render_expr(&rel.expr).contains("std::env::consts::OS.to_string()"));

    let ver = lower_intrinsic("platform_version", &[]).expect("platform_version");
    assert!(render_expr(&ver.expr).contains("Command::new(\"uname\").arg(\"-v\")"));
    assert!(render_expr(&ver.expr).contains("std::env::consts::OS.to_string()"));

    let proc = lower_intrinsic("platform_processor", &[]).expect("platform_processor");
    assert_eq!(
        render_expr(&proc.expr),
        "std::env::consts::ARCH.to_string()"
    );
}
