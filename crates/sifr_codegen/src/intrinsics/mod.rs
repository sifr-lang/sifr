//! Intrinsic registry and dispatch for incremental migration.

mod math;
mod json;
mod env;
mod os;
mod io;
mod pathlib;
mod test;
mod collections;
mod bytes;
mod time;
mod random;
mod re;
mod hash;
mod platform;
mod uuid;
mod toml;
mod datetime;
mod sys;
mod subprocess;
mod html;
mod calendar;
mod gzip;
mod zipfile;
mod base64;
mod base32;
mod hashlib;

use crate::RustExpr;

pub(crate) struct LoweredIntrinsic {
    pub(crate) expr: RustExpr,
    pub(crate) required_crate: Option<&'static str>,
}

pub(crate) fn lower_intrinsic(name: &str, rendered_args: &[String]) -> Option<LoweredIntrinsic> {
    let (expr, required_crate) = match name {
        "sqrt" => (math::lower_sqrt(rendered_args), None),
        "floor" => (math::lower_floor(rendered_args), None),
        "ceil" => (math::lower_ceil(rendered_args), None),
        "abs_val" => (math::lower_abs_val(rendered_args), None),
        "log" => (math::lower_log(rendered_args), None),
        "cbrt" => (math::lower_cbrt(rendered_args), None),
        "exp2" => (math::lower_exp2(rendered_args), None),
        "sin" => (math::lower_sin(rendered_args), None),
        "cos" => (math::lower_cos(rendered_args), None),
        "tan" => (math::lower_tan(rendered_args), None),
        "pow_val" => (math::lower_pow_val(rendered_args), None),
        "min_val" => (math::lower_min_val(rendered_args), None),
        "max_val" => (math::lower_max_val(rendered_args), None),
        "round_val" => (math::lower_round_val(rendered_args), None),
        "asin" => (math::lower_asin(rendered_args), None),
        "acos" => (math::lower_acos(rendered_args), None),
        "atan" => (math::lower_atan(rendered_args), None),
        "atan2" => (math::lower_atan2(rendered_args), None),
        "sinh" => (math::lower_sinh(rendered_args), None),
        "cosh" => (math::lower_cosh(rendered_args), None),
        "tanh" => (math::lower_tanh(rendered_args), None),
        "log10" => (math::lower_log10(rendered_args), None),
        "log2" => (math::lower_log2(rendered_args), None),
        "degrees" => (math::lower_degrees(rendered_args), None),
        "radians" => (math::lower_radians(rendered_args), None),
        "isnan" => (math::lower_isnan(rendered_args), None),
        "isinf" => (math::lower_isinf(rendered_args), None),
        "trunc" => (math::lower_trunc(rendered_args), None),
        "copysign" => (math::lower_copysign(rendered_args), None),
        "signbit" => (math::lower_signbit(rendered_args), None),
        "fmod" => (math::lower_fmod(rendered_args), None),
        "hypot" => (math::lower_hypot(rendered_args), None),
        "fma" => (math::lower_fma(rendered_args), None),
        "fmax" => (math::lower_fmax(rendered_args), None),
        "fmin" => (math::lower_fmin(rendered_args), None),
        "exp" => (math::lower_exp(rendered_args), None),
        "expm1" => (math::lower_expm1(rendered_args), None),
        "log1p" => (math::lower_log1p(rendered_args), None),
        "fabs" => (math::lower_fabs(rendered_args), None),
        "isfinite" => (math::lower_isfinite(rendered_args), None),
        "isnormal" => (math::lower_isnormal(rendered_args), None),
        "issubnormal" => (math::lower_issubnormal(rendered_args), None),
        "acosh" => (math::lower_acosh(rendered_args), None),
        "asinh" => (math::lower_asinh(rendered_args), None),
        "atanh" => (math::lower_atanh(rendered_args), None),
        "isqrt" => (math::lower_isqrt(rendered_args), None),
        "env_get" => (env::lower_env_get(rendered_args), None),
        "env_set" => (env::lower_env_set(rendered_args), None),
        "env_unset" => (env::lower_env_unset(rendered_args), None),
        "env_keys" => (env::lower_env_keys(rendered_args), None),
        "env_values" => (env::lower_env_values(rendered_args), None),
        "env_items" => (env::lower_env_items(rendered_args), None),
        "run_command" => (os::lower_run_command(rendered_args), None),
        "get_args" => (os::lower_get_args(rendered_args), None),
        "chdir" => (os::lower_chdir(rendered_args), None),
        "getpid" => (os::lower_getpid(rendered_args), None),
        "cpu_count" => (os::lower_cpu_count(rendered_args), None),
        "stat_size" => (os::lower_stat_size(rendered_args), None),
        "which" => (os::lower_which(rendered_args), None),
        "disk_usage" => (os::lower_disk_usage(rendered_args), None),
        "os_sep" => (os::lower_os_sep(rendered_args), None),
        "os_linesep" => (os::lower_os_linesep(rendered_args), None),
        "os_name" => (os::lower_os_name(rendered_args), None),
        "touch" => (pathlib::lower_touch(rendered_args), None),
        "resolve_path" => (pathlib::lower_resolve_path(rendered_args), None),
        "iterdir" => (pathlib::lower_iterdir(rendered_args), None),
        "glob_pattern" => (pathlib::lower_glob_pattern(rendered_args), None),
        "rglob_pattern" => (pathlib::lower_rglob_pattern(rendered_args), None),
        "read_text" => (io::lower_read_text(rendered_args), None),
        "write_text" => (io::lower_write_text(rendered_args), None),
        "exists" => (io::lower_exists(rendered_args), None),
        "read_lines" => (io::lower_read_lines(rendered_args), None),
        "append_text" => (io::lower_append_text(rendered_args), None),
        "getcwd" => (io::lower_getcwd(rendered_args), None),
        "listdir" => (io::lower_listdir(rendered_args), None),
        "mkdir" => (io::lower_mkdir(rendered_args), None),
        "rmdir" => (io::lower_rmdir(rendered_args), None),
        "remove_file" => (io::lower_remove_file(rendered_args), None),
        "rename" => (io::lower_rename(rendered_args), None),
        "is_file" => (io::lower_is_file(rendered_args), None),
        "is_dir" => (io::lower_is_dir(rendered_args), None),
        "copy_file" => (io::lower_copy_file(rendered_args), None),
        "walk_dir" => (io::lower_walk_dir(rendered_args), None),
        "rmdir_all" => (io::lower_rmdir_all(rendered_args), None),
        "gettempdir" => (io::lower_gettempdir(rendered_args), None),
        "makedirs" => (io::lower_makedirs(rendered_args), None),
        "json_loads" => (json::lower_json_loads(rendered_args), Some("serde_json")),
        "json_dumps" => (json::lower_json_dumps(rendered_args), Some("serde_json")),
        "assert_eq" => (test::lower_assert_eq(rendered_args), None),
        "assert_ne" => (test::lower_assert_ne(rendered_args), None),
        "assert_true" => (test::lower_assert_true(rendered_args), None),
        "assert_false" => (test::lower_assert_false(rendered_args), None),
        "assert_almost_eq" => (test::lower_assert_almost_eq(rendered_args), None),
        "assert_gt" => (test::lower_assert_gt(rendered_args), None),
        "assert_lt" => (test::lower_assert_lt(rendered_args), None),
        "new_set" => (collections::lower_new_set(rendered_args), None),
        "set_from_list" => (collections::lower_set_from_list(rendered_args), None),
        "set_add" => (collections::lower_set_add(rendered_args), None),
        "set_contains" => (collections::lower_set_contains(rendered_args), None),
        "set_remove" => (collections::lower_set_remove(rendered_args), None),
        "set_len" => (collections::lower_set_len(rendered_args), None),
        "set_union" => (collections::lower_set_union(rendered_args), None),
        "set_intersection" => (collections::lower_set_intersection(rendered_args), None),
        "counter_from_list" => (collections::lower_counter_from_list(rendered_args), Some("serde_json")),
        "counter_get" => (collections::lower_counter_get(rendered_args), Some("serde_json")),
        "counter_most_common" => (
            collections::lower_counter_most_common(rendered_args),
            Some("serde_json"),
        ),
        "counter_total" => (collections::lower_counter_total(rendered_args), Some("serde_json")),
        "counter_values" => (collections::lower_counter_values(rendered_args), Some("serde_json")),
        "counter_keys" => (collections::lower_counter_keys(rendered_args), Some("serde_json")),
        "counter_items" => (collections::lower_counter_items(rendered_args), Some("serde_json")),
        "counter_increment" => (
            collections::lower_counter_increment(rendered_args),
            Some("serde_json"),
        ),
        "defaultdict_new" => (collections::lower_defaultdict_new(rendered_args), None),
        "defaultdict_get" => (collections::lower_defaultdict_get(rendered_args), Some("serde_json")),
        "defaultdict_set" => (collections::lower_defaultdict_set(rendered_args), Some("serde_json")),
        "encode_utf8" => (bytes::lower_encode_utf8(rendered_args), None),
        "decode_utf8" => (bytes::lower_decode_utf8(rendered_args), None),
        "bytes_to_hex" => (bytes::lower_bytes_to_hex(rendered_args), None),
        "bytes_from_hex" => (bytes::lower_bytes_from_hex(rendered_args), None),
        "time_now" => (time::lower_time_now(rendered_args), None),
        "sleep" => (time::lower_sleep(rendered_args), None),
        "time_format" => (time::lower_time_format(rendered_args), Some("chrono")),
        "perf_counter" => (time::lower_perf_counter(rendered_args), None),
        "monotonic" => (time::lower_monotonic(rendered_args), None),
        "strptime" => (time::lower_strptime(rendered_args), Some("chrono")),
        "gmtime" => (time::lower_gmtime(rendered_args), Some("chrono")),
        "localtime" => (time::lower_localtime(rendered_args), Some("chrono")),
        "_strptime_intrinsic" => (time::lower_strptime(rendered_args), Some("chrono")),
        "_gmtime_intrinsic" => (time::lower_gmtime(rendered_args), Some("chrono")),
        "_localtime_intrinsic" => (time::lower_localtime(rendered_args), Some("chrono")),
        "time_strptime" => (time::lower_time_strptime_compat(rendered_args), Some("chrono")),
        "time_gmtime" => (time::lower_time_gmtime_compat(rendered_args), Some("chrono")),
        "time_localtime" => (time::lower_time_localtime_compat(rendered_args), Some("chrono")),
        "random_int" => (random::lower_random_int(rendered_args), Some("rand")),
        "random_float" => (random::lower_random_float(rendered_args), Some("rand")),
        "random_choice" => (random::lower_random_choice(rendered_args), Some("rand")),
        "random_uniform" => (random::lower_random_uniform(rendered_args), Some("rand")),
        "random_shuffle" => (random::lower_random_shuffle(rendered_args), Some("rand")),
        "random_sample" => (random::lower_random_sample(rendered_args), Some("rand")),
        "random_randrange" => (random::lower_random_randrange(rendered_args), Some("rand")),
        "random_gauss" => (random::lower_random_gauss(rendered_args), Some("rand")),
        "re_match" => (re::lower_re_match(rendered_args), Some("regex")),
        "re_find" => (re::lower_re_find(rendered_args), Some("regex")),
        "re_replace" => (re::lower_re_replace(rendered_args), Some("regex")),
        "re_findall" => (re::lower_re_findall(rendered_args), Some("regex")),
        "re_split" => (re::lower_re_split(rendered_args), Some("regex")),
        "re_find_start" => (re::lower_re_find_start(rendered_args), Some("regex")),
        "re_find_end" => (re::lower_re_find_end(rendered_args), Some("regex")),
        "re_match_flags" => (re::lower_re_match_flags(rendered_args), Some("regex")),
        "re_find_flags" => (re::lower_re_find_flags(rendered_args), Some("regex")),
        "re_replace_flags" => (re::lower_re_replace_flags(rendered_args), Some("regex")),
        "re_findall_flags" => (re::lower_re_findall_flags(rendered_args), Some("regex")),
        "re_split_flags" => (re::lower_re_split_flags(rendered_args), Some("regex")),
        "sha256" => (hash::lower_sha256(rendered_args), Some("sha2")),
        "md5" => (hash::lower_md5(rendered_args), Some("md5")),
        "platform_system" => (platform::lower_platform_system(rendered_args), None),
        "platform_arch" => (platform::lower_platform_arch(rendered_args), None),
        "platform_node" => (platform::lower_platform_node(rendered_args), None),
        "platform_release" => (platform::lower_platform_release(rendered_args), None),
        "platform_version" => (platform::lower_platform_version(rendered_args), None),
        "platform_processor" => (platform::lower_platform_processor(rendered_args), None),
        "uuid4" => (uuid::lower_uuid4(rendered_args), Some("rand")),
        "toml_parse" => (toml::lower_toml_parse(rendered_args), Some("toml")),
        "datetime_now" => (datetime::lower_datetime_now(rendered_args), Some("chrono")),
        "datetime_now_struct" => (datetime::lower_datetime_now_struct(rendered_args), Some("chrono")),
        "datetime_format" => (datetime::lower_datetime_format(rendered_args), None),
        "datetime_from_timestamp" => (
            datetime::lower_datetime_from_timestamp(rendered_args),
            Some("chrono"),
        ),
        "sys_exit" => (sys::lower_sys_exit(rendered_args), None),
        "sys_version" => (sys::lower_sys_version(rendered_args), None),
        "sys_platform" => (sys::lower_sys_platform(rendered_args), None),
        "sys_maxsize" => (sys::lower_sys_maxsize(rendered_args), None),
        "subprocess_run" => (subprocess::lower_subprocess_run(rendered_args), None),
        "subprocess_run_with_input" => (
            subprocess::lower_subprocess_run_with_input(rendered_args),
            None,
        ),
        "subprocess_run_structured" => (
            subprocess::lower_subprocess_run_structured(rendered_args),
            None,
        ),
        "html_escape" => (html::lower_html_escape(rendered_args), None),
        "html_unescape" => (html::lower_html_unescape(rendered_args), None),
        "calendar_isleap" => (calendar::lower_calendar_isleap(rendered_args), None),
        "calendar_weekday" => (calendar::lower_calendar_weekday(rendered_args), None),
        "calendar_monthrange" => (calendar::lower_calendar_monthrange(rendered_args), None),
        "gzip_compress" => (gzip::lower_gzip_compress(rendered_args), Some("flate2")),
        "gzip_decompress" => (gzip::lower_gzip_decompress(rendered_args), Some("flate2")),
        "zip_create" => (zipfile::lower_zip_create(rendered_args), Some("zip")),
        "zip_add_file" => (zipfile::lower_zip_add_file(rendered_args), Some("zip")),
        "zip_read_file" => (zipfile::lower_zip_read_file(rendered_args), Some("zip")),
        "zip_namelist" => (zipfile::lower_zip_namelist(rendered_args), Some("zip")),
        "base64_encode" => (base64::lower_base64_encode(rendered_args), Some("base64")),
        "base64_decode" => (base64::lower_base64_decode(rendered_args), Some("base64")),
        "base64_encode_opts" => (base64::lower_base64_encode_opts(rendered_args), Some("base64")),
        "base64_decode_opts" => (base64::lower_base64_decode_opts(rendered_args), Some("base64")),
        "urlsafe_b64encode" => (base64::lower_urlsafe_b64encode(rendered_args), Some("base64")),
        "urlsafe_b64decode" => (base64::lower_urlsafe_b64decode(rendered_args), Some("base64")),
        "b32encode" => (base32::lower_b32encode(rendered_args), None),
        "b32decode" => (base32::lower_b32decode(rendered_args), None),
        "b32hexencode" => (base32::lower_b32hexencode(rendered_args), None),
        "b32hexdecode" => (base32::lower_b32hexdecode(rendered_args), None),
        "sha1" => (hashlib::lower_sha1(rendered_args), Some("sha1")),
        "sha512" => (hashlib::lower_sha512(rendered_args), Some("sha2")),
        "sha224" => (hashlib::lower_sha224(rendered_args), Some("sha2")),
        "sha384" => (hashlib::lower_sha384(rendered_args), Some("sha2")),
        "blake2b" => (hashlib::lower_blake2b(rendered_args), Some("blake2")),
        "blake2s" => (hashlib::lower_blake2s(rendered_args), Some("blake2")),
        _ => return None,
    };

    Some(LoweredIntrinsic {
        expr: expr?,
        required_crate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_expr;

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

        let lowered = lower_intrinsic("round_val", &["n".to_string()])
            .expect("round_val should lower");
        assert_eq!(render_expr(&lowered.expr), "(n).round() as i64");

        let lowered = lower_intrinsic("isfinite", &["f".to_string()])
            .expect("isfinite should lower");
        assert_eq!(render_expr(&lowered.expr), "(f).is_finite()");

        let lowered = lower_intrinsic("isqrt", &["v".to_string()]).expect("isqrt should lower");
        assert_eq!(render_expr(&lowered.expr), "{ let __n = v as f64; __n.sqrt() as i64 }");
    }

    #[test]
    fn lowers_json_intrinsics_with_dependency_metadata() {
        let loads = lower_intrinsic("json_loads", &["payload".to_string()])
            .expect("json_loads should lower");
        assert_eq!(loads.required_crate, Some("serde_json"));
        assert!(render_expr(&loads.expr).contains("serde_json::from_str"));

        let dumps = lower_intrinsic("json_dumps", &["value".to_string()])
            .expect("json_dumps should lower");
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
        let run = lower_intrinsic("run_command", &["cmd".to_string()])
            .expect("run_command should lower");
        assert!(render_expr(&run.expr).contains("std::process::Command::new(\"sh\")"));

        let args = lower_intrinsic("get_args", &[]).expect("get_args should lower");
        assert_eq!(render_expr(&args.expr), "std::env::args().collect::<Vec<String>>()");

        let pid = lower_intrinsic("getpid", &[]).expect("getpid should lower");
        assert_eq!(render_expr(&pid.expr), "std::process::id() as i64");

        let cpus = lower_intrinsic("cpu_count", &[]).expect("cpu_count should lower");
        assert!(render_expr(&cpus.expr).contains("available_parallelism"));

        let which = lower_intrinsic("which", &["tool".to_string()]).expect("which should lower");
        assert!(render_expr(&which.expr).contains("std::env::var(\"PATH\")"));

        let disk = lower_intrinsic("disk_usage", &["path".to_string()]).expect("disk_usage lowers");
        assert!(render_expr(&disk.expr).contains("std::process::Command::new(\"df\")"));

        let sep = lower_intrinsic("os_sep", &[]).expect("os_sep lowers");
        assert_eq!(render_expr(&sep.expr), "std::path::MAIN_SEPARATOR.to_string()");

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
        assert!(render_expr(&walk.expr).contains("fn __walk"));
    }

    #[test]
    fn lowers_pathlib_intrinsics_via_registry() {
        let touch = lower_intrinsic("touch", &["p".to_string()]).expect("touch lowers");
        assert!(render_expr(&touch.expr).contains("OpenOptions::new().create(true)"));

        let resolve = lower_intrinsic("resolve_path", &["p".to_string()])
            .expect("resolve_path lowers");
        assert!(render_expr(&resolve.expr).contains("std::fs::canonicalize"));

        let iterdir = lower_intrinsic("iterdir", &["p".to_string()]).expect("iterdir lowers");
        assert!(render_expr(&iterdir.expr).contains("std::fs::read_dir"));

        let glob =
            lower_intrinsic("glob_pattern", &["dir".to_string(), "pat".to_string()])
                .expect("glob_pattern lowers");
        assert!(render_expr(&glob.expr).contains("fn __matches_glob"));

        let rglob =
            lower_intrinsic("rglob_pattern", &["dir".to_string(), "pat".to_string()])
                .expect("rglob_pattern lowers");
        assert!(render_expr(&rglob.expr).contains("fn __rglob_walk"));
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

        let inter =
            lower_intrinsic("set_intersection", &["a".to_string(), "b".to_string()])
                .expect("set_intersection lowers");
        assert!(render_expr(&inter.expr).contains("collect::<Vec<i64>>()"));
    }

    #[test]
    fn lowers_collections_counter_intrinsics_via_registry() {
        let from_list =
            lower_intrinsic("counter_from_list", &["vals".to_string()]).expect("counter_from_list");
        assert!(render_expr(&from_list.expr).contains("HashMap::<String, i64>"));

        let get = lower_intrinsic("counter_get", &["data".to_string(), "k".to_string()])
            .expect("counter_get");
        assert!(render_expr(&get.expr).contains("serde_json::from_str"));

        let incr = lower_intrinsic("counter_increment", &["data".to_string(), "k".to_string()])
            .expect("counter_increment");
        assert!(render_expr(&incr.expr).contains("or_insert(0) += 1"));

        let dd_set = lower_intrinsic(
            "defaultdict_set",
            &["dd".to_string(), "key".to_string(), "v".to_string()],
        )
        .expect("defaultdict_set");
        assert!(render_expr(&dd_set.expr).contains("serde_json::json!"));
    }

    #[test]
    fn lowers_bytes_intrinsics_via_registry() {
        let enc = lower_intrinsic("encode_utf8", &["s".to_string()]).expect("encode_utf8");
        assert!(render_expr(&enc.expr).contains("as_bytes()"));

        let dec = lower_intrinsic("decode_utf8", &["vals".to_string()]).expect("decode_utf8");
        assert!(render_expr(&dec.expr).contains("String::from_utf8"));

        let to_hex = lower_intrinsic("bytes_to_hex", &["vals".to_string()]).expect("bytes_to_hex");
        assert!(render_expr(&to_hex.expr).contains("byte out of range"));

        let from_hex =
            lower_intrinsic("bytes_from_hex", &["hex".to_string()]).expect("bytes_from_hex");
        assert!(render_expr(&from_hex.expr).contains("invalid hex character"));
    }

    #[test]
    fn lowers_time_intrinsics_via_registry() {
        let now = lower_intrinsic("time_now", &[]).expect("time_now");
        assert!(render_expr(&now.expr).contains("SystemTime::now()"));

        let sleep = lower_intrinsic("sleep", &["0.1".to_string()]).expect("sleep");
        assert!(render_expr(&sleep.expr).contains("from_secs_f64"));

        let fmt = lower_intrinsic("time_format", &["secs".to_string(), "mask".to_string()])
            .expect("time_format");
        assert_eq!(fmt.required_crate, Some("chrono"));
        assert!(render_expr(&fmt.expr).contains("DateTime::from_timestamp"));

        let perf = lower_intrinsic("perf_counter", &[]).expect("perf_counter");
        assert!(render_expr(&perf.expr).contains("OnceLock<std::time::Instant>"));

        let mono = lower_intrinsic("monotonic", &[]).expect("monotonic");
        assert!(render_expr(&mono.expr).contains("OnceLock<std::time::Instant>"));

        let parse =
            lower_intrinsic("strptime", &["s".to_string(), "f".to_string()]).expect("strptime");
        assert_eq!(parse.required_crate, Some("chrono"));
        assert!(render_expr(&parse.expr).contains("NaiveDateTime::parse_from_str"));

        let gmt = lower_intrinsic("gmtime", &["ts".to_string()]).expect("gmtime");
        assert_eq!(gmt.required_crate, Some("chrono"));
        assert!(render_expr(&gmt.expr).contains("DateTime::<Utc>::from_timestamp"));

        let local = lower_intrinsic("localtime", &["ts".to_string()]).expect("localtime");
        assert_eq!(local.required_crate, Some("chrono"));
        assert!(render_expr(&local.expr).contains("with_timezone(&Local)"));

        let parse_alias = lower_intrinsic("_strptime_intrinsic", &["s".to_string(), "f".to_string()])
            .expect("_strptime_intrinsic");
        assert_eq!(parse_alias.required_crate, Some("chrono"));
        assert!(render_expr(&parse_alias.expr).contains("NaiveDateTime::parse_from_str"));

        let compat_parse =
            lower_intrinsic("time_strptime", &["s".to_string(), "f".to_string()])
                .expect("time_strptime");
        assert_eq!(compat_parse.required_crate, Some("chrono"));
        assert!(render_expr(&compat_parse.expr).contains("Result<Vec<i64>, ValueError>"));

        let compat_gmt = lower_intrinsic("time_gmtime", &[]).expect("time_gmtime");
        assert_eq!(compat_gmt.required_crate, Some("chrono"));
        assert!(render_expr(&compat_gmt.expr).contains("Utc::now().naive_utc()"));

        let compat_local = lower_intrinsic("time_localtime", &[]).expect("time_localtime");
        assert_eq!(compat_local.required_crate, Some("chrono"));
        assert!(render_expr(&compat_local.expr).contains("Local::now().naive_local()"));
    }

    #[test]
    fn lowers_random_intrinsics_via_registry() {
        let rint = lower_intrinsic("random_int", &["1".to_string(), "9".to_string()])
            .expect("random_int");
        assert_eq!(rint.required_crate, Some("rand"));
        assert!(render_expr(&rint.expr).contains("gen_range(1..=9)"));

        let rfloat = lower_intrinsic("random_float", &[]).expect("random_float");
        assert_eq!(rfloat.required_crate, Some("rand"));
        assert!(render_expr(&rfloat.expr).contains("gen::<f64>()"));

        let choice = lower_intrinsic("random_choice", &["items".to_string()]).expect("random_choice");
        assert!(render_expr(&choice.expr).contains("items.len()"));

        let uniform = lower_intrinsic("random_uniform", &["0.0".to_string(), "1.0".to_string()])
            .expect("random_uniform");
        assert!(render_expr(&uniform.expr).contains("gen_range(0.0..=1.0)"));

        let shuffle =
            lower_intrinsic("random_shuffle", &["vals".to_string()]).expect("random_shuffle");
        assert!(render_expr(&shuffle.expr).contains("SliceRandom"));

        let sample = lower_intrinsic("random_sample", &["vals".to_string(), "3".to_string()])
            .expect("random_sample");
        assert!(render_expr(&sample.expr).contains("choose_multiple"));

        let randrange = lower_intrinsic(
            "random_randrange",
            &["0".to_string(), "10".to_string(), "1".to_string()],
        )
        .expect("random_randrange");
        assert!(render_expr(&randrange.expr).contains("randrange: step must not be zero"));

        let gauss = lower_intrinsic("random_gauss", &["0.0".to_string(), "1.0".to_string()])
            .expect("random_gauss");
        assert!(render_expr(&gauss.expr).contains("rand_distr"));
    }

    #[test]
    fn lowers_re_intrinsics_via_registry() {
        let m = lower_intrinsic("re_match", &["pat".to_string(), "txt".to_string()]).expect("re_match");
        assert_eq!(m.required_crate, Some("regex"));
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
        assert_eq!(rf.required_crate, Some("regex"));
        assert!(render_expr(&rf.expr).contains("replace_all"));
    }

    #[test]
    fn lowers_hash_intrinsics_via_registry() {
        let sha = lower_intrinsic("sha256", &["payload".to_string()]).expect("sha256");
        assert_eq!(sha.required_crate, Some("sha2"));
        assert!(render_expr(&sha.expr).contains("sha2::Sha256::digest"));
        assert!(render_expr(&sha.expr).contains(".as_bytes()"));

        let md5 = lower_intrinsic("md5", &["payload".to_string()]).expect("md5");
        assert_eq!(md5.required_crate, Some("md5"));
        assert!(render_expr(&md5.expr).contains("md5::compute"));
        assert!(render_expr(&md5.expr).contains(".as_bytes()"));
    }

    #[test]
    fn lowers_platform_intrinsics_via_registry() {
        let system = lower_intrinsic("platform_system", &[]).expect("platform_system");
        assert_eq!(render_expr(&system.expr), "std::env::consts::OS.to_string()");

        let arch = lower_intrinsic("platform_arch", &[]).expect("platform_arch");
        assert_eq!(render_expr(&arch.expr), "std::env::consts::ARCH.to_string()");

        let node = lower_intrinsic("platform_node", &[]).expect("platform_node");
        assert!(render_expr(&node.expr).contains("Command::new(\"hostname\")"));

        let rel = lower_intrinsic("platform_release", &[]).expect("platform_release");
        assert!(render_expr(&rel.expr).contains("Command::new(\"uname\").arg(\"-r\")"));

        let ver = lower_intrinsic("platform_version", &[]).expect("platform_version");
        assert!(render_expr(&ver.expr).contains("Command::new(\"uname\").arg(\"-v\")"));

        let proc = lower_intrinsic("platform_processor", &[]).expect("platform_processor");
        assert_eq!(render_expr(&proc.expr), "std::env::consts::ARCH.to_string()");
    }

    #[test]
    fn lowers_uuid_intrinsic_via_registry() {
        let uuid = lower_intrinsic("uuid4", &[]).expect("uuid4");
        assert_eq!(uuid.required_crate, Some("rand"));
        assert!(render_expr(&uuid.expr).contains("rand::thread_rng"));
        assert!(render_expr(&uuid.expr).contains("format!(\"{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}\""));
    }

    #[test]
    fn lowers_toml_intrinsic_with_dependency_metadata() {
        let parsed = lower_intrinsic("toml_parse", &["payload".to_string()]).expect("toml_parse");
        assert_eq!(parsed.required_crate, Some("toml"));
        assert!(render_expr(&parsed.expr).contains("parse::<toml::Value>()"));
        assert!(render_expr(&parsed.expr).contains("TOMLDecodeError"));
    }

    #[test]
    fn lowers_datetime_intrinsics_via_registry() {
        let now = lower_intrinsic("datetime_now", &[]).expect("datetime_now");
        assert_eq!(now.required_crate, Some("chrono"));
        assert!(render_expr(&now.expr).contains("chrono::Local::now()"));

        let now_struct = lower_intrinsic("datetime_now_struct", &[]).expect("datetime_now_struct");
        assert_eq!(now_struct.required_crate, Some("chrono"));
        assert!(render_expr(&now_struct.expr).contains("vec![__dt.year() as i64"));

        let fmt = lower_intrinsic("datetime_format", &["dt".to_string(), "mask".to_string()])
            .expect("datetime_format");
        assert!(render_expr(&fmt.expr).contains("__dt_str.to_string()"));

        let from_ts =
            lower_intrinsic("datetime_from_timestamp", &["ts".to_string()]).expect("from_timestamp");
        assert_eq!(from_ts.required_crate, Some("chrono"));
        assert!(render_expr(&from_ts.expr).contains("DateTime::from_timestamp"));
    }

    #[test]
    fn lowers_sys_intrinsics_via_registry() {
        let exit = lower_intrinsic("sys_exit", &["code".to_string()]).expect("sys_exit");
        assert!(render_expr(&exit.expr).contains("std::process::exit(code as i32)"));

        let version = lower_intrinsic("sys_version", &[]).expect("sys_version");
        assert_eq!(render_expr(&version.expr), "\"sifr 0.1.0\".to_string()");

        let platform = lower_intrinsic("sys_platform", &[]).expect("sys_platform");
        assert_eq!(render_expr(&platform.expr), "std::env::consts::OS.to_string()");

        let maxsize = lower_intrinsic("sys_maxsize", &[]).expect("sys_maxsize");
        assert_eq!(render_expr(&maxsize.expr), "i64::MAX");
    }

    #[test]
    fn lowers_subprocess_intrinsics_via_registry() {
        let run = lower_intrinsic("subprocess_run", &["cmd".to_string()]).expect("subprocess_run");
        assert!(render_expr(&run.expr).contains("Command::new(\"sh\")"));
        assert!(render_expr(&run.expr).contains("Result<String, IOError>"));

        let with_input = lower_intrinsic(
            "subprocess_run_with_input",
            &["cmd".to_string(), "stdin_data".to_string()],
        )
        .expect("subprocess_run_with_input");
        assert!(render_expr(&with_input.expr).contains("use std::io::Write"));
        assert!(render_expr(&with_input.expr).contains("stdin.write_all"));

        let structured = lower_intrinsic("subprocess_run_structured", &["cmd".to_string()])
            .expect("subprocess_run_structured");
        assert!(render_expr(&structured.expr).contains("Result<Vec<String>, IOError>"));
        assert!(render_expr(&structured.expr).contains("returncode"));
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
        let leap = lower_intrinsic("calendar_isleap", &["year".to_string()])
            .expect("calendar_isleap");
        assert!(render_expr(&leap.expr).contains("__y % 4 == 0"));

        let weekday = lower_intrinsic(
            "calendar_weekday",
            &["y".to_string(), "m".to_string(), "d".to_string()],
        )
        .expect("calendar_weekday");
        assert!(render_expr(&weekday.expr).contains("__t = [0i64, 3, 2, 5"));
        assert!(render_expr(&weekday.expr).contains("__t[(__m0-1) as usize]"));

        let monthrange = lower_intrinsic("calendar_monthrange", &["y".to_string(), "m".to_string()])
            .expect("calendar_monthrange");
        assert!(render_expr(&monthrange.expr).contains("vec![__wd, __days]"));
    }

    #[test]
    fn lowers_gzip_intrinsics_with_dependency_metadata() {
        let compress = lower_intrinsic("gzip_compress", &["data".to_string()]).expect("gzip_compress");
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
            &["path".to_string(), "name".to_string(), "content".to_string()],
        )
        .expect("zip_add_file");
        assert_eq!(add.required_crate, Some("zip"));
        assert!(render_expr(&add.expr).contains("start_file"));

        let read =
            lower_intrinsic("zip_read_file", &["path".to_string(), "name".to_string()]).expect("zip_read_file");
        assert_eq!(read.required_crate, Some("zip"));
        assert!(render_expr(&read.expr).contains("ZipArchive::new"));

        let names = lower_intrinsic("zip_namelist", &["path".to_string()]).expect("zip_namelist");
        assert_eq!(names.required_crate, Some("zip"));
        assert!(render_expr(&names.expr).contains("__zip.by_index"));
    }

    #[test]
    fn lowers_base64_intrinsics_with_dependency_metadata() {
        let enc = lower_intrinsic("base64_encode", &["text".to_string()]).expect("base64_encode");
        assert_eq!(enc.required_crate, Some("base64"));
        assert!(render_expr(&enc.expr).contains("general_purpose::STANDARD.encode"));

        let dec = lower_intrinsic("base64_decode", &["s".to_string()]).expect("base64_decode");
        assert_eq!(dec.required_crate, Some("base64"));
        assert!(render_expr(&dec.expr).contains("general_purpose::STANDARD.decode"));

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
        assert!(render_expr(&url_enc.expr).contains("general_purpose::URL_SAFE.encode"));

        let url_dec =
            lower_intrinsic("urlsafe_b64decode", &["s".to_string()]).expect("urlsafe_b64decode");
        assert_eq!(url_dec.required_crate, Some("base64"));
        assert!(render_expr(&url_dec.expr).contains("general_purpose::URL_SAFE.decode"));
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
        assert!(render_expr(&sha1.expr).contains("sha1::Sha1::digest"));

        let sha512 = lower_intrinsic("sha512", &["s".to_string()]).expect("sha512");
        assert_eq!(sha512.required_crate, Some("sha2"));
        assert!(render_expr(&sha512.expr).contains("sha2::Sha512::digest"));

        let sha224 = lower_intrinsic("sha224", &["s".to_string()]).expect("sha224");
        assert_eq!(sha224.required_crate, Some("sha2"));
        assert!(render_expr(&sha224.expr).contains("sha2::Sha224::new"));

        let sha384 = lower_intrinsic("sha384", &["s".to_string()]).expect("sha384");
        assert_eq!(sha384.required_crate, Some("sha2"));
        assert!(render_expr(&sha384.expr).contains("sha2::Sha384::new"));

        let blake2b = lower_intrinsic("blake2b", &["s".to_string()]).expect("blake2b");
        assert_eq!(blake2b.required_crate, Some("blake2"));
        assert!(render_expr(&blake2b.expr).contains("Blake2b512"));

        let blake2s = lower_intrinsic("blake2s", &["s".to_string()]).expect("blake2s");
        assert_eq!(blake2s.required_crate, Some("blake2"));
        assert!(render_expr(&blake2s.expr).contains("Blake2s256"));
    }
}
