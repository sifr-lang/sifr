//! Time intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_time_now(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64()".to_string(),
    ))
}

pub(super) fn lower_sleep(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::thread::sleep(std::time::Duration::from_secs_f64({}))",
        args[0]
    )))
}

pub(super) fn lower_time_format(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let secs = {} as i64; let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default(); dt.format({}).to_string() }}",
        args[0],
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_perf_counter(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ fn __monotonic() -> f64 { static __START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new(); let s = __START.get_or_init(std::time::Instant::now); s.elapsed().as_secs_f64() } __monotonic() }".to_string(),
    ))
}

pub(super) fn lower_monotonic(args: &[String]) -> Option<RustExpr> {
    lower_perf_counter(args)
}

pub(super) fn lower_strptime(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, ValueError> {{ use chrono::NaiveDateTime; let __s = {}; let __fmt = {}; NaiveDateTime::parse_from_str(__s, __fmt).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).map_err(|e| ValueError {{ message: e.to_string() }}) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_gmtime(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ use chrono::{{DateTime, Utc}}; let __ts = {} as i64; DateTime::<Utc>::from_timestamp(__ts, 0).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).unwrap_or_default() }}",
        args[0]
    )))
}

pub(super) fn lower_localtime(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ use chrono::{{DateTime, Utc, Local}}; let __ts = {} as i64; DateTime::<Utc>::from_timestamp(__ts, 0).map(|dt| dt.with_timezone(&Local).format(\"%Y-%m-%dT%H:%M:%S\").to_string()).unwrap_or_default() }}",
        args[0]
    )))
}
