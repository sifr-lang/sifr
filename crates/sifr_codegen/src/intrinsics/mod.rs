//! Intrinsic registry and dispatch for incremental migration.

mod math;
mod json;
mod env;
mod os;
mod io;
mod pathlib;

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
}
