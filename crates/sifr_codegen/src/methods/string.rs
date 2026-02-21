//! String method lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_upper(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.to_uppercase()")))
}

pub(super) fn lower_lower(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.to_lowercase()")))
}

pub(super) fn lower_strip(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.trim().to_string()")))
}
