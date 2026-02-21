//! Datetime intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_datetime_now(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "chrono::Local::now().format(\"%Y-%m-%dT%H:%M:%S\").to_string()".to_string(),
    ))
}

pub(super) fn lower_datetime_now_struct(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ use chrono::{Datelike, Timelike}; let __dt = chrono::Local::now(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64] }".to_string(),
    ))
}

pub(super) fn lower_datetime_format(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __dt_str = {}; let __fmt = {}; __dt_str.to_string() }}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_datetime_from_timestamp(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __ts = {} as i64; chrono::DateTime::from_timestamp(__ts, 0).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).ok_or_else(|| ValueError {{ message: \"invalid timestamp\".to_string() }}) }}",
        args[0]
    )))
}
