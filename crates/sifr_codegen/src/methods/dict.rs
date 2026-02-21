//! Dict method lowerers for registry migration.

use crate::RustExpr;

fn render_key_arg(arg: &str) -> String {
    if arg.ends_with(".as_str()") || arg.starts_with('&') {
        arg.to_string()
    } else {
        format!("&({arg})")
    }
}

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

pub(super) fn lower_contains(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let key = render_key_arg(&args[0]);
    Some(RustExpr::RawCode(format!("{object}.contains_key({key})")))
}

pub(super) fn lower_get(object: &str, args: &[String]) -> Option<RustExpr> {
    match args.len() {
        1 => {
            let key = render_key_arg(&args[0]);
            Some(RustExpr::RawCode(format!("{object}.get({key}).cloned()")))
        }
        2 => {
            let key = render_key_arg(&args[0]);
            Some(RustExpr::RawCode(format!(
                "{object}.get({key}).cloned().unwrap_or({})",
                args[1]
            )))
        }
        _ => None,
    }
}

pub(super) fn lower_pop(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let key = render_key_arg(&args[0]);
    Some(RustExpr::RawCode(format!("{object}.remove({key})")))
}
