//! String method lowerers for registry migration.

use crate::{RustExpr, RustParam, RustType};

fn lower_zero_arg_method(object: &str, args: &[String], method: &str) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: method.to_string(),
        args: vec![],
    })
}

fn lower_trim_to_string(object: &str, args: &[String], trim_method: &str) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(object.to_string())),
            method: trim_method.to_string(),
            args: vec![],
        }),
        method: "to_string".to_string(),
        args: vec![],
    })
}

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

pub(super) fn lower_upper(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_zero_arg_method(object, args, "to_uppercase")
}

pub(super) fn lower_lower(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_zero_arg_method(object, args, "to_lowercase")
}

pub(super) fn lower_strip(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim")
}

pub(super) fn lower_startswith(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "starts_with".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_endswith(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(object.to_string())),
        method: "ends_with".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
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
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(object.to_string())),
            method: "find".to_string(),
            args: vec![render_borrowed_arg_expr(&args[0])],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "i".to_string(),
                ty: RustType::RawCode("_".to_string()),
            }],
            body: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Ident("i".to_string())),
                ty: RustType::I64,
            }),
            is_move: false,
        }],
    })
}

pub(super) fn lower_lstrip(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim_start")
}

pub(super) fn lower_rstrip(object: &str, args: &[String]) -> Option<RustExpr> {
    lower_trim_to_string(object, args, "trim_end")
}

pub(super) fn lower_count(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "matches".to_string(),
                args: vec![render_borrowed_arg_expr(&args[0])],
            }),
            method: "count".to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}

pub(super) fn lower_join(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::RawCode(args[0].clone())),
        method: "join".to_string(),
        args: vec![render_borrowed_arg_expr(object)],
    })
}

pub(super) fn lower_title(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.split_whitespace().map(|w| {{ let mut c = w.chars(); match c.next() {{ None => String::new(), Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase() }} }}).collect::<Vec<_>>().join(\" \")"
    )))
}

pub(super) fn lower_capitalize(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let _s = ({object}).clone(); let mut _c = _s.chars(); match _c.next() {{ None => String::new(), Some(f) => f.to_uppercase().to_string() + &_c.as_str().to_lowercase() }} }}"
    )))
}

pub(super) fn lower_swapcase(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.chars().map(|c| if c.is_uppercase() {{ c.to_lowercase().to_string() }} else {{ c.to_uppercase().to_string() }}).collect::<String>()"
    )))
}

pub(super) fn lower_isdigit(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "!{object}.is_empty() && {object}.chars().all(|c| c.is_ascii_digit())"
    )))
}

pub(super) fn lower_isalpha(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "!{object}.is_empty() && {object}.chars().all(|c| c.is_alphabetic())"
    )))
}

pub(super) fn lower_isalnum(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "!{object}.is_empty() && {object}.chars().all(|c| c.is_alphanumeric())"
    )))
}

pub(super) fn lower_isspace(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "!{object}.is_empty() && {object}.chars().all(|c| c.is_whitespace())"
    )))
}

pub(super) fn lower_isupper(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.chars().any(|c| c.is_alphabetic()) && {object}.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase())"
    )))
}

pub(super) fn lower_islower(object: &str, args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{object}.chars().any(|c| c.is_alphabetic()) && {object}.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_lowercase())"
    )))
}

pub(super) fn lower_center(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let _s = ({object}).clone(); let _w = {} as usize; let _len = _s.chars().count(); if _len >= _w {{ _s }} else {{ let _pad = _w - _len; let _left = _pad / 2; let _right = _pad - _left; format!(\"{{}}{{}}{{}}\", \" \".repeat(_left), _s, \" \".repeat(_right)) }} }}",
        args[0]
    )))
}

pub(super) fn lower_ljust(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "format!(\"{{:<width$}}\", {object}, width = {} as usize)",
        args[0]
    )))
}

pub(super) fn lower_rjust(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "format!(\"{{:>width$}}\", {object}, width = {} as usize)",
        args[0]
    )))
}

pub(super) fn lower_zfill(object: &str, args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "format!(\"{{:0>width$}}\", {object}, width = {} as usize)",
        args[0]
    )))
}
