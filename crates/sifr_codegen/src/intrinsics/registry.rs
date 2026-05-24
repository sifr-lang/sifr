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

pub(crate) fn additional_required_crates(name: &str) -> &'static [&'static str] {
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

pub(crate) fn lower_intrinsic_rendered(name: &str, args: &[RustExpr]) -> Option<LoweredIntrinsic> {
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
