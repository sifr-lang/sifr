//! Sys intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_sys_exit(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ std::process::exit({} as i32) }}",
        args[0]
    )))
}

pub(super) fn lower_sys_version(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("\"sifr 0.1.0\".to_string()".to_string()))
}

pub(super) fn lower_sys_platform(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("std::env::consts::OS.to_string()".to_string()))
}

pub(super) fn lower_sys_maxsize(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("i64::MAX".to_string()))
}
