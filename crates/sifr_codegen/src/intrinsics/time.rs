//! Time intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_time_now(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "time".to_string(),
                        "SystemTime".to_string(),
                        "now".to_string(),
                    ])),
                    args: vec![],
                }),
                method: "duration_since".to_string(),
                args: vec![RustExpr::Path(vec![
                    "std".to_string(),
                    "time".to_string(),
                    "UNIX_EPOCH".to_string(),
                ])],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        }),
        method: "as_secs_f64".to_string(),
        args: vec![],
    })
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

pub(super) fn lower_time_strptime_compat(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<i64>, ValueError> {{ let __s = {}; let __fmt = {}; chrono::NaiveDateTime::parse_from_str(__s, __fmt).map(|dt| {{ use chrono::Datelike; use chrono::Timelike; vec![dt.year() as i64, dt.month() as i64, dt.day() as i64, dt.hour() as i64, dt.minute() as i64, dt.second() as i64, dt.weekday().num_days_from_monday() as i64, dt.ordinal() as i64] }}).map_err(|e| ValueError {{ message: e.to_string() }}) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_time_gmtime_compat(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ use chrono::{Datelike, Timelike, Utc}; let __dt = Utc::now().naive_utc(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64, __dt.weekday().num_days_from_monday() as i64, __dt.ordinal() as i64] }".to_string(),
    ))
}

pub(super) fn lower_time_localtime_compat(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ use chrono::{Datelike, Timelike, Local}; let __dt = Local::now().naive_local(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64, __dt.weekday().num_days_from_monday() as i64, __dt.ordinal() as i64] }".to_string(),
    ))
}
