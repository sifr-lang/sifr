//! OS intrinsic lowerers for registry migration.

use crate::RustExpr;

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
