//! Collections intrinsic lowerers for registry migration.

use crate::RustExpr;

fn cloned_vec(expr: &str) -> String {
    format!("({expr}).clone()")
}

pub(super) fn lower_new_set(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("Vec::<i64>::new()".to_string()))
}

pub(super) fn lower_set_from_list(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut s = {}; s.sort(); s.dedup(); s }}",
        cloned_vec(&args[0])
    )))
}

pub(super) fn lower_set_add(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut s = {}; let v = {}; if !s.contains(&v) {{ s.push(v); }} s }}",
        cloned_vec(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_set_contains(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{}.contains(&{})",
        args[0], args[1]
    )))
}

pub(super) fn lower_set_remove(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut s = {}; s.retain(|x| *x != {}); s }}",
        cloned_vec(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_set_len(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!("{}.len() as i64", args[0])))
}

pub(super) fn lower_set_union(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut s = {}; for v in {}.iter() {{ if !s.contains(v) {{ s.push(*v); }} }} s.sort(); s }}",
        cloned_vec(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_set_intersection(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __a = {}; let __b = {}; __a.iter().filter(|x| __b.contains(x)).cloned().collect::<Vec<i64>>() }}",
        cloned_vec(&args[0]),
        cloned_vec(&args[1])
    )))
}
