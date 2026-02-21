//! List method lowerers for registry migration.

use crate::RustExpr;

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

pub(super) fn lower_reverse(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.reverse()")))
}

pub(super) fn lower_sort(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.sort()")))
}

pub(super) fn lower_count(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.iter().filter(|x| **x == {}).count() as i64",
        args[0]
    )))
}

pub(super) fn lower_contains(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.contains(&{})", args[0])))
}

pub(super) fn lower_pop(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!("{object}.pop()")))
}

pub(super) fn lower_remove(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ if let Some(__pos) = {object}.iter().position(|__x| *__x == {}) {{ {object}.remove(__pos); }} }}",
        args[0]
    )))
}

pub(super) fn lower_index(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.iter().position(|__x| *__x == {}).map(|__p| __p as i64)",
        args[0]
    )))
}
