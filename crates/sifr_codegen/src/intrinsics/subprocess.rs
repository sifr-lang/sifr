//! Subprocess intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_subprocess_run(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, IOError> {{ let output = std::process::Command::new(\"sh\").args([\"-c\", {}]).output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) }})()",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_subprocess_run_with_input(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, IOError> {{ use std::io::Write; let mut child = std::process::Command::new(\"sh\").args([\"-c\", {}]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().map_err(__io_err)?; if let Some(mut stdin) = child.stdin.take() {{ stdin.write_all({}.as_bytes()).map_err(__io_err)?; }} let output = child.wait_with_output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_subprocess_run_structured(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<String>, IOError> {{ let output = std::process::Command::new(\"sh\").args([\"-c\", {}]).output().map_err(__io_err)?; let stdout = String::from_utf8_lossy(&output.stdout).to_string(); let stderr = String::from_utf8_lossy(&output.stderr).to_string(); let returncode = output.status.code().unwrap_or(-1).to_string(); Ok(vec![stdout, stderr, returncode]) }})()",
        borrowed_str(&args[0])
    )))
}
