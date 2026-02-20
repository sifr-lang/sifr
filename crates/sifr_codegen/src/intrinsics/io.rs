//! I/O intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrow_expr(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_read_text(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::read_to_string({}).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_write_text(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::write({}, {}.as_bytes()).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_exists(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::path::Path::new(&({})).exists()",
        args[0]
    )))
}

pub(super) fn lower_read_lines(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::read_to_string({}).map(|s| s.lines().map(|l| l.to_string()).collect::<Vec<String>>()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_append_text(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ use std::io::Write; (|| -> Result<(), IOError> {{ let mut _f = std::fs::OpenOptions::new().append(true).create(true).open({}).map_err(__io_err)?; write!(_f, \"{{}}\", {}).map_err(__io_err)?; Ok(()) }})() }}",
        borrow_expr(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_getcwd(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "std::env::current_dir().map(|p| p.to_string_lossy().to_string()).map_err(__io_err)"
            .to_string(),
    ))
}

pub(super) fn lower_listdir(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::read_dir({}).map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect::<Vec<String>>()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_walk_dir(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ fn __walk(p: &std::path::Path) -> Result<Vec<String>, IOError> {{ let mut r = Vec::new(); let entries = std::fs::read_dir(p).map_err(__io_err)?; for e in entries {{ let e = e.map_err(__io_err)?; let path = e.path(); r.push(path.display().to_string()); if path.is_dir() {{ r.extend(__walk(&path)?); }} }} Ok(r) }} __walk(std::path::Path::new({})) }}",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_mkdir(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::create_dir_all({}).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_rmdir(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::remove_dir({}).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_remove_file(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::remove_file({}).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_rename(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::rename({}, {}).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0]),
        borrow_expr(&args[1])
    )))
}

pub(super) fn lower_is_file(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::path::Path::new(&({})).is_file()",
        args[0]
    )))
}

pub(super) fn lower_is_dir(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::path::Path::new(&({})).is_dir()",
        args[0]
    )))
}

pub(super) fn lower_copy_file(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::copy({}, {}).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0]),
        borrow_expr(&args[1])
    )))
}

pub(super) fn lower_rmdir_all(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::remove_dir_all({}).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_gettempdir(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "std::env::temp_dir().display().to_string()".to_string(),
    ))
}

pub(super) fn lower_makedirs(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "std::fs::create_dir_all({}).map(|_| ()).map_err(__io_err)",
        borrow_expr(&args[0])
    )))
}
