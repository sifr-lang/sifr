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

pub(super) fn lower_glob_pattern(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<String>, IOError> {{ let __dir = {}; let __pat = {}; let __full_pat = if __dir.is_empty() {{ __pat.to_string() }} else {{ format!(\"{{}}/{{}}\", __dir, __pat) }}; let __entries = std::fs::read_dir(__dir).map_err(__io_err)?; let mut __results: Vec<String> = Vec::new(); fn __matches_glob(name: &str, pattern: &str) -> bool {{ let __parts: Vec<&str> = pattern.split('*').collect(); if __parts.len() == 1 {{ return name == pattern; }} if !name.starts_with(__parts[0]) {{ return false; }} let mut __pos = __parts[0].len(); for __i in 1..__parts.len() {{ if __parts[__i].is_empty() {{ __pos = name.len(); continue; }} match name[__pos..].find(__parts[__i]) {{ Some(__idx) => __pos += __idx + __parts[__i].len(), None => return false, }} }} true }} for __entry in __entries {{ let __e = __entry.map_err(__io_err)?; let __name = __e.file_name().to_string_lossy().to_string(); if __matches_glob(&__name, __pat) {{ __results.push(__e.path().to_string_lossy().to_string()); }} }} __results.sort(); Ok(__results) }})()",
        borrow_expr(&args[0]),
        borrow_expr(&args[1])
    )))
}

pub(super) fn lower_rglob_pattern(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<String>, IOError> {{ let __dir = {}; let __pat = {}; fn __rglob_walk(dir: &str, pattern: &str, results: &mut Vec<String>) -> Result<(), IOError> {{ fn __io_err_inner(e: std::io::Error) -> IOError {{ IOError {{ message: e.to_string(), kind: \"Other\".to_string() }} }} fn __matches_glob(name: &str, pat: &str) -> bool {{ let parts: Vec<&str> = pat.split('*').collect(); if parts.len() == 1 {{ return name == pat; }} if !name.starts_with(parts[0]) {{ return false; }} let mut pos = parts[0].len(); for i in 1..parts.len() {{ if parts[i].is_empty() {{ pos = name.len(); continue; }} match name[pos..].find(parts[i]) {{ Some(idx) => pos += idx + parts[i].len(), None => return false, }} }} true }} let entries = std::fs::read_dir(dir).map_err(__io_err_inner)?; for entry in entries {{ let e = entry.map_err(__io_err_inner)?; let path = e.path(); let name = e.file_name().to_string_lossy().to_string(); if path.is_dir() {{ __rglob_walk(&path.to_string_lossy(), pattern, results)?; }} if __matches_glob(&name, pattern) {{ results.push(path.to_string_lossy().to_string()); }} }} Ok(()) }} let mut __results: Vec<String> = Vec::new(); __rglob_walk(__dir, __pat, &mut __results).map_err(|e| e)?; __results.sort(); Ok(__results) }})()",
        borrow_expr(&args[0]),
        borrow_expr(&args[1])
    )))
}
