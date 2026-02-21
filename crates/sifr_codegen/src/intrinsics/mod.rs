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
        "touch" => (pathlib::lower_touch(rendered_args), None),
        "resolve_path" => (pathlib::lower_resolve_path(rendered_args), None),
        "iterdir" => (pathlib::lower_iterdir(rendered_args), None),
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
        "time_format" => (time::lower_time_format(rendered_args), None),
        "perf_counter" => (time::lower_perf_counter(rendered_args), None),
        "monotonic" => (time::lower_monotonic(rendered_args), None),
        "strptime" => (time::lower_strptime(rendered_args), None),
        "gmtime" => (time::lower_gmtime(rendered_args), None),
        "localtime" => (time::lower_localtime(rendered_args), None),
        "_strptime_intrinsic" => (time::lower_strptime(rendered_args), None),
        "_gmtime_intrinsic" => (time::lower_gmtime(rendered_args), None),
        "_localtime_intrinsic" => (time::lower_localtime(rendered_args), None),
        "random_int" => (random::lower_random_int(rendered_args), None),
        "random_float" => (random::lower_random_float(rendered_args), None),
        "random_choice" => (random::lower_random_choice(rendered_args), None),
        "random_uniform" => (random::lower_random_uniform(rendered_args), None),
        "random_shuffle" => (random::lower_random_shuffle(rendered_args), None),
        "random_sample" => (random::lower_random_sample(rendered_args), None),
        "random_randrange" => (random::lower_random_randrange(rendered_args), None),
        "random_gauss" => (random::lower_random_gauss(rendered_args), None),
        "re_match" => (re::lower_re_match(rendered_args), None),
        "re_find" => (re::lower_re_find(rendered_args), None),
        "re_replace" => (re::lower_re_replace(rendered_args), None),
        "re_findall" => (re::lower_re_findall(rendered_args), None),
        "re_split" => (re::lower_re_split(rendered_args), None),
        "re_find_start" => (re::lower_re_find_start(rendered_args), None),
        "re_find_end" => (re::lower_re_find_end(rendered_args), None),
        "re_match_flags" => (re::lower_re_match_flags(rendered_args), None),
        "re_find_flags" => (re::lower_re_find_flags(rendered_args), None),
        "re_replace_flags" => (re::lower_re_replace_flags(rendered_args), None),
        "re_findall_flags" => (re::lower_re_findall_flags(rendered_args), None),
        "re_split_flags" => (re::lower_re_split_flags(rendered_args), None),
        "sha256" => (hash::lower_sha256(rendered_args), None),
        "md5" => (hash::lower_md5(rendered_args), None),
        "platform_system" => (platform::lower_platform_system(rendered_args), None),
        "platform_arch" => (platform::lower_platform_arch(rendered_args), None),
        "platform_node" => (platform::lower_platform_node(rendered_args), None),
        "platform_release" => (platform::lower_platform_release(rendered_args), None),
        "platform_version" => (platform::lower_platform_version(rendered_args), None),
        "platform_processor" => (platform::lower_platform_processor(rendered_args), None),
        "uuid4" => (uuid::lower_uuid4(rendered_args), None),
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
        assert!(render_expr(&fmt.expr).contains("DateTime::from_timestamp"));

        let perf = lower_intrinsic("perf_counter", &[]).expect("perf_counter");
        assert!(render_expr(&perf.expr).contains("OnceLock<std::time::Instant>"));

        let mono = lower_intrinsic("monotonic", &[]).expect("monotonic");
        assert!(render_expr(&mono.expr).contains("OnceLock<std::time::Instant>"));

        let parse =
            lower_intrinsic("strptime", &["s".to_string(), "f".to_string()]).expect("strptime");
        assert!(render_expr(&parse.expr).contains("NaiveDateTime::parse_from_str"));

        let gmt = lower_intrinsic("gmtime", &["ts".to_string()]).expect("gmtime");
        assert!(render_expr(&gmt.expr).contains("DateTime::<Utc>::from_timestamp"));

        let local = lower_intrinsic("localtime", &["ts".to_string()]).expect("localtime");
        assert!(render_expr(&local.expr).contains("with_timezone(&Local)"));

        let parse_alias = lower_intrinsic("_strptime_intrinsic", &["s".to_string(), "f".to_string()])
            .expect("_strptime_intrinsic");
        assert!(render_expr(&parse_alias.expr).contains("NaiveDateTime::parse_from_str"));
    }

    #[test]
    fn lowers_random_intrinsics_via_registry() {
        let rint = lower_intrinsic("random_int", &["1".to_string(), "9".to_string()])
            .expect("random_int");
        assert!(render_expr(&rint.expr).contains("gen_range(1..=9)"));

        let rfloat = lower_intrinsic("random_float", &[]).expect("random_float");
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
        assert!(render_expr(&rf.expr).contains("replace_all"));
    }

    #[test]
    fn lowers_hash_intrinsics_via_registry() {
        let sha = lower_intrinsic("sha256", &["payload".to_string()]).expect("sha256");
        assert!(render_expr(&sha.expr).contains("sha2::Sha256::digest"));
        assert!(render_expr(&sha.expr).contains(".as_bytes()"));

        let md5 = lower_intrinsic("md5", &["payload".to_string()]).expect("md5");
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
        assert!(render_expr(&uuid.expr).contains("rand::thread_rng"));
        assert!(render_expr(&uuid.expr).contains("format!(\"{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}\""));
    }
}
