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
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "env".to_string(),
                "args".to_string(),
            ])),
            args: vec![],
        }),
        method: "collect::<Vec<String>>".to_string(),
        args: vec![],
    })
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
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "process".to_string(),
                "id".to_string(),
            ])),
            args: vec![],
        }),
        ty: crate::RustType::I64,
    })
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

pub(super) fn lower_which(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __name = {}; std::env::var(\"PATH\").ok().and_then(|__path| __path.split(':').map(|d| std::path::Path::new(d).join(__name)).find(|p| p.is_file()).map(|p| p.to_string_lossy().to_string())) }}",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_disk_usage(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __path = {}; let __stat = std::fs::metadata(__path); match __stat {{ Ok(_) => {{ let __out = std::process::Command::new(\"df\").args([\"-k\", __path]).output(); match __out {{ Ok(__o) => {{ let __s = String::from_utf8_lossy(&__o.stdout); let __lines: Vec<&str> = __s.lines().collect(); if __lines.len() >= 2 {{ let __parts: Vec<&str> = __lines[1].split_whitespace().collect(); if __parts.len() >= 4 {{ let __total = __parts[1].parse::<i64>().unwrap_or(0) * 1024; let __used = __parts[2].parse::<i64>().unwrap_or(0) * 1024; let __free = __parts[3].parse::<i64>().unwrap_or(0) * 1024; vec![__total, __used, __free] }} else {{ vec![0i64, 0, 0] }} }} else {{ vec![0i64, 0, 0] }} }}, Err(_) => vec![0i64, 0, 0] }} }}, Err(_) => vec![0i64, 0, 0] }} }}",
        borrow_expr(&args[0])
    )))
}

pub(super) fn lower_os_sep(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "path".to_string(),
            "MAIN_SEPARATOR".to_string(),
        ])),
        method: "to_string".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_os_linesep(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ if cfg!(target_os = \"windows\") { \"\\r\\n\".to_string() } else { \"\\n\".to_string() } }"
            .to_string(),
    ))
}

pub(super) fn lower_os_name(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ if cfg!(target_os = \"windows\") { \"nt\".to_string() } else { \"posix\".to_string() } }"
            .to_string(),
    ))
}
