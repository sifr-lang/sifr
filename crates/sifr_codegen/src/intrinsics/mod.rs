//! Intrinsic registry and dispatch for incremental IR rollout.

mod base32;
mod base64;
mod bytes;
mod calendar;
mod collections;
mod datetime;
mod digest_format;
mod env;
mod file_handles;
mod gzip;
mod hash;
mod hashlib;
mod html;
mod io;
mod json;
mod logging;
mod math;
mod os;
mod pathlib;
mod platform;
mod random;
mod re;
mod subprocess;
mod sys;
mod test;
mod time;
mod toml;
mod uuid;
mod zipfile;

use crate::RustExpr;

pub(crate) struct LoweredIntrinsic {
    pub(crate) expr: RustExpr,
    pub(crate) required_crate: Option<&'static str>,
    pub(crate) additional_required_crates: &'static [&'static str],
}

fn additional_required_crates(name: &str) -> &'static [&'static str] {
    match name {
        // random_gauss uses rand_distr::Normal in addition to rand::rng.
        "random_gauss" => &["rand_distr"],
        "json_loads"
        | "json_validate_integer_digit_limits"
        | "json_dumps_value_exact"
        | "json_dumps_value_web"
        | "json_dumps_value_string_ints" => &["sifr_runtime"],
        _ => &[],
    }
}

pub(crate) fn lower_intrinsic(name: &str, args: &[RustExpr]) -> Option<LoweredIntrinsic> {
    lower_intrinsic_rendered(name, args)
}

