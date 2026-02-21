//! Set method lowerers for registry migration.

use crate::RustExpr;

fn render_borrowed_arg_expr(arg: &str) -> RustExpr {
    if arg.ends_with(".as_str()") || arg.starts_with('&') {
        RustExpr::RawCode(arg.to_string())
    } else {
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::RawCode(format!("({arg})"))),
        }
    }
}

pub(super) fn lower_add(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "insert".to_string(),
        args: vec![RustExpr::RawCode(args[0].clone())],
    })
}

pub(super) fn lower_remove(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "remove".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_discard(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "remove".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_contains(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "contains".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_clear(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "clear".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_copy(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "clone".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_issubset(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "is_subset".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_issuperset(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "is_superset".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_isdisjoint(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "is_disjoint".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_pop(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __v = {object}.iter().next().cloned(); if let Some(ref __val) = __v {{ {object}.remove(__val); }} __v }}"
    )))
}

pub(super) fn lower_union(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.union(&{}).cloned().collect::<std::collections::HashSet<_>>()",
        args[0]
    )))
}

pub(super) fn lower_intersection(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.intersection(&{}).cloned().collect::<std::collections::HashSet<_>>()",
        args[0]
    )))
}

pub(super) fn lower_difference(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.difference(&{}).cloned().collect::<std::collections::HashSet<_>>()",
        args[0]
    )))
}

pub(super) fn lower_symmetric_difference(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.symmetric_difference(&{}).cloned().collect::<std::collections::HashSet<_>>()",
        args[0]
    )))
}
