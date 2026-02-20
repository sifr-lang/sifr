//! OS intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrow_expr(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_run_command(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, IOError> {{ let __cmd = {}; let output = std::process::Command::new(\"sh\").args([\"-c\", &__cmd]).output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) }})()",
        args[0]
    )))
}

pub(super) fn lower_get_args(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "std::env::args().collect::<Vec<String>>()".to_string(),
    ))
}

pub(super) fn lower_chdir(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::env::set_current_dir({}).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_getpid(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("std::process::id() as i64".to_string()))
}

pub(super) fn lower_cpu_count(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ let __n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1); __n as i64 }".to_string(),
    ))
}

pub(super) fn lower_stat_size(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::metadata({}).map(|m| m.len() as i64).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}