fn lower_intrinsic_rendered(name: &str, args: &[RustExpr]) -> Option<LoweredIntrinsic> {
    let (expr, required_crate) = match name {
        "sqrt" => (math::lower_sqrt(args), None),
        "floor" => (math::lower_floor(args), None),
        "ceil" => (math::lower_ceil(args), None),
        "abs_val" => (math::lower_abs_val(args), None),
        "log" => (math::lower_log(args), None),
        "cbrt" => (math::lower_cbrt(args), None),
        "exp2" => (math::lower_exp2(args), None),
        "sin" => (math::lower_sin(args), None),
        "cos" => (math::lower_cos(args), None),
        "tan" => (math::lower_tan(args), None),
        "pow_val" => (math::lower_pow_val(args), None),
        "min_val" => (math::lower_min_val(args), None),
        "max_val" => (math::lower_max_val(args), None),
        "round_val" => (math::lower_round_val(args), None),
        "asin" => (math::lower_asin(args), None),
        "acos" => (math::lower_acos(args), None),
        "atan" => (math::lower_atan(args), None),
        "atan2" => (math::lower_atan2(args), None),
        "sinh" => (math::lower_sinh(args), None),
        "cosh" => (math::lower_cosh(args), None),
        "tanh" => (math::lower_tanh(args), None),
        "log10" => (math::lower_log10(args), None),
        "log2" => (math::lower_log2(args), None),
        "degrees" => (math::lower_degrees(args), None),
        "radians" => (math::lower_radians(args), None),
        "isnan" => (math::lower_isnan(args), None),
        "isinf" => (math::lower_isinf(args), None),
        "trunc" => (math::lower_trunc(args), None),
        "copysign" => (math::lower_copysign(args), None),
        "signbit" => (math::lower_signbit(args), None),
        "fmod" => (math::lower_fmod(args), None),
        "hypot" => (math::lower_hypot(args), None),
        "fma" => (math::lower_fma(args), None),
        "fmax" => (math::lower_fmax(args), None),
        "fmin" => (math::lower_fmin(args), None),
        "exp" => (math::lower_exp(args), None),
        "expm1" => (math::lower_expm1(args), None),
        "log1p" => (math::lower_log1p(args), None),
        "fabs" => (math::lower_fabs(args), None),
        "isfinite" => (math::lower_isfinite(args), None),
        "isnormal" => (math::lower_isnormal(args), None),
        "issubnormal" => (math::lower_issubnormal(args), None),
        "acosh" => (math::lower_acosh(args), None),
        "asinh" => (math::lower_asinh(args), None),
        "atanh" => (math::lower_atanh(args), None),
        "isqrt" => (math::lower_isqrt(args), None),
        "remainder" => (math::lower_remainder(args), None),
        "dist" => (math::lower_dist(args), None),
        "fsum" => (math::lower_fsum(args), None),
        "sumprod" => (math::lower_sumprod(args), None),
        "erf" => (math::lower_erf(args), None),
        "erfc" => (math::lower_erfc(args), None),
        "gamma" => (math::lower_gamma(args), None),
        "lgamma" => (math::lower_lgamma(args), None),
        "frexp" => (math::lower_frexp(args), None),
        "ldexp" => (math::lower_ldexp(args), None),
        "modf" => (math::lower_modf(args), None),
        "nextafter" => (math::lower_nextafter(args), None),
        "ulp" => (math::lower_ulp(args), None),
        "env_get" => (env::lower_env_get(args), None),
        "env_set" => (env::lower_env_set(args), None),
        "env_unset" => (env::lower_env_unset(args), None),
        "env_keys" => (env::lower_env_keys(args), None),
        "env_values" => (env::lower_env_values(args), None),
        "env_items" => (env::lower_env_items(args), None),
        "run_command" => (os::lower_run_command(args), None),
        "get_args" => (os::lower_get_args(args), None),
        "chdir" => (os::lower_chdir(args), None),
        "getpid" => (os::lower_getpid(args), None),
        "cpu_count" => (os::lower_cpu_count(args), None),
        "stat_size" => (os::lower_stat_size(args), None),
        "which" => (os::lower_which(args), None),
        "disk_usage" => (os::lower_disk_usage(args), None),
        "os_sep" => (os::lower_os_sep(args), None),
        "os_linesep" => (os::lower_os_linesep(args), None),
        "os_name" => (os::lower_os_name(args), None),
        "touch" => (pathlib::lower_touch(args), None),
        "resolve_path" => (pathlib::lower_resolve_path(args), None),
        "iterdir" => (pathlib::lower_iterdir(args), None),
        "glob_pattern" => (pathlib::lower_glob_pattern(args), Some("regex")),
        "rglob_pattern" => (pathlib::lower_rglob_pattern(args), Some("regex")),
        "read_text" => (io::lower_read_text(args), None),
        "write_text" => (io::lower_write_text(args), None),
        "exists" => (io::lower_exists(args), None),
        "read_lines" => (io::lower_read_lines(args), None),
        "append_text" => (io::lower_append_text(args), None),
        "getcwd" => (io::lower_getcwd(args), None),
        "listdir" => (io::lower_listdir(args), None),
        "mkdir" => (io::lower_mkdir(args), None),
        "rmdir" => (io::lower_rmdir(args), None),
        "remove_file" => (io::lower_remove_file(args), None),
        "rename" => (io::lower_rename(args), None),
        "is_file" => (io::lower_is_file(args), None),
        "is_dir" => (io::lower_is_dir(args), None),
        "copy_file" => (io::lower_copy_file(args), None),
        "walk_dir" => (io::lower_walk_dir(args), None),
        "rmdir_all" => (io::lower_rmdir_all(args), None),
        "gettempdir" => (io::lower_gettempdir(args), None),
        "makedirs" => (io::lower_makedirs(args), None),
        "builtin_open" => (file_handles::lower_builtin_open(args), None),
        "open_file" => (file_handles::lower_open_file(args), None),
        "file_read" => (file_handles::lower_file_read(args), None),
        "file_write" => (file_handles::lower_file_write(args), None),
        "file_readline" => (file_handles::lower_file_readline(args), None),
        "file_readlines" => (file_handles::lower_file_readlines(args), None),
        "file_close" => (file_handles::lower_file_close(args), None),
        "file_read_bytes" => (file_handles::lower_file_read_bytes(args), None),
        "file_write_bytes" => (file_handles::lower_file_write_bytes(args), None),
        "json_loads" => (json::lower_json_loads(args), Some("serde_json")),
        "json_validate_integer_digit_limits" => {
            (json::lower_json_validate_integer_digit_limits(args), None)
        }
        "json_dumps" => (json::lower_json_dumps(args), Some("serde_json")),
        "json_dumps_value" => (json::lower_json_dumps_value(args), Some("serde_json")),
        "json_dumps_value_exact" => (json::lower_json_dumps_value_exact(args), Some("serde_json")),
        "json_dumps_value_web" => (json::lower_json_dumps_value_web(args), Some("serde_json")),
        "json_dumps_value_string_ints" => (
            json::lower_json_dumps_value_string_ints(args),
            Some("serde_json"),
        ),
        "assert_eq" => (test::lower_assert_eq(args), None),
        "assert_ne" => (test::lower_assert_ne(args), None),
        "assert_true" => (test::lower_assert_true(args), None),
        "assert_false" => (test::lower_assert_false(args), None),
        "assert_almost_eq" => (test::lower_assert_almost_eq(args), None),
        "assert_gt" => (test::lower_assert_gt(args), None),
        "assert_lt" => (test::lower_assert_lt(args), None),
        "new_set" => (collections::lower_new_set(args), None),
        "set_from_list" => (collections::lower_set_from_list(args), None),
        "set_add" => (collections::lower_set_add(args), None),
        "set_contains" => (collections::lower_set_contains(args), None),
        "set_remove" => (collections::lower_set_remove(args), None),
        "set_len" => (collections::lower_set_len(args), None),
        "set_union" => (collections::lower_set_union(args), None),
        "set_intersection" => (collections::lower_set_intersection(args), None),
        "counter_from_list" => (
            collections::lower_counter_from_list(args),
            Some("serde_json"),
        ),
        "counter_get" => (collections::lower_counter_get(args), Some("serde_json")),
        "counter_most_common" => (
            collections::lower_counter_most_common(args),
            Some("serde_json"),
        ),
        "counter_total" => (collections::lower_counter_total(args), Some("serde_json")),
        "counter_values" => (collections::lower_counter_values(args), Some("serde_json")),
        "counter_keys" => (collections::lower_counter_keys(args), Some("serde_json")),
        "counter_items" => (collections::lower_counter_items(args), Some("serde_json")),
        "counter_increment" => (
            collections::lower_counter_increment(args),
            Some("serde_json"),
        ),
        "defaultdict_new" => (collections::lower_defaultdict_new(args), None),
        "defaultdict_get" => (collections::lower_defaultdict_get(args), Some("serde_json")),
        "defaultdict_set" => (collections::lower_defaultdict_set(args), Some("serde_json")),
        "encode_utf8" => (bytes::lower_encode_utf8(args), None),
        "str_encode_utf8_result" => (bytes::lower_str_encode_utf8_result(args), None),
        "str_encode_utf8_result_with_encoding" => (
            bytes::lower_str_encode_utf8_result_with_encoding(args),
            None,
        ),
        "decode_utf8" => (bytes::lower_decode_utf8(args), None),
        "decode_utf8_with_encoding" => (bytes::lower_decode_utf8_with_encoding(args), None),
        "bytes_to_hex" => (bytes::lower_bytes_to_hex(args), None),
        "bytes_to_hex_strict" => (bytes::lower_bytes_to_hex_strict(args), None),
        "bytes_from_hex" => (bytes::lower_bytes_from_hex(args), None),
        "bytes_with_size" => (bytes::lower_bytes_with_size(args), None),
        "bytes_from_ints" => (bytes::lower_bytes_from_ints(args), None),
        "time_now" => (time::lower_time_now(args), None),
        "sleep" => (time::lower_sleep(args), None),
        "time_format" => (time::lower_time_format(args), Some("chrono")),
        "perf_counter" => (time::lower_perf_counter(args), None),
        "monotonic" => (time::lower_monotonic(args), None),
        "strptime" => (time::lower_strptime(args), Some("chrono")),
        "gmtime" => (time::lower_gmtime(args), Some("chrono")),
        "localtime" => (time::lower_localtime(args), Some("chrono")),
        "_strptime_intrinsic" => (time::lower_strptime(args), Some("chrono")),
        "_gmtime_intrinsic" => (time::lower_gmtime(args), Some("chrono")),
        "_localtime_intrinsic" => (time::lower_localtime(args), Some("chrono")),
        "time_strptime" => (time::lower_time_strptime_parts(args), Some("chrono")),
        "time_gmtime" => (time::lower_time_gmtime_parts(args), Some("chrono")),
        "time_localtime" => (time::lower_time_localtime_parts(args), Some("chrono")),
        "random_int" => (random::lower_random_int(args), Some("rand")),
        "random_float" => (random::lower_random_float(args), Some("rand")),
        "random_choice" => (random::lower_random_choice(args), Some("rand")),
        "random_uniform" => (random::lower_random_uniform(args), Some("rand")),
        "random_shuffle" => (random::lower_random_shuffle(args), Some("rand")),
        "random_sample" => (random::lower_random_sample(args), Some("rand")),
        "random_randrange" => (random::lower_random_randrange(args), Some("rand")),
        "random_gauss" => (random::lower_random_gauss(args), Some("rand")),
        "random_module_state_words" => (random::lower_random_module_state_words(args), None),
        "random_module_state_index" => (random::lower_random_module_state_index(args), None),
        "random_module_state_gauss_next" => {
            (random::lower_random_module_state_gauss_next(args), None)
        }
        "random_module_set_state" => (random::lower_random_module_set_state(args), None),
        "re_match" => (re::lower_re_match(args), Some("regex")),
        "re_find" => (re::lower_re_find(args), Some("regex")),
        "re_replace" => (re::lower_re_replace(args), Some("regex")),
        "re_findall" => (re::lower_re_findall(args), Some("regex")),
        "re_split" => (re::lower_re_split(args), Some("regex")),
        "re_find_start" => (re::lower_re_find_start(args), Some("regex")),
        "re_find_end" => (re::lower_re_find_end(args), Some("regex")),
        "re_match_flags" => (re::lower_re_match_flags(args), Some("regex")),
        "re_find_flags" => (re::lower_re_find_flags(args), Some("regex")),
        "re_replace_flags" => (re::lower_re_replace_flags(args), Some("regex")),
        "re_findall_flags" => (re::lower_re_findall_flags(args), Some("regex")),
        "re_split_flags" => (re::lower_re_split_flags(args), Some("regex")),
        "sha256" => (hash::lower_sha256(args), Some("sha2")),
        "md5" => (hash::lower_md5(args), Some("md5")),
        "sha256_bytes" => (hashlib::lower_sha256_bytes(args), Some("sha2")),
        "md5_bytes" => (hashlib::lower_md5_bytes(args), Some("md5")),
        "platform_system" => (platform::lower_platform_system(args), None),
        "platform_arch" => (platform::lower_platform_arch(args), None),
        "platform_node" => (platform::lower_platform_node(args), None),
        "platform_release" => (platform::lower_platform_release(args), None),
        "platform_version" => (platform::lower_platform_version(args), None),
        "platform_processor" => (platform::lower_platform_processor(args), None),
        "uuid4" => (uuid::lower_uuid4(args), Some("rand")),
        "uuid3_text" => (uuid::lower_uuid3(args), Some("uuid")),
        "uuid5_text" => (uuid::lower_uuid5(args), Some("uuid")),
        "toml_parse" => (toml::lower_toml_parse(args), Some("toml")),
        "datetime_now" => (datetime::lower_datetime_now(args), Some("chrono")),
        "datetime_now_struct" => (datetime::lower_datetime_now_struct(args), Some("chrono")),
        "datetime_format" => (datetime::lower_datetime_format(args), None),
        "datetime_from_timestamp" => (
            datetime::lower_datetime_from_timestamp(args),
            Some("chrono"),
        ),
        "sys_exit" => (sys::lower_sys_exit(args), None),
        "sys_version" => (sys::lower_sys_version(args), None),
        "sys_platform" => (sys::lower_sys_platform(args), None),
        "sys_maxsize" => (sys::lower_sys_maxsize(args), None),
        "subprocess_run" => (subprocess::lower_subprocess_run(args), None),
        "subprocess_run_with_input" => (subprocess::lower_subprocess_run_with_input(args), None),
        "subprocess_run_structured" => (subprocess::lower_subprocess_run_structured(args), None),
        "html_escape" => (html::lower_html_escape(args), None),
        "html_unescape" => (html::lower_html_unescape(args), None),
        "calendar_isleap" => (calendar::lower_calendar_isleap(args), None),
        "calendar_weekday" => (calendar::lower_calendar_weekday(args), None),
        "calendar_monthrange" => (calendar::lower_calendar_monthrange(args), None),
        "gzip_compress" => (gzip::lower_gzip_compress(args), Some("flate2")),
        "gzip_decompress" => (gzip::lower_gzip_decompress(args), Some("flate2")),
        "zip_create" => (zipfile::lower_zip_create(args), Some("zip")),
        "zip_add_file" => (zipfile::lower_zip_add_file(args), Some("zip")),
        "zip_add_file_bytes" => (zipfile::lower_zip_add_file_bytes(args), Some("zip")),
        "zip_read_file" => (zipfile::lower_zip_read_file(args), Some("zip")),
        "zip_read_file_bytes" => (zipfile::lower_zip_read_file_bytes(args), Some("zip")),
        "zip_namelist" => (zipfile::lower_zip_namelist(args), Some("zip")),
        "base64_encode" => (base64::lower_base64_encode(args), Some("base64")),
        "base64_decode" => (base64::lower_base64_decode(args), Some("base64")),
        "base64_encode_bytes" => (base64::lower_base64_encode_bytes(args), Some("base64")),
        "base64_decode_bytes" => (base64::lower_base64_decode_bytes(args), Some("base64")),
        "base64_encode_opts" => (base64::lower_base64_encode_opts(args), Some("base64")),
        "base64_decode_opts" => (base64::lower_base64_decode_opts(args), Some("base64")),
        "urlsafe_b64encode" => (base64::lower_urlsafe_b64encode(args), Some("base64")),
        "urlsafe_b64decode" => (base64::lower_urlsafe_b64decode(args), Some("base64")),
        "urlsafe_b64encode_bytes" => (base64::lower_urlsafe_b64encode_bytes(args), Some("base64")),
        "urlsafe_b64decode_bytes" => (base64::lower_urlsafe_b64decode_bytes(args), Some("base64")),
        "b32encode" => (base32::lower_b32encode(args), None),
        "b32decode" => (base32::lower_b32decode(args), None),
        "b32hexencode" => (base32::lower_b32hexencode(args), None),
        "b32hexdecode" => (base32::lower_b32hexdecode(args), None),
        "sha1" => (hashlib::lower_sha1(args), Some("sha1")),
        "sha1_bytes" => (hashlib::lower_sha1_bytes(args), Some("sha1")),
        "sha512" => (hashlib::lower_sha512(args), Some("sha2")),
        "sha512_bytes" => (hashlib::lower_sha512_bytes(args), Some("sha2")),
        "sha224" => (hashlib::lower_sha224(args), Some("sha2")),
        "sha224_bytes" => (hashlib::lower_sha224_bytes(args), Some("sha2")),
        "sha384" => (hashlib::lower_sha384(args), Some("sha2")),
        "sha384_bytes" => (hashlib::lower_sha384_bytes(args), Some("sha2")),
        "blake2b" => (hashlib::lower_blake2b(args), Some("blake2")),
        "blake2b_bytes" => (hashlib::lower_blake2b_bytes(args), Some("blake2")),
        "blake2s" => (hashlib::lower_blake2s(args), Some("blake2")),
        "blake2s_bytes" => (hashlib::lower_blake2s_bytes(args), Some("blake2")),
        "set_global_level" => (logging::lower_set_global_level(args), None),
        "get_global_level" => (logging::lower_get_global_level(args), None),
        _ => return None,
    };

    Some(LoweredIntrinsic {
        expr: expr?,
        required_crate,
        additional_required_crates: additional_required_crates(name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_expr;

    fn parse_test_arg(rendered: &str) -> RustExpr {
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

    fn lower_intrinsic(name: &str, rendered_args: &[String]) -> Option<LoweredIntrinsic> {
        let args = rendered_args
            .iter()
            .cloned()
            .map(|arg| parse_test_arg(&arg))
            .collect::<Vec<_>>();
        super::lower_intrinsic(name, &args)
    }

    #[test]
    fn lowers_math_intrinsics_via_registry() {
        let lowered = lower_intrinsic("sqrt", &["x".to_string()]).expect("sqrt should lower");
        assert_eq!(render_expr(&lowered.expr), "(x).sqrt()");

        let lowered = lower_intrinsic("pow_val", &["a".to_string(), "b".to_string()])
            .expect("pow_val should lower");
        assert_eq!(render_expr(&lowered.expr), "(a).powf(b)");

        let lowered = lower_intrinsic("atan2", &["y".to_string(), "x".to_string()])
            .expect("atan2 should lower");
        assert_eq!(render_expr(&lowered.expr), "(y).atan2(x)");

        let lowered =
            lower_intrinsic("round_val", &["n".to_string()]).expect("round_val should lower");
        assert_eq!(render_expr(&lowered.expr), "(n).round() as i64");

        let lowered = lower_intrinsic("floor", &["n".to_string()]).expect("floor should lower");
        assert_eq!(render_expr(&lowered.expr), "(n).floor() as i64");

        let lowered = lower_intrinsic("ceil", &["n".to_string()]).expect("ceil should lower");
        assert_eq!(render_expr(&lowered.expr), "(n).ceil() as i64");

        let lowered =
            lower_intrinsic("isfinite", &["f".to_string()]).expect("isfinite should lower");
        assert_eq!(render_expr(&lowered.expr), "(f).is_finite()");

        let lowered = lower_intrinsic("isqrt", &["v".to_string()]).expect("isqrt should lower");
        assert_eq!(render_expr(&lowered.expr), "((v) as f64).sqrt() as i64");
    }

    #[test]
    fn lowers_json_intrinsics_with_dependency_metadata() {
        let loads = lower_intrinsic("json_loads", &["payload".to_string()])
            .expect("json_loads should lower");
        assert_eq!(loads.required_crate, Some("serde_json"));
        assert!(loads.additional_required_crates.contains(&"sifr_runtime"));
        let loads_rendered = render_expr(&loads.expr);
        assert!(loads_rendered.contains("serde_json::from_str"));
        assert!(loads_rendered.contains("validate_json_integer_digit_limits"));

        let validate = lower_intrinsic(
            "json_validate_integer_digit_limits",
            &["payload".to_string()],
        )
        .expect("json_validate_integer_digit_limits should lower");
        assert_eq!(validate.required_crate, None);
        assert!(validate
            .additional_required_crates
            .contains(&"sifr_runtime"));
        assert!(render_expr(&validate.expr).contains("JsonLimitError"));

        let dumps =
            lower_intrinsic("json_dumps", &["value".to_string()]).expect("json_dumps should lower");
        assert_eq!(dumps.required_crate, Some("serde_json"));
        assert_eq!(
            render_expr(&dumps.expr),
            "serde_json::to_string(&value).unwrap_or_default()"
        );
    }

    #[test]
    fn lowers_env_intrinsics_via_registry() {
        let get = lower_intrinsic("env_get", &["key".to_string()]).expect("env_get should lower");
        assert!(render_expr(&get.expr).contains("std::env::var"));

        let set = lower_intrinsic("env_set", &["k".to_string(), "v".to_string()])
            .expect("env_set should lower");
        assert!(render_expr(&set.expr).contains("std::env::set_var"));

        let keys = lower_intrinsic("env_keys", &[]).expect("env_keys should lower");
        assert!(render_expr(&keys.expr).contains("std::env::vars_os()"));
    }

    #[test]
    fn lowers_os_intrinsics_via_registry() {
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
    fn lowers_io_intrinsics_via_registry() {
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
    fn lowers_pathlib_intrinsics_via_registry() {
        let touch = lower_intrinsic("touch", &["p".to_string()]).expect("touch lowers");
        assert!(render_expr(&touch.expr).contains("OpenOptions::new().create(true)"));

        let resolve =
            lower_intrinsic("resolve_path", &["p".to_string()]).expect("resolve_path lowers");
        assert!(render_expr(&resolve.expr).contains("std::fs::canonicalize"));

        let iterdir = lower_intrinsic("iterdir", &["p".to_string()]).expect("iterdir lowers");
        assert!(render_expr(&iterdir.expr).contains("std::fs::read_dir"));

        let glob = lower_intrinsic("glob_pattern", &["dir".to_string(), "pat".to_string()])
            .expect("glob_pattern lowers");
        assert_eq!(glob.required_crate, Some("regex"));
        assert!(render_expr(&glob.expr).contains("regex::Regex::new"));
        assert!(render_expr(&glob.expr).contains("__re.is_match(&__name)"));

        let rglob = lower_intrinsic("rglob_pattern", &["dir".to_string(), "pat".to_string()])
            .expect("rglob_pattern lowers");
        assert_eq!(rglob.required_crate, Some("regex"));
        assert!(render_expr(&rglob.expr).contains("__stack.pop()"));
        assert!(render_expr(&rglob.expr).contains("__re.is_match(&__name)"));
    }

    #[test]
    fn lowers_test_intrinsics_via_registry() {
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
    fn lowers_collections_set_intrinsics_via_registry() {
        let new_set = lower_intrinsic("new_set", &[]).expect("new_set lowers");
        assert_eq!(render_expr(&new_set.expr), "Vec::<i64>::new()");

        let add = lower_intrinsic("set_add", &["s".to_string(), "v".to_string()])
            .expect("set_add lowers");
        assert!(render_expr(&add.expr).contains("s.push(v)"));

        let inter = lower_intrinsic("set_intersection", &["a".to_string(), "b".to_string()])
            .expect("set_intersection lowers");
        assert!(render_expr(&inter.expr).contains("collect::<Vec<i64>>()"));
    }

    #[test]
    fn lowers_collections_counter_intrinsics_via_registry() {
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
    fn lowers_bytes_intrinsics_via_registry() {
        let enc = lower_intrinsic("encode_utf8", &["s".to_string()]).expect("encode_utf8");
        assert!(render_expr(&enc.expr).contains("as_bytes()"));

        let enc_result = lower_intrinsic("str_encode_utf8_result", &["s".to_string()])
            .expect("str_encode_utf8_result");
        assert!(render_expr(&enc_result.expr).contains("Ok"));

        let enc_with_codec = lower_intrinsic(
            "str_encode_utf8_result_with_encoding",
            &["s".to_string(), "codec".to_string()],
        )
        .expect("str_encode_utf8_result_with_encoding");
        assert!(render_expr(&enc_with_codec.expr).contains("UTF-8 encoding"));

        let dec = lower_intrinsic("decode_utf8", &["vals".to_string()]).expect("decode_utf8");
        assert!(render_expr(&dec.expr).contains("String::from_utf8"));

        let dec_with_codec = lower_intrinsic(
            "decode_utf8_with_encoding",
            &["vals".to_string(), "codec".to_string()],
        )
        .expect("decode_utf8_with_encoding");
        assert!(render_expr(&dec_with_codec.expr).contains("UTF-8 encoding"));

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

        let to_hex_strict = lower_intrinsic("bytes_to_hex_strict", &["vals".to_string()])
            .expect("bytes_to_hex_strict");
        assert!(render_expr(&to_hex_strict.expr).contains("{:02x}"));
        assert!(!render_expr(&to_hex_strict.expr).contains("Ok"));

        let from_hex =
            lower_intrinsic("bytes_from_hex", &["hex".to_string()]).expect("bytes_from_hex");
        let from_hex_rendered = render_expr(&from_hex.expr);
        assert!(from_hex_rendered.contains("invalid hex character"));
        assert!(from_hex_rendered.contains("Ok::<Vec<u8>, ParseError>"));
    }

    #[test]
    fn lowers_time_intrinsics_via_registry() {
        let now = lower_intrinsic("time_now", &[]).expect("time_now");
        assert!(render_expr(&now.expr).contains("SystemTime::now()"));

        let sleep = lower_intrinsic("sleep", &["0.1".to_string()]).expect("sleep");
        assert!(render_expr(&sleep.expr).contains("is_finite()"));
        assert!(render_expr(&sleep.expr).contains("Duration::from_nanos"));

        let fmt = lower_intrinsic("time_format", &["secs".to_string(), "mask".to_string()])
            .expect("time_format");
        assert_eq!(fmt.required_crate, Some("chrono"));
        assert!(render_expr(&fmt.expr).contains("DateTime::from_timestamp"));

        let perf = lower_intrinsic("perf_counter", &[]).expect("perf_counter");
        assert!(render_expr(&perf.expr).contains("SystemTime::now()"));

        let mono = lower_intrinsic("monotonic", &[]).expect("monotonic");
        assert!(render_expr(&mono.expr).contains("SystemTime::now()"));

        let parse =
            lower_intrinsic("strptime", &["s".to_string(), "f".to_string()]).expect("strptime");
        assert_eq!(parse.required_crate, Some("chrono"));
        assert!(render_expr(&parse.expr).contains("NaiveDateTime::parse_from_str"));

        let gmt = lower_intrinsic("gmtime", &["ts".to_string()]).expect("gmtime");
        assert_eq!(gmt.required_crate, Some("chrono"));
        assert!(render_expr(&gmt.expr).contains("chrono::DateTime::<chrono::Utc>::from_timestamp"));

        let local = lower_intrinsic("localtime", &["ts".to_string()]).expect("localtime");
        assert_eq!(local.required_crate, Some("chrono"));
        assert!(render_expr(&local.expr).contains("with_timezone(&chrono::Local)"));

        let parse_alias =
            lower_intrinsic("_strptime_intrinsic", &["s".to_string(), "f".to_string()])
                .expect("_strptime_intrinsic");
        assert_eq!(parse_alias.required_crate, Some("chrono"));
        assert!(render_expr(&parse_alias.expr).contains("NaiveDateTime::parse_from_str"));

        let parsed_parts = lower_intrinsic("time_strptime", &["s".to_string(), "f".to_string()])
            .expect("time_strptime");
        assert_eq!(parsed_parts.required_crate, Some("chrono"));
        assert!(render_expr(&parsed_parts.expr).contains("Result<Vec<i64>, ValueError>"));

        let gmtime_parts = lower_intrinsic("time_gmtime", &[]).expect("time_gmtime");
        assert_eq!(gmtime_parts.required_crate, Some("chrono"));
        assert!(render_expr(&gmtime_parts.expr).contains("Utc::now().naive_utc()"));

        let localtime_parts = lower_intrinsic("time_localtime", &[]).expect("time_localtime");
        assert_eq!(localtime_parts.required_crate, Some("chrono"));
        assert!(render_expr(&localtime_parts.expr).contains("Local::now().naive_local()"));
    }

    #[test]
    fn lowers_random_intrinsics_via_registry() {
        let rint =
            lower_intrinsic("random_int", &["1".to_string(), "9".to_string()]).expect("random_int");
        assert_eq!(rint.required_crate, Some("rand"));
        assert!(render_expr(&rint.expr).contains("rand::RngExt::random_range"));

        let rfloat = lower_intrinsic("random_float", &[]).expect("random_float");
        assert_eq!(rfloat.required_crate, Some("rand"));
        assert!(render_expr(&rfloat.expr).contains("rand::random::<f64>()"));

        let choice =
            lower_intrinsic("random_choice", &["items".to_string()]).expect("random_choice");
        assert!(render_expr(&choice.expr).contains("items.len()"));

        let uniform = lower_intrinsic("random_uniform", &["0.0".to_string(), "1.0".to_string()])
            .expect("random_uniform");
        assert!(render_expr(&uniform.expr).contains("rand::random::<f64>()"));

        let shuffle =
            lower_intrinsic("random_shuffle", &["vals".to_string()]).expect("random_shuffle");
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
        assert!(gauss.additional_required_crates.contains(&"rand_distr"));
        assert!(render_expr(&gauss.expr).contains("rand_distr"));

        let state_words =
            lower_intrinsic("random_module_state_words", &[]).expect("random_module_state_words");
        assert_eq!(state_words.required_crate, None);
        assert!(render_expr(&state_words.expr).contains("__SIFR_RANDOM_MODULE_STATE"));
        assert!(render_expr(&state_words.expr).contains(".words"));

        let state_index =
            lower_intrinsic("random_module_state_index", &[]).expect("random_module_state_index");
        assert_eq!(state_index.required_crate, None);
        assert!(render_expr(&state_index.expr).contains(".index"));

        let state_gauss = lower_intrinsic("random_module_state_gauss_next", &[])
            .expect("random_module_state_gauss_next");
        assert_eq!(state_gauss.required_crate, None);
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
        assert_eq!(set_state.required_crate, None);
        assert!(render_expr(&set_state.expr).contains("length 624"));
        assert!(render_expr(&set_state.expr).contains("random module state index"));
    }

    #[test]
    fn lowers_re_intrinsics_via_registry() {
        let m =
            lower_intrinsic("re_match", &["pat".to_string(), "txt".to_string()]).expect("re_match");
        assert_eq!(m.required_crate, Some("regex"));
        assert!(render_expr(&m.expr).contains("is_match"));

        let f =
            lower_intrinsic("re_find", &["pat".to_string(), "txt".to_string()]).expect("re_find");
        assert!(render_expr(&f.expr).contains("re.find"));

        let rep = lower_intrinsic(
            "re_replace",
            &["pat".to_string(), "repl".to_string(), "txt".to_string()],
        )
        .expect("re_replace");
        assert!(render_expr(&rep.expr).contains("replace_all"));

        let all = lower_intrinsic("re_findall", &["pat".to_string(), "txt".to_string()])
            .expect("re_findall");
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
        assert_eq!(rf.required_crate, Some("regex"));
        assert!(render_expr(&rf.expr).contains("replace_all"));
    }

    #[test]
    fn lowers_hash_intrinsics_via_registry() {
        let sha = lower_intrinsic("sha256", &["payload".to_string()]).expect("sha256");
        assert_eq!(sha.required_crate, Some("sha2"));
        assert!(render_expr(&sha.expr).contains("<sha2::Sha256 as sha2::Digest>::digest"));
        assert!(render_expr(&sha.expr).contains(".as_bytes()"));

        let md5 = lower_intrinsic("md5", &["payload".to_string()]).expect("md5");
        assert_eq!(md5.required_crate, Some("md5"));
        assert!(render_expr(&md5.expr).contains("md5::compute"));
        assert!(render_expr(&md5.expr).contains(".as_bytes()"));

        let sha_bytes =
            lower_intrinsic("sha256_bytes", &["payload".to_string()]).expect("sha256_bytes");
        assert_eq!(sha_bytes.required_crate, Some("sha2"));
        assert!(render_expr(&sha_bytes.expr).contains("to_vec"));

        let md5_bytes = lower_intrinsic("md5_bytes", &["payload".to_string()]).expect("md5_bytes");
        assert_eq!(md5_bytes.required_crate, Some("md5"));
        assert!(render_expr(&md5_bytes.expr).contains("md5::compute"));
        assert!(render_expr(&md5_bytes.expr).contains(".0"));
        assert!(render_expr(&md5_bytes.expr).contains("to_vec"));
    }

    #[test]
    fn lowers_platform_intrinsics_via_registry() {
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

    #[test]
    fn lowers_uuid_intrinsic_via_registry() {
        let uuid = lower_intrinsic("uuid4", &[]).expect("uuid4");
        assert_eq!(uuid.required_crate, Some("rand"));
        assert!(render_expr(&uuid.expr).contains("rand::random::<u32>()"));
        assert!(render_expr(&uuid.expr).contains("format!(\"{:08x}-{:04x}-{:04x}-{:04x}-{:012x}\""));
        assert!(render_expr(&uuid.expr).contains("(rand::random::<u16>() & 4095)"));

        let uuid3 =
            lower_intrinsic("uuid3_text", &["ns".to_string(), "name".to_string()]).expect("uuid3");
        assert_eq!(uuid3.required_crate, Some("uuid"));
        assert!(render_expr(&uuid3.expr).contains("uuid::Uuid::parse_str"));
        assert!(render_expr(&uuid3.expr).contains("uuid::Uuid::new_v3"));

        let uuid5 =
            lower_intrinsic("uuid5_text", &["ns".to_string(), "name".to_string()]).expect("uuid5");
        assert_eq!(uuid5.required_crate, Some("uuid"));
        assert!(render_expr(&uuid5.expr).contains("uuid::Uuid::new_v5"));
    }

    #[test]
    fn lowers_toml_intrinsic_with_dependency_metadata() {
        let parsed = lower_intrinsic("toml_parse", &["payload".to_string()]).expect("toml_parse");
        assert_eq!(parsed.required_crate, Some("toml"));
        assert!(render_expr(&parsed.expr).contains("parse::<toml::Table>()"));
        assert!(render_expr(&parsed.expr).contains("TOMLDecodeError"));
    }

    #[test]
    fn lowers_datetime_intrinsics_via_registry() {
        let now = lower_intrinsic("datetime_now", &[]).expect("datetime_now");
        assert_eq!(now.required_crate, Some("chrono"));
        assert!(render_expr(&now.expr).contains("chrono::Local::now()"));

        let now_struct = lower_intrinsic("datetime_now_struct", &[]).expect("datetime_now_struct");
        assert_eq!(now_struct.required_crate, Some("chrono"));
        assert!(render_expr(&now_struct.expr).contains("chrono::Datelike::year(&__dt) as i64"));
        assert!(render_expr(&now_struct.expr).contains("chrono::Timelike::second(&__dt) as i64"));

        let fmt = lower_intrinsic("datetime_format", &["dt".to_string(), "mask".to_string()])
            .expect("datetime_format");
        assert!(render_expr(&fmt.expr).contains("NaiveDateTime::parse_from_str"));

        let from_ts = lower_intrinsic("datetime_from_timestamp", &["ts".to_string()])
            .expect("from_timestamp");
        assert_eq!(from_ts.required_crate, Some("chrono"));
        assert!(render_expr(&from_ts.expr).contains("DateTime::from_timestamp"));
        assert!(render_expr(&from_ts.expr).contains("ok_or_else"));
        assert!(render_expr(&from_ts.expr).contains("\"invalid timestamp\".to_string()"));
    }

    #[test]
    fn lowers_sys_intrinsics_via_registry() {
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
    fn lowers_subprocess_intrinsics_via_registry() {
        let run = lower_intrinsic("subprocess_run", &["cmd".to_string()]).expect("subprocess_run");
        assert!(render_expr(&run.expr).contains("Command::new(\"sh\".to_string())"));
        assert!(render_expr(&run.expr).contains(".arg(\"-c\".to_string())"));
        assert!(render_expr(&run.expr).contains("String::from_utf8_lossy"));

        let with_input = lower_intrinsic(
            "subprocess_run_with_input",
            &["cmd".to_string(), "stdin_data".to_string()],
        )
        .expect("subprocess_run_with_input");
        assert!(render_expr(&with_input.expr).contains("std::io::Write::write_all"));
        assert!(render_expr(&with_input.expr).contains("__child.stdin.take()"));

        let structured = lower_intrinsic("subprocess_run_structured", &["cmd".to_string()])
            .expect("subprocess_run_structured");
        assert!(render_expr(&structured.expr).contains("__output.status.code().unwrap_or(-1)"));
        assert!(render_expr(&structured.expr).contains("vec![__stdout, __stderr, __returncode]"));
    }

    #[test]
    fn lowers_html_intrinsics_via_registry() {
        let esc = lower_intrinsic("html_escape", &["s".to_string()]).expect("html_escape");
        assert!(render_expr(&esc.expr).contains("replace('&', \"&amp;\")"));

        let unesc = lower_intrinsic("html_unescape", &["s".to_string()]).expect("html_unescape");
        assert!(render_expr(&unesc.expr).contains("replace(\"&amp;\", \"&\")"));
    }

    #[test]
    fn lowers_calendar_intrinsics_via_registry() {
        let leap =
            lower_intrinsic("calendar_isleap", &["year".to_string()]).expect("calendar_isleap");
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

        let monthrange =
            lower_intrinsic("calendar_monthrange", &["y".to_string(), "m".to_string()])
                .expect("calendar_monthrange");
        assert!(render_expr(&monthrange.expr).contains("vec![__wd, __days]"));
    }

    #[test]
    fn lowers_gzip_intrinsics_with_dependency_metadata() {
        let compress =
            lower_intrinsic("gzip_compress", &["data".to_string()]).expect("gzip_compress");
        assert_eq!(compress.required_crate, Some("flate2"));
        assert!(render_expr(&compress.expr).contains("GzEncoder"));

        let decompress =
            lower_intrinsic("gzip_decompress", &["bytes".to_string()]).expect("gzip_decompress");
        assert_eq!(decompress.required_crate, Some("flate2"));
        assert!(render_expr(&decompress.expr).contains("GzDecoder"));
    }

    #[test]
    fn lowers_zip_intrinsics_with_dependency_metadata() {
        let create = lower_intrinsic("zip_create", &["path".to_string()]).expect("zip_create");
        assert_eq!(create.required_crate, Some("zip"));
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
        assert_eq!(add.required_crate, Some("zip"));
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
        assert_eq!(add_bytes.required_crate, Some("zip"));
        assert!(render_expr(&add_bytes.expr).contains("write_all"));

        let read = lower_intrinsic("zip_read_file", &["path".to_string(), "name".to_string()])
            .expect("zip_read_file");
        assert_eq!(read.required_crate, Some("zip"));
        assert!(render_expr(&read.expr).contains("ZipArchive::new"));

        let read_bytes = lower_intrinsic(
            "zip_read_file_bytes",
            &["path".to_string(), "name".to_string()],
        )
        .expect("zip_read_file_bytes");
        assert_eq!(read_bytes.required_crate, Some("zip"));
        assert!(render_expr(&read_bytes.expr).contains("read_to_end"));

        let names = lower_intrinsic("zip_namelist", &["path".to_string()]).expect("zip_namelist");
        assert_eq!(names.required_crate, Some("zip"));
        assert!(render_expr(&names.expr).contains("__zip.by_index"));
    }

    #[test]
    fn lowers_base64_intrinsics_with_dependency_metadata() {
        let enc = lower_intrinsic("base64_encode", &["text".to_string()]).expect("base64_encode");
        assert_eq!(enc.required_crate, Some("base64"));
        assert!(render_expr(&enc.expr).contains("base64::Engine::encode"));
        assert!(render_expr(&enc.expr).contains("general_purpose::STANDARD"));

        let dec = lower_intrinsic("base64_decode", &["s".to_string()]).expect("base64_decode");
        assert_eq!(dec.required_crate, Some("base64"));
        assert!(render_expr(&dec.expr).contains("base64::Engine::decode"));
        assert!(render_expr(&dec.expr).contains("general_purpose::STANDARD"));

        let enc_bytes = lower_intrinsic("base64_encode_bytes", &["b".to_string()])
            .expect("base64_encode_bytes");
        assert_eq!(enc_bytes.required_crate, Some("base64"));
        assert!(render_expr(&enc_bytes.expr).contains("into_bytes"));

        let dec_bytes = lower_intrinsic("base64_decode_bytes", &["b".to_string()])
            .expect("base64_decode_bytes");
        assert_eq!(dec_bytes.required_crate, Some("base64"));
        assert!(render_expr(&dec_bytes.expr).contains("base64::Engine::decode"));

        let enc_opts = lower_intrinsic(
            "base64_encode_opts",
            &["s".to_string(), "alt".to_string(), "wrap".to_string()],
        )
        .expect("base64_encode_opts");
        assert_eq!(enc_opts.required_crate, Some("base64"));
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
        assert_eq!(dec_opts.required_crate, Some("base64"));
        assert!(render_expr(&dec_opts.expr).contains("invalid base64 character"));

        let url_enc =
            lower_intrinsic("urlsafe_b64encode", &["s".to_string()]).expect("urlsafe_b64encode");
        assert_eq!(url_enc.required_crate, Some("base64"));
        assert!(render_expr(&url_enc.expr).contains("base64::Engine::encode"));
        assert!(render_expr(&url_enc.expr).contains("general_purpose::URL_SAFE"));

        let url_dec =
            lower_intrinsic("urlsafe_b64decode", &["s".to_string()]).expect("urlsafe_b64decode");
        assert_eq!(url_dec.required_crate, Some("base64"));
        assert!(render_expr(&url_dec.expr).contains("base64::Engine::decode"));
        assert!(render_expr(&url_dec.expr).contains("general_purpose::URL_SAFE"));

        let url_enc_bytes = lower_intrinsic("urlsafe_b64encode_bytes", &["b".to_string()])
            .expect("urlsafe_b64encode_bytes");
        assert_eq!(url_enc_bytes.required_crate, Some("base64"));
        assert!(render_expr(&url_enc_bytes.expr).contains("into_bytes"));

        let url_dec_bytes = lower_intrinsic("urlsafe_b64decode_bytes", &["b".to_string()])
            .expect("urlsafe_b64decode_bytes");
        assert_eq!(url_dec_bytes.required_crate, Some("base64"));
        assert!(render_expr(&url_dec_bytes.expr).contains("base64::Engine::decode"));
    }

    #[test]
    fn lowers_base32_intrinsics_via_registry() {
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
    fn lowers_hashlib_intrinsics_with_dependency_metadata() {
        let sha1 = lower_intrinsic("sha1", &["s".to_string()]).expect("sha1");
        assert_eq!(sha1.required_crate, Some("sha1"));
        assert!(render_expr(&sha1.expr).contains("<sha1::Sha1 as sha1::Digest>::digest"));

        let sha1_bytes = lower_intrinsic("sha1_bytes", &["b".to_string()]).expect("sha1_bytes");
        assert_eq!(sha1_bytes.required_crate, Some("sha1"));
        assert!(render_expr(&sha1_bytes.expr).contains("to_vec"));

        let sha512 = lower_intrinsic("sha512", &["s".to_string()]).expect("sha512");
        assert_eq!(sha512.required_crate, Some("sha2"));
        assert!(render_expr(&sha512.expr).contains("<sha2::Sha512 as sha2::Digest>::digest"));

        let sha512_bytes =
            lower_intrinsic("sha512_bytes", &["b".to_string()]).expect("sha512_bytes");
        assert_eq!(sha512_bytes.required_crate, Some("sha2"));
        assert!(render_expr(&sha512_bytes.expr).contains("to_vec"));

        let sha224 = lower_intrinsic("sha224", &["s".to_string()]).expect("sha224");
        assert_eq!(sha224.required_crate, Some("sha2"));
        assert!(render_expr(&sha224.expr).contains("<sha2::Sha224 as sha2::Digest>::digest"));

        let sha224_bytes =
            lower_intrinsic("sha224_bytes", &["b".to_string()]).expect("sha224_bytes");
        assert_eq!(sha224_bytes.required_crate, Some("sha2"));
        assert!(render_expr(&sha224_bytes.expr).contains("to_vec"));

        let sha384 = lower_intrinsic("sha384", &["s".to_string()]).expect("sha384");
        assert_eq!(sha384.required_crate, Some("sha2"));
        assert!(render_expr(&sha384.expr).contains("<sha2::Sha384 as sha2::Digest>::digest"));

        let sha384_bytes =
            lower_intrinsic("sha384_bytes", &["b".to_string()]).expect("sha384_bytes");
        assert_eq!(sha384_bytes.required_crate, Some("sha2"));
        assert!(render_expr(&sha384_bytes.expr).contains("to_vec"));

        let blake2b = lower_intrinsic("blake2b", &["s".to_string()]).expect("blake2b");
        assert_eq!(blake2b.required_crate, Some("blake2"));
        assert!(render_expr(&blake2b.expr).contains("Blake2b512"));

        let blake2b_bytes =
            lower_intrinsic("blake2b_bytes", &["b".to_string()]).expect("blake2b_bytes");
        assert_eq!(blake2b_bytes.required_crate, Some("blake2"));
        assert!(render_expr(&blake2b_bytes.expr).contains("to_vec"));

        let blake2s = lower_intrinsic("blake2s", &["s".to_string()]).expect("blake2s");
        assert_eq!(blake2s.required_crate, Some("blake2"));
        assert!(render_expr(&blake2s.expr).contains("Blake2s256"));

        let blake2s_bytes =
            lower_intrinsic("blake2s_bytes", &["b".to_string()]).expect("blake2s_bytes");
        assert_eq!(blake2s_bytes.required_crate, Some("blake2"));
        assert!(render_expr(&blake2s_bytes.expr).contains("to_vec"));
    }

    #[test]
    fn lowers_extended_math_intrinsics_via_registry() {
        let remainder =
            lower_intrinsic("remainder", &["x".to_string(), "y".to_string()]).expect("remainder");
        assert!(render_expr(&remainder.expr).contains("__abs_frac < 0.5"));

        let dist = lower_intrinsic("dist", &["p".to_string(), "q".to_string()]).expect("dist");
        assert!(render_expr(&dist.expr).contains("__p.len() != __q.len()"));

        let fsum = lower_intrinsic("fsum", &["vals".to_string()]).expect("fsum");
        assert!(render_expr(&fsum.expr).contains("__sum + __comp"));

        let sumprod =
            lower_intrinsic("sumprod", &["a".to_string(), "b".to_string()]).expect("sumprod");
        assert!(render_expr(&sumprod.expr).contains("__p.len().min(__q.len())"));

        let ldexp = lower_intrinsic("ldexp", &["m".to_string(), "e".to_string()]).expect("ldexp");
        assert!(render_expr(&ldexp.expr).contains("(2.0 as f64).powi"));

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
    fn lowers_file_handle_and_logging_intrinsics_via_registry() {
        let open = lower_intrinsic("open_file", &["path".to_string(), "mode".to_string()])
            .expect("open_file");
        assert!(render_expr(&open.expr).contains("__SIFR_FILE_HANDLES"));
        assert!(render_expr(&open.expr).contains("__sifr_next_file_handle_id()"));

        let read = lower_intrinsic("file_read", &["hid".to_string()]).expect("file_read");
        assert!(render_expr(&read.expr).contains("TextRead"));

        let write = lower_intrinsic("file_write", &["hid".to_string(), "text".to_string()])
            .expect("file_write");
        assert!(render_expr(&write.expr).contains("TextWrite"));

        let close = lower_intrinsic("file_close", &["hid".to_string()]).expect("file_close");
        assert!(render_expr(&close.expr).contains("__SIFR_FILE_HANDLES"));

        let builtin_open =
            lower_intrinsic("builtin_open", &["path".to_string(), "mode".to_string()])
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
    fn lower_intrinsic_accepts_ir_inputs() {
        let ir = super::lower_intrinsic(
            "file_write",
            &[
                RustExpr::Ident("hid".to_string()),
                RustExpr::Ident("text".to_string()),
            ],
        )
        .expect("ir file_write");

        assert!(render_expr(&ir.expr).contains("TextWrite"));
        assert_eq!(ir.required_crate, None);
    }
}
