//! Pathlib intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrow_expr(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_touch(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::OpenOptions::new().create(true).write(true).open({}).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_resolve_path(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::canonicalize({}).map(|p| p.to_string_lossy().to_string()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_iterdir(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<String>, IOError> {{ let __entries = std::fs::read_dir({}).map_err(__io_err)?; Ok(__entries.filter_map(|e| e.ok().map(|e| e.path().to_string_lossy().to_string())).collect()) }})()",
        borrow_expr(&args[0])
    )))
}
