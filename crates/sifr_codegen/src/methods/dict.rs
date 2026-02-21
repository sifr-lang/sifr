//! Dict method lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_keys(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.keys().cloned().collect::<Vec<_>>()"
    )))
}

pub(super) fn lower_values(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.values().cloned().collect::<Vec<_>>()"
    )))
}

pub(super) fn lower_items(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()"
    )))
}

pub(super) fn lower_update(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.extend({})", args[0])))
}

pub(super) fn lower_clear(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.clear()")))
}

pub(super) fn lower_copy(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.clone()")))
}
