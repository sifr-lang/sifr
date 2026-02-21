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

pub(super) fn lower_startswith(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.starts_with(&({}))",
        args[0]
    )))
}

pub(super) fn lower_endswith(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.ends_with(&({}))", args[0])))
}

pub(super) fn lower_split(object: &str, args: &[String]) -> Option<RustExpr> {
    match args.len() {
        0 => Some(RustExpr::RawCode(format!(
            "{object}.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>()"
        ))),
        1 => Some(RustExpr::RawCode(format!(
            "{object}.split(&({})).map(|s| s.to_string()).collect::<Vec<String>>()",
            args[0]
        ))),
        _ => None,
    }
}

pub(super) fn lower_replace(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.replace(&({}), &({}))",
        args[0], args[1]
    )))
}

pub(super) fn lower_find(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.find(&({})).map(|i| i as i64)",
        args[0]
    )))
}
