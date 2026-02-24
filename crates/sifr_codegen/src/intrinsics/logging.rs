//! Logging state intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_set_global_level(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ *__SIFR_GLOBAL_LOG_LEVEL.lock().unwrap() = ({}); () }}",
        args[0]
    )))
}

pub(super) fn lower_get_global_level(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "*__SIFR_GLOBAL_LOG_LEVEL.lock().unwrap()".to_string(),
    ))
}
